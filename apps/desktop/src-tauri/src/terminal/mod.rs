pub mod types;

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    io::{self, Read, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use uuid::Uuid;

use crate::project::{ProjectExecutionError, ProjectService, StoredTerminalSession};
use types::{
    TerminalDiagnosticCode, TerminalOutputChunk, TerminalPollRequest, TerminalRegistrySnapshot,
    TerminalResizeRequest, TerminalSnapshot, TerminalStartRequest, TerminalState,
    TerminalWriteRequest, TERMINAL_REGISTRY_SCHEMA_VERSION, TERMINAL_SCHEMA_VERSION,
};

const TERMINAL_CAPACITY: usize = 8;
const MIN_COLUMNS: u16 = 2;
const MAX_COLUMNS: u16 = 500;
const MIN_ROWS: u16 = 2;
const MAX_ROWS: u16 = 200;
const MAX_INPUT_BYTES: usize = 64 * 1024;
const MAX_INPUT_BASE64_BYTES: usize = MAX_INPUT_BYTES.div_ceil(3) * 4;
const MAX_RETAINED_OUTPUT_BYTES: usize = 1024 * 1024;
const MAX_OUTPUT_CHUNKS: usize = 512;
const MAX_POLL_OUTPUT_BYTES: usize = 128 * 1024;
const MAX_POLL_CHUNKS: usize = 64;
const READER_BUFFER_BYTES: usize = 8 * 1024;
const READER_POLL_MILLIS: i32 = 100;
const PROCESS_SETTLE_TIMEOUT: Duration = Duration::from_millis(250);
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);
const SYSTEM_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
const INHERITED_ENVIRONMENT: &[&str] = &[
    "DBUS_SESSION_BUS_ADDRESS",
    "DESKTOP_SESSION",
    "DISPLAY",
    "HOME",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "LOGNAME",
    "USER",
    "WAYLAND_DISPLAY",
    "XAUTHORITY",
    "XDG_CONFIG_HOME",
    "XDG_CURRENT_DESKTOP",
    "XDG_DATA_DIRS",
    "XDG_DATA_HOME",
    "XDG_RUNTIME_DIR",
    "XDG_SESSION_DESKTOP",
    "XDG_SESSION_TYPE",
];

#[derive(Default)]
struct ServiceState {
    sessions: HashMap<String, Arc<TerminalSession>>,
    pending_starts: usize,
    pending_titles: HashSet<String>,
}

#[derive(Default)]
pub struct TerminalService {
    state: Mutex<ServiceState>,
}

struct TerminalSession {
    terminal_id: String,
    project_id: String,
    title: String,
    runtime: Mutex<TerminalRuntime>,
    output: Arc<Mutex<OutputStore>>,
    reader_stop: Arc<AtomicBool>,
    reader_failed: Arc<AtomicBool>,
    reader_thread: Mutex<Option<thread::JoinHandle<()>>>,
}

struct TerminalRuntime {
    state: TerminalState,
    columns: u16,
    rows: u16,
    exit_code: Option<i32>,
    master: Option<Box<dyn MasterPty + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    child: Option<Box<dyn Child + Send + Sync>>,
    session_id: i32,
    leader_start_time: u64,
}

#[derive(Default)]
struct OutputStore {
    chunks: VecDeque<StoredOutputChunk>,
    retained_bytes: usize,
    next_sequence: u64,
    dropped_through: u64,
}

struct StoredOutputChunk {
    sequence: u64,
    data: Vec<u8>,
}

#[derive(Default)]
struct OutputWindow {
    chunks: Vec<TerminalOutputChunk>,
    first_sequence: u64,
    last_sequence: u64,
    truncated: bool,
    has_more: bool,
}

#[derive(Debug)]
enum TerminalSpawnError {
    Pty,
    Shell,
}

impl TerminalService {
    pub async fn status(&self, projects: &ProjectService) -> TerminalRegistrySnapshot {
        let sessions = match self.live_sessions() {
            Ok(sessions) => sessions,
            Err(code) => return TerminalRegistrySnapshot::unavailable(code),
        };
        for session in sessions {
            let _ = refresh_and_record(&session, projects);
        }
        self.registry_snapshot(projects)
    }

    pub async fn start(
        &self,
        request: TerminalStartRequest,
        projects: &ProjectService,
    ) -> TerminalSnapshot {
        if !valid_dimensions(request.columns, request.rows) {
            return TerminalSnapshot::unavailable(
                Some(request.project_id),
                TerminalDiagnosticCode::InvalidRequest,
            );
        }
        let records = match projects.terminal_records() {
            Ok(records) => records,
            Err(error) => {
                return TerminalSnapshot::unavailable(
                    Some(request.project_id),
                    map_project_error(error),
                );
            }
        };
        let title = match self.begin_start(&records) {
            Ok(title) => title,
            Err(code) => {
                return TerminalSnapshot::unavailable(Some(request.project_id), code);
            }
        };
        if let Err(error) = projects.reserve_terminal(&request.project_id) {
            self.finish_start(&title);
            return TerminalSnapshot::unavailable(
                Some(request.project_id),
                map_project_error(error),
            );
        }
        let cwd = match projects.execution_cwd(&request.project_id) {
            Ok(cwd) => cwd,
            Err(error) => {
                projects.release_terminal(&request.project_id);
                self.finish_start(&title);
                return TerminalSnapshot::unavailable(
                    Some(request.project_id),
                    map_project_error(error),
                );
            }
        };

        let terminal_id = Uuid::now_v7().to_string();
        let spawn_id = terminal_id.clone();
        let project_id = request.project_id.clone();
        let spawn_title = title.clone();
        let columns = request.columns;
        let rows = request.rows;
        let spawned = tokio::task::spawn_blocking(move || {
            TerminalSession::spawn(spawn_id, project_id, spawn_title, cwd, columns, rows)
        })
        .await;
        let session = match spawned {
            Ok(Ok(session)) => Arc::new(session),
            Ok(Err(error)) => {
                projects.release_terminal(&request.project_id);
                self.finish_start(&title);
                return TerminalSnapshot::unavailable(
                    Some(request.project_id),
                    match error {
                        TerminalSpawnError::Pty => TerminalDiagnosticCode::PtyUnavailable,
                        TerminalSpawnError::Shell => TerminalDiagnosticCode::ShellUnavailable,
                    },
                );
            }
            Err(_) => {
                projects.release_terminal(&request.project_id);
                self.finish_start(&title);
                return TerminalSnapshot::unavailable(
                    Some(request.project_id),
                    TerminalDiagnosticCode::PtyUnavailable,
                );
            }
        };
        if projects
            .record_terminal_start(
                &terminal_id,
                &request.project_id,
                &title,
                request.columns,
                request.rows,
            )
            .is_err()
        {
            let cleanup = Arc::clone(&session);
            let _ = tokio::task::spawn_blocking(move || cleanup.close()).await;
            projects.release_terminal(&request.project_id);
            self.finish_start(&title);
            return TerminalSnapshot::unavailable(
                Some(request.project_id),
                TerminalDiagnosticCode::MetadataUnavailable,
            );
        }
        let inserted = self
            .state
            .lock()
            .map(|mut state| {
                state.pending_starts = state.pending_starts.saturating_sub(1);
                state.pending_titles.remove(&title);
                state.sessions.insert(terminal_id, Arc::clone(&session));
            })
            .is_ok();
        if !inserted {
            let cleanup = Arc::clone(&session);
            let _ = tokio::task::spawn_blocking(move || cleanup.close()).await;
            projects.release_terminal(&request.project_id);
            let _ = projects.remove_terminal_record(&session.terminal_id);
            return TerminalSnapshot::unavailable(
                Some(request.project_id),
                TerminalDiagnosticCode::MetadataUnavailable,
            );
        }
        session.snapshot(0, false, None)
    }

    pub async fn poll(
        &self,
        request: TerminalPollRequest,
        projects: &ProjectService,
    ) -> TerminalSnapshot {
        let Some(session) = self.session(&request.terminal_id) else {
            return TerminalSnapshot::unavailable(None, TerminalDiagnosticCode::TerminalNotFound);
        };
        let metadata_failed = refresh_and_record(&session, projects).is_err();
        let diagnostic = metadata_failed.then_some(TerminalDiagnosticCode::MetadataUnavailable);
        session.snapshot(request.after_sequence, true, diagnostic)
    }

    pub async fn write(
        &self,
        request: TerminalWriteRequest,
        projects: &ProjectService,
    ) -> TerminalSnapshot {
        let Some(session) = self.session(&request.terminal_id) else {
            return TerminalSnapshot::unavailable(None, TerminalDiagnosticCode::TerminalNotFound);
        };
        let input = match decode_terminal_input(&request.data_base64) {
            Ok(input) => input,
            Err(()) => {
                return session.snapshot(0, false, Some(TerminalDiagnosticCode::InputTooLarge));
            }
        };
        refresh_and_record(&session, projects).ok();
        let diagnostic = session
            .write(&input)
            .err()
            .map(|_| TerminalDiagnosticCode::InputUnavailable);
        session.snapshot(0, false, diagnostic)
    }

    pub async fn resize(
        &self,
        request: TerminalResizeRequest,
        projects: &ProjectService,
    ) -> TerminalSnapshot {
        let Some(session) = self.session(&request.terminal_id) else {
            return TerminalSnapshot::unavailable(None, TerminalDiagnosticCode::TerminalNotFound);
        };
        if !valid_dimensions(request.columns, request.rows) {
            return session.snapshot(0, false, Some(TerminalDiagnosticCode::InvalidRequest));
        }
        refresh_and_record(&session, projects).ok();
        let diagnostic = match session.resize(request.columns, request.rows) {
            Ok(()) => projects
                .record_terminal_state(
                    &session.terminal_id,
                    session.state().storage_value().unwrap_or("failed"),
                    request.columns,
                    request.rows,
                    session.exit_code(),
                )
                .err()
                .map(|_| TerminalDiagnosticCode::MetadataUnavailable),
            Err(()) => Some(TerminalDiagnosticCode::ResizeUnavailable),
        };
        session.snapshot(0, false, diagnostic)
    }

    pub async fn close(
        &self,
        terminal_id: String,
        projects: &ProjectService,
    ) -> TerminalRegistrySnapshot {
        if !valid_terminal_id(&terminal_id) {
            return TerminalRegistrySnapshot::unavailable(TerminalDiagnosticCode::TerminalNotFound);
        }
        let session = self.session(&terminal_id);
        if session.is_none() {
            match projects.terminal_records() {
                Ok(records) if !records.iter().any(|record| record.id == terminal_id) => {
                    return TerminalRegistrySnapshot::unavailable(
                        TerminalDiagnosticCode::TerminalNotFound,
                    );
                }
                Err(_) => {
                    return TerminalRegistrySnapshot::unavailable(
                        TerminalDiagnosticCode::MetadataUnavailable,
                    );
                }
                Ok(_) => {}
            }
        }
        if let Some(session) = session {
            session.mark_closing();
            let dimensions = session.dimensions();
            let _ = projects.record_terminal_state(
                &terminal_id,
                "closing",
                dimensions.0,
                dimensions.1,
                session.exit_code(),
            );
            let cleanup = Arc::clone(&session);
            let cleaned = tokio::task::spawn_blocking(move || cleanup.close())
                .await
                .unwrap_or(false);
            if !cleaned {
                let dimensions = session.dimensions();
                let _ = projects.record_terminal_state(
                    &terminal_id,
                    "failed",
                    dimensions.0,
                    dimensions.1,
                    session.exit_code(),
                );
                let mut registry = self.registry_snapshot(projects);
                if let Some(terminal) = registry
                    .terminals
                    .iter_mut()
                    .find(|terminal| terminal.terminal_id.as_deref() == Some(&terminal_id))
                {
                    terminal.diagnostic_code = Some(TerminalDiagnosticCode::CleanupIncomplete);
                }
                return registry;
            }
            let removed = self
                .state
                .lock()
                .map(|mut state| {
                    state
                        .sessions
                        .get(&terminal_id)
                        .is_some_and(|registered| Arc::ptr_eq(registered, &session))
                        .then(|| state.sessions.remove(&terminal_id))
                        .flatten()
                        .is_some()
                })
                .unwrap_or(false);
            if removed {
                projects.release_terminal(&session.project_id);
            }
        }
        if projects.remove_terminal_record(&terminal_id).is_err() {
            return TerminalRegistrySnapshot::unavailable(
                TerminalDiagnosticCode::MetadataUnavailable,
            );
        }
        self.registry_snapshot(projects)
    }

    fn begin_start(
        &self,
        records: &[StoredTerminalSession],
    ) -> Result<String, TerminalDiagnosticCode> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| TerminalDiagnosticCode::MetadataUnavailable)?;
        let record_ids: HashSet<&str> = records.iter().map(|record| record.id.as_str()).collect();
        let unrecorded_live = state
            .sessions
            .keys()
            .filter(|terminal_id| !record_ids.contains(terminal_id.as_str()))
            .count();
        if records.len() + unrecorded_live + state.pending_starts >= TERMINAL_CAPACITY {
            return Err(TerminalDiagnosticCode::CapacityReached);
        }
        state.pending_starts += 1;
        let mut titles: HashSet<&str> =
            records.iter().map(|record| record.title.as_str()).collect();
        titles.extend(
            state
                .sessions
                .values()
                .map(|session| session.title.as_str()),
        );
        let number = (1..=TERMINAL_CAPACITY)
            .find(|number| {
                let title = format!("Terminal {number}");
                !titles.contains(title.as_str()) && !state.pending_titles.contains(&title)
            })
            .unwrap_or(records.len() + 1);
        let title = format!("Terminal {number}");
        state.pending_titles.insert(title.clone());
        Ok(title)
    }

    fn finish_start(&self, title: &str) {
        if let Ok(mut state) = self.state.lock() {
            state.pending_starts = state.pending_starts.saturating_sub(1);
            state.pending_titles.remove(title);
        }
    }

    fn session(&self, terminal_id: &str) -> Option<Arc<TerminalSession>> {
        if !valid_terminal_id(terminal_id) {
            return None;
        }
        self.state
            .lock()
            .ok()
            .and_then(|state| state.sessions.get(terminal_id).cloned())
    }

    fn live_sessions(&self) -> Result<Vec<Arc<TerminalSession>>, TerminalDiagnosticCode> {
        self.state
            .lock()
            .map(|state| state.sessions.values().cloned().collect())
            .map_err(|_| TerminalDiagnosticCode::MetadataUnavailable)
    }

    fn registry_snapshot(&self, projects: &ProjectService) -> TerminalRegistrySnapshot {
        let records = match projects.terminal_records() {
            Ok(records) => records,
            Err(_) => {
                return TerminalRegistrySnapshot::unavailable(
                    TerminalDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let live = match self.state.lock() {
            Ok(state) => state.sessions.clone(),
            Err(_) => {
                return TerminalRegistrySnapshot::unavailable(
                    TerminalDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let terminals = records
            .into_iter()
            .map(|record| {
                live.get(&record.id).map_or_else(
                    || snapshot_from_record(record),
                    |session| session.snapshot(0, false, None),
                )
            })
            .collect();
        TerminalRegistrySnapshot {
            schema_version: TERMINAL_REGISTRY_SCHEMA_VERSION,
            capacity: TERMINAL_CAPACITY as u8,
            terminals,
            diagnostic_code: None,
        }
    }
}

impl Drop for TerminalService {
    fn drop(&mut self) {
        let sessions = self
            .state
            .lock()
            .map(|mut state| state.sessions.drain().map(|(_, session)| session).collect())
            .unwrap_or_else(|_| Vec::new());
        for session in sessions {
            session.close();
        }
    }
}

impl TerminalSession {
    fn spawn(
        terminal_id: String,
        project_id: String,
        title: String,
        cwd: PathBuf,
        columns: u16,
        rows: u16,
    ) -> Result<Self, TerminalSpawnError> {
        let command = terminal_command();
        Self::spawn_with_command(terminal_id, project_id, title, cwd, columns, rows, command)
    }

    fn spawn_with_command(
        terminal_id: String,
        project_id: String,
        title: String,
        cwd: PathBuf,
        columns: u16,
        rows: u16,
        mut command: CommandBuilder,
    ) -> Result<Self, TerminalSpawnError> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols: columns,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| TerminalSpawnError::Pty)?;
        command.cwd(&cwd);
        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|_| TerminalSpawnError::Shell)?;
        drop(pair.slave);
        let Some(session_id) = child.process_id().and_then(|pid| i32::try_from(pid).ok()) else {
            let _ = child.kill();
            let _ = child.wait();
            return Err(TerminalSpawnError::Shell);
        };
        let leader_start_time = match wait_for_process_identity(session_id) {
            Some((session, start_time)) if session == session_id => start_time,
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(TerminalSpawnError::Shell);
            }
        };
        let reader_fd = match pair.master.as_raw_fd() {
            Some(fd) => fd,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(TerminalSpawnError::Pty);
            }
        };
        let reader = match pair.master.try_clone_reader() {
            Ok(reader) => reader,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(TerminalSpawnError::Pty);
            }
        };
        let writer = match pair.master.take_writer() {
            Ok(writer) => writer,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(TerminalSpawnError::Pty);
            }
        };
        let output = Arc::new(Mutex::new(OutputStore::default()));
        let reader_stop = Arc::new(AtomicBool::new(false));
        let reader_failed = Arc::new(AtomicBool::new(false));
        let reader_output = Arc::clone(&output);
        let stop = Arc::clone(&reader_stop);
        let failed = Arc::clone(&reader_failed);
        let reader_thread = match thread::Builder::new()
            .name("quireforge-terminal-reader".to_owned())
            .spawn(move || read_terminal_output(reader, reader_fd, reader_output, stop, failed))
        {
            Ok(reader_thread) => reader_thread,
            Err(_) => {
                let _ = terminate_owned_session(session_id, leader_start_time, child.as_mut());
                return Err(TerminalSpawnError::Pty);
            }
        };

        Ok(Self {
            terminal_id,
            project_id,
            title,
            runtime: Mutex::new(TerminalRuntime {
                state: TerminalState::Running,
                columns,
                rows,
                exit_code: None,
                master: Some(pair.master),
                writer: Some(writer),
                child: Some(child),
                session_id,
                leader_start_time,
            }),
            output,
            reader_stop,
            reader_failed,
            reader_thread: Mutex::new(Some(reader_thread)),
        })
    }

    fn refresh(&self) -> Result<bool, ()> {
        let mut runtime = self.runtime.lock().map_err(|_| ())?;
        if runtime.state != TerminalState::Running {
            return Ok(false);
        }
        let Some(child) = runtime.child.as_mut() else {
            runtime.state = TerminalState::Failed;
            return Ok(true);
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                runtime.exit_code = Some(status.exit_code().min(i32::MAX as u32) as i32);
                runtime.state = TerminalState::Exited;
                Ok(true)
            }
            Ok(None) => Ok(false),
            Err(_) => {
                runtime.state = TerminalState::Failed;
                Ok(true)
            }
        }
    }

    fn write(&self, input: &[u8]) -> Result<(), ()> {
        let mut runtime = self.runtime.lock().map_err(|_| ())?;
        if runtime.state != TerminalState::Running {
            return Err(());
        }
        let writer = runtime.writer.as_mut().ok_or(())?;
        writer
            .write_all(input)
            .and_then(|_| writer.flush())
            .map_err(|_| ())
    }

    fn resize(&self, columns: u16, rows: u16) -> Result<(), ()> {
        let mut runtime = self.runtime.lock().map_err(|_| ())?;
        if !matches!(
            runtime.state,
            TerminalState::Running | TerminalState::Exited
        ) {
            return Err(());
        }
        runtime
            .master
            .as_ref()
            .ok_or(())?
            .resize(PtySize {
                rows,
                cols: columns,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| ())?;
        runtime.columns = columns;
        runtime.rows = rows;
        Ok(())
    }

    fn close(&self) -> bool {
        let cleaned = {
            let mut runtime = match self.runtime.lock() {
                Ok(runtime) => runtime,
                Err(_) => return false,
            };
            runtime.state = TerminalState::Closing;
            let session_id = runtime.session_id;
            let leader_start_time = runtime.leader_start_time;
            let child_cleaned = runtime.child.as_mut().is_some_and(|child| {
                terminate_owned_session(session_id, leader_start_time, child.as_mut())
            });
            if child_cleaned {
                runtime.child.take();
                runtime.writer.take();
            }
            child_cleaned
        };
        if !cleaned {
            if let Ok(mut runtime) = self.runtime.lock() {
                runtime.state = TerminalState::Failed;
            }
            return false;
        }
        self.reader_stop.store(true, Ordering::Release);
        if let Ok(mut reader_thread) = self.reader_thread.lock() {
            if let Some(reader_thread) = reader_thread.take() {
                let _ = reader_thread.join();
            }
        }
        if let Ok(mut runtime) = self.runtime.lock() {
            runtime.master.take();
            runtime.state = TerminalState::Interrupted;
        }
        true
    }

    fn mark_closing(&self) {
        if let Ok(mut runtime) = self.runtime.lock() {
            runtime.state = TerminalState::Closing;
        }
    }

    fn state(&self) -> TerminalState {
        self.runtime
            .lock()
            .map(|runtime| runtime.state)
            .unwrap_or(TerminalState::Failed)
    }

    fn dimensions(&self) -> (u16, u16) {
        self.runtime
            .lock()
            .map(|runtime| (runtime.columns, runtime.rows))
            .unwrap_or((0, 0))
    }

    fn exit_code(&self) -> Option<i32> {
        self.runtime
            .lock()
            .ok()
            .and_then(|runtime| runtime.exit_code)
    }

    fn snapshot(
        &self,
        after_sequence: u64,
        include_output: bool,
        diagnostic_code: Option<TerminalDiagnosticCode>,
    ) -> TerminalSnapshot {
        let (state, columns, rows, exit_code) = self
            .runtime
            .lock()
            .map(|runtime| {
                (
                    runtime.state,
                    runtime.columns,
                    runtime.rows,
                    runtime.exit_code,
                )
            })
            .unwrap_or((TerminalState::Failed, 0, 0, None));
        let output = self
            .output
            .lock()
            .map(|output| output.window(after_sequence, include_output))
            .unwrap_or_default();
        let diagnostic_code = diagnostic_code.or_else(|| {
            self.reader_failed
                .load(Ordering::Acquire)
                .then_some(TerminalDiagnosticCode::OutputUnavailable)
        });
        TerminalSnapshot {
            schema_version: TERMINAL_SCHEMA_VERSION,
            state,
            terminal_id: Some(self.terminal_id.clone()),
            project_id: Some(self.project_id.clone()),
            title: Some(self.title.clone()),
            live: true,
            columns,
            rows,
            output: output.chunks,
            first_sequence: output.first_sequence,
            last_sequence: output.last_sequence,
            truncated: output.truncated,
            has_more: output.has_more,
            exit_code,
            diagnostic_code,
        }
    }
}

impl OutputStore {
    fn push(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.retained_bytes = self.retained_bytes.saturating_add(data.len());
        self.chunks.push_back(StoredOutputChunk {
            sequence: self.next_sequence,
            data: data.to_vec(),
        });
        while self.retained_bytes > MAX_RETAINED_OUTPUT_BYTES
            || self.chunks.len() > MAX_OUTPUT_CHUNKS
        {
            let Some(dropped) = self.chunks.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(dropped.data.len());
            self.dropped_through = dropped.sequence;
        }
    }

    fn window(&self, after_sequence: u64, include_output: bool) -> OutputWindow {
        let first_sequence = self.chunks.front().map(|chunk| chunk.sequence).unwrap_or(0);
        let truncated = after_sequence < self.dropped_through;
        let effective_after = after_sequence.max(self.dropped_through);
        if !include_output {
            return OutputWindow {
                chunks: Vec::new(),
                first_sequence,
                last_sequence: self.next_sequence,
                truncated,
                has_more: false,
            };
        }
        let mut bytes = 0usize;
        let chunks: Vec<_> = self
            .chunks
            .iter()
            .filter(|chunk| chunk.sequence > effective_after)
            .take_while(|chunk| {
                let within_limit = bytes.saturating_add(chunk.data.len()) <= MAX_POLL_OUTPUT_BYTES;
                if within_limit {
                    bytes += chunk.data.len();
                }
                within_limit
            })
            .take(MAX_POLL_CHUNKS)
            .map(|chunk| TerminalOutputChunk {
                sequence: chunk.sequence,
                data_base64: BASE64.encode(&chunk.data),
            })
            .collect();
        let returned_through = chunks
            .last()
            .map(|chunk| chunk.sequence)
            .unwrap_or(effective_after);
        OutputWindow {
            chunks,
            first_sequence,
            last_sequence: self.next_sequence,
            truncated,
            has_more: self.next_sequence > returned_through,
        }
    }
}

fn terminal_command() -> CommandBuilder {
    let mut command = CommandBuilder::new_default_prog();
    command.env_clear();
    for name in INHERITED_ENVIRONMENT {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command.env("PATH", SYSTEM_PATH);
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");
    command.env("TERM_PROGRAM", "QuireForge");
    command.env("TERM_PROGRAM_VERSION", env!("CARGO_PKG_VERSION"));
    command
}

fn read_terminal_output(
    mut reader: Box<dyn Read + Send>,
    reader_fd: i32,
    output: Arc<Mutex<OutputStore>>,
    stop: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
) {
    let mut buffer = vec![0_u8; READER_BUFFER_BYTES];
    while !stop.load(Ordering::Acquire) {
        let mut descriptor = libc::pollfd {
            fd: reader_fd,
            events: libc::POLLIN | libc::POLLHUP | libc::POLLERR,
            revents: 0,
        };
        let polled = unsafe { libc::poll(&mut descriptor, 1, READER_POLL_MILLIS) };
        if polled == 0 {
            continue;
        }
        if polled < 0 {
            if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                continue;
            }
            failed.store(true, Ordering::Release);
            break;
        }
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                if let Ok(mut output) = output.lock() {
                    output.push(&buffer[..read]);
                } else {
                    failed.store(true, Ordering::Release);
                    break;
                }
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => {
                failed.store(true, Ordering::Release);
                break;
            }
        }
    }
}

fn refresh_and_record(
    session: &TerminalSession,
    projects: &ProjectService,
) -> Result<(), ProjectExecutionError> {
    if !session.refresh().unwrap_or(false) {
        return Ok(());
    }
    let (columns, rows) = session.dimensions();
    projects.record_terminal_state(
        &session.terminal_id,
        session.state().storage_value().unwrap_or("failed"),
        columns,
        rows,
        session.exit_code(),
    )
}

fn snapshot_from_record(record: StoredTerminalSession) -> TerminalSnapshot {
    TerminalSnapshot {
        schema_version: TERMINAL_SCHEMA_VERSION,
        state: TerminalState::from_storage_value(&record.status).unwrap_or(TerminalState::Failed),
        terminal_id: Some(record.id),
        project_id: Some(record.project_id),
        title: Some(record.title),
        live: false,
        columns: record.columns,
        rows: record.rows,
        output: Vec::new(),
        first_sequence: 0,
        last_sequence: 0,
        truncated: false,
        has_more: false,
        exit_code: record.exit_code,
        diagnostic_code: None,
    }
}

fn valid_dimensions(columns: u16, rows: u16) -> bool {
    (MIN_COLUMNS..=MAX_COLUMNS).contains(&columns) && (MIN_ROWS..=MAX_ROWS).contains(&rows)
}

fn valid_terminal_id(value: &str) -> bool {
    value.len() == 36
        && Uuid::parse_str(value)
            .is_ok_and(|uuid| uuid.get_version_num() == 7 && uuid.to_string() == value)
}

fn decode_terminal_input(value: &str) -> Result<Vec<u8>, ()> {
    if value.is_empty() || value.len() > MAX_INPUT_BASE64_BYTES {
        return Err(());
    }
    BASE64
        .decode(value.as_bytes())
        .ok()
        .filter(|input| !input.is_empty() && input.len() <= MAX_INPUT_BYTES)
        .ok_or(())
}

fn map_project_error(error: ProjectExecutionError) -> TerminalDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId | ProjectExecutionError::ProjectNotFound => {
            TerminalDiagnosticCode::ProjectUnavailable
        }
        ProjectExecutionError::MetadataUnavailable => TerminalDiagnosticCode::MetadataUnavailable,
        ProjectExecutionError::DirectoryUnavailable | ProjectExecutionError::NotRepository => {
            TerminalDiagnosticCode::ProjectUnavailable
        }
        ProjectExecutionError::IdentityChanged => TerminalDiagnosticCode::ProjectIdentityChanged,
        ProjectExecutionError::NotWritable => TerminalDiagnosticCode::ProjectNotWritable,
        ProjectExecutionError::ProjectBusy => TerminalDiagnosticCode::ProjectBusy,
    }
}

fn wait_for_process_identity(pid: i32) -> Option<(i32, u64)> {
    let deadline = Instant::now() + Duration::from_millis(100);
    loop {
        if let Some(identity) = process_identity(pid) {
            return Some(identity);
        }
        if Instant::now() >= deadline {
            return None;
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn process_identity(pid: i32) -> Option<(i32, u64)> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    parse_process_stat(&stat)
}

fn parse_process_stat(stat: &str) -> Option<(i32, u64)> {
    let command_end = stat.rfind(')')?;
    let fields: Vec<_> = stat.get(command_end + 1..)?.split_whitespace().collect();
    let session_id = fields.get(3)?.parse().ok()?;
    let start_time = fields.get(19)?.parse().ok()?;
    Some((session_id, start_time))
}

fn owned_session_members(session_id: i32, leader_start_time: u64) -> Result<Vec<i32>, ()> {
    if let Some((leader_session, current_start_time)) = process_identity(session_id) {
        if leader_session != session_id || current_start_time != leader_start_time {
            return Err(());
        }
    }
    let mut members = Vec::new();
    for entry in fs::read_dir("/proc").map_err(|_| ())? {
        let Ok(entry) = entry else {
            continue;
        };
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.parse::<i32>().ok())
        else {
            continue;
        };
        if pid == std::process::id() as i32 {
            continue;
        }
        if process_identity(pid)
            .is_some_and(|(candidate_session, _)| candidate_session == session_id)
        {
            members.push(pid);
        }
    }
    members.sort_unstable_by_key(|pid| *pid == session_id);
    Ok(members)
}

fn signal_owned_session(session_id: i32, leader_start_time: u64, signal: i32) -> Result<(), ()> {
    for pid in owned_session_members(session_id, leader_start_time)? {
        let result = unsafe { libc::kill(pid, signal) };
        if result != 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            return Err(());
        }
    }
    Ok(())
}

fn wait_for_cleanup(
    session_id: i32,
    leader_start_time: u64,
    child: &mut (dyn Child + Send + Sync),
) -> bool {
    let deadline = Instant::now() + PROCESS_SETTLE_TIMEOUT;
    loop {
        let child_done = child.try_wait().ok().flatten().is_some();
        let members_empty = owned_session_members(session_id, leader_start_time)
            .map(|members| members.is_empty())
            .unwrap_or(false);
        if child_done && members_empty {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(PROCESS_POLL_INTERVAL);
    }
}

fn terminate_owned_session(
    session_id: i32,
    leader_start_time: u64,
    child: &mut (dyn Child + Send + Sync),
) -> bool {
    let _ = signal_owned_session(session_id, leader_start_time, libc::SIGHUP);
    if wait_for_cleanup(session_id, leader_start_time, child) {
        return true;
    }
    let _ = signal_owned_session(session_id, leader_start_time, libc::SIGTERM);
    if wait_for_cleanup(session_id, leader_start_time, child) {
        return true;
    }
    let _ = signal_owned_session(session_id, leader_start_time, libc::SIGKILL);
    let _ = child.kill();
    wait_for_cleanup(session_id, leader_start_time, child)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::Path,
        thread,
        time::{Duration, Instant},
    };

    use super::{
        decode_terminal_input, parse_process_stat,
        types::{
            TerminalDiagnosticCode, TerminalPollRequest, TerminalStartRequest, TerminalState,
            TerminalWriteRequest,
        },
        OutputStore, TerminalService, TerminalSession, INHERITED_ENVIRONMENT,
        MAX_INPUT_BASE64_BYTES, MAX_RETAINED_OUTPUT_BYTES, SYSTEM_PATH,
    };
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use portable_pty::CommandBuilder;
    use uuid::Uuid;

    use crate::project::ProjectService;

    #[test]
    fn output_window_preserves_bytes_and_reports_a_slow_consumer() {
        let mut output = OutputStore::default();
        output.push(&[0xff, 0x00, b'A']);
        let current = output.window(0, true);
        assert_eq!(current.chunks.len(), 1);
        assert_eq!(
            BASE64.decode(&current.chunks[0].data_base64).unwrap(),
            [0xff, 0x00, b'A']
        );
        output.push(&vec![b'x'; MAX_RETAINED_OUTPUT_BYTES]);
        let truncated = output.window(0, true);
        assert!(truncated.truncated);
        assert!(truncated.first_sequence > 1);
    }

    #[test]
    fn parses_proc_stat_with_spaces_and_parentheses_in_the_process_name() {
        let mut fields = vec!["S", "1", "42", "42"];
        fields.extend(std::iter::repeat_n("0", 15));
        fields.push("98765");
        assert_eq!(
            parse_process_stat(&format!("42 (shell name)) {}", fields.join(" "))),
            Some((42, 98765))
        );
    }

    #[test]
    fn terminal_environment_is_an_explicit_noncredential_allowlist() {
        for required in ["HOME", "LANG", "XDG_RUNTIME_DIR", "WAYLAND_DISPLAY"] {
            assert!(INHERITED_ENVIRONMENT.contains(&required));
        }
        for forbidden in [
            "OPENAI_API_KEY",
            "GITHUB_TOKEN",
            "AWS_PROFILE",
            "SSH_AUTH_SOCK",
            "GNUPGHOME",
            "CODEX_HOME",
        ] {
            assert!(!INHERITED_ENVIRONMENT.contains(&forbidden));
        }
    }

    #[test]
    fn terminal_input_is_bounded_before_base64_decoding() {
        assert_eq!(decode_terminal_input("/wA=").unwrap(), [0xff, 0x00]);
        assert!(decode_terminal_input("").is_err());
        assert!(decode_terminal_input("not-base64").is_err());
        assert!(decode_terminal_input(&"A".repeat(MAX_INPUT_BASE64_BYTES + 1)).is_err());
    }

    #[test]
    fn concurrent_provisional_starts_receive_distinct_titles() {
        let service = TerminalService::default();
        let first = service.begin_start(&[]).expect("first start must reserve");
        let second = service.begin_start(&[]).expect("second start must reserve");

        assert_eq!(first, "Terminal 1");
        assert_eq!(second, "Terminal 2");
        service.finish_start(&first);
        service.finish_start(&second);
    }

    #[tokio::test]
    async fn close_rejects_an_unknown_app_owned_identifier() {
        let service = TerminalService::default();
        let projects = ProjectService::in_memory();

        let result = service.close(Uuid::now_v7().to_string(), &projects).await;

        assert_eq!(
            result.diagnostic_code,
            Some(TerminalDiagnosticCode::TerminalNotFound)
        );
        assert!(result.terminals.is_empty());
    }

    #[tokio::test]
    async fn service_starts_in_a_revalidated_project_and_removes_its_metadata() {
        let directory =
            std::env::temp_dir().join(format!("quireforge-terminal-service-{}", Uuid::now_v7()));
        fs::create_dir_all(&directory).expect("terminal project fixture must exist");
        let projects = ProjectService::in_memory();
        projects.prepare_attachment(directory.clone());
        let attached = projects.confirm_pending();
        let project_id = attached.projects[0].id.clone();
        let service = TerminalService::default();

        let started = service
            .start(
                TerminalStartRequest {
                    project_id: project_id.clone(),
                    columns: 80,
                    rows: 24,
                },
                &projects,
            )
            .await;
        let terminal_id = started
            .terminal_id
            .clone()
            .expect("terminal service must return its app-owned ID");
        assert_eq!(started.project_id, Some(project_id));
        assert_eq!(started.state, TerminalState::Running);
        assert!(started.live);
        assert!(started.output.is_empty());

        let status = service.status(&projects).await;
        assert_eq!(status.terminals.len(), 1);
        assert!(status.terminals[0].output.is_empty());
        assert!(!serde_json::to_string(&status)
            .expect("terminal status must serialize")
            .contains(directory.to_string_lossy().as_ref()));

        service
            .write(
                TerminalWriteRequest {
                    terminal_id: terminal_id.clone(),
                    data_base64: BASE64.encode(b"pwd\r"),
                },
                &projects,
            )
            .await;
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut cursor = 0;
        let mut output = Vec::new();
        loop {
            let snapshot = service
                .poll(
                    TerminalPollRequest {
                        terminal_id: terminal_id.clone(),
                        after_sequence: cursor,
                    },
                    &projects,
                )
                .await;
            for chunk in snapshot.output {
                cursor = chunk.sequence;
                output.extend(
                    BASE64
                        .decode(chunk.data_base64)
                        .expect("terminal output must be base64"),
                );
            }
            if String::from_utf8_lossy(&output).contains(directory.to_string_lossy().as_ref()) {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "terminal output must prove the verified cwd: {:?}",
                String::from_utf8_lossy(&output)
            );
            thread::sleep(Duration::from_millis(10));
        }

        let closed = service.close(terminal_id, &projects).await;
        assert!(closed.terminals.is_empty());
        assert!(projects.terminal_records().unwrap().is_empty());
        fs::remove_dir_all(directory).expect("terminal project fixture must be removed");
    }

    #[test]
    fn closes_an_already_exited_and_refreshed_shell() {
        let directory =
            std::env::temp_dir().join(format!("quireforge-terminal-exit-{}", Uuid::now_v7()));
        fs::create_dir_all(&directory).expect("terminal fixture must exist");
        let mut command = CommandBuilder::new("/bin/sh");
        command.env_clear();
        command.env("PATH", SYSTEM_PATH);
        command.env("TERM", "xterm-256color");
        let session = TerminalSession::spawn_with_command(
            Uuid::now_v7().to_string(),
            Uuid::now_v7().to_string(),
            "Terminal 1".to_owned(),
            directory.clone(),
            80,
            24,
            command,
        )
        .expect("PTY fixture must start");

        session.write(b"exit\r").expect("shell must accept exit");
        let deadline = Instant::now() + Duration::from_secs(2);
        while !session
            .refresh()
            .expect("shell status must remain available")
        {
            assert!(Instant::now() < deadline, "shell must exit promptly");
            thread::sleep(Duration::from_millis(10));
        }

        assert_eq!(session.state(), TerminalState::Exited);
        assert!(session.close());
        fs::remove_dir_all(directory).expect("terminal fixture must be removed");
    }

    #[test]
    fn owns_byte_output_and_reaps_a_background_job_on_close() {
        let directory =
            std::env::temp_dir().join(format!("quireforge-terminal-process-{}", Uuid::now_v7()));
        fs::create_dir_all(&directory).expect("terminal fixture must exist");
        let mut command = CommandBuilder::new("/bin/sh");
        command.env_clear();
        command.env("PATH", SYSTEM_PATH);
        command.env("TERM", "xterm-256color");
        let session = TerminalSession::spawn_with_command(
            Uuid::now_v7().to_string(),
            Uuid::now_v7().to_string(),
            "Terminal 1".to_owned(),
            directory.clone(),
            80,
            24,
            command,
        )
        .expect("PTY fixture must start");
        session
            .write(b"sleep 30 & printf 'QF_BG:%s\\n' \"$!\"\r")
            .expect("terminal input must be written");

        let mut output = Vec::new();
        let mut cursor = 0;
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        let background_pid = loop {
            let snapshot = session.snapshot(cursor, true, None);
            for chunk in snapshot.output {
                cursor = chunk.sequence;
                output.extend(
                    BASE64
                        .decode(chunk.data_base64)
                        .expect("terminal output must be base64"),
                );
            }
            let text = String::from_utf8_lossy(&output);
            let parsed = text.match_indices("QF_BG:").find_map(|(offset, _)| {
                text.get(offset + 6..)?
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .parse::<i32>()
                    .ok()
            });
            if let Some(pid) = parsed {
                break pid;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "background PID must appear in bounded output: {text:?}"
            );
            thread::sleep(Duration::from_millis(10));
        };

        assert!(Path::new(&format!("/proc/{background_pid}")).exists());
        assert!(session.close());
        assert!(!Path::new(&format!("/proc/{background_pid}")).exists());
        fs::remove_dir_all(directory).expect("terminal fixture must be removed");
    }
}
