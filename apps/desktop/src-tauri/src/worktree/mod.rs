pub mod types;

use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
    sync::Mutex,
    time::{Duration, Instant},
};

use tokio::{io::AsyncReadExt, process::Command, time::timeout};
use uuid::Uuid;

use crate::project::{
    ProjectExecutionError, ProjectReviewRoot, ProjectService, ProjectWorktreeCandidate,
    ProjectWorktreeContext, WorktreeRegistrationError,
};
use types::{
    WorktreeCancelRequest, WorktreeConfirmRequest, WorktreeCreatePreviewRequest,
    WorktreeDiagnosticCode, WorktreeEntry, WorktreeEntryState, WorktreeOperation,
    WorktreeOwnership, WorktreePreviewSnapshot, WorktreePreviewState, WorktreeResultSnapshot,
    WorktreeResultState, WorktreeWorkspaceSnapshot, WorktreeWorkspaceState,
    WORKTREE_SCHEMA_VERSION,
};

const GIT_TIMEOUT: Duration = Duration::from_secs(20);
const CONFIRMATION_TTL: Duration = Duration::from_secs(5 * 60);
const MAX_OUTPUT_BYTES: usize = 256 * 1024;
const MAX_STDERR_BYTES: usize = 8 * 1024;
const MAX_WORKTREES: usize = 256;
const MAX_DISCOVERED_WORKTREES: usize = 512;
const MAX_BRANCH_BYTES: usize = 96;

#[derive(Clone, Debug)]
enum PendingOperation {
    Create { destination: PathBuf },
    Attach { candidate: ProjectWorktreeCandidate },
}

#[derive(Clone, Debug)]
struct PendingWorktree {
    confirmation_id: String,
    expires_at: Instant,
    source_project_id: String,
    source_root: PathBuf,
    common_dir: PathBuf,
    branch_name: Option<String>,
    base_commit: Option<String>,
    operation: PendingOperation,
}

#[derive(Clone, Debug)]
struct DiscoveredWorktree {
    path: PathBuf,
    branch_name: Option<String>,
    detached: bool,
    locked: bool,
    prunable: bool,
}

#[derive(Debug)]
enum GitRunError {
    Unavailable,
    Failed,
    TooLarge,
    TimedOut,
}

struct GitOutput {
    stdout: Vec<u8>,
    success: bool,
    code: Option<i32>,
}

pub struct WorktreeService {
    storage_root: Option<PathBuf>,
    pending: Mutex<Option<PendingWorktree>>,
}

impl WorktreeService {
    pub fn unavailable() -> Self {
        Self {
            storage_root: None,
            pending: Mutex::new(None),
        }
    }

    pub fn open(storage_root: &Path) -> Self {
        let root = prepare_storage_root(storage_root).ok();
        Self {
            storage_root: root,
            pending: Mutex::new(None),
        }
    }

    #[cfg(test)]
    fn for_test(storage_root: &Path) -> Self {
        Self::open(storage_root)
    }

    pub async fn status(
        &self,
        project_id: String,
        projects: &ProjectService,
    ) -> WorktreeWorkspaceSnapshot {
        let context = match projects.worktree_context(&project_id) {
            Ok(context) => context,
            Err(error) => {
                return WorktreeWorkspaceSnapshot::unavailable(None, map_project_error(error));
            }
        };
        let source_root = match projects.review_root(&context.source_project_id) {
            Ok(root) => root,
            Err(error) => {
                return WorktreeWorkspaceSnapshot::unavailable(
                    Some(context.source_project_id),
                    map_project_error(error),
                );
            }
        };
        let discovered = match list_worktrees(&source_root).await {
            Ok(worktrees) => worktrees,
            Err(error) => {
                return WorktreeWorkspaceSnapshot::unavailable(
                    Some(context.source_project_id),
                    map_git_error(error),
                );
            }
        };
        build_workspace(
            project_id,
            context,
            source_root.worktree_root.clone(),
            discovered,
        )
    }

    pub async fn preview_create(
        &self,
        request: WorktreeCreatePreviewRequest,
        projects: &ProjectService,
    ) -> WorktreePreviewSnapshot {
        let source_project_id = match projects.worktree_context(&request.project_id) {
            Ok(context) => context.source_project_id,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    request.project_id,
                    WorktreeOperation::Create,
                    map_project_error(error),
                );
            }
        };
        if !valid_branch_name(&request.branch_name) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Create,
                WorktreeDiagnosticCode::InvalidBranch,
            );
        }
        let Some(storage_root) = self.storage_root.as_ref() else {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Create,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        };
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root) if root.writable => root,
            Ok(_) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    WorktreeDiagnosticCode::DirectoryUnavailable,
                );
            }
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    map_project_error(error),
                );
            }
        };
        match check_branch(&source_root, &request.branch_name).await {
            Ok(false) => {}
            Ok(true) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    WorktreeDiagnosticCode::BranchExists,
                );
            }
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    map_git_error(error),
                );
            }
        }
        let base_commit = match read_head(&source_root).await {
            Ok(base_commit) => base_commit,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Create,
                    map_git_error(error),
                );
            }
        };

        let destination =
            generated_destination(storage_root, &source_project_id, &request.branch_name);
        let confirmation_id = Uuid::now_v7().to_string();
        let pending = PendingWorktree {
            confirmation_id: confirmation_id.clone(),
            expires_at: Instant::now() + CONFIRMATION_TTL,
            source_project_id: source_project_id.clone(),
            source_root: source_root.worktree_root,
            common_dir: source_root.common_dir,
            branch_name: Some(request.branch_name.clone()),
            base_commit: Some(base_commit),
            operation: PendingOperation::Create {
                destination: destination.clone(),
            },
        };
        if !self.replace_pending(pending) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Create,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        }
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Ready,
            source_project_id,
            operation: WorktreeOperation::Create,
            branch_name: Some(request.branch_name),
            display_path: Some(display_path(&destination)),
            ownership: Some(WorktreeOwnership::Managed),
            destructive: false,
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub async fn preview_attach(
        &self,
        project_id: String,
        selected_path: PathBuf,
        projects: &ProjectService,
    ) -> WorktreePreviewSnapshot {
        let source_project_id = match projects.worktree_context(&project_id) {
            Ok(context) => context.source_project_id,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    project_id,
                    WorktreeOperation::Attach,
                    map_project_error(error),
                );
            }
        };
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root) => root,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    map_project_error(error),
                );
            }
        };
        let candidate = match projects.inspect_worktree_candidate(&selected_path) {
            Ok(candidate)
                if candidate.is_linked_worktree
                    && candidate.resolved_path == candidate.worktree_root =>
            {
                candidate
            }
            Ok(_) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    WorktreeDiagnosticCode::NotLinkedWorktree,
                );
            }
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    map_project_error(error),
                );
            }
        };
        if candidate.common_dir != source_root.common_dir {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::DifferentRepository,
            );
        }
        let discovered = match list_worktrees(&source_root).await {
            Ok(discovered) => discovered,
            Err(error) => {
                return WorktreePreviewSnapshot::unavailable(
                    source_project_id,
                    WorktreeOperation::Attach,
                    map_git_error(error),
                );
            }
        };
        let Some(entry) = discovered
            .iter()
            .find(|entry| same_path(&entry.path, &candidate.worktree_root))
        else {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::StalePreview,
            );
        };
        if entry.locked || entry.prunable {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::DirectoryUnavailable,
            );
        }
        let branch_name = entry.branch_name.clone();
        let confirmation_id = Uuid::now_v7().to_string();
        let pending = PendingWorktree {
            confirmation_id: confirmation_id.clone(),
            expires_at: Instant::now() + CONFIRMATION_TTL,
            source_project_id: source_project_id.clone(),
            source_root: source_root.worktree_root,
            common_dir: source_root.common_dir,
            branch_name: branch_name.clone(),
            base_commit: None,
            operation: PendingOperation::Attach {
                candidate: candidate.clone(),
            },
        };
        if !self.replace_pending(pending) {
            return WorktreePreviewSnapshot::unavailable(
                source_project_id,
                WorktreeOperation::Attach,
                WorktreeDiagnosticCode::MetadataUnavailable,
            );
        }
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Ready,
            source_project_id,
            operation: WorktreeOperation::Attach,
            branch_name,
            display_path: Some(candidate.display_path),
            ownership: Some(WorktreeOwnership::Attached),
            destructive: false,
            confirmation_id: Some(confirmation_id),
            diagnostic_code: None,
        }
    }

    pub fn picker_unavailable(&self, project_id: String) -> WorktreePreviewSnapshot {
        WorktreePreviewSnapshot::unavailable(
            project_id,
            WorktreeOperation::Attach,
            WorktreeDiagnosticCode::PickerUnavailable,
        )
    }

    pub fn picker_cancelled(&self, project_id: String) -> WorktreePreviewSnapshot {
        WorktreePreviewSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Cancelled,
            source_project_id: project_id,
            operation: WorktreeOperation::Attach,
            branch_name: None,
            display_path: None,
            ownership: None,
            destructive: false,
            confirmation_id: None,
            diagnostic_code: None,
        }
    }

    pub fn cancel(&self, request: WorktreeCancelRequest) -> bool {
        if !valid_confirmation_id(&request.confirmation_id) {
            return false;
        }
        self.pending
            .lock()
            .map(|mut pending| {
                let matches = pending
                    .as_ref()
                    .is_some_and(|value| value.confirmation_id == request.confirmation_id);
                if matches {
                    *pending = None;
                }
                matches
            })
            .unwrap_or(false)
    }

    pub async fn confirm(
        &self,
        request: WorktreeConfirmRequest,
        projects: &ProjectService,
    ) -> WorktreeResultSnapshot {
        let Some(pending) = self.take_pending(&request.confirmation_id) else {
            return WorktreeResultSnapshot::unavailable(
                None,
                WorktreeDiagnosticCode::ConfirmationExpired,
            );
        };
        let source_project_id = pending.source_project_id.clone();
        if pending.expires_at <= Instant::now() {
            return WorktreeResultSnapshot::unavailable(
                Some(source_project_id),
                WorktreeDiagnosticCode::ConfirmationExpired,
            );
        }
        let reserved = match reserve_project_group(projects, &source_project_id) {
            Ok(reserved) => reserved,
            Err(error) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_project_error(error),
                );
            }
        };
        let result = self.confirm_reserved(pending, projects).await;
        for project_id in reserved {
            projects.release_execution(&project_id);
        }
        result
    }

    async fn confirm_reserved(
        &self,
        pending: PendingWorktree,
        projects: &ProjectService,
    ) -> WorktreeResultSnapshot {
        let source_project_id = pending.source_project_id.clone();
        let source_root = match projects.review_root(&source_project_id) {
            Ok(root)
                if root.worktree_root == pending.source_root
                    && root.common_dir == pending.common_dir =>
            {
                root
            }
            Ok(_) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    WorktreeDiagnosticCode::IdentityChanged,
                );
            }
            Err(error) => {
                return WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_project_error(error),
                );
            }
        };

        let (selected_path, ownership) = match pending.operation {
            PendingOperation::Create { destination } => {
                let branch_name = pending
                    .branch_name
                    .as_deref()
                    .expect("create preview always has a branch name");
                match check_branch(&source_root, branch_name).await {
                    Ok(false) => {}
                    Ok(true) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::BranchExists,
                        );
                    }
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_git_error(error),
                        );
                    }
                }
                let base_commit = match read_head(&source_root).await {
                    Ok(base_commit)
                        if pending.base_commit.as_deref() == Some(base_commit.as_str()) =>
                    {
                        base_commit
                    }
                    Ok(_) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::StalePreview,
                        );
                    }
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_git_error(error),
                        );
                    }
                };
                if destination.exists() || !destination_is_managed(&self.storage_root, &destination)
                {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::StalePreview,
                    );
                }
                if let Some(parent) = destination.parent() {
                    if fs::create_dir_all(parent).is_err()
                        || fs::set_permissions(parent, fs::Permissions::from_mode(0o700)).is_err()
                    {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::MetadataUnavailable,
                        );
                    }
                }
                match add_worktree(&source_root, branch_name, &destination, &base_commit).await {
                    Ok(()) => (destination, WorktreeOwnership::Managed),
                    Err(error) => {
                        let mut result = WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            if destination.exists() {
                                WorktreeDiagnosticCode::WorktreeRemains
                            } else {
                                map_git_error(error)
                            },
                        );
                        if destination.exists() {
                            result.recoverable_display_path = Some(display_path(&destination));
                        }
                        return result;
                    }
                }
            }
            PendingOperation::Attach { candidate } => {
                let current = match projects.inspect_worktree_candidate(&candidate.selected_path) {
                    Ok(current) if current == candidate => current,
                    Ok(_) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            WorktreeDiagnosticCode::IdentityChanged,
                        );
                    }
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_project_error(error),
                        );
                    }
                };
                let discovered = match list_worktrees(&source_root).await {
                    Ok(discovered) => discovered,
                    Err(error) => {
                        return WorktreeResultSnapshot::unavailable(
                            Some(source_project_id),
                            map_git_error(error),
                        );
                    }
                };
                let valid = discovered.iter().any(|entry| {
                    same_path(&entry.path, &current.worktree_root)
                        && entry.branch_name == pending.branch_name
                        && !entry.locked
                        && !entry.prunable
                });
                if !valid || current.common_dir != pending.common_dir {
                    return WorktreeResultSnapshot::unavailable(
                        Some(source_project_id),
                        WorktreeDiagnosticCode::StalePreview,
                    );
                }
                (current.selected_path, WorktreeOwnership::Attached)
            }
        };

        let ownership_value = ownership
            .as_storage_value()
            .expect("persisted ownership must have a storage value");
        let project_id = match projects.register_worktree_project(
            &source_project_id,
            &selected_path,
            &pending.common_dir,
            ownership_value,
            pending.branch_name.as_deref(),
        ) {
            Ok(project_id) => project_id,
            Err(error) => {
                let mut result = WorktreeResultSnapshot::unavailable(
                    Some(source_project_id),
                    map_registration_error(error),
                );
                if ownership == WorktreeOwnership::Managed {
                    result.diagnostic_code = Some(WorktreeDiagnosticCode::WorktreeRemains);
                    result.recoverable_display_path = Some(display_path(&selected_path));
                }
                return result;
            }
        };
        let workspace = self.status(source_project_id.clone(), projects).await;
        WorktreeResultSnapshot {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreeResultState::Applied,
            source_project_id: Some(source_project_id),
            project_id: Some(project_id),
            workspace: Some(workspace),
            recoverable_display_path: None,
            diagnostic_code: None,
        }
    }

    fn replace_pending(&self, pending: PendingWorktree) -> bool {
        self.pending
            .lock()
            .map(|mut current| *current = Some(pending))
            .is_ok()
    }

    fn take_pending(&self, confirmation_id: &str) -> Option<PendingWorktree> {
        if !valid_confirmation_id(confirmation_id) {
            return None;
        }
        self.pending.lock().ok().and_then(|mut current| {
            let matches = current
                .as_ref()
                .is_some_and(|pending| pending.confirmation_id == confirmation_id);
            if matches {
                current.take()
            } else {
                None
            }
        })
    }
}

fn prepare_storage_root(path: &Path) -> Result<PathBuf, ()> {
    if !path.is_absolute() || path.to_str().is_none() {
        return Err(());
    }
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(());
        }
    } else {
        fs::create_dir_all(path).map_err(|_| ())?;
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|_| ())?;
    fs::canonicalize(path).map_err(|_| ())
}

fn generated_destination(root: &Path, source_project_id: &str, branch_name: &str) -> PathBuf {
    let slug: String = branch_name
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .take(64)
        .collect();
    let suffix = Uuid::now_v7().simple().to_string();
    root.join(source_project_id)
        .join(format!("{slug}-{}", &suffix[..8]))
}

fn destination_is_managed(root: &Option<PathBuf>, destination: &Path) -> bool {
    root.as_ref().is_some_and(|root| {
        destination.starts_with(root)
            && destination.parent().and_then(Path::parent) == Some(root.as_path())
    })
}

fn valid_branch_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_BRANCH_BYTES
        && value.is_ascii()
        && !value.starts_with('-')
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.ends_with('.')
        && !value.ends_with(".lock")
        && value != "HEAD"
        && !value.contains("..")
        && !value.contains("@{")
        && !value.contains("//")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/._-".contains(&byte))
}

fn valid_confirmation_id(value: &str) -> bool {
    value.len() == 36 && Uuid::parse_str(value).is_ok()
}

fn reserve_project_group(
    projects: &ProjectService,
    source_project_id: &str,
) -> Result<Vec<String>, ProjectExecutionError> {
    let context = projects.worktree_context(source_project_id)?;
    let mut project_ids = vec![context.source_project_id];
    project_ids.extend(context.records.into_iter().map(|record| record.project_id));
    project_ids.sort();
    project_ids.dedup();
    let mut reserved: Vec<String> = Vec::with_capacity(project_ids.len());
    for project_id in project_ids {
        if let Err(error) = projects.reserve_execution(&project_id) {
            for reserved_id in &reserved {
                projects.release_execution(reserved_id);
            }
            return Err(error);
        }
        reserved.push(project_id);
    }
    Ok(reserved)
}

fn build_workspace(
    current_project_id: String,
    context: ProjectWorktreeContext,
    source_worktree_root: PathBuf,
    discovered: Vec<DiscoveredWorktree>,
) -> WorktreeWorkspaceSnapshot {
    let mut records: HashMap<PathBuf, _> = context
        .records
        .into_iter()
        .filter_map(|record| {
            record
                .selected_path
                .clone()
                .map(|path| (normalized_path(&path), record))
        })
        .collect();
    let mut entries = Vec::new();
    for discovered_entry in discovered {
        let normalized = normalized_path(&discovered_entry.path);
        let source = same_path(&discovered_entry.path, &source_worktree_root);
        let record = records.remove(&normalized);
        let (project_id, display_name, ownership, branch_name, archived) = if source {
            (
                Some(context.source_project_id.clone()),
                context.source_display_name.clone(),
                WorktreeOwnership::Source,
                discovered_entry.branch_name.clone(),
                false,
            )
        } else if let Some(record) = record {
            let ownership = WorktreeOwnership::from_storage_value(&record.ownership)
                .unwrap_or(WorktreeOwnership::External);
            (
                Some(record.project_id),
                record.display_name,
                ownership,
                record.branch_name.or(discovered_entry.branch_name.clone()),
                record.archived,
            )
        } else {
            (
                None,
                discovered_entry
                    .branch_name
                    .clone()
                    .unwrap_or_else(|| directory_name(&discovered_entry.path)),
                WorktreeOwnership::External,
                discovered_entry.branch_name.clone(),
                false,
            )
        };
        let state = if archived {
            WorktreeEntryState::Archived
        } else if discovered_entry.prunable {
            WorktreeEntryState::Prunable
        } else if discovered_entry.locked {
            WorktreeEntryState::Locked
        } else if discovered_entry.detached {
            WorktreeEntryState::Detached
        } else {
            WorktreeEntryState::Ready
        };
        entries.push(WorktreeEntry {
            current: project_id.as_deref() == Some(&current_project_id),
            project_id,
            display_name,
            display_path: display_path(&discovered_entry.path),
            branch_name,
            ownership,
            state,
        });
    }
    for (_, record) in records {
        let Some(path) = record.selected_path else {
            continue;
        };
        let ownership = WorktreeOwnership::from_storage_value(&record.ownership)
            .unwrap_or(WorktreeOwnership::External);
        entries.push(WorktreeEntry {
            current: record.project_id == current_project_id,
            project_id: Some(record.project_id),
            display_name: record.display_name,
            display_path: display_path(&path),
            branch_name: record.branch_name,
            ownership,
            state: if record.archived {
                WorktreeEntryState::Archived
            } else {
                WorktreeEntryState::Missing
            },
        });
    }
    entries.sort_by(|left, right| {
        ownership_rank(left.ownership)
            .cmp(&ownership_rank(right.ownership))
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    let truncated = entries.len() > MAX_WORKTREES;
    entries.truncate(MAX_WORKTREES);
    WorktreeWorkspaceSnapshot {
        schema_version: WORKTREE_SCHEMA_VERSION,
        state: if entries.is_empty() {
            WorktreeWorkspaceState::Empty
        } else {
            WorktreeWorkspaceState::Ready
        },
        source_project_id: Some(context.source_project_id),
        worktrees: entries,
        truncated,
        diagnostic_code: None,
    }
}

fn ownership_rank(ownership: WorktreeOwnership) -> u8 {
    match ownership {
        WorktreeOwnership::Source => 0,
        WorktreeOwnership::Managed => 1,
        WorktreeOwnership::Attached => 2,
        WorktreeOwnership::External => 3,
    }
}

fn directory_name(path: &Path) -> String {
    path.file_name()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("Detached worktree")
        .chars()
        .take(120)
        .collect()
}

fn normalized_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn same_path(left: &Path, right: &Path) -> bool {
    normalized_path(left) == normalized_path(right)
}

fn display_path(path: &Path) -> String {
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        if path == home {
            return "~".to_owned();
        }
        if let Ok(relative) = path.strip_prefix(home) {
            return format!("~/{}", relative.to_string_lossy());
        }
    }
    path.to_string_lossy().into_owned()
}

async fn check_branch(root: &ProjectReviewRoot, branch_name: &str) -> Result<bool, GitRunError> {
    let checked = run_git(
        &root.attached_root,
        [
            OsString::from("check-ref-format"),
            OsString::from("--branch"),
            OsString::from(branch_name),
        ],
        false,
    )
    .await?;
    if !checked.success {
        return Err(GitRunError::Failed);
    }
    let reference = format!("refs/heads/{branch_name}");
    let existing = run_git(
        &root.attached_root,
        [
            OsString::from("show-ref"),
            OsString::from("--verify"),
            OsString::from("--quiet"),
            OsString::from(reference),
        ],
        false,
    )
    .await?;
    match existing.code {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(GitRunError::Failed),
    }
}

async fn add_worktree(
    root: &ProjectReviewRoot,
    branch_name: &str,
    destination: &Path,
    base_commit: &str,
) -> Result<(), GitRunError> {
    let filter_overrides = checkout_filter_overrides(root).await?;
    let output = run_git_with_config(
        &root.attached_root,
        [
            OsString::from("worktree"),
            OsString::from("add"),
            OsString::from("--no-track"),
            OsString::from("-b"),
            OsString::from(branch_name),
            destination.as_os_str().to_os_string(),
            OsString::from(base_commit),
        ],
        true,
        &filter_overrides,
    )
    .await?;
    if output.success {
        Ok(())
    } else {
        Err(GitRunError::Failed)
    }
}

async fn checkout_filter_overrides(root: &ProjectReviewRoot) -> Result<Vec<String>, GitRunError> {
    let output = run_git(
        &root.attached_root,
        [
            OsString::from("config"),
            OsString::from("--null"),
            OsString::from("--name-only"),
            OsString::from("--get-regexp"),
            OsString::from(r"^filter\..*\.(clean|smudge|process|required)$"),
        ],
        false,
    )
    .await?;
    if output.code == Some(1) {
        return Ok(Vec::new());
    }
    if !output.success {
        return Err(GitRunError::Failed);
    }
    let mut drivers = Vec::new();
    for record in output.stdout.split(|byte| *byte == 0) {
        if record.is_empty() {
            continue;
        }
        let key = std::str::from_utf8(record).map_err(|_| GitRunError::Failed)?;
        let normalized = key.to_ascii_lowercase();
        let Some(remainder) = normalized.strip_prefix("filter.") else {
            return Err(GitRunError::Failed);
        };
        let Some((driver, property)) = remainder.rsplit_once('.') else {
            return Err(GitRunError::Failed);
        };
        if driver.is_empty()
            || driver.len() > 128
            || !driver
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            || !matches!(property, "clean" | "smudge" | "process" | "required")
        {
            return Err(GitRunError::Failed);
        }
        drivers.push(driver.to_owned());
    }
    drivers.sort();
    drivers.dedup();
    if drivers.len() > 64 {
        return Err(GitRunError::TooLarge);
    }
    let mut overrides = Vec::with_capacity(drivers.len() * 4);
    for driver in drivers {
        overrides.extend([
            format!("filter.{driver}.clean="),
            format!("filter.{driver}.smudge="),
            format!("filter.{driver}.process="),
            format!("filter.{driver}.required=false"),
        ]);
    }
    Ok(overrides)
}

async fn read_head(root: &ProjectReviewRoot) -> Result<String, GitRunError> {
    let output = run_git(
        &root.attached_root,
        [
            OsString::from("rev-parse"),
            OsString::from("--verify"),
            OsString::from("HEAD"),
        ],
        false,
    )
    .await?;
    if !output.success {
        return Err(GitRunError::Failed);
    }
    let value = std::str::from_utf8(&output.stdout)
        .map_err(|_| GitRunError::Failed)?
        .trim();
    if !matches!(value.len(), 40 | 64) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(GitRunError::Failed);
    }
    Ok(value.to_ascii_lowercase())
}

async fn list_worktrees(root: &ProjectReviewRoot) -> Result<Vec<DiscoveredWorktree>, GitRunError> {
    let output = run_git(
        &root.attached_root,
        [
            OsString::from("worktree"),
            OsString::from("list"),
            OsString::from("--porcelain"),
            OsString::from("-z"),
        ],
        false,
    )
    .await?;
    if !output.success {
        return Err(GitRunError::Failed);
    }
    parse_worktree_list(&output.stdout)
}

fn parse_worktree_list(bytes: &[u8]) -> Result<Vec<DiscoveredWorktree>, GitRunError> {
    let mut worktrees = Vec::new();
    let mut current: Option<DiscoveredWorktree> = None;
    for record in bytes.split(|byte| *byte == 0) {
        if record.is_empty() {
            if let Some(entry) = current.take() {
                worktrees.push(entry);
                if worktrees.len() > MAX_DISCOVERED_WORKTREES {
                    return Err(GitRunError::TooLarge);
                }
            }
            continue;
        }
        let text = std::str::from_utf8(record).map_err(|_| GitRunError::Failed)?;
        if let Some(path) = text.strip_prefix("worktree ") {
            if current.is_some() || !valid_inventory_path(path) {
                return Err(GitRunError::Failed);
            }
            current = Some(DiscoveredWorktree {
                path: PathBuf::from(path),
                branch_name: None,
                detached: false,
                locked: false,
                prunable: false,
            });
        } else if let Some(entry) = current.as_mut() {
            if let Some(branch) = text.strip_prefix("branch refs/heads/") {
                if valid_branch_name(branch) {
                    entry.branch_name = Some(branch.to_owned());
                } else {
                    return Err(GitRunError::Failed);
                }
            } else if text == "detached" {
                entry.detached = true;
            } else if text == "locked" || text.starts_with("locked ") {
                entry.locked = true;
            } else if text == "prunable" || text.starts_with("prunable ") {
                entry.prunable = true;
            } else if !text.starts_with("HEAD ") && !text.starts_with("bare") {
                return Err(GitRunError::Failed);
            }
        } else {
            return Err(GitRunError::Failed);
        }
    }
    if let Some(entry) = current {
        worktrees.push(entry);
    }
    if worktrees.len() > MAX_DISCOVERED_WORKTREES {
        return Err(GitRunError::TooLarge);
    }
    if worktrees.iter().any(|entry| !entry.path.is_absolute()) {
        return Err(GitRunError::Failed);
    }
    Ok(worktrees)
}

fn valid_inventory_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && !value.contains('\\')
        && !value.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '\u{061c}'
                        | '\u{200e}'
                        | '\u{200f}'
                        | '\u{202a}'..='\u{202e}'
                        | '\u{2066}'..='\u{2069}'
                )
        })
}

async fn run_git<const N: usize>(
    cwd: &Path,
    arguments: [OsString; N],
    mutation: bool,
) -> Result<GitOutput, GitRunError> {
    run_git_with_config(cwd, arguments, mutation, &[]).await
}

async fn run_git_with_config<const N: usize>(
    cwd: &Path,
    arguments: [OsString; N],
    mutation: bool,
    extra_config: &[String],
) -> Result<GitOutput, GitRunError> {
    let mut command = Command::new("git");
    command
        .current_dir(cwd)
        .env_clear()
        .env("PATH", "/usr/local/bin:/usr/bin:/bin")
        .env("HOME", "/nonexistent")
        .env("XDG_CONFIG_HOME", "/nonexistent")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_ATTR_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LITERAL_PATHSPECS", "1")
        .env("GIT_PAGER", "cat")
        .env("PAGER", "cat")
        .arg("--no-pager")
        .args(["-c", "core.quotepath=false"])
        .args(["-c", "color.ui=false"])
        .args(["-c", "core.hooksPath=/dev/null"])
        .args(["-c", "core.fsmonitor=false"])
        .args(["-c", "credential.helper="])
        .args(["-c", "submodule.recurse=false"]);
    for value in extra_config {
        command.arg("-c").arg(value);
    }
    if !mutation {
        command.env("GIT_OPTIONAL_LOCKS", "0");
    }
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command.spawn().map_err(|_| GitRunError::Unavailable)?;
    let stdout = child.stdout.take().ok_or(GitRunError::Unavailable)?;
    let stderr = child.stderr.take().ok_or(GitRunError::Unavailable)?;
    let result = timeout(GIT_TIMEOUT, async {
        let (stdout, stderr, status) = tokio::join!(
            read_bounded(stdout, MAX_OUTPUT_BYTES),
            read_bounded(stderr, MAX_STDERR_BYTES),
            child.wait(),
        );
        (stdout, stderr, status)
    })
    .await;
    let (stdout, stderr, status) = match result {
        Ok(result) => result,
        Err(_) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(GitRunError::TimedOut);
        }
    };
    let stdout = stdout?;
    let _ = stderr?;
    let status = status.map_err(|_| GitRunError::Failed)?;
    Ok(GitOutput {
        stdout,
        success: status.success(),
        code: status.code(),
    })
}

async fn read_bounded(
    reader: impl tokio::io::AsyncRead + Unpin,
    limit: usize,
) -> Result<Vec<u8>, GitRunError> {
    let mut bytes = Vec::with_capacity(limit.min(16 * 1024));
    reader
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| GitRunError::Failed)?;
    if bytes.len() > limit {
        return Err(GitRunError::TooLarge);
    }
    Ok(bytes)
}

fn map_project_error(error: ProjectExecutionError) -> WorktreeDiagnosticCode {
    match error {
        ProjectExecutionError::InvalidProjectId | ProjectExecutionError::ProjectNotFound => {
            WorktreeDiagnosticCode::ProjectNotFound
        }
        ProjectExecutionError::MetadataUnavailable => WorktreeDiagnosticCode::MetadataUnavailable,
        ProjectExecutionError::DirectoryUnavailable | ProjectExecutionError::NotWritable => {
            WorktreeDiagnosticCode::DirectoryUnavailable
        }
        ProjectExecutionError::IdentityChanged => WorktreeDiagnosticCode::IdentityChanged,
        ProjectExecutionError::NotRepository => WorktreeDiagnosticCode::NotRepository,
        ProjectExecutionError::ProjectBusy => WorktreeDiagnosticCode::ProjectBusy,
    }
}

fn map_git_error(error: GitRunError) -> WorktreeDiagnosticCode {
    match error {
        GitRunError::Unavailable => WorktreeDiagnosticCode::GitUnavailable,
        GitRunError::TooLarge => WorktreeDiagnosticCode::OutputTooLarge,
        GitRunError::Failed | GitRunError::TimedOut => WorktreeDiagnosticCode::GitFailed,
    }
}

fn map_registration_error(error: WorktreeRegistrationError) -> WorktreeDiagnosticCode {
    match error {
        WorktreeRegistrationError::Project(error) => map_project_error(error),
        WorktreeRegistrationError::DuplicateDirectory => WorktreeDiagnosticCode::DuplicateDirectory,
        WorktreeRegistrationError::NotLinkedWorktree => WorktreeDiagnosticCode::NotLinkedWorktree,
        WorktreeRegistrationError::DifferentRepository => {
            WorktreeDiagnosticCode::DifferentRepository
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt, process::Command as StdCommand};

    use uuid::Uuid;

    use super::{
        parse_worktree_list, types::WorktreeDiagnosticCode, types::WorktreeOwnership,
        types::WorktreePreviewState, types::WorktreeResultState, types::WorktreeWorkspaceState,
        valid_branch_name, WorktreeService,
    };
    use crate::project::ProjectService;
    use crate::worktree::types::{WorktreeConfirmRequest, WorktreeCreatePreviewRequest};

    fn temporary_directory(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("quireforge-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("temporary directory must be created");
        path
    }

    fn repository_fixture() -> (std::path::PathBuf, ProjectService, String) {
        let root = temporary_directory("worktree-repository");
        assert!(StdCommand::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .expect("Git must run")
            .success());
        assert!(StdCommand::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "--quiet",
                "-m",
                "initial",
            ])
            .current_dir(&root)
            .status()
            .expect("Git must run")
            .success());
        let projects = ProjectService::in_memory();
        projects.prepare_attachment(root.clone());
        let attached = projects.confirm_pending();
        (root, projects, attached.projects[0].id.clone())
    }

    #[test]
    fn validates_a_bounded_non_option_branch_contract() {
        assert!(valid_branch_name("feature/worktree-11a"));
        for invalid in [
            "", "-force", "HEAD", "a..b", "a@{b", "a.lock", "a b", "a\\b",
        ] {
            assert!(!valid_branch_name(invalid), "{invalid} must be rejected");
        }
    }

    #[test]
    fn parses_porcelain_without_retaining_object_ids() {
        let bytes = b"worktree /tmp/source\0HEAD 0123456789abcdef\0branch refs/heads/main\0\0worktree /tmp/linked\0HEAD fedcba9876543210\0detached\0locked reason\0\0";
        let parsed = parse_worktree_list(bytes).expect("fixture must parse");
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].branch_name.as_deref(), Some("main"));
        assert!(parsed[1].detached);
        assert!(parsed[1].locked);
        let serialized = format!("{parsed:?}");
        assert!(!serialized.contains("0123456789abcdef"));
        assert!(!serialized.contains("fedcba9876543210"));
    }

    #[tokio::test]
    async fn creates_a_confirmed_managed_worktree_and_registers_a_project() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id: project_id.clone(),
                    branch_name: "feature/managed-fixture".to_owned(),
                },
                &projects,
            )
            .await;
        assert_eq!(preview.state, WorktreePreviewState::Ready);
        assert!(!preview.destructive);
        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(result.state, WorktreeResultState::Applied);
        let workspace = result.workspace.expect("workspace must be returned");
        assert_eq!(workspace.state, WorktreeWorkspaceState::Ready);
        assert!(workspace.worktrees.iter().any(|entry| {
            entry.ownership == WorktreeOwnership::Managed
                && entry.branch_name.as_deref() == Some("feature/managed-fixture")
        }));
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn recognizes_the_source_when_the_project_attaches_a_repository_subdirectory() {
        let (repository, projects, project_id) = repository_fixture();
        let subdirectory = repository.join("nested-project");
        fs::create_dir(&subdirectory).expect("project subdirectory must exist");
        projects.prepare_relink(project_id.clone(), subdirectory);
        let relinked = projects.confirm_pending();
        assert!(relinked.diagnostic_code.is_none());
        let app_data = temporary_directory("worktree-subdirectory-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));

        let workspace = service.status(project_id.clone(), &projects).await;

        assert_eq!(workspace.state, WorktreeWorkspaceState::Ready);
        let source = workspace
            .worktrees
            .iter()
            .find(|entry| entry.ownership == WorktreeOwnership::Source)
            .expect("repository source must be identified");
        assert_eq!(source.project_id.as_deref(), Some(project_id.as_str()));
        assert!(source.current);
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn keeps_external_worktrees_unselectable_until_native_picker_attachment() {
        let (repository, projects, project_id) = repository_fixture();
        let external = repository.with_extension(format!("linked-{}", Uuid::now_v7()));
        assert!(StdCommand::new("git")
            .args([
                "worktree",
                "add",
                "--quiet",
                "-b",
                "feature/external-fixture",
            ])
            .arg(&external)
            .arg("HEAD")
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let app_data = temporary_directory("worktree-attach-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));

        let before = service.status(project_id.clone(), &projects).await;
        let external_entry = before
            .worktrees
            .iter()
            .find(|entry| entry.branch_name.as_deref() == Some("feature/external-fixture"))
            .expect("external worktree must be discovered");
        assert_eq!(external_entry.ownership, WorktreeOwnership::External);
        assert!(external_entry.project_id.is_none());

        let preview = service
            .preview_attach(project_id, external.clone(), &projects)
            .await;
        assert_eq!(preview.state, WorktreePreviewState::Ready);
        assert_eq!(preview.ownership, Some(WorktreeOwnership::Attached));
        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;
        assert_eq!(result.state, WorktreeResultState::Applied);
        assert!(result
            .workspace
            .expect("workspace must be returned")
            .worktrees
            .iter()
            .any(|entry| {
                entry.ownership == WorktreeOwnership::Attached
                    && entry.project_id.is_some()
                    && entry.branch_name.as_deref() == Some("feature/external-fixture")
            }));

        drop(service);
        drop(projects);
        fs::remove_dir_all(external).expect("external worktree fixture must be removed");
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn leaves_a_created_worktree_recoverable_when_metadata_registration_fails() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-recovery-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id,
                    branch_name: "feature/recoverable-fixture".to_owned(),
                },
                &projects,
            )
            .await;
        projects.fail_worktree_registration_for_test();

        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(result.state, WorktreeResultState::Unavailable);
        assert_eq!(
            result.diagnostic_code,
            Some(WorktreeDiagnosticCode::WorktreeRemains)
        );
        assert!(result.recoverable_display_path.is_some());
        let managed_parent = app_data.join("worktrees");
        assert!(managed_parent
            .read_dir()
            .expect("managed root must exist")
            .next()
            .is_some());

        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("recoverable fixture must be removed explicitly");
    }

    #[tokio::test]
    async fn consumes_confirmation_tokens_once() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-token-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id,
                    branch_name: "feature/one-use".to_owned(),
                },
                &projects,
            )
            .await;
        let confirmation_id = preview.confirmation_id.expect("confirmation must exist");
        let first = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: confirmation_id.clone(),
                },
                &projects,
            )
            .await;
        assert_eq!(first.state, WorktreeResultState::Applied);
        let second = service
            .confirm(WorktreeConfirmRequest { confirmation_id }, &projects)
            .await;
        assert_eq!(
            second.diagnostic_code,
            Some(WorktreeDiagnosticCode::ConfirmationExpired)
        );
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn refuses_creation_when_head_changes_after_preview() {
        let (repository, projects, project_id) = repository_fixture();
        let app_data = temporary_directory("worktree-stale-head-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id,
                    branch_name: "feature/stale-head".to_owned(),
                },
                &projects,
            )
            .await;
        assert!(StdCommand::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--allow-empty",
                "--quiet",
                "-m",
                "changed after preview",
            ])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());

        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(result.state, WorktreeResultState::Unavailable);
        assert_eq!(
            result.diagnostic_code,
            Some(WorktreeDiagnosticCode::StalePreview)
        );
        assert!(!StdCommand::new("git")
            .args([
                "show-ref",
                "--verify",
                "--quiet",
                "refs/heads/feature/stale-head"
            ])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }

    #[tokio::test]
    async fn creates_without_running_repository_checkout_filters() {
        let (repository, projects, project_id) = repository_fixture();
        fs::write(
            repository.join(".gitattributes"),
            "payload.txt filter=fixture\n",
        )
        .expect("attributes fixture must be written");
        fs::write(repository.join("payload.txt"), "safe payload\n")
            .expect("payload fixture must be written");
        assert!(StdCommand::new("git")
            .args(["add", ".gitattributes", "payload.txt"])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        assert!(StdCommand::new("git")
            .args([
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "filter fixture",
            ])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let marker = std::env::temp_dir().join(format!("quireforge-filter-{}", Uuid::now_v7()));
        let hook_marker = std::env::temp_dir().join(format!("quireforge-hook-{}", Uuid::now_v7()));
        let filter_command = format!("touch {}; cat", marker.display());
        assert!(StdCommand::new("git")
            .args(["config", "--local", "filter.fixture.smudge"])
            .arg(&filter_command)
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let hook = repository.join(".git/hooks/post-checkout");
        fs::write(
            &hook,
            format!("#!/bin/sh\n: > '{}'\n", hook_marker.display()),
        )
        .expect("hook fixture must be written");
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755))
            .expect("hook fixture must be executable");
        assert!(StdCommand::new("git")
            .args(["config", "--local", "filter.fixture.required", "true"])
            .current_dir(&repository)
            .status()
            .expect("Git must run")
            .success());
        let app_data = temporary_directory("worktree-filter-app-data");
        let service = WorktreeService::for_test(&app_data.join("worktrees"));
        let preview = service
            .preview_create(
                WorktreeCreatePreviewRequest {
                    project_id,
                    branch_name: "feature/no-filter".to_owned(),
                },
                &projects,
            )
            .await;

        let result = service
            .confirm(
                WorktreeConfirmRequest {
                    confirmation_id: preview.confirmation_id.expect("confirmation must exist"),
                },
                &projects,
            )
            .await;

        assert_eq!(result.state, WorktreeResultState::Applied);
        let filter_executed = marker.exists();
        if filter_executed {
            fs::remove_file(&marker).expect("unexpected filter marker must be removed");
        }
        let hook_executed = hook_marker.exists();
        if hook_executed {
            fs::remove_file(&hook_marker).expect("unexpected hook marker must be removed");
        }
        assert!(!filter_executed, "checkout filter must not execute");
        assert!(!hook_executed, "checkout hook must not execute");
        drop(service);
        drop(projects);
        fs::remove_dir_all(repository).expect("repository fixture must be removed");
        fs::remove_dir_all(app_data).expect("app data fixture must be removed");
    }
}
