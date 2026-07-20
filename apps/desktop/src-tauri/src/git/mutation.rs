use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::project::{ProjectReviewRoot, ProjectService};

use super::types::{
    GitChangeKind, GitDiagnosticCode, GitFileChange, GitMutationConfirmRequest,
    GitMutationOperation, GitMutationPreviewRequest, GitMutationPreviewSnapshot,
    GitMutationPreviewState, GitMutationResultSnapshot, GitMutationResultState, GitMutationTarget,
    GitRecoveryRequest, GitSecretFinding, GitSecretFindingKind, GitSecretFindingLocation,
    GIT_MUTATION_SCHEMA_VERSION,
};
use super::{
    inspect_status, map_project_error, map_run_error, run_git, safe_worktree_file,
    scoped_status_path, unsafe_display_character, valid_relative_path, workspace_from_root,
    GitRunError,
};

const CONFIRMATION_TTL: Duration = Duration::from_secs(5 * 60);
const RECOVERY_TTL: Duration = Duration::from_secs(30 * 60);
const MAX_PENDING_MUTATIONS: usize = 32;
const MAX_RECOVERIES: usize = 16;
const MAX_RECOVERY_BYTES: u64 = 1024 * 1024;
const MAX_SCANNED_BLOB_BYTES: usize = 1024 * 1024;
const MAX_SCANNED_TOTAL_BYTES: usize = 4 * 1024 * 1024;
const MAX_GIT_METADATA_BYTES: usize = 64 * 1024;
const MAX_COMMIT_MESSAGE_CHARACTERS: usize = 512;
const MAX_SECRET_FINDINGS: usize = 64;

#[derive(Default)]
pub(super) struct MutationCoordinator {
    state: Mutex<MutationState>,
}

#[derive(Default)]
struct MutationState {
    pending: HashMap<String, PendingMutation>,
    recoveries: HashMap<String, CompletedRecovery>,
}

struct PendingMutation {
    project_id: String,
    root: ProjectReviewRoot,
    operation: GitMutationOperation,
    expires_at: Instant,
    action: PendingAction,
}

enum PendingAction {
    Stage(TargetEvidence),
    Unstage(UnstageEvidence),
    Revert(RevertEvidence),
    Commit(CommitPlan),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TargetEvidence {
    path: String,
    repo_path: String,
    change: GitFileChange,
    index_entry: Option<IndexEntry>,
    worktree_oid: Option<String>,
    worktree_mode: Option<u32>,
}

struct UnstageEvidence {
    target: TargetEvidence,
    head_entry: Option<IndexEntry>,
}

struct RevertEvidence {
    target: TargetEvidence,
    backup: FileBackup,
}

#[derive(Clone)]
struct FileBackup {
    bytes: Vec<u8>,
    mode: u32,
}

struct CommitPlan {
    message: String,
    evidence: CommitEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommitEvidence {
    head_oid: Option<String>,
    identity_name: String,
    identity_email: String,
    staged: Vec<CommitTargetEvidence>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CommitTargetEvidence {
    change: GitFileChange,
    repo_path: String,
    index_entry: Option<IndexEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct IndexEntry {
    mode: String,
    oid: String,
}

#[derive(Clone)]
struct CompletedRecovery {
    recovery_id: String,
    project_id: String,
    root: ProjectReviewRoot,
    path: String,
    repo_path: String,
    backup: FileBackup,
    reverted_oid: String,
    original_oid: String,
    expires_at: Instant,
}

struct PreparedAction {
    action: PendingAction,
    targets: Vec<GitMutationTarget>,
}

struct PreparationFailure {
    diagnostic: GitDiagnosticCode,
    blocked: bool,
    targets: Vec<GitMutationTarget>,
    findings: Vec<GitSecretFinding>,
}

struct ExecutionSuccess {
    recovery: Option<CompletedRecovery>,
}

struct IndexLock {
    path: PathBuf,
    device: u64,
    inode: u64,
}

impl Drop for IndexLock {
    fn drop(&mut self) {
        if self.path.symlink_metadata().is_ok_and(|metadata| {
            metadata.is_file() && metadata.dev() == self.device && metadata.ino() == self.inode
        }) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl MutationCoordinator {
    pub(super) async fn preview(
        &self,
        request: GitMutationPreviewRequest,
        projects: &ProjectService,
    ) -> GitMutationPreviewSnapshot {
        let path = request
            .path
            .as_ref()
            .filter(|path| valid_relative_path(path))
            .cloned();
        if !valid_request_shape(&request) {
            return GitMutationPreviewSnapshot::unavailable(
                &request,
                path,
                GitDiagnosticCode::MutationUnavailable,
            );
        }
        if request.path.is_some() && path.is_none() {
            return GitMutationPreviewSnapshot::unavailable(
                &request,
                None,
                GitDiagnosticCode::InvalidPath,
            );
        }
        let root = match projects.review_root(&request.project_id) {
            Ok(root) => root,
            Err(error) => {
                return GitMutationPreviewSnapshot::unavailable(
                    &request,
                    path,
                    map_project_error(error),
                );
            }
        };
        if !mutation_root_writable(&root) {
            return GitMutationPreviewSnapshot::unavailable(
                &request,
                path,
                GitDiagnosticCode::ReadOnly,
            );
        }
        if let Err(error) = projects.reserve_execution(&request.project_id) {
            return GitMutationPreviewSnapshot::unavailable(
                &request,
                path,
                map_project_error(error),
            );
        }
        projects.release_execution(&request.project_id);
        {
            let mut state = self.state.lock().await;
            clear_expired(&mut state);
            state
                .pending
                .retain(|_, pending| pending.project_id != request.project_id);
        }

        let prepared = prepare_action(&request, &root).await;
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(failure) => {
                return GitMutationPreviewSnapshot {
                    schema_version: GIT_MUTATION_SCHEMA_VERSION,
                    state: if failure.blocked {
                        GitMutationPreviewState::Blocked
                    } else {
                        GitMutationPreviewState::Unavailable
                    },
                    project_id: request.project_id,
                    operation: request.operation,
                    path,
                    targets: failure.targets,
                    destructive: request.operation == GitMutationOperation::Revert,
                    confirmation_id: None,
                    secret_findings: failure.findings,
                    diagnostic_code: Some(failure.diagnostic),
                };
            }
        };

        let confirmation_id = Uuid::now_v7().to_string();
        let mut state = self.state.lock().await;
        clear_expired(&mut state);
        state
            .pending
            .retain(|_, pending| pending.project_id != request.project_id);
        if state.pending.len() >= MAX_PENDING_MUTATIONS {
            return GitMutationPreviewSnapshot::unavailable(
                &request,
                path,
                GitDiagnosticCode::MutationUnavailable,
            );
        }
        state.pending.insert(
            confirmation_id.clone(),
            PendingMutation {
                project_id: request.project_id.clone(),
                root,
                operation: request.operation,
                expires_at: Instant::now() + CONFIRMATION_TTL,
                action: prepared.action,
            },
        );
        GitMutationPreviewSnapshot {
            schema_version: GIT_MUTATION_SCHEMA_VERSION,
            state: GitMutationPreviewState::Ready,
            project_id: request.project_id,
            operation: request.operation,
            path,
            targets: prepared.targets,
            destructive: request.operation == GitMutationOperation::Revert,
            confirmation_id: Some(confirmation_id),
            secret_findings: Vec::new(),
            diagnostic_code: None,
        }
    }

    pub(super) async fn confirm(
        &self,
        request: GitMutationConfirmRequest,
        projects: &ProjectService,
    ) -> GitMutationResultSnapshot {
        if !valid_token(&request.confirmation_id) {
            return GitMutationResultSnapshot::unavailable(
                None,
                None,
                GitDiagnosticCode::ConfirmationExpired,
            );
        }
        let pending = {
            let mut state = self.state.lock().await;
            clear_expired(&mut state);
            state.pending.remove(&request.confirmation_id)
        };
        let Some(pending) = pending else {
            return GitMutationResultSnapshot::unavailable(
                None,
                None,
                GitDiagnosticCode::ConfirmationExpired,
            );
        };
        if pending.expires_at <= Instant::now() {
            return GitMutationResultSnapshot::unavailable(
                Some(pending.project_id),
                Some(pending.operation),
                GitDiagnosticCode::ConfirmationExpired,
            );
        }
        if let Err(error) = projects.reserve_execution(&pending.project_id) {
            return GitMutationResultSnapshot::unavailable(
                Some(pending.project_id),
                Some(pending.operation),
                map_project_error(error),
            );
        }
        let reserved_project_id = pending.project_id.clone();
        let result = self.confirm_reserved(pending, projects).await;
        projects.release_execution(&reserved_project_id);
        result
    }

    async fn confirm_reserved(
        &self,
        pending: PendingMutation,
        projects: &ProjectService,
    ) -> GitMutationResultSnapshot {
        let project_id = pending.project_id.clone();
        let operation = pending.operation;
        let root = match projects.review_root(&project_id) {
            Ok(root) if root == pending.root && mutation_root_writable(&root) => root,
            Ok(_) => {
                return GitMutationResultSnapshot::unavailable(
                    Some(project_id),
                    Some(operation),
                    GitDiagnosticCode::StalePreview,
                );
            }
            Err(error) => {
                return GitMutationResultSnapshot::unavailable(
                    Some(project_id),
                    Some(operation),
                    map_project_error(error),
                );
            }
        };
        let execution = match execute_action(&root, &project_id, pending.action).await {
            Ok(execution) => execution,
            Err(diagnostic) => {
                return GitMutationResultSnapshot::unavailable(
                    Some(project_id),
                    Some(operation),
                    diagnostic,
                );
            }
        };
        let recovery_id = execution
            .recovery
            .as_ref()
            .map(|recovery| recovery.recovery_id.clone());
        if let Some(recovery) = execution.recovery {
            let mut state = self.state.lock().await;
            clear_expired(&mut state);
            if state.recoveries.len() >= MAX_RECOVERIES {
                if let Some(oldest) = state
                    .recoveries
                    .iter()
                    .min_by_key(|(_, recovery)| recovery.expires_at)
                    .map(|(id, _)| id.clone())
                {
                    state.recoveries.remove(&oldest);
                }
            }
            state
                .recoveries
                .insert(recovery.recovery_id.clone(), recovery);
        }
        let workspace = workspace_from_root(project_id.clone(), &root).await;
        GitMutationResultSnapshot {
            schema_version: GIT_MUTATION_SCHEMA_VERSION,
            state: GitMutationResultState::Applied,
            project_id: Some(project_id),
            operation: Some(operation),
            recovery_id,
            workspace: Some(workspace),
            diagnostic_code: None,
        }
    }

    pub(super) async fn recover(
        &self,
        request: GitRecoveryRequest,
        projects: &ProjectService,
    ) -> GitMutationResultSnapshot {
        if !valid_token(&request.recovery_id) {
            return GitMutationResultSnapshot::unavailable(
                None,
                Some(GitMutationOperation::Revert),
                GitDiagnosticCode::RecoveryUnavailable,
            );
        }
        let recovery = {
            let mut state = self.state.lock().await;
            clear_expired(&mut state);
            state.recoveries.get(&request.recovery_id).cloned()
        };
        let Some(recovery) = recovery else {
            return GitMutationResultSnapshot::unavailable(
                None,
                Some(GitMutationOperation::Revert),
                GitDiagnosticCode::RecoveryUnavailable,
            );
        };
        if let Err(error) = projects.reserve_execution(&recovery.project_id) {
            return GitMutationResultSnapshot::unavailable(
                Some(recovery.project_id),
                Some(GitMutationOperation::Revert),
                map_project_error(error),
            );
        }
        let result = self.recover_reserved(&recovery, projects).await;
        projects.release_execution(&recovery.project_id);
        if result.state == GitMutationResultState::Applied {
            let mut state = self.state.lock().await;
            state.recoveries.remove(&request.recovery_id);
        }
        result
    }

    async fn recover_reserved(
        &self,
        recovery: &CompletedRecovery,
        projects: &ProjectService,
    ) -> GitMutationResultSnapshot {
        let root = match projects.review_root(&recovery.project_id) {
            Ok(root) if root == recovery.root && mutation_root_writable(&root) => root,
            Ok(_) => {
                return GitMutationResultSnapshot::unavailable(
                    Some(recovery.project_id.clone()),
                    Some(GitMutationOperation::Revert),
                    GitDiagnosticCode::StalePreview,
                );
            }
            Err(error) => {
                return GitMutationResultSnapshot::unavailable(
                    Some(recovery.project_id.clone()),
                    Some(GitMutationOperation::Revert),
                    map_project_error(error),
                );
            }
        };
        if recovery.expires_at <= Instant::now()
            || hash_worktree(&root, &recovery.path, &recovery.repo_path)
                .await
                .ok()
                .as_deref()
                != Some(recovery.reverted_oid.as_str())
        {
            return GitMutationResultSnapshot::unavailable(
                Some(recovery.project_id.clone()),
                Some(GitMutationOperation::Revert),
                GitDiagnosticCode::StalePreview,
            );
        }
        if restore_backup(&root, &recovery.path, &recovery.backup).is_err() {
            return GitMutationResultSnapshot::unavailable(
                Some(recovery.project_id.clone()),
                Some(GitMutationOperation::Revert),
                GitDiagnosticCode::PostconditionFailed,
            );
        }
        let restored = hash_worktree(&root, &recovery.path, &recovery.repo_path).await;
        if restored.as_deref() != Ok(recovery.original_oid.as_str()) {
            return GitMutationResultSnapshot::unavailable(
                Some(recovery.project_id.clone()),
                Some(GitMutationOperation::Revert),
                GitDiagnosticCode::PostconditionFailed,
            );
        }
        let workspace = workspace_from_root(recovery.project_id.clone(), &root).await;
        GitMutationResultSnapshot {
            schema_version: GIT_MUTATION_SCHEMA_VERSION,
            state: GitMutationResultState::Applied,
            project_id: Some(recovery.project_id.clone()),
            operation: Some(GitMutationOperation::Revert),
            recovery_id: None,
            workspace: Some(workspace),
            diagnostic_code: None,
        }
    }
}

fn valid_request_shape(request: &GitMutationPreviewRequest) -> bool {
    match request.operation {
        GitMutationOperation::Commit => {
            request.path.is_none() && request.message.as_deref().is_some_and(valid_commit_message)
        }
        GitMutationOperation::Stage
        | GitMutationOperation::Unstage
        | GitMutationOperation::Revert => {
            request.message.is_none() && request.path.as_deref().is_some_and(valid_relative_path)
        }
    }
}

fn valid_commit_message(message: &str) -> bool {
    !message.is_empty()
        && message.trim() == message
        && message.chars().count() <= MAX_COMMIT_MESSAGE_CHARACTERS
        && !message.chars().any(unsafe_display_character)
}

fn valid_token(value: &str) -> bool {
    Uuid::parse_str(value).is_ok_and(|uuid| uuid.get_version_num() == 7)
}

fn mutation_root_writable(root: &ProjectReviewRoot) -> bool {
    root.writable
        && writable_directory(&root.attached_root)
        && writable_directory(&root.git_dir)
        && writable_directory(&root.common_dir)
}

fn writable_directory(path: &Path) -> bool {
    fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_dir() && metadata.permissions().mode() & 0o222 != 0)
}

fn clear_expired(state: &mut MutationState) {
    let now = Instant::now();
    state.pending.retain(|_, pending| pending.expires_at > now);
    state
        .recoveries
        .retain(|_, recovery| recovery.expires_at > now);
}

async fn prepare_action(
    request: &GitMutationPreviewRequest,
    root: &ProjectReviewRoot,
) -> Result<PreparedAction, PreparationFailure> {
    match request.operation {
        GitMutationOperation::Stage => prepare_stage(request, root).await,
        GitMutationOperation::Unstage => prepare_unstage(request, root).await,
        GitMutationOperation::Revert => prepare_revert(request, root).await,
        GitMutationOperation::Commit => prepare_commit(request, root).await,
    }
}

async fn prepare_stage(
    request: &GitMutationPreviewRequest,
    root: &ProjectReviewRoot,
) -> Result<PreparedAction, PreparationFailure> {
    let path = request.path.as_deref().expect("validated stage path");
    let target = target_evidence(root, path, true)
        .await
        .map_err(unavailable)?;
    if !target.change.reviewable
        || !matches!(
            target.change.worktree,
            Some(
                GitChangeKind::Modified
                    | GitChangeKind::Added
                    | GitChangeKind::Deleted
                    | GitChangeKind::Untracked
            )
        )
        || (target.change.worktree != Some(GitChangeKind::Deleted)
            && (target.worktree_oid.is_none() || target.worktree_mode.is_none()))
    {
        return Err(unavailable(GitDiagnosticCode::MutationUnavailable));
    }
    Ok(PreparedAction {
        targets: vec![mutation_target(&target.change)],
        action: PendingAction::Stage(target),
    })
}

async fn prepare_unstage(
    request: &GitMutationPreviewRequest,
    root: &ProjectReviewRoot,
) -> Result<PreparedAction, PreparationFailure> {
    let path = request.path.as_deref().expect("validated unstage path");
    let target = target_evidence(root, path, false)
        .await
        .map_err(unavailable)?;
    if !target.change.reviewable
        || !matches!(
            target.change.staged,
            Some(GitChangeKind::Modified | GitChangeKind::Added | GitChangeKind::Deleted)
        )
    {
        return Err(unavailable(GitDiagnosticCode::MutationUnavailable));
    }
    let head_entry = head_index_entry(root, &target.repo_path)
        .await
        .map_err(unavailable)?;
    Ok(PreparedAction {
        targets: vec![mutation_target(&target.change)],
        action: PendingAction::Unstage(UnstageEvidence { target, head_entry }),
    })
}

async fn prepare_revert(
    request: &GitMutationPreviewRequest,
    root: &ProjectReviewRoot,
) -> Result<PreparedAction, PreparationFailure> {
    let path = request.path.as_deref().expect("validated revert path");
    let target = target_evidence(root, path, true)
        .await
        .map_err(unavailable)?;
    if !target.change.reviewable
        || target.change.worktree != Some(GitChangeKind::Modified)
        || target.index_entry.is_none()
        || target.worktree_oid.is_none()
    {
        return Err(unavailable(GitDiagnosticCode::MutationUnavailable));
    }
    let backup =
        read_backup(root, path).map_err(|_| unavailable(GitDiagnosticCode::RecoveryUnavailable))?;
    Ok(PreparedAction {
        targets: vec![mutation_target(&target.change)],
        action: PendingAction::Revert(RevertEvidence { target, backup }),
    })
}

async fn prepare_commit(
    request: &GitMutationPreviewRequest,
    root: &ProjectReviewRoot,
) -> Result<PreparedAction, PreparationFailure> {
    let (evidence, targets) = commit_evidence(root).await.map_err(unavailable)?;
    let findings = scan_commit_secrets(root, &evidence).await;
    let mut findings = match findings {
        Ok(findings) => findings,
        Err(diagnostic) => {
            return Err(PreparationFailure {
                diagnostic,
                blocked: true,
                targets,
                findings: Vec::new(),
            });
        }
    };
    for kind in secret_kinds(
        request
            .message
            .as_deref()
            .expect("validated commit message")
            .as_bytes(),
    ) {
        push_finding(
            &mut findings,
            GitSecretFindingLocation::CommitMessage,
            None,
            kind,
        );
    }
    if !findings.is_empty() {
        return Err(PreparationFailure {
            diagnostic: GitDiagnosticCode::SecretDetected,
            blocked: true,
            targets,
            findings,
        });
    }
    Ok(PreparedAction {
        targets,
        action: PendingAction::Commit(CommitPlan {
            message: request.message.clone().expect("validated commit message"),
            evidence,
        }),
    })
}

fn unavailable(diagnostic: GitDiagnosticCode) -> PreparationFailure {
    PreparationFailure {
        diagnostic,
        blocked: false,
        targets: Vec::new(),
        findings: Vec::new(),
    }
}

fn mutation_target(change: &GitFileChange) -> GitMutationTarget {
    GitMutationTarget {
        path: change.path.clone(),
        staged: change.staged,
        worktree: change.worktree,
    }
}

async fn target_evidence(
    root: &ProjectReviewRoot,
    path: &str,
    inspect_attributes: bool,
) -> Result<TargetEvidence, GitDiagnosticCode> {
    let (_, changes, truncated) = inspect_status(&root.attached_root, &root.worktree_root)
        .await
        .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if truncated {
        return Err(GitDiagnosticCode::OutputTooLarge);
    }
    let change = changes
        .into_iter()
        .find(|change| change.path == path)
        .ok_or(GitDiagnosticCode::StalePreview)?;
    let repo_path = repository_path(root, path)?;
    if inspect_attributes && change.worktree != Some(GitChangeKind::Deleted) {
        ensure_safe_attributes(root, &repo_path).await?;
    }
    let index_entry = index_entry(root, &repo_path).await?;
    let (worktree_oid, worktree_mode) = if change.worktree == Some(GitChangeKind::Deleted) {
        (None, None)
    } else if safe_worktree_file(&root.attached_root, path) {
        let mode = root
            .attached_root
            .join(path)
            .symlink_metadata()
            .map_err(|_| GitDiagnosticCode::StalePreview)?
            .permissions()
            .mode()
            & 0o7777;
        (
            Some(hash_worktree(root, path, &repo_path).await?),
            Some(mode),
        )
    } else {
        (None, None)
    };
    Ok(TargetEvidence {
        path: path.to_owned(),
        repo_path,
        change,
        index_entry,
        worktree_oid,
        worktree_mode,
    })
}

fn repository_path(root: &ProjectReviewRoot, path: &str) -> Result<String, GitDiagnosticCode> {
    let scope = root
        .attached_root
        .strip_prefix(&root.worktree_root)
        .map_err(|_| GitDiagnosticCode::IdentityChanged)?;
    let scope = scope.to_str().ok_or(GitDiagnosticCode::IdentityChanged)?;
    let repo_path = if scope.is_empty() {
        path.to_owned()
    } else {
        format!("{scope}/{path}")
    };
    valid_relative_path(&repo_path)
        .then_some(repo_path)
        .ok_or(GitDiagnosticCode::InvalidPath)
}

async fn ensure_safe_attributes(
    root: &ProjectReviewRoot,
    repo_path: &str,
) -> Result<(), GitDiagnosticCode> {
    let output = run_git(
        &root.worktree_root,
        &[
            "check-attr",
            "-z",
            "filter",
            "working-tree-encoding",
            "--",
            repo_path,
        ],
        MAX_GIT_METADATA_BYTES,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !output.success {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    let fields: Vec<&[u8]> = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty())
        .collect();
    if fields.len() != 6 {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    for chunk in fields.chunks_exact(3) {
        if chunk[0] != repo_path.as_bytes()
            || !matches!(chunk[1], b"filter" | b"working-tree-encoding")
            || !matches!(chunk[2], b"unspecified" | b"unset")
        {
            return Err(GitDiagnosticCode::MutationUnavailable);
        }
    }
    Ok(())
}

async fn hash_worktree(
    root: &ProjectReviewRoot,
    path: &str,
    repo_path: &str,
) -> Result<String, GitDiagnosticCode> {
    if !safe_worktree_file(&root.attached_root, path) {
        return Err(GitDiagnosticCode::InvalidPath);
    }
    let path_option = format!("--path={repo_path}");
    let output = run_git(
        &root.worktree_root,
        &["hash-object", &path_option, "--", repo_path],
        128,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !output.success {
        return Err(GitDiagnosticCode::GitFailed);
    }
    parse_oid(&output.stdout).ok_or(GitDiagnosticCode::GitFailed)
}

async fn index_entry(
    root: &ProjectReviewRoot,
    repo_path: &str,
) -> Result<Option<IndexEntry>, GitDiagnosticCode> {
    let output = run_git(
        &root.worktree_root,
        &["ls-files", "--stage", "-z", "--", repo_path],
        MAX_GIT_METADATA_BYTES,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !output.success {
        return Err(GitDiagnosticCode::GitFailed);
    }
    parse_index_entry(&output.stdout, repo_path)
}

fn parse_index_entry(
    bytes: &[u8],
    expected_path: &str,
) -> Result<Option<IndexEntry>, GitDiagnosticCode> {
    let records: Vec<&[u8]> = bytes
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .collect();
    if records.is_empty() {
        return Ok(None);
    }
    if records.len() != 1 {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    let record = std::str::from_utf8(records[0]).map_err(|_| GitDiagnosticCode::InvalidPath)?;
    let (metadata, path) = record
        .split_once('\t')
        .ok_or(GitDiagnosticCode::GitFailed)?;
    if path != expected_path {
        return Err(GitDiagnosticCode::InvalidPath);
    }
    let fields: Vec<&str> = metadata.split_whitespace().collect();
    if fields.len() != 3
        || fields[2] != "0"
        || !valid_index_mode(fields[0])
        || !valid_oid(fields[1])
    {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    Ok(Some(IndexEntry {
        mode: fields[0].to_owned(),
        oid: fields[1].to_owned(),
    }))
}

fn valid_index_mode(value: &str) -> bool {
    matches!(value, "100644" | "100755" | "120000" | "160000")
}

async fn head_oid(root: &ProjectReviewRoot) -> Result<Option<String>, GitDiagnosticCode> {
    let output = run_git(&root.worktree_root, &["rev-parse", "--verify", "HEAD"], 128)
        .await
        .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !output.success {
        return Ok(None);
    }
    parse_oid(&output.stdout)
        .map(Some)
        .ok_or(GitDiagnosticCode::GitFailed)
}

async fn head_index_entry(
    root: &ProjectReviewRoot,
    repo_path: &str,
) -> Result<Option<IndexEntry>, GitDiagnosticCode> {
    if head_oid(root).await?.is_none() {
        return Ok(None);
    }
    let output = run_git(
        &root.worktree_root,
        &["ls-tree", "-z", "HEAD", "--", repo_path],
        MAX_GIT_METADATA_BYTES,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !output.success {
        return Err(GitDiagnosticCode::GitFailed);
    }
    let records: Vec<&[u8]> = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
        .collect();
    if records.is_empty() {
        return Ok(None);
    }
    if records.len() != 1 {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    let record = std::str::from_utf8(records[0]).map_err(|_| GitDiagnosticCode::InvalidPath)?;
    let (metadata, path) = record
        .split_once('\t')
        .ok_or(GitDiagnosticCode::GitFailed)?;
    let fields: Vec<&str> = metadata.split_whitespace().collect();
    if path != repo_path
        || fields.len() != 3
        || fields[1] != "blob"
        || !valid_index_mode(fields[0])
        || !valid_oid(fields[2])
    {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    Ok(Some(IndexEntry {
        mode: fields[0].to_owned(),
        oid: fields[2].to_owned(),
    }))
}

fn parse_oid(bytes: &[u8]) -> Option<String> {
    let value = std::str::from_utf8(bytes).ok()?.trim();
    valid_oid(value).then(|| value.to_owned())
}

fn valid_oid(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn read_backup(root: &ProjectReviewRoot, path: &str) -> Result<FileBackup, ()> {
    let candidate = root.attached_root.join(path);
    let metadata = candidate.symlink_metadata().map_err(|_| ())?;
    if metadata.file_type().is_symlink()
        || !metadata.is_file()
        || metadata.len() > MAX_RECOVERY_BYTES
        || metadata.permissions().mode() & 0o7000 != 0
    {
        return Err(());
    }
    let resolved = candidate.canonicalize().map_err(|_| ())?;
    if !resolved.starts_with(&root.attached_root) {
        return Err(());
    }
    Ok(FileBackup {
        bytes: fs::read(candidate).map_err(|_| ())?,
        mode: metadata.permissions().mode() & 0o777,
    })
}

fn restore_backup(root: &ProjectReviewRoot, path: &str, backup: &FileBackup) -> Result<(), ()> {
    let candidate = root.attached_root.join(path);
    let parent = candidate.parent().ok_or(())?;
    let parent_resolved = parent.canonicalize().map_err(|_| ())?;
    if !parent_resolved.starts_with(&root.attached_root) || !parent_resolved.is_dir() {
        return Err(());
    }
    if !safe_worktree_file(&root.attached_root, path) {
        return Err(());
    }

    let temporary = parent.join(format!(".quireforge-recovery-{}.tmp", Uuid::now_v7()));
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|_| ())?;
        file.write_all(&backup.bytes).map_err(|_| ())?;
        file.sync_all().map_err(|_| ())?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(backup.mode)).map_err(|_| ())?;
        if !safe_worktree_file(&root.attached_root, path) {
            return Err(());
        }
        fs::rename(&temporary, &candidate).map_err(|_| ())?;
        fs::File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|_| ())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

async fn commit_evidence(
    root: &ProjectReviewRoot,
) -> Result<(CommitEvidence, Vec<GitMutationTarget>), GitDiagnosticCode> {
    if repository_operation_in_progress(root).await? {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    let (_, root_changes, truncated) = inspect_status(&root.worktree_root, &root.worktree_root)
        .await
        .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if truncated {
        return Err(GitDiagnosticCode::OutputTooLarge);
    }
    let scope = root
        .attached_root
        .strip_prefix(&root.worktree_root)
        .map_err(|_| GitDiagnosticCode::IdentityChanged)?
        .to_str()
        .ok_or(GitDiagnosticCode::IdentityChanged)?;
    let mut staged = Vec::new();
    let mut targets = Vec::new();
    for change in root_changes
        .into_iter()
        .filter(|change| change.staged.is_some())
    {
        if !change.reviewable || change.submodule || change.conflict {
            return Err(GitDiagnosticCode::MutationUnavailable);
        }
        let path =
            scoped_status_path(&change.path, scope).ok_or(GitDiagnosticCode::OutsideAttachment)?;
        let previous_path = match change.previous_path.as_deref() {
            Some(previous) => Some(
                scoped_status_path(previous, scope).ok_or(GitDiagnosticCode::OutsideAttachment)?,
            ),
            None => None,
        };
        let relative_change = GitFileChange {
            path,
            previous_path,
            staged: change.staged,
            worktree: change.worktree,
            conflict: change.conflict,
            submodule: change.submodule,
            reviewable: change.reviewable,
        };
        let index_entry = index_entry(root, &change.path).await?;
        if change.staged != Some(GitChangeKind::Deleted) && index_entry.is_none() {
            return Err(GitDiagnosticCode::MutationUnavailable);
        }
        if index_entry
            .as_ref()
            .is_some_and(|entry| entry.mode == "160000")
        {
            return Err(GitDiagnosticCode::MutationUnavailable);
        }
        targets.push(mutation_target(&relative_change));
        staged.push(CommitTargetEvidence {
            change: relative_change,
            repo_path: change.path,
            index_entry,
        });
    }
    if staged.is_empty() {
        return Err(GitDiagnosticCode::MutationUnavailable);
    }
    staged.sort_by(|left, right| left.repo_path.cmp(&right.repo_path));
    targets.sort_by(|left, right| left.path.cmp(&right.path));
    let (identity_name, identity_email) = repository_identity(root).await?;
    Ok((
        CommitEvidence {
            head_oid: head_oid(root).await?,
            identity_name,
            identity_email,
            staged,
        },
        targets,
    ))
}

async fn repository_operation_in_progress(
    root: &ProjectReviewRoot,
) -> Result<bool, GitDiagnosticCode> {
    for reference in ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD"] {
        let output = run_git(
            &root.worktree_root,
            &["rev-parse", "--verify", "-q", reference],
            128,
        )
        .await
        .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
        if output.success {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn repository_identity(
    root: &ProjectReviewRoot,
) -> Result<(String, String), GitDiagnosticCode> {
    let name = local_config_value(root, "user.name").await?;
    let email = local_config_value(root, "user.email").await?;
    if !valid_identity_value(&name) || !valid_identity_value(&email) || !email.contains('@') {
        return Err(GitDiagnosticCode::IdentityUnavailable);
    }
    Ok((name, email))
}

async fn local_config_value(
    root: &ProjectReviewRoot,
    key: &str,
) -> Result<String, GitDiagnosticCode> {
    let output = run_git(
        &root.worktree_root,
        &["config", "--local", "--get", key],
        1024,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !output.success {
        return Err(GitDiagnosticCode::IdentityUnavailable);
    }
    std::str::from_utf8(&output.stdout)
        .ok()
        .map(str::trim)
        .filter(|value| valid_identity_value(value))
        .map(str::to_owned)
        .ok_or(GitDiagnosticCode::IdentityUnavailable)
}

fn valid_identity_value(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= 256
        && !value.chars().any(unsafe_display_character)
}

async fn scan_commit_secrets(
    root: &ProjectReviewRoot,
    evidence: &CommitEvidence,
) -> Result<Vec<GitSecretFinding>, GitDiagnosticCode> {
    let mut findings = Vec::new();
    let mut scanned = 0_usize;
    for target in &evidence.staged {
        let Some(entry) = &target.index_entry else {
            continue;
        };
        if forbidden_secret_path(&target.change.path) {
            push_finding(
                &mut findings,
                GitSecretFindingLocation::StagedFile,
                Some(&target.change.path),
                GitSecretFindingKind::ForbiddenPath,
            );
        }
        let size_output = run_git(&root.worktree_root, &["cat-file", "-s", &entry.oid], 128)
            .await
            .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
        let size = std::str::from_utf8(&size_output.stdout)
            .ok()
            .map(str::trim)
            .and_then(|value| value.parse::<usize>().ok())
            .ok_or(GitDiagnosticCode::UnscannableContent)?;
        if !size_output.success
            || size > MAX_SCANNED_BLOB_BYTES
            || scanned.saturating_add(size) > MAX_SCANNED_TOTAL_BYTES
        {
            return Err(GitDiagnosticCode::UnscannableContent);
        }
        let blob = run_git(
            &root.worktree_root,
            &["cat-file", "blob", &entry.oid],
            MAX_SCANNED_BLOB_BYTES,
        )
        .await
        .map_err(|error| map_run_error(error, GitDiagnosticCode::UnscannableContent))?;
        if !blob.success || blob.stdout.len() != size {
            return Err(GitDiagnosticCode::UnscannableContent);
        }
        scanned += size;
        for kind in secret_kinds(&blob.stdout) {
            push_finding(
                &mut findings,
                GitSecretFindingLocation::StagedFile,
                Some(&target.change.path),
                kind,
            );
        }
    }
    Ok(findings)
}

fn push_finding(
    findings: &mut Vec<GitSecretFinding>,
    location: GitSecretFindingLocation,
    path: Option<&str>,
    kind: GitSecretFindingKind,
) {
    if findings.len() < MAX_SECRET_FINDINGS
        && !findings.iter().any(|finding| {
            finding.location == location && finding.path.as_deref() == path && finding.kind == kind
        })
    {
        findings.push(GitSecretFinding {
            location,
            path: path.map(str::to_owned),
            kind,
        });
    }
}

fn forbidden_secret_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.split('/').any(|part| {
        part == ".env"
            || part.starts_with(".env.")
            || matches!(part, "id_rsa" | "id_dsa" | "id_ecdsa" | "id_ed25519")
            || [".key", ".pem", ".p12", ".pfx"]
                .iter()
                .any(|suffix| part.ends_with(suffix))
    })
}

fn secret_kinds(bytes: &[u8]) -> Vec<GitSecretFindingKind> {
    let mut kinds = Vec::new();
    if contains_private_key(bytes) {
        kinds.push(GitSecretFindingKind::PrivateKey);
    }
    if contains_prefixed_token(
        bytes,
        &[
            b"ghp_".as_slice(),
            b"gho_".as_slice(),
            b"ghu_".as_slice(),
            b"ghs_".as_slice(),
            b"ghr_".as_slice(),
        ],
        20,
    ) {
        kinds.push(GitSecretFindingKind::GitHubToken);
    }
    if contains_openai_key(bytes) {
        kinds.push(GitSecretFindingKind::OpenAiApiKey);
    }
    kinds
}

fn contains_private_key(bytes: &[u8]) -> bool {
    bytes
        .windows(b"-----BEGIN ".len())
        .any(|window| window == b"-----BEGIN ")
        && bytes
            .windows(b"PRIVATE KEY-----".len())
            .any(|window| window == b"PRIVATE KEY-----")
}

fn contains_prefixed_token(bytes: &[u8], prefixes: &[&[u8]], minimum: usize) -> bool {
    prefixes.iter().any(|prefix| {
        bytes
            .windows(prefix.len())
            .enumerate()
            .any(|(index, window)| {
                window == *prefix
                    && bytes[index + prefix.len()..]
                        .iter()
                        .take_while(|byte| byte.is_ascii_alphanumeric() || **byte == b'_')
                        .count()
                        >= minimum
            })
    })
}

fn contains_openai_key(bytes: &[u8]) -> bool {
    bytes.windows(3).enumerate().any(|(index, window)| {
        if window != b"sk-" {
            return false;
        }
        let remainder = &bytes[index + 3..];
        let remainder = remainder.strip_prefix(b"proj-").unwrap_or(remainder);
        remainder
            .iter()
            .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(**byte, b'_' | b'-'))
            .count()
            >= 20
    })
}

async fn execute_action(
    root: &ProjectReviewRoot,
    project_id: &str,
    action: PendingAction,
) -> Result<ExecutionSuccess, GitDiagnosticCode> {
    match action {
        PendingAction::Stage(target) => execute_stage(root, target).await,
        PendingAction::Unstage(unstage) => execute_unstage(root, unstage).await,
        PendingAction::Revert(revert) => execute_revert(root, project_id, revert).await,
        PendingAction::Commit(commit) => execute_commit(root, commit).await,
    }
}

async fn execute_stage(
    root: &ProjectReviewRoot,
    expected: TargetEvidence,
) -> Result<ExecutionSuccess, GitDiagnosticCode> {
    let current = target_evidence(root, &expected.path, true).await?;
    if current != expected {
        return Err(GitDiagnosticCode::StalePreview);
    }
    let output = run_git(
        &root.worktree_root,
        &["add", "--", &expected.repo_path],
        MAX_GIT_METADATA_BYTES,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !output.success {
        return Err(GitDiagnosticCode::GitFailed);
    }
    let post = target_evidence(root, &expected.path, false).await;
    let expected_oid = expected.worktree_oid.as_deref();
    let valid = post.as_ref().is_ok_and(|post| {
        post.change.worktree.is_none()
            && post.change.staged.is_some()
            && post.index_entry.as_ref().map(|entry| entry.oid.as_str()) == expected_oid
    });
    if !valid {
        let _ = apply_index_entry(root, &expected.repo_path, expected.index_entry.as_ref()).await;
        return Err(GitDiagnosticCode::PostconditionFailed);
    }
    Ok(ExecutionSuccess { recovery: None })
}

async fn execute_unstage(
    root: &ProjectReviewRoot,
    expected: UnstageEvidence,
) -> Result<ExecutionSuccess, GitDiagnosticCode> {
    let current = target_evidence(root, &expected.target.path, false).await?;
    if current != expected.target {
        return Err(GitDiagnosticCode::StalePreview);
    }
    apply_index_entry(
        root,
        &expected.target.repo_path,
        expected.head_entry.as_ref(),
    )
    .await?;
    let post_entry = index_entry(root, &expected.target.repo_path).await;
    let status = inspect_status(&root.attached_root, &root.worktree_root).await;
    let valid = post_entry
        .as_ref()
        .is_ok_and(|entry| entry == &expected.head_entry)
        && status.as_ref().is_ok_and(|(_, changes, truncated)| {
            !*truncated
                && changes
                    .iter()
                    .find(|change| change.path == expected.target.path)
                    .is_some_and(|change| change.staged.is_none())
        });
    if !valid {
        let _ = apply_index_entry(
            root,
            &expected.target.repo_path,
            expected.target.index_entry.as_ref(),
        )
        .await;
        return Err(GitDiagnosticCode::PostconditionFailed);
    }
    Ok(ExecutionSuccess { recovery: None })
}

async fn apply_index_entry(
    root: &ProjectReviewRoot,
    repo_path: &str,
    entry: Option<&IndexEntry>,
) -> Result<(), GitDiagnosticCode> {
    let output = if let Some(entry) = entry {
        run_git(
            &root.worktree_root,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                &entry.mode,
                &entry.oid,
                repo_path,
            ],
            MAX_GIT_METADATA_BYTES,
        )
        .await
    } else {
        run_git(
            &root.worktree_root,
            &["update-index", "--force-remove", "--", repo_path],
            MAX_GIT_METADATA_BYTES,
        )
        .await
    }
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    output
        .success
        .then_some(())
        .ok_or(GitDiagnosticCode::GitFailed)
}

async fn execute_revert(
    root: &ProjectReviewRoot,
    project_id: &str,
    expected: RevertEvidence,
) -> Result<ExecutionSuccess, GitDiagnosticCode> {
    let current = target_evidence(root, &expected.target.path, true).await?;
    if current != expected.target {
        return Err(GitDiagnosticCode::StalePreview);
    }
    let output = run_git(
        &root.worktree_root,
        &[
            "restore",
            "--worktree",
            "--no-recurse-submodules",
            "--",
            &expected.target.repo_path,
        ],
        MAX_GIT_METADATA_BYTES,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    let index_oid = expected
        .target
        .index_entry
        .as_ref()
        .map(|entry| entry.oid.clone())
        .ok_or(GitDiagnosticCode::MutationUnavailable)?;
    let original_oid = expected
        .target
        .worktree_oid
        .clone()
        .ok_or(GitDiagnosticCode::MutationUnavailable)?;
    let post_oid = hash_worktree(root, &expected.target.path, &expected.target.repo_path).await;
    let status = inspect_status(&root.attached_root, &root.worktree_root).await;
    let valid = output.success
        && post_oid.as_deref() == Ok(index_oid.as_str())
        && status.as_ref().is_ok_and(|(_, changes, truncated)| {
            !*truncated
                && changes
                    .iter()
                    .find(|change| change.path == expected.target.path)
                    .is_none_or(|change| change.worktree.is_none())
        });
    if !valid {
        let _ = restore_backup(root, &expected.target.path, &expected.backup);
        return Err(GitDiagnosticCode::PostconditionFailed);
    }
    let recovery_id = Uuid::now_v7().to_string();
    Ok(ExecutionSuccess {
        recovery: Some(CompletedRecovery {
            recovery_id,
            project_id: project_id.to_owned(),
            root: root.clone(),
            path: expected.target.path,
            repo_path: expected.target.repo_path,
            backup: expected.backup,
            reverted_oid: index_oid,
            original_oid,
            expires_at: Instant::now() + RECOVERY_TTL,
        }),
    })
}

async fn execute_commit(
    root: &ProjectReviewRoot,
    expected: CommitPlan,
) -> Result<ExecutionSuccess, GitDiagnosticCode> {
    drop(IndexLock::acquire(root)?);
    let (before_tree, _) = commit_evidence(root).await?;
    if before_tree != expected.evidence {
        return Err(GitDiagnosticCode::StalePreview);
    }
    let tree = successful_oid(
        run_git(&root.worktree_root, &["write-tree"], 128).await,
        GitDiagnosticCode::GitFailed,
    )?;
    let _lock = IndexLock::acquire(root)?;
    let (current, _) = commit_evidence(root).await?;
    if current != expected.evidence {
        return Err(GitDiagnosticCode::StalePreview);
    }
    let tree_matches_index = run_git(
        &root.worktree_root,
        &["diff-index", "--cached", "--quiet", &tree, "--"],
        MAX_GIT_METADATA_BYTES,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !tree_matches_index.success {
        return Err(GitDiagnosticCode::StalePreview);
    }
    let mut arguments = vec![
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "commit.gpgSign=false",
        "commit-tree",
        tree.as_str(),
    ];
    if let Some(head) = current.head_oid.as_deref() {
        arguments.extend(["-p", head]);
    }
    arguments.extend(["-m", expected.message.as_str()]);
    let commit = successful_oid(
        run_git(&root.worktree_root, &arguments, 128).await,
        GitDiagnosticCode::GitFailed,
    )?;
    let zero_oid = "0".repeat(commit.len());
    let old = current.head_oid.as_deref().unwrap_or(&zero_oid);
    let updated = run_git(
        &root.worktree_root,
        &[
            "-c",
            "core.hooksPath=/dev/null",
            "update-ref",
            "-m",
            "QuireForge commit",
            "HEAD",
            &commit,
            old,
        ],
        MAX_GIT_METADATA_BYTES,
    )
    .await
    .map_err(|error| map_run_error(error, GitDiagnosticCode::GitFailed))?;
    if !updated.success {
        return Err(GitDiagnosticCode::StalePreview);
    }
    let post_head = head_oid(root).await;
    let post_status = inspect_status(&root.worktree_root, &root.worktree_root).await;
    let valid = post_head
        .as_ref()
        .is_ok_and(|head| head.as_deref() == Some(commit.as_str()))
        && post_status.as_ref().is_ok_and(|(_, changes, truncated)| {
            !*truncated && changes.iter().all(|change| change.staged.is_none())
        });
    if !valid {
        let _ = rollback_head(root, current.head_oid.as_deref(), &commit).await;
        return Err(GitDiagnosticCode::PostconditionFailed);
    }
    Ok(ExecutionSuccess { recovery: None })
}

fn successful_oid(
    output: Result<super::GitOutput, GitRunError>,
    fallback: GitDiagnosticCode,
) -> Result<String, GitDiagnosticCode> {
    let output = output.map_err(|error| map_run_error(error, fallback))?;
    if !output.success {
        return Err(fallback);
    }
    parse_oid(&output.stdout).ok_or(fallback)
}

async fn rollback_head(
    root: &ProjectReviewRoot,
    old: Option<&str>,
    current: &str,
) -> Result<(), GitDiagnosticCode> {
    let arguments = if let Some(old) = old {
        vec![
            "-c",
            "core.hooksPath=/dev/null",
            "update-ref",
            "-m",
            "QuireForge commit rollback",
            "HEAD",
            old,
            current,
        ]
    } else {
        vec![
            "-c",
            "core.hooksPath=/dev/null",
            "update-ref",
            "-d",
            "HEAD",
            current,
        ]
    };
    let output = run_git(&root.worktree_root, &arguments, MAX_GIT_METADATA_BYTES)
        .await
        .map_err(|error| map_run_error(error, GitDiagnosticCode::PostconditionFailed))?;
    output
        .success
        .then_some(())
        .ok_or(GitDiagnosticCode::PostconditionFailed)
}

impl IndexLock {
    fn acquire(root: &ProjectReviewRoot) -> Result<Self, GitDiagnosticCode> {
        let git_dir = root
            .git_dir
            .canonicalize()
            .map_err(|_| GitDiagnosticCode::IdentityChanged)?;
        if git_dir != root.git_dir || !git_dir.is_dir() {
            return Err(GitDiagnosticCode::IdentityChanged);
        }
        let path = git_dir.join("index.lock");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .map_err(|_| GitDiagnosticCode::ProjectBusy)?;
        if file
            .write_all(b"QuireForge holds this index lock during an exact commit.\n")
            .and_then(|()| file.sync_all())
            .is_err()
        {
            drop(file);
            let _ = fs::remove_file(&path);
            return Err(GitDiagnosticCode::GitFailed);
        }
        let metadata = match file.metadata() {
            Ok(metadata) => metadata,
            Err(_) => {
                drop(file);
                let _ = fs::remove_file(&path);
                return Err(GitDiagnosticCode::GitFailed);
            }
        };
        Ok(Self {
            path,
            device: metadata.dev(),
            inode: metadata.ino(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
        process::{Command, Output},
    };

    use super::*;
    use crate::project::ProjectService;

    struct TestRepository {
        root: PathBuf,
    }

    impl TestRepository {
        fn new(with_identity: bool) -> Self {
            let root = std::env::temp_dir()
                .join(format!("quireforge-git-mutation-test-{}", Uuid::now_v7()));
            fs::create_dir(&root).expect("temporary repository directory must be created");
            let repository = Self { root };
            repository.git_success(&["init", "--quiet"]);
            if with_identity {
                repository.git_success(&["config", "--local", "user.name", "QuireForge Test"]);
                repository.git_success(&[
                    "config",
                    "--local",
                    "user.email",
                    "quireforge@example.invalid",
                ]);
            }
            repository
        }

        fn path(&self, relative: &str) -> PathBuf {
            self.root.join(relative)
        }

        fn write(&self, relative: &str, contents: &str) {
            let path = self.path(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("test file parent must be created");
            }
            fs::write(path, contents).expect("test file must be written");
        }

        fn git(&self, arguments: &[&str]) -> Output {
            Command::new("git")
                .args(arguments)
                .current_dir(&self.root)
                .env("LC_ALL", "C")
                .output()
                .expect("git must start for the mutation test")
        }

        fn git_success(&self, arguments: &[&str]) -> Output {
            let output = self.git(arguments);
            assert!(
                output.status.success(),
                "git {arguments:?} failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            output
        }

        fn commit_all(&self, message: &str) {
            self.git_success(&["add", "--all"]);
            self.git_success(&["commit", "--quiet", "-m", message]);
        }

        fn attach(&self, selected: &Path) -> (ProjectService, String) {
            let service = ProjectService::in_memory();
            service.prepare_attachment(selected.to_path_buf());
            let snapshot = service.confirm_pending();
            assert_eq!(snapshot.projects.len(), 1);
            (service, snapshot.projects[0].id.clone())
        }
    }

    impl Drop for TestRepository {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn preview_request(
        project_id: &str,
        operation: GitMutationOperation,
        path: Option<&str>,
        message: Option<&str>,
    ) -> GitMutationPreviewRequest {
        GitMutationPreviewRequest {
            project_id: project_id.to_owned(),
            operation,
            path: path.map(str::to_owned),
            message: message.map(str::to_owned),
        }
    }

    #[test]
    fn unavailable_snapshots_match_the_strict_frontend_fixtures() {
        let request = preview_request(
            "018f0000-0000-7000-8000-000000000001",
            GitMutationOperation::Stage,
            Some("README.md"),
            None,
        );
        let preview = GitMutationPreviewSnapshot::unavailable(
            &request,
            request.path.clone(),
            GitDiagnosticCode::MutationUnavailable,
        );
        let result = GitMutationResultSnapshot::unavailable(
            None,
            None,
            GitDiagnosticCode::ConfirmationExpired,
        );
        let preview_fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/git-mutation-preview.json"))
                .unwrap();
        let result_fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/git-mutation-result.json"))
                .unwrap();

        assert_eq!(serde_json::to_value(preview).unwrap(), preview_fixture);
        assert_eq!(serde_json::to_value(result).unwrap(), result_fixture);
    }

    #[test]
    fn mutation_requests_reject_unknown_fields_and_non_v7_tokens() {
        assert!(
            serde_json::from_value::<GitMutationPreviewRequest>(serde_json::json!({
                "projectId": "018f0000-0000-7000-8000-000000000001",
                "operation": "stage",
                "path": "README.md",
                "message": null,
                "arguments": ["add", "--all"]
            }))
            .is_err()
        );
        assert!(
            serde_json::from_value::<GitMutationConfirmRequest>(serde_json::json!({
                "confirmationId": "018f0000-0000-4000-8000-000000000001"
            }))
            .is_ok()
        );
        assert!(!valid_token("018f0000-0000-4000-8000-000000000001"));
    }

    #[test]
    fn index_lock_cleanup_never_removes_a_replacement_lock() {
        let repository = TestRepository::new(true);
        let (projects, project_id) = repository.attach(&repository.root);
        let root = projects.review_root(&project_id).unwrap();
        let lock = IndexLock::acquire(&root).unwrap();
        let lock_path = repository.path(".git/index.lock");
        fs::remove_file(&lock_path).unwrap();
        fs::write(&lock_path, "replacement lock\n").unwrap();

        drop(lock);

        assert_eq!(fs::read_to_string(lock_path).unwrap(), "replacement lock\n");
    }

    async fn preview_and_confirm(
        coordinator: &MutationCoordinator,
        projects: &ProjectService,
        request: GitMutationPreviewRequest,
    ) -> GitMutationResultSnapshot {
        let preview = coordinator.preview(request, projects).await;
        assert_eq!(preview.state, GitMutationPreviewState::Ready);
        coordinator
            .confirm(
                GitMutationConfirmRequest {
                    confirmation_id: preview
                        .confirmation_id
                        .expect("ready preview must have a confirmation token"),
                },
                projects,
            )
            .await
    }

    #[tokio::test]
    async fn stage_unstage_revert_and_recovery_preserve_exact_content() {
        let repository = TestRepository::new(true);
        repository.write("tracked.txt", "base\n");
        repository.commit_all("initial");
        repository.write("tracked.txt", "changed\n");
        let (projects, project_id) = repository.attach(&repository.root);
        let coordinator = MutationCoordinator::default();

        let stale_preview = coordinator
            .preview(
                preview_request(
                    &project_id,
                    GitMutationOperation::Stage,
                    Some("tracked.txt"),
                    None,
                ),
                &projects,
            )
            .await;
        repository.write("tracked.txt", "changed after preview\n");
        let stale = coordinator
            .confirm(
                GitMutationConfirmRequest {
                    confirmation_id: stale_preview.confirmation_id.unwrap(),
                },
                &projects,
            )
            .await;
        assert_eq!(stale.state, GitMutationResultState::Unavailable);
        assert_eq!(stale.diagnostic_code, Some(GitDiagnosticCode::StalePreview));
        assert!(repository
            .git(&["diff", "--cached", "--quiet"])
            .status
            .success());

        repository.write("tracked.txt", "changed\n");
        let staged = preview_and_confirm(
            &coordinator,
            &projects,
            preview_request(
                &project_id,
                GitMutationOperation::Stage,
                Some("tracked.txt"),
                None,
            ),
        )
        .await;
        assert_eq!(staged.state, GitMutationResultState::Applied);
        assert!(!repository
            .git(&["diff", "--cached", "--quiet"])
            .status
            .success());
        assert!(repository.git(&["diff", "--quiet"]).status.success());

        let unstaged = preview_and_confirm(
            &coordinator,
            &projects,
            preview_request(
                &project_id,
                GitMutationOperation::Unstage,
                Some("tracked.txt"),
                None,
            ),
        )
        .await;
        assert_eq!(unstaged.state, GitMutationResultState::Applied);
        assert!(repository
            .git(&["diff", "--cached", "--quiet"])
            .status
            .success());
        assert!(!repository.git(&["diff", "--quiet"]).status.success());

        let reverted = preview_and_confirm(
            &coordinator,
            &projects,
            preview_request(
                &project_id,
                GitMutationOperation::Revert,
                Some("tracked.txt"),
                None,
            ),
        )
        .await;
        assert_eq!(reverted.state, GitMutationResultState::Applied);
        assert_eq!(
            fs::read_to_string(repository.path("tracked.txt")).unwrap(),
            "base\n"
        );
        let recovery_id = reverted
            .recovery_id
            .expect("a completed revert must be recoverable");

        repository.write("tracked.txt", "newer work\n");
        let stale_recovery = coordinator
            .recover(
                GitRecoveryRequest {
                    recovery_id: recovery_id.clone(),
                },
                &projects,
            )
            .await;
        assert_eq!(stale_recovery.state, GitMutationResultState::Unavailable);
        assert_eq!(
            stale_recovery.diagnostic_code,
            Some(GitDiagnosticCode::StalePreview)
        );
        assert_eq!(
            fs::read_to_string(repository.path("tracked.txt")).unwrap(),
            "newer work\n"
        );
        repository.write("tracked.txt", "base\n");
        let recovered = coordinator
            .recover(
                GitRecoveryRequest {
                    recovery_id: recovery_id.clone(),
                },
                &projects,
            )
            .await;
        assert_eq!(recovered.state, GitMutationResultState::Applied);
        assert_eq!(
            fs::read_to_string(repository.path("tracked.txt")).unwrap(),
            "changed\n"
        );
        assert!(!repository.path(".quireforge-recovery").exists());

        let repeated = coordinator
            .recover(GitRecoveryRequest { recovery_id }, &projects)
            .await;
        assert_eq!(repeated.state, GitMutationResultState::Unavailable);
        assert_eq!(
            repeated.diagnostic_code,
            Some(GitDiagnosticCode::RecoveryUnavailable)
        );

        repository.write("untracked.txt", "never delete\n");
        let untracked_revert = coordinator
            .preview(
                preview_request(
                    &project_id,
                    GitMutationOperation::Revert,
                    Some("untracked.txt"),
                    None,
                ),
                &projects,
            )
            .await;
        assert_eq!(untracked_revert.state, GitMutationPreviewState::Unavailable);
        assert_eq!(
            fs::read_to_string(repository.path("untracked.txt")).unwrap(),
            "never delete\n"
        );
    }

    #[tokio::test]
    async fn commit_uses_the_reviewed_index_without_hooks_or_worktree_loss() {
        let repository = TestRepository::new(true);
        repository.write("staged.txt", "base\n");
        repository.write("unstaged.txt", "base\n");
        repository.commit_all("initial");
        repository.write("staged.txt", "staged change\n");
        repository.write("unstaged.txt", "unstaged change\n");
        repository.git_success(&["add", "--", "staged.txt"]);
        repository.git_success(&["config", "--local", "commit.gpgSign", "true"]);

        let marker = repository.path("hook-ran");
        let hook = repository.path(".git/hooks/post-commit");
        fs::write(&hook, format!("#!/bin/sh\ntouch '{}'\n", marker.display())).unwrap();
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();

        let old_head =
            String::from_utf8(repository.git_success(&["rev-parse", "HEAD"]).stdout).unwrap();
        let (projects, project_id) = repository.attach(&repository.root);
        let result = preview_and_confirm(
            &MutationCoordinator::default(),
            &projects,
            preview_request(
                &project_id,
                GitMutationOperation::Commit,
                None,
                Some("Reviewed commit"),
            ),
        )
        .await;

        assert_eq!(
            result.state,
            GitMutationResultState::Applied,
            "commit failed with {:?}",
            result.diagnostic_code
        );
        assert!(!marker.exists(), "commit hooks must never run");
        assert_ne!(
            repository.git_success(&["rev-parse", "HEAD"]).stdout,
            old_head.as_bytes()
        );
        assert_eq!(
            String::from_utf8(repository.git_success(&["log", "-1", "--format=%s"]).stdout)
                .unwrap(),
            "Reviewed commit\n"
        );
        assert_eq!(
            String::from_utf8(repository.git_success(&["show", "HEAD:staged.txt"]).stdout).unwrap(),
            "staged change\n"
        );
        assert_eq!(
            fs::read_to_string(repository.path("unstaged.txt")).unwrap(),
            "unstaged change\n"
        );
        assert!(!repository.git(&["diff", "--quiet"]).status.success());
        assert!(!repository.path(".git/index.lock").exists());
    }

    #[tokio::test]
    async fn commit_blocks_secrets_missing_identity_and_paths_outside_the_attachment() {
        let secret_repository = TestRepository::new(true);
        let example_key = ["sk-proj-", "abcdefghijklmnopqrstuvwxyz"].concat();
        secret_repository.write(".env", &format!("OPENAI_API_KEY={example_key}\n"));
        secret_repository.git_success(&["add", "--", ".env"]);
        let (secret_projects, secret_project_id) =
            secret_repository.attach(&secret_repository.root);
        let secret = MutationCoordinator::default()
            .preview(
                preview_request(
                    &secret_project_id,
                    GitMutationOperation::Commit,
                    None,
                    Some("unsafe"),
                ),
                &secret_projects,
            )
            .await;
        assert_eq!(secret.state, GitMutationPreviewState::Blocked);
        assert_eq!(
            secret.diagnostic_code,
            Some(GitDiagnosticCode::SecretDetected)
        );
        assert_eq!(secret.secret_findings.len(), 2);
        assert!(secret.secret_findings.iter().all(|finding| {
            finding.location == GitSecretFindingLocation::StagedFile
                && finding.path.as_deref() == Some(".env")
        }));
        assert!(secret.confirmation_id.is_none());

        let message_repository = TestRepository::new(true);
        message_repository.write("safe.txt", "safe content\n");
        message_repository.git_success(&["add", "--", "safe.txt"]);
        let (message_projects, message_project_id) =
            message_repository.attach(&message_repository.root);
        let message_secret = MutationCoordinator::default()
            .preview(
                preview_request(
                    &message_project_id,
                    GitMutationOperation::Commit,
                    None,
                    Some(&example_key),
                ),
                &message_projects,
            )
            .await;
        assert_eq!(message_secret.state, GitMutationPreviewState::Blocked);
        assert_eq!(message_secret.secret_findings.len(), 1);
        assert_eq!(
            message_secret.secret_findings[0].location,
            GitSecretFindingLocation::CommitMessage
        );
        assert!(message_secret.secret_findings[0].path.is_none());

        let unidentified_repository = TestRepository::new(false);
        unidentified_repository.write("new.txt", "content\n");
        unidentified_repository.git_success(&["add", "--", "new.txt"]);
        let (unidentified_projects, unidentified_project_id) =
            unidentified_repository.attach(&unidentified_repository.root);
        let unidentified = MutationCoordinator::default()
            .preview(
                preview_request(
                    &unidentified_project_id,
                    GitMutationOperation::Commit,
                    None,
                    Some("missing identity"),
                ),
                &unidentified_projects,
            )
            .await;
        assert_eq!(unidentified.state, GitMutationPreviewState::Unavailable);
        assert_eq!(
            unidentified.diagnostic_code,
            Some(GitDiagnosticCode::IdentityUnavailable)
        );

        let scoped_repository = TestRepository::new(true);
        scoped_repository.write("inside/tracked.txt", "base\n");
        scoped_repository.write("outside.txt", "base\n");
        scoped_repository.commit_all("initial");
        scoped_repository.write("outside.txt", "outside staged change\n");
        scoped_repository.git_success(&["add", "--", "outside.txt"]);
        let (scoped_projects, scoped_project_id) =
            scoped_repository.attach(&scoped_repository.path("inside"));
        let outside = MutationCoordinator::default()
            .preview(
                preview_request(
                    &scoped_project_id,
                    GitMutationOperation::Commit,
                    None,
                    Some("outside"),
                ),
                &scoped_projects,
            )
            .await;
        assert_eq!(outside.state, GitMutationPreviewState::Unavailable);
        assert_eq!(
            outside.diagnostic_code,
            Some(GitDiagnosticCode::OutsideAttachment)
        );
    }

    #[tokio::test]
    async fn commit_respects_an_existing_index_lock_and_preserves_head() {
        let repository = TestRepository::new(true);
        repository.write("tracked.txt", "base\n");
        repository.commit_all("initial");
        repository.write("tracked.txt", "next\n");
        repository.git_success(&["add", "--", "tracked.txt"]);
        let head = repository.git_success(&["rev-parse", "HEAD"]).stdout;
        let (projects, project_id) = repository.attach(&repository.root);
        let coordinator = MutationCoordinator::default();
        let preview = coordinator
            .preview(
                preview_request(
                    &project_id,
                    GitMutationOperation::Commit,
                    None,
                    Some("locked"),
                ),
                &projects,
            )
            .await;
        fs::write(repository.path(".git/index.lock"), "held elsewhere\n").unwrap();
        let result = coordinator
            .confirm(
                GitMutationConfirmRequest {
                    confirmation_id: preview.confirmation_id.unwrap(),
                },
                &projects,
            )
            .await;

        assert_eq!(result.state, GitMutationResultState::Unavailable);
        assert_eq!(result.diagnostic_code, Some(GitDiagnosticCode::ProjectBusy));
        assert_eq!(repository.git_success(&["rev-parse", "HEAD"]).stdout, head);
        assert_eq!(
            fs::read_to_string(repository.path(".git/index.lock")).unwrap(),
            "held elsewhere\n"
        );
    }

    #[tokio::test]
    async fn commit_supports_an_unborn_branch_and_preview_respects_project_ownership() {
        let repository = TestRepository::new(true);
        repository.write("first.txt", "first commit\n");
        repository.git_success(&["add", "--", "first.txt"]);
        let (projects, project_id) = repository.attach(&repository.root);
        let coordinator = MutationCoordinator::default();

        projects.reserve_execution(&project_id).unwrap();
        let busy = coordinator
            .preview(
                preview_request(
                    &project_id,
                    GitMutationOperation::Commit,
                    None,
                    Some("first"),
                ),
                &projects,
            )
            .await;
        projects.release_execution(&project_id);
        assert_eq!(busy.state, GitMutationPreviewState::Unavailable);
        assert_eq!(busy.diagnostic_code, Some(GitDiagnosticCode::ProjectBusy));

        let applied = preview_and_confirm(
            &coordinator,
            &projects,
            preview_request(
                &project_id,
                GitMutationOperation::Commit,
                None,
                Some("first"),
            ),
        )
        .await;
        assert_eq!(applied.state, GitMutationResultState::Applied);
        assert_eq!(
            String::from_utf8(repository.git_success(&["show", "HEAD:first.txt"]).stdout).unwrap(),
            "first commit\n"
        );
    }
}
