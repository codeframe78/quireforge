pub mod types;

use std::{path::Path, time::Duration};

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::project::{ConversationReference, ProjectExecutionError, ProjectService};

use super::{
    app_server::{
        validate_uuid_v7, AppServerCommand, AppServerNotification, AppServerProcess,
        ConversationErrorCode as WireErrorCode, ConversationItemKind as WireItemKind,
        ConversationItemStatus as WireItemStatus, ConversationNotification,
        ConversationPlanStepStatus as WirePlanStepStatus, ConversationTurnStatus as WireTurnStatus,
    },
    error::CodexAdapterError,
};
use types::{
    ConversationActivityKind, ConversationActivityStatus, ConversationApprovalPolicy,
    ConversationDiagnosticCode, ConversationEvent, ConversationLifecyclePhase,
    ConversationPlanStep, ConversationPlanStepStatus, ConversationSandboxMode,
    ConversationSnapshot, ConversationStartRequest, ConversationState, ConversationStreamErrorCode,
    CONVERSATION_SCHEMA_VERSION,
};

const MAX_PROMPT_BYTES: usize = 64 * 1024;
const MAX_POLL_EVENTS: usize = 32;
const FIRST_POLL_WAIT: Duration = Duration::from_millis(200);
const DRAIN_POLL_WAIT: Duration = Duration::from_millis(1);

pub struct ConversationService {
    state: Mutex<ConversationServiceState>,
    command: AppServerCommand,
}

struct ConversationServiceState {
    active: Option<ActiveConversation>,
    last: ConversationSnapshot,
}

struct ActiveConversation {
    conversation_id: String,
    project_id: String,
    model_id: String,
    reasoning_effort: String,
    sandbox_mode: ConversationSandboxMode,
    approval_policy: ConversationApprovalPolicy,
    thread_id: String,
    turn_id: String,
    state: ConversationState,
    next_sequence: u64,
    process: AppServerProcess,
}

struct StartedConversation {
    conversation_id: String,
    thread_id: String,
    turn_id: String,
}

#[derive(Clone, Copy)]
struct TerminalState {
    state: ConversationState,
    phase: ConversationLifecyclePhase,
    storage_status: &'static str,
    diagnostic_code: Option<ConversationDiagnosticCode>,
}

impl Default for ConversationService {
    fn default() -> Self {
        Self {
            state: Mutex::new(ConversationServiceState {
                active: None,
                last: ConversationSnapshot::empty(),
            }),
            command: AppServerCommand::codex("codex"),
        }
    }
}

impl ConversationService {
    #[cfg(test)]
    fn with_command(command: AppServerCommand) -> Self {
        Self {
            state: Mutex::new(ConversationServiceState {
                active: None,
                last: ConversationSnapshot::empty(),
            }),
            command,
        }
    }

    pub async fn status(&self) -> ConversationSnapshot {
        let state = self.state.lock().await;
        state
            .active
            .as_ref()
            .map(|active| active.snapshot(Vec::new(), None))
            .unwrap_or_else(|| state.last.clone())
    }

    pub async fn start(
        &self,
        request: ConversationStartRequest,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        if let Err(code) = validate_start_request(&request) {
            return ConversationSnapshot::unavailable(code);
        }

        let mut state = self.state.lock().await;
        if state.active.is_some() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationActive,
            );
        }

        if let Err(error) = projects.reserve_execution(&request.project_id) {
            return ConversationSnapshot::unavailable(map_project_error(error));
        }
        let cwd = match projects.execution_cwd(&request.project_id) {
            Ok(cwd) => cwd,
            Err(error) => {
                projects.release_execution(&request.project_id);
                return ConversationSnapshot::unavailable(map_project_error(error));
            }
        };

        match self.start_reserved(&request, &cwd, projects).await {
            Ok(active) => {
                let events = vec![
                    ConversationEvent::Lifecycle {
                        sequence: 1,
                        phase: ConversationLifecyclePhase::Starting,
                    },
                    ConversationEvent::Lifecycle {
                        sequence: 2,
                        phase: ConversationLifecyclePhase::Running,
                    },
                ];
                let snapshot = active.snapshot(events, None);
                state.active = Some(active);
                state.last = snapshot.clone();
                snapshot
            }
            Err(code) => {
                projects.release_execution(&request.project_id);
                let snapshot = ConversationSnapshot::unavailable(code);
                state.last = snapshot.clone();
                snapshot
            }
        }
    }

    async fn start_reserved(
        &self,
        request: &ConversationStartRequest,
        cwd: &Path,
        projects: &ProjectService,
    ) -> Result<ActiveConversation, ConversationDiagnosticCode> {
        let mut process = AppServerProcess::spawn(self.command.clone())
            .map_err(|error| map_adapter_error(&error))?;
        let started = match start_on_process(&mut process, request, cwd, projects).await {
            Ok(started) => started,
            Err(error) => {
                let _ = process.shutdown().await;
                return Err(error);
            }
        };

        Ok(ActiveConversation {
            conversation_id: started.conversation_id,
            project_id: request.project_id.clone(),
            model_id: request.model_id.clone(),
            reasoning_effort: request.reasoning_effort.clone(),
            sandbox_mode: request.sandbox_mode,
            approval_policy: request.approval_policy,
            thread_id: started.thread_id,
            turn_id: started.turn_id,
            state: ConversationState::Running,
            next_sequence: 3,
            process,
        })
    }

    pub async fn poll(
        &self,
        conversation_id: String,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return matching_last_or_not_found(&state.last, &conversation_id);
        };
        if active.conversation_id != conversation_id {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }

        let mut events = Vec::new();
        let mut terminal = None;
        for index in 0..MAX_POLL_EVENTS {
            let wait = if index == 0 {
                FIRST_POLL_WAIT
            } else {
                DRAIN_POLL_WAIT
            };
            match active.process.next_notification_with_timeout(wait).await {
                Ok(Some(notification)) => match apply_notification(active, notification) {
                    Ok((event, completed)) => {
                        if let Some(event) = event {
                            events.push(event);
                        }
                        if completed.is_some() {
                            terminal = completed;
                            break;
                        }
                    }
                    Err(code) => {
                        terminal = Some(protocol_terminal(code));
                        break;
                    }
                },
                Ok(None) => break,
                Err(CodexAdapterError::UnexpectedServerRequest) => {
                    terminal = Some(TerminalState {
                        state: ConversationState::Blocked,
                        phase: ConversationLifecyclePhase::Blocked,
                        storage_status: "blocked",
                        diagnostic_code: Some(ConversationDiagnosticCode::ApprovalRequired),
                    });
                    break;
                }
                Err(error) => {
                    terminal = Some(protocol_terminal(map_adapter_error(&error)));
                    break;
                }
            }
        }

        if let Some(terminal) = terminal {
            return finish_active(&mut state, projects, terminal, events).await;
        }
        let snapshot = state
            .active
            .as_ref()
            .expect("active conversation remains available")
            .snapshot(events, None);
        state.last = snapshot.clone();
        snapshot
    }

    pub async fn interrupt(
        &self,
        conversation_id: String,
        projects: &ProjectService,
    ) -> ConversationSnapshot {
        if validate_uuid_v7(&conversation_id).is_err() {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }
        let mut state = self.state.lock().await;
        let Some(active) = state.active.as_mut() else {
            return matching_last_or_not_found(&state.last, &conversation_id);
        };
        if active.conversation_id != conversation_id {
            return ConversationSnapshot::unavailable(
                ConversationDiagnosticCode::ConversationNotFound,
            );
        }
        if active.state == ConversationState::Stopping {
            return active.snapshot(Vec::new(), None);
        }

        let result = active
            .process
            .request(
                "turn/interrupt",
                json!({
                    "threadId": active.thread_id,
                    "turnId": active.turn_id,
                }),
            )
            .await;
        if let Err(error) = result {
            let terminal = if matches!(error, CodexAdapterError::UnexpectedServerRequest) {
                TerminalState {
                    state: ConversationState::Blocked,
                    phase: ConversationLifecyclePhase::Blocked,
                    storage_status: "blocked",
                    diagnostic_code: Some(ConversationDiagnosticCode::ApprovalRequired),
                }
            } else {
                protocol_terminal(map_adapter_error(&error))
            };
            return finish_active(&mut state, projects, terminal, Vec::new()).await;
        }

        let event = active.lifecycle_event(ConversationLifecyclePhase::Stopping);
        active.state = ConversationState::Stopping;
        if projects
            .record_conversation_status(&active.conversation_id, "stopping")
            .is_err()
        {
            return finish_active(
                &mut state,
                projects,
                protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable),
                vec![event],
            )
            .await;
        }
        let snapshot = active.snapshot(vec![event], None);
        state.last = snapshot.clone();
        snapshot
    }
}

async fn start_on_process(
    process: &mut AppServerProcess,
    request: &ConversationStartRequest,
    cwd: &Path,
    projects: &ProjectService,
) -> Result<StartedConversation, ConversationDiagnosticCode> {
    let (models, _) = process
        .discover_models()
        .await
        .map_err(|error| map_adapter_error(&error))?;
    let Some(model) = models.iter().find(|model| model.id == request.model_id) else {
        return Err(ConversationDiagnosticCode::ModelUnavailable);
    };
    if !model
        .supported_reasoning_efforts
        .contains(&request.reasoning_effort)
    {
        return Err(ConversationDiagnosticCode::ReasoningUnavailable);
    }

    let thread_result = process
        .request(
            "thread/start",
            json!({
                "cwd": cwd,
                "model": request.model_id,
                "approvalPolicy": request.approval_policy.as_protocol_value(),
                "sandbox": request.sandbox_mode.as_protocol_value(),
            }),
        )
        .await
        .map_err(|error| map_adapter_error(&error))?;
    let thread = parse_thread_start(thread_result, cwd)?;
    let conversation_id = Uuid::now_v7().to_string();
    projects
        .record_conversation_reference(ConversationReference {
            conversation_id: &conversation_id,
            project_id: &request.project_id,
            codex_thread_id: &thread.thread.id,
            model_id: &request.model_id,
            reasoning_effort: &request.reasoning_effort,
            sandbox_mode: request.sandbox_mode.as_protocol_value(),
            approval_policy: request.approval_policy.as_protocol_value(),
        })
        .map_err(|_| ConversationDiagnosticCode::MetadataUnavailable)?;

    let turn_result = process
        .request(
            "turn/start",
            json!({
                "threadId": thread.thread.id,
                "input": [{"type": "text", "text": request.prompt}],
                "cwd": cwd,
                "model": request.model_id,
                "effort": request.reasoning_effort,
                "approvalPolicy": request.approval_policy.as_protocol_value(),
                "sandboxPolicy": sandbox_policy(request.sandbox_mode, cwd),
            }),
        )
        .await
        .map_err(|error| {
            let _ = projects.record_conversation_status(&conversation_id, "failed");
            map_adapter_error(&error)
        })?;
    let turn = match parse_turn_start(turn_result) {
        Ok(turn) => turn,
        Err(error) => {
            let _ = projects.record_conversation_status(&conversation_id, "failed");
            return Err(error);
        }
    };
    if projects
        .record_conversation_turn(&conversation_id, &turn.turn.id)
        .is_err()
    {
        let _ = projects.record_conversation_status(&conversation_id, "failed");
        return Err(ConversationDiagnosticCode::MetadataUnavailable);
    }

    Ok(StartedConversation {
        conversation_id,
        thread_id: thread.thread.id,
        turn_id: turn.turn.id,
    })
}

impl ActiveConversation {
    fn snapshot(
        &self,
        events: Vec<ConversationEvent>,
        diagnostic_code: Option<ConversationDiagnosticCode>,
    ) -> ConversationSnapshot {
        ConversationSnapshot {
            schema_version: CONVERSATION_SCHEMA_VERSION,
            state: self.state,
            conversation_id: Some(self.conversation_id.clone()),
            project_id: Some(self.project_id.clone()),
            model_id: Some(self.model_id.clone()),
            reasoning_effort: Some(self.reasoning_effort.clone()),
            sandbox_mode: Some(self.sandbox_mode),
            approval_policy: Some(self.approval_policy),
            events,
            diagnostic_code,
        }
    }

    fn lifecycle_event(&mut self, phase: ConversationLifecyclePhase) -> ConversationEvent {
        let sequence = self.take_sequence();
        ConversationEvent::Lifecycle { sequence, phase }
    }

    fn take_sequence(&mut self) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        sequence
    }
}

async fn finish_active(
    state: &mut ConversationServiceState,
    projects: &ProjectService,
    mut terminal: TerminalState,
    mut events: Vec<ConversationEvent>,
) -> ConversationSnapshot {
    let mut active = state
        .active
        .take()
        .expect("terminal transition requires an active conversation");
    if projects
        .record_conversation_status(&active.conversation_id, terminal.storage_status)
        .is_err()
    {
        terminal = protocol_terminal(ConversationDiagnosticCode::MetadataUnavailable);
    }
    events.push(active.lifecycle_event(terminal.phase));
    active.state = terminal.state;
    let _ = active.process.shutdown().await;
    projects.release_execution(&active.project_id);
    let snapshot = active.snapshot(events, terminal.diagnostic_code);
    state.last = snapshot.clone();
    snapshot
}

fn apply_notification(
    active: &mut ActiveConversation,
    notification: AppServerNotification,
) -> Result<(Option<ConversationEvent>, Option<TerminalState>), ConversationDiagnosticCode> {
    let AppServerNotification::Conversation(notification) = notification else {
        return Ok((None, None));
    };
    match notification {
        ConversationNotification::ThreadStarted { thread_id } => {
            ensure_thread(active, &thread_id)?;
            Ok((None, None))
        }
        ConversationNotification::TurnStarted { thread_id, turn_id } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((None, None))
        }
        ConversationNotification::AgentMessageDelta {
            thread_id,
            turn_id,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::AgentMessageDelta {
                    sequence: active.take_sequence(),
                    delta,
                }),
                None,
            ))
        }
        ConversationNotification::ReasoningSummaryDelta {
            thread_id,
            turn_id,
            delta,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::ReasoningSummaryDelta {
                    sequence: active.take_sequence(),
                    delta,
                }),
                None,
            ))
        }
        ConversationNotification::PlanUpdated {
            thread_id,
            turn_id,
            explanation,
            steps,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::PlanUpdated {
                    sequence: active.take_sequence(),
                    explanation,
                    steps: steps
                        .into_iter()
                        .map(|step| ConversationPlanStep {
                            step: step.step,
                            status: match step.status {
                                WirePlanStepStatus::Pending => ConversationPlanStepStatus::Pending,
                                WirePlanStepStatus::InProgress => {
                                    ConversationPlanStepStatus::InProgress
                                }
                                WirePlanStepStatus::Completed => {
                                    ConversationPlanStepStatus::Completed
                                }
                            },
                        })
                        .collect(),
                }),
                None,
            ))
        }
        ConversationNotification::ItemLifecycle {
            thread_id,
            turn_id,
            kind,
            status,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::Activity {
                    sequence: active.take_sequence(),
                    kind: map_item_kind(kind),
                    status: match status {
                        WireItemStatus::Started => ConversationActivityStatus::Started,
                        WireItemStatus::Completed => ConversationActivityStatus::Completed,
                    },
                }),
                None,
            ))
        }
        ConversationNotification::TurnCompleted {
            thread_id,
            turn_id,
            status,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            let terminal = match status {
                WireTurnStatus::Completed => TerminalState {
                    state: ConversationState::Completed,
                    phase: ConversationLifecyclePhase::Completed,
                    storage_status: "completed",
                    diagnostic_code: None,
                },
                WireTurnStatus::Interrupted => TerminalState {
                    state: ConversationState::Interrupted,
                    phase: ConversationLifecyclePhase::Interrupted,
                    storage_status: "interrupted",
                    diagnostic_code: None,
                },
                WireTurnStatus::Failed => TerminalState {
                    state: ConversationState::Failed,
                    phase: ConversationLifecyclePhase::Failed,
                    storage_status: "failed",
                    diagnostic_code: Some(ConversationDiagnosticCode::RuntimeUnavailable),
                },
            };
            Ok((None, Some(terminal)))
        }
        ConversationNotification::Error {
            thread_id,
            turn_id,
            code,
            will_retry,
        } => {
            ensure_turn(active, &thread_id, &turn_id)?;
            Ok((
                Some(ConversationEvent::Error {
                    sequence: active.take_sequence(),
                    code: map_stream_error(code),
                    will_retry,
                }),
                None,
            ))
        }
    }
}

fn validate_start_request(
    request: &ConversationStartRequest,
) -> Result<(), ConversationDiagnosticCode> {
    validate_uuid_v7(&request.project_id)
        .map_err(|_| ConversationDiagnosticCode::InvalidRequest)?;
    validate_user_text(&request.prompt)?;
    validate_protocol_choice(&request.model_id, 128)?;
    validate_protocol_choice(&request.reasoning_effort, 32)?;
    if request.sandbox_mode == ConversationSandboxMode::DangerFullAccess
        && request.approval_policy == ConversationApprovalPolicy::Never
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

fn validate_user_text(value: &str) -> Result<(), ConversationDiagnosticCode> {
    if value.trim().is_empty()
        || value.len() > MAX_PROMPT_BYTES
        || value.chars().any(|character| {
            character == '\0'
                || (character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
                || matches!(
                    character,
                    '\u{200B}'..='\u{200F}'
                        | '\u{202A}'..='\u{202E}'
                        | '\u{2060}'..='\u{206F}'
                        | '\u{FEFF}'
                )
        })
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

fn validate_protocol_choice(
    value: &str,
    max_bytes: usize,
) -> Result<(), ConversationDiagnosticCode> {
    if value.is_empty()
        || value.len() > max_bytes
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
    {
        return Err(ConversationDiagnosticCode::InvalidRequest);
    }
    Ok(())
}

fn sandbox_policy(mode: ConversationSandboxMode, cwd: &Path) -> Value {
    match mode {
        ConversationSandboxMode::ReadOnly => {
            json!({"type": "readOnly", "networkAccess": false})
        }
        ConversationSandboxMode::WorkspaceWrite => json!({
            "type": "workspaceWrite",
            "writableRoots": [cwd],
            "networkAccess": false,
            "excludeSlashTmp": false,
            "excludeTmpdirEnvVar": false,
        }),
        ConversationSandboxMode::DangerFullAccess => json!({"type": "dangerFullAccess"}),
    }
}

#[derive(Deserialize)]
struct ProtocolId {
    id: String,
}

#[derive(Deserialize)]
struct ThreadStartResult {
    cwd: String,
    thread: ProtocolId,
}

#[derive(Deserialize)]
struct TurnStartResult {
    turn: TurnStartTurn,
}

#[derive(Deserialize)]
struct TurnStartTurn {
    id: String,
    status: String,
}

fn parse_thread_start(
    value: Value,
    expected_cwd: &Path,
) -> Result<ThreadStartResult, ConversationDiagnosticCode> {
    let result: ThreadStartResult =
        serde_json::from_value(value).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&result.thread.id).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    if Path::new(&result.cwd) != expected_cwd {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(result)
}

fn parse_turn_start(value: Value) -> Result<TurnStartResult, ConversationDiagnosticCode> {
    let result: TurnStartResult =
        serde_json::from_value(value).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    validate_uuid_v7(&result.turn.id).map_err(|_| ConversationDiagnosticCode::ProtocolInvalid)?;
    if result.turn.status != "inProgress" {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(result)
}

fn ensure_thread(
    active: &ActiveConversation,
    thread_id: &str,
) -> Result<(), ConversationDiagnosticCode> {
    if active.thread_id != thread_id {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

fn ensure_turn(
    active: &ActiveConversation,
    thread_id: &str,
    turn_id: &str,
) -> Result<(), ConversationDiagnosticCode> {
    ensure_thread(active, thread_id)?;
    if active.turn_id != turn_id {
        return Err(ConversationDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

fn map_project_error(error: ProjectExecutionError) -> ConversationDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId | ProjectExecutionError::ProjectNotFound => {
            ConversationDiagnosticCode::ProjectUnavailable
        }
        ProjectExecutionError::MetadataUnavailable => {
            ConversationDiagnosticCode::MetadataUnavailable
        }
        ProjectExecutionError::DirectoryUnavailable => {
            ConversationDiagnosticCode::ProjectUnavailable
        }
        ProjectExecutionError::IdentityChanged => {
            ConversationDiagnosticCode::ProjectIdentityChanged
        }
        ProjectExecutionError::NotWritable => ConversationDiagnosticCode::ProjectNotWritable,
        ProjectExecutionError::ProjectBusy => ConversationDiagnosticCode::ProjectBusy,
    }
}

fn map_adapter_error(error: &CodexAdapterError) -> ConversationDiagnosticCode {
    match error {
        CodexAdapterError::ProcessSpawnFailed | CodexAdapterError::CliNotFound => {
            ConversationDiagnosticCode::RuntimeUnavailable
        }
        CodexAdapterError::ProcessExited => ConversationDiagnosticCode::ProcessExited,
        CodexAdapterError::TransportTimeout | CodexAdapterError::TransportClosed => {
            ConversationDiagnosticCode::TransportFailed
        }
        CodexAdapterError::RpcRejected => ConversationDiagnosticCode::RpcRejected,
        CodexAdapterError::UnexpectedServerRequest => ConversationDiagnosticCode::ApprovalRequired,
        CodexAdapterError::MessageTooLarge
        | CodexAdapterError::InvalidProtocolMessage
        | CodexAdapterError::InvalidModelCatalog
        | CodexAdapterError::CliVersionInvalid => ConversationDiagnosticCode::ProtocolInvalid,
    }
}

fn protocol_terminal(code: ConversationDiagnosticCode) -> TerminalState {
    TerminalState {
        state: if code == ConversationDiagnosticCode::ApprovalRequired {
            ConversationState::Blocked
        } else {
            ConversationState::Failed
        },
        phase: if code == ConversationDiagnosticCode::ApprovalRequired {
            ConversationLifecyclePhase::Blocked
        } else {
            ConversationLifecyclePhase::Failed
        },
        storage_status: if code == ConversationDiagnosticCode::ApprovalRequired {
            "blocked"
        } else {
            "failed"
        },
        diagnostic_code: Some(code),
    }
}

fn matching_last_or_not_found(
    last: &ConversationSnapshot,
    conversation_id: &str,
) -> ConversationSnapshot {
    if last.conversation_id.as_deref() == Some(conversation_id) {
        let mut snapshot = last.clone();
        snapshot.events.clear();
        snapshot
    } else {
        ConversationSnapshot::unavailable(ConversationDiagnosticCode::ConversationNotFound)
    }
}

fn map_item_kind(kind: WireItemKind) -> ConversationActivityKind {
    match kind {
        WireItemKind::UserMessage => ConversationActivityKind::UserMessage,
        WireItemKind::AgentMessage => ConversationActivityKind::AgentMessage,
        WireItemKind::Plan => ConversationActivityKind::Plan,
        WireItemKind::Reasoning => ConversationActivityKind::Reasoning,
        WireItemKind::CommandExecution => ConversationActivityKind::CommandExecution,
        WireItemKind::FileChange => ConversationActivityKind::FileChange,
        WireItemKind::ToolCall => ConversationActivityKind::ToolCall,
        WireItemKind::WebSearch => ConversationActivityKind::WebSearch,
        WireItemKind::Image => ConversationActivityKind::Image,
        WireItemKind::Other => ConversationActivityKind::Other,
    }
}

fn map_stream_error(code: WireErrorCode) -> ConversationStreamErrorCode {
    match code {
        WireErrorCode::ContextWindowExceeded => ConversationStreamErrorCode::ContextWindowExceeded,
        WireErrorCode::UsageLimitExceeded => ConversationStreamErrorCode::UsageLimitExceeded,
        WireErrorCode::Unauthorized => ConversationStreamErrorCode::Unauthorized,
        WireErrorCode::Sandbox => ConversationStreamErrorCode::Sandbox,
        WireErrorCode::Server => ConversationStreamErrorCode::Server,
        WireErrorCode::Other => ConversationStreamErrorCode::Other,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use serde_json::Value;
    use uuid::Uuid;

    use super::*;

    const THREAD_ID: &str = "018f0000-0000-7000-8000-000000000020";
    const TURN_ID: &str = "018f0000-0000-7000-8000-000000000030";

    #[test]
    fn serialized_empty_snapshot_matches_the_shared_frontend_fixture() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../../../fixtures/conversation.json"))
                .expect("conversation fixture must be JSON");
        let snapshot = serde_json::to_value(ConversationSnapshot::empty())
            .expect("conversation snapshot must serialize");

        assert_eq!(snapshot, fixture);
    }

    #[tokio::test]
    async fn starts_in_the_verified_cwd_and_normalizes_streamed_events() {
        let (projects, directory, project_id) = attached_project();
        let trailing = format!(
            r#"
printf '%s\n' '{{"method":"item/agentMessage/delta","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","itemId":"item-1","delta":"Review complete."}}}}'
printf '%s\n' '{{"method":"turn/plan/updated","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","explanation":"Checked safely.","plan":[{{"step":"Inspect project","status":"completed"}}]}}}}'
printf '%s\n' '{{"method":"item/started","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","startedAtMs":1,"item":{{"id":"item-2","type":"commandExecution"}}}}}}'
printf '%s\n' '{{"method":"item/completed","params":{{"threadId":"{THREAD_ID}","turnId":"{TURN_ID}","completedAtMs":2,"item":{{"id":"item-2","type":"commandExecution"}}}}}}'
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"completed"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));

        let started = service
            .start(start_request(project_id.clone()), &projects)
            .await;
        assert_eq!(started.state, ConversationState::Running);
        assert_eq!(
            projects.archive(project_id.clone()).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );
        let serialized = serde_json::to_string(&started).expect("snapshot must serialize");
        assert!(!serialized.contains(THREAD_ID));
        assert!(!serialized.contains(TURN_ID));
        assert!(!serialized.contains(directory.to_string_lossy().as_ref()));

        let completed = service
            .poll(
                started
                    .conversation_id
                    .clone()
                    .expect("conversation ID must exist"),
                &projects,
            )
            .await;
        assert_eq!(completed.state, ConversationState::Completed);
        assert!(completed.events.iter().any(|event| matches!(
            event,
            ConversationEvent::AgentMessageDelta { delta, .. } if delta == "Review complete."
        )));
        assert!(completed.events.iter().any(|event| matches!(
            event,
            ConversationEvent::Activity {
                kind: ConversationActivityKind::CommandExecution,
                status: ConversationActivityStatus::Completed,
                ..
            }
        )));
        assert!(completed.diagnostic_code.is_none());
        assert_ne!(
            projects.archive(project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn interrupts_only_the_exact_app_owned_conversation() {
        let (projects, directory, project_id) = attached_project();
        let trailing = format!(
            r#"
read -r _interrupt
printf '%s\n' '{{"id":5,"result":null}}'
printf '%s\n' '{{"method":"turn/completed","params":{{"threadId":"{THREAD_ID}","turn":{{"id":"{TURN_ID}","status":"interrupted"}}}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service.start(start_request(project_id), &projects).await;
        let conversation_id = started
            .conversation_id
            .clone()
            .expect("conversation ID must exist");

        let stopping = service.interrupt(conversation_id.clone(), &projects).await;
        assert_eq!(stopping.state, ConversationState::Stopping);
        let interrupted = service.poll(conversation_id, &projects).await;
        assert_eq!(interrupted.state, ConversationState::Interrupted);

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn fails_closed_on_approval_requests_and_releases_the_project() {
        let (projects, directory, project_id) = attached_project();
        let trailing = r#"
printf '%s\n' '{"id":99,"method":"item/commandExecution/requestApproval","params":{"private":"discarded"}}'
"#;
        let script = successful_start_script(&directory, trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service
            .start(start_request(project_id.clone()), &projects)
            .await;

        let blocked = service
            .poll(
                started.conversation_id.expect("conversation ID must exist"),
                &projects,
            )
            .await;
        assert_eq!(blocked.state, ConversationState::Blocked);
        assert_eq!(
            blocked.diagnostic_code,
            Some(ConversationDiagnosticCode::ApprovalRequired)
        );
        assert_ne!(
            projects.archive(project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn rejects_unadvertised_reasoning_and_never_spawns_for_a_missing_project() {
        let projects = ProjectService::in_memory();
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", "exit 91"]));
        let missing = service
            .start(
                start_request("018f0000-0000-7000-8000-000000000001".to_owned()),
                &projects,
            )
            .await;
        assert_eq!(
            missing.diagnostic_code,
            Some(ConversationDiagnosticCode::ProjectUnavailable)
        );

        let (projects, directory, project_id) = attached_project();
        let shutdown_marker = directory.join("shutdown-marker");
        let script = r#"
trap 'printf reaped > __SHUTDOWN_MARKER__' EXIT
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _models
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
read -r _wait_until_shutdown || true
"#
        .replace("__SHUTDOWN_MARKER__", shutdown_marker.to_string_lossy().as_ref());
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let mut request = start_request(project_id.clone());
        request.reasoning_effort = "xhigh".to_owned();
        let unavailable = service.start(request, &projects).await;
        assert_eq!(
            unavailable.diagnostic_code,
            Some(ConversationDiagnosticCode::ReasoningUnavailable)
        );
        assert_ne!(
            projects.archive(project_id).diagnostic_code,
            Some(crate::project::types::ProjectDiagnosticCode::ProjectBusy)
        );
        assert_eq!(
            fs::read_to_string(&shutdown_marker).expect("child exit trap must run before return"),
            "reaped"
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    #[tokio::test]
    async fn fails_closed_when_stream_identity_does_not_match_the_active_turn() {
        let (projects, directory, project_id) = attached_project();
        let trailing = format!(
            r#"
printf '%s\n' '{{"method":"item/agentMessage/delta","params":{{"threadId":"018f0000-0000-7000-8000-000000000099","turnId":"{TURN_ID}","itemId":"item-1","delta":"wrong thread"}}}}'
"#
        );
        let script = successful_start_script(&directory, &trailing);
        let service =
            ConversationService::with_command(AppServerCommand::test("sh", &["-c", &script]));
        let started = service.start(start_request(project_id), &projects).await;

        let failed = service
            .poll(
                started.conversation_id.expect("conversation ID must exist"),
                &projects,
            )
            .await;
        assert_eq!(failed.state, ConversationState::Failed);
        assert_eq!(
            failed.diagnostic_code,
            Some(ConversationDiagnosticCode::ProtocolInvalid)
        );

        fs::remove_dir_all(directory).expect("temporary project must be removed");
    }

    fn attached_project() -> (ProjectService, PathBuf, String) {
        let directory = std::env::temp_dir().join(format!(
            "quireforge-conversation-project-{}",
            Uuid::now_v7()
        ));
        fs::create_dir(&directory).expect("temporary project must be created");
        let projects = ProjectService::in_memory();
        let preview = projects.prepare_attachment(directory.clone());
        assert!(preview.pending_attachment.is_some());
        let snapshot = projects.confirm_pending();
        let project_id = snapshot
            .projects
            .first()
            .expect("project must attach")
            .id
            .clone();
        (projects, directory, project_id)
    }

    fn start_request(project_id: String) -> ConversationStartRequest {
        ConversationStartRequest {
            project_id,
            prompt: "Review the attached project.".to_owned(),
            model_id: "fixture-model".to_owned(),
            reasoning_effort: "medium".to_owned(),
            sandbox_mode: ConversationSandboxMode::ReadOnly,
            approval_policy: ConversationApprovalPolicy::Untrusted,
        }
    }

    fn successful_start_script(directory: &Path, trailing: &str) -> String {
        let cwd_json = serde_json::to_string(&directory.to_string_lossy())
            .expect("temporary cwd must serialize");
        r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _models
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
read -r _thread
printf '%s\n' '{"method":"thread/started","params":{"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
printf '%s\n' '{"id":3,"result":{"cwd":__CWD__,"thread":{"id":"018f0000-0000-7000-8000-000000000020"}}}'
read -r _turn
printf '%s\n' '{"method":"turn/started","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turn":{"id":"018f0000-0000-7000-8000-000000000030"}}}'
printf '%s\n' '{"id":4,"result":{"turn":{"id":"018f0000-0000-7000-8000-000000000030","status":"inProgress"}}}'
__TRAILING__
"#
        .replace("__CWD__", &cwd_json)
        .replace("__TRAILING__", trailing)
    }
}
