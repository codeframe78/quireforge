mod identity;
mod storage;
pub mod types;

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Mutex,
};

pub(crate) use storage::StoredConversationReference;
use storage::{ProjectRepository, StorageError, StoredAssociation, StoredProject};
use types::{
    DirectoryAccessibilityState, DirectorySummary, GitSummary, PendingAttachmentKind,
    PendingAttachmentPreview, ProjectDiagnosticCode, ProjectPreflightSnapshot, ProjectSummary,
    ProjectWorkspaceSnapshot, ProjectWorkspaceState, PROJECT_SCHEMA_VERSION,
};
use uuid::Uuid;

use self::identity::{
    disconnected_state, display_path, inspect_directory, DirectoryIdentity,
    DirectoryInspectionError,
};

#[derive(Clone)]
struct PendingAttachment {
    kind: PendingAttachmentKind,
    project_id: Option<String>,
    display_name: String,
    identity: DirectoryIdentity,
}

pub struct ProjectService {
    repository: Mutex<Option<ProjectRepository>>,
    pending: Mutex<Option<PendingAttachment>>,
    active_executions: Mutex<HashSet<String>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProjectExecutionError {
    InvalidProjectId,
    MetadataUnavailable,
    ProjectNotFound,
    DirectoryUnavailable,
    IdentityChanged,
    NotWritable,
    ProjectBusy,
}

pub(crate) struct ConversationReference<'a> {
    pub conversation_id: &'a str,
    pub project_id: &'a str,
    pub codex_thread_id: &'a str,
    pub model_id: &'a str,
    pub reasoning_effort: &'a str,
    pub sandbox_mode: &'a str,
    pub approval_policy: &'a str,
    pub parent_conversation_id: Option<&'a str>,
}

impl ProjectService {
    pub fn unavailable() -> Self {
        Self {
            repository: Mutex::new(None),
            pending: Mutex::new(None),
            active_executions: Mutex::new(HashSet::new()),
        }
    }

    pub fn open(database_path: &Path) -> Self {
        Self {
            repository: Mutex::new(ProjectRepository::open(database_path).ok()),
            pending: Mutex::new(None),
            active_executions: Mutex::new(HashSet::new()),
        }
    }

    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        Self {
            repository: Mutex::new(ProjectRepository::in_memory().ok()),
            pending: Mutex::new(None),
            active_executions: Mutex::new(HashSet::new()),
        }
    }

    pub fn status(&self) -> ProjectWorkspaceSnapshot {
        self.build_snapshot(None)
    }

    pub fn picker_unavailable(&self) -> ProjectWorkspaceSnapshot {
        self.build_snapshot(Some(ProjectDiagnosticCode::PickerUnavailable))
    }

    pub fn prepare_attachment(&self, selected_path: PathBuf) -> ProjectWorkspaceSnapshot {
        self.prepare(PendingAttachmentKind::Attach, None, selected_path)
    }

    pub fn prepare_relink(
        &self,
        project_id: String,
        selected_path: PathBuf,
    ) -> ProjectWorkspaceSnapshot {
        if !valid_id(&project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectNotFound));
        }
        if self.execution_active(&project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectBusy));
        }
        self.prepare(
            PendingAttachmentKind::Relink,
            Some(project_id),
            selected_path,
        )
    }

    pub fn cancel_pending(&self) -> ProjectWorkspaceSnapshot {
        if let Ok(mut pending) = self.pending.lock() {
            *pending = None;
        }
        self.status()
    }

    pub fn confirm_pending(&self) -> ProjectWorkspaceSnapshot {
        let pending = self
            .pending
            .lock()
            .ok()
            .and_then(|mut pending| pending.take());
        let Some(pending) = pending else {
            return self.build_snapshot(Some(ProjectDiagnosticCode::AttachmentNotPending));
        };
        if pending
            .project_id
            .as_deref()
            .is_some_and(|project_id| self.execution_active(project_id))
        {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectBusy));
        }

        let current_identity = match inspect_directory(&pending.identity.selected_path) {
            Ok(identity) => identity,
            Err(_) => return self.build_snapshot(Some(ProjectDiagnosticCode::IdentityChanged)),
        };
        if !same_identity(&pending.identity, &current_identity) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::IdentityChanged));
        }

        let mut repository_guard = match self.repository.lock() {
            Ok(repository) => repository,
            Err(_) => {
                return ProjectWorkspaceSnapshot::unavailable(
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let Some(repository) = repository_guard.as_mut() else {
            return ProjectWorkspaceSnapshot::unavailable(
                ProjectDiagnosticCode::MetadataUnavailable,
            );
        };
        let result = match pending.kind {
            PendingAttachmentKind::Attach => repository
                .insert_project(&pending.display_name, &current_identity)
                .map(|_| ()),
            PendingAttachmentKind::Relink => repository.relink_project(
                pending
                    .project_id
                    .as_deref()
                    .expect("relink pending state always has a project ID"),
                &current_identity,
            ),
        };
        drop(repository_guard);
        match result {
            Ok(()) => self.status(),
            Err(error) => self.build_snapshot(Some(map_storage_error(&error))),
        }
    }

    pub fn detach(&self, project_id: String) -> ProjectWorkspaceSnapshot {
        self.metadata_action(&project_id, |repository, project_id| {
            repository.detach_project(project_id)
        })
    }

    pub fn archive(&self, project_id: String) -> ProjectWorkspaceSnapshot {
        self.metadata_action(&project_id, |repository, project_id| {
            repository.archive_project(project_id)
        })
    }

    pub fn preflight(&self, project_id: String) -> ProjectPreflightSnapshot {
        if !valid_id(&project_id) {
            return unavailable_preflight(project_id, ProjectDiagnosticCode::ProjectNotFound);
        }
        let repository_guard = match self.repository.lock() {
            Ok(repository) => repository,
            Err(_) => {
                return unavailable_preflight(
                    project_id,
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let Some(repository) = repository_guard.as_ref() else {
            return unavailable_preflight(project_id, ProjectDiagnosticCode::MetadataUnavailable);
        };
        let project = match repository.project(&project_id) {
            Ok(project) => project,
            Err(StorageError::ProjectNotFound) => {
                return unavailable_preflight(project_id, ProjectDiagnosticCode::ProjectNotFound);
            }
            Err(_) => {
                return unavailable_preflight(
                    project_id,
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        drop(repository_guard);
        let Some(association) = project.association else {
            return ProjectPreflightSnapshot {
                schema_version: PROJECT_SCHEMA_VERSION,
                project_id,
                cwd_ready: false,
                display_path: None,
                state: DirectoryAccessibilityState::MissingOrMoved,
                diagnostic_code: None,
            };
        };

        let selected_path = PathBuf::from(&association.selected_path);
        match inspect_directory(&selected_path) {
            Ok(identity) if same_stored_identity(&association, &identity) => {
                let cwd_ready =
                    identity.accessibility == DirectoryAccessibilityState::ConnectedAccessible;
                ProjectPreflightSnapshot {
                    schema_version: PROJECT_SCHEMA_VERSION,
                    project_id,
                    cwd_ready,
                    display_path: Some(identity.selected_display_path),
                    state: identity.accessibility,
                    diagnostic_code: None,
                }
            }
            Ok(_) => ProjectPreflightSnapshot {
                schema_version: PROJECT_SCHEMA_VERSION,
                project_id,
                cwd_ready: false,
                display_path: Some(display_path(&selected_path)),
                state: DirectoryAccessibilityState::IdentityChanged,
                diagnostic_code: Some(ProjectDiagnosticCode::IdentityChanged),
            },
            Err(error) => ProjectPreflightSnapshot {
                schema_version: PROJECT_SCHEMA_VERSION,
                project_id,
                cwd_ready: false,
                display_path: Some(display_path(&selected_path)),
                state: preflight_failure_state(&association, error),
                diagnostic_code: None,
            },
        }
    }

    pub(crate) fn execution_cwd(&self, project_id: &str) -> Result<PathBuf, ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        let project = repository
            .project(project_id)
            .map_err(|error| match error {
                StorageError::ProjectNotFound => ProjectExecutionError::ProjectNotFound,
                _ => ProjectExecutionError::MetadataUnavailable,
            })?;
        if project.archived {
            return Err(ProjectExecutionError::ProjectNotFound);
        }
        let association = project
            .association
            .ok_or(ProjectExecutionError::DirectoryUnavailable)?;
        drop(repository_guard);

        let identity = inspect_directory(Path::new(&association.selected_path))
            .map_err(|_| ProjectExecutionError::DirectoryUnavailable)?;
        if !same_stored_identity(&association, &identity) {
            return Err(ProjectExecutionError::IdentityChanged);
        }
        if identity.accessibility != DirectoryAccessibilityState::ConnectedAccessible {
            return Err(ProjectExecutionError::NotWritable);
        }
        Ok(identity.resolved_path)
    }

    pub(crate) fn reserve_execution(&self, project_id: &str) -> Result<(), ProjectExecutionError> {
        if !valid_id(project_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let mut active = self
            .active_executions
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        if !active.insert(project_id.to_owned()) {
            return Err(ProjectExecutionError::ProjectBusy);
        }
        Ok(())
    }

    pub(crate) fn release_execution(&self, project_id: &str) {
        if let Ok(mut active) = self.active_executions.lock() {
            active.remove(project_id);
        }
    }

    pub(crate) fn record_conversation_reference(
        &self,
        reference: ConversationReference<'_>,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .insert_conversation_reference(&reference)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn conversation_reference(
        &self,
        conversation_id: &str,
    ) -> Result<StoredConversationReference, ProjectExecutionError> {
        if !valid_id(conversation_id) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .conversation_reference(conversation_id)
            .map_err(|error| match error {
                StorageError::InvalidStoredValue => ProjectExecutionError::ProjectNotFound,
                _ => ProjectExecutionError::MetadataUnavailable,
            })
    }

    pub(crate) fn conversation_references(
        &self,
        project_id: Option<&str>,
    ) -> Result<Vec<StoredConversationReference>, ProjectExecutionError> {
        if project_id.is_some_and(|value| !valid_id(value)) {
            return Err(ProjectExecutionError::InvalidProjectId);
        }
        let repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_ref()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .list_conversation_references(project_id)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_conversation_turn(
        &self,
        conversation_id: &str,
        active_turn_id: &str,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_conversation_turn(conversation_id, active_turn_id)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_conversation_status(
        &self,
        conversation_id: &str,
        status: &str,
    ) -> Result<(), ProjectExecutionError> {
        if !matches!(
            status,
            "stopping" | "completed" | "interrupted" | "blocked" | "failed"
        ) {
            return Err(ProjectExecutionError::MetadataUnavailable);
        }
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_conversation_status(conversation_id, status)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    pub(crate) fn record_conversation_archived(
        &self,
        conversation_id: &str,
        archived: bool,
    ) -> Result<(), ProjectExecutionError> {
        let mut repository_guard = self
            .repository
            .lock()
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)?;
        let repository = repository_guard
            .as_mut()
            .ok_or(ProjectExecutionError::MetadataUnavailable)?;
        repository
            .update_conversation_archived(conversation_id, archived)
            .map_err(|_| ProjectExecutionError::MetadataUnavailable)
    }

    fn prepare(
        &self,
        kind: PendingAttachmentKind,
        project_id: Option<String>,
        selected_path: PathBuf,
    ) -> ProjectWorkspaceSnapshot {
        let identity = match inspect_directory(&selected_path) {
            Ok(identity)
                if matches!(
                    identity.accessibility,
                    DirectoryAccessibilityState::ConnectedAccessible
                        | DirectoryAccessibilityState::ConnectedReadOnly
                ) =>
            {
                identity
            }
            Ok(_) | Err(_) => {
                return self.build_snapshot(Some(ProjectDiagnosticCode::DirectoryUnavailable));
            }
        };

        let repository_guard = match self.repository.lock() {
            Ok(repository) => repository,
            Err(_) => {
                return ProjectWorkspaceSnapshot::unavailable(
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let Some(repository) = repository_guard.as_ref() else {
            return ProjectWorkspaceSnapshot::unavailable(
                ProjectDiagnosticCode::MetadataUnavailable,
            );
        };
        let availability = (|| {
            let excluding_association = project_id
                .as_deref()
                .map(|project_id| {
                    repository
                        .project(project_id)
                        .map(|project| project.association.map(|association| association.id))
                })
                .transpose()?
                .flatten();
            repository.ensure_directory_available(&identity, excluding_association.as_deref())
        })();
        drop(repository_guard);
        if let Err(error) = availability {
            return self.build_snapshot(Some(map_storage_error(&error)));
        }

        let display_name = directory_display_name(&identity.selected_path);
        if let Ok(mut pending) = self.pending.lock() {
            *pending = Some(PendingAttachment {
                kind,
                project_id,
                display_name,
                identity,
            });
        } else {
            return ProjectWorkspaceSnapshot::unavailable(
                ProjectDiagnosticCode::MetadataUnavailable,
            );
        }
        self.status()
    }

    fn metadata_action<F>(&self, project_id: &str, action: F) -> ProjectWorkspaceSnapshot
    where
        F: FnOnce(&mut ProjectRepository, &str) -> Result<(), StorageError>,
    {
        if !valid_id(project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectNotFound));
        }
        if self.execution_active(project_id) {
            return self.build_snapshot(Some(ProjectDiagnosticCode::ProjectBusy));
        }
        let result =
            self.repository.lock().ok().and_then(|mut repository| {
                repository.as_mut().map(|repo| action(repo, project_id))
            });
        match result {
            Some(Ok(())) => self.status(),
            Some(Err(error)) => self.build_snapshot(Some(map_storage_error(&error))),
            None => {
                ProjectWorkspaceSnapshot::unavailable(ProjectDiagnosticCode::MetadataUnavailable)
            }
        }
    }

    fn execution_active(&self, project_id: &str) -> bool {
        self.active_executions
            .lock()
            .map(|active| active.contains(project_id))
            .unwrap_or(true)
    }

    fn build_snapshot(
        &self,
        diagnostic_code: Option<ProjectDiagnosticCode>,
    ) -> ProjectWorkspaceSnapshot {
        let projects = match self.repository.lock() {
            Ok(repository) => match repository.as_ref() {
                Some(repository) => match repository.list_projects() {
                    Ok(projects) => projects,
                    Err(_) => {
                        return ProjectWorkspaceSnapshot::unavailable(
                            ProjectDiagnosticCode::MetadataUnavailable,
                        );
                    }
                },
                None => {
                    return ProjectWorkspaceSnapshot::unavailable(
                        ProjectDiagnosticCode::MetadataUnavailable,
                    );
                }
            },
            Err(_) => {
                return ProjectWorkspaceSnapshot::unavailable(
                    ProjectDiagnosticCode::MetadataUnavailable,
                );
            }
        };
        let projects: Vec<_> = projects.into_iter().map(project_summary).collect();
        let pending_attachment = self
            .pending
            .lock()
            .ok()
            .and_then(|pending| pending.as_ref().map(pending_preview));
        ProjectWorkspaceSnapshot {
            schema_version: PROJECT_SCHEMA_VERSION,
            state: if projects.is_empty() {
                ProjectWorkspaceState::Empty
            } else {
                ProjectWorkspaceState::Ready
            },
            projects,
            pending_attachment,
            diagnostic_code,
        }
    }
}

fn project_summary(project: StoredProject) -> ProjectSummary {
    ProjectSummary {
        id: project.id,
        display_name: project.display_name,
        archived: project.archived,
        directory: project.association.map(directory_summary),
    }
}

fn directory_summary(association: StoredAssociation) -> DirectorySummary {
    let selected_path = PathBuf::from(&association.selected_path);
    let stored_resolved_path = PathBuf::from(&association.resolved_path);
    match inspect_directory(&selected_path) {
        Ok(identity) if same_stored_identity(&association, &identity) => {
            let git = identity.git_summary();
            DirectorySummary {
                association_id: association.id,
                display_path: identity.selected_display_path,
                resolved_display_path: Some(identity.resolved_display_path),
                state: identity.accessibility,
                expected_access: association.expected_access,
                is_primary: true,
                git,
                has_agents_guidance: identity.has_agents_guidance,
                has_codex_config: identity.has_codex_config,
            }
        }
        Ok(identity) => {
            let git = identity.git_summary();
            DirectorySummary {
                association_id: association.id,
                display_path: display_path(&selected_path),
                resolved_display_path: Some(identity.resolved_display_path),
                state: DirectoryAccessibilityState::IdentityChanged,
                expected_access: association.expected_access,
                is_primary: true,
                git,
                has_agents_guidance: identity.has_agents_guidance,
                has_codex_config: identity.has_codex_config,
            }
        }
        Err(error) => {
            let state = preflight_failure_state(&association, error);
            DirectorySummary {
                association_id: association.id,
                display_path: display_path(&selected_path),
                resolved_display_path: Some(display_path(&stored_resolved_path)),
                state,
                expected_access: association.expected_access,
                is_primary: true,
                git: GitSummary {
                    is_repository: association.git_common_dir.is_some(),
                    is_linked_worktree: association.git_is_linked_worktree,
                },
                has_agents_guidance: association.has_agents_guidance,
                has_codex_config: association.has_codex_config,
            }
        }
    }
}

fn pending_preview(pending: &PendingAttachment) -> PendingAttachmentPreview {
    PendingAttachmentPreview {
        operation: pending.kind,
        project_id: pending.project_id.clone(),
        display_name: pending.display_name.clone(),
        selected_display_path: pending.identity.selected_display_path.clone(),
        resolved_display_path: pending.identity.resolved_display_path.clone(),
        state: pending.identity.accessibility,
        git: pending.identity.git_summary(),
        has_agents_guidance: pending.identity.has_agents_guidance,
        has_codex_config: pending.identity.has_codex_config,
    }
}

fn same_identity(expected: &DirectoryIdentity, current: &DirectoryIdentity) -> bool {
    expected.resolved_path == current.resolved_path
        && expected.device_id == current.device_id
        && expected.inode == current.inode
        && expected.mount_id == current.mount_id
        && expected.filesystem_type == current.filesystem_type
        && expected.git == current.git
        && expected.accessibility == current.accessibility
        && expected.has_agents_guidance == current.has_agents_guidance
        && expected.has_codex_config == current.has_codex_config
}

fn same_stored_identity(stored: &StoredAssociation, current: &DirectoryIdentity) -> bool {
    stored.resolved_path == current.resolved_path.to_string_lossy()
        && stored.device_id == Some(current.device_id)
        && stored.inode == Some(current.inode)
        && stored.mount_id == current.mount_id
        && stored.filesystem_type == current.filesystem_type
        && stored.git_common_dir.as_deref()
            == current.git.as_ref().and_then(|git| git.common_dir.to_str())
        && stored.git_worktree_root.as_deref()
            == current
                .git
                .as_ref()
                .and_then(|git| git.worktree_root.to_str())
        && stored.git_is_linked_worktree
            == current
                .git
                .as_ref()
                .is_some_and(|git| git.is_linked_worktree)
}

fn preflight_failure_state(
    stored: &StoredAssociation,
    failure: DirectoryInspectionError,
) -> DirectoryAccessibilityState {
    if failure.state == DirectoryAccessibilityState::MissingOrMoved {
        disconnected_state(stored.filesystem_type.as_deref())
    } else {
        failure.state
    }
}

fn directory_display_name(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty() && !name.chars().any(char::is_control))
        .map(|name| name.chars().take(120).collect())
        .unwrap_or_else(|| "Local project".to_owned())
}

fn valid_id(value: &str) -> bool {
    Uuid::parse_str(value).is_ok() && value.len() == 36
}

fn map_storage_error(error: &StorageError) -> ProjectDiagnosticCode {
    match error {
        StorageError::DuplicateDirectory => ProjectDiagnosticCode::DuplicateDirectory,
        StorageError::ProjectNotFound => ProjectDiagnosticCode::ProjectNotFound,
        StorageError::InvalidStoredValue
        | StorageError::FutureSchema
        | StorageError::Filesystem
        | StorageError::Sqlite(_) => ProjectDiagnosticCode::MetadataUnavailable,
    }
}

fn unavailable_preflight(
    project_id: String,
    diagnostic_code: ProjectDiagnosticCode,
) -> ProjectPreflightSnapshot {
    ProjectPreflightSnapshot {
        schema_version: PROJECT_SCHEMA_VERSION,
        project_id,
        cwd_ready: false,
        display_path: None,
        state: DirectoryAccessibilityState::VerificationUnknown,
        diagnostic_code: Some(diagnostic_code),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        os::unix::fs::{symlink, PermissionsExt},
        sync::Arc,
        thread,
    };

    use uuid::Uuid;

    use super::{
        types::{DirectoryAccessibilityState, ProjectDiagnosticCode, ProjectWorkspaceState},
        ProjectService,
    };

    fn temporary_directory(label: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("quireforge-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&path).expect("temporary directory must be created");
        path
    }

    #[test]
    fn serialized_empty_workspace_matches_the_shared_frontend_fixture() {
        let service = ProjectService::in_memory();
        let actual =
            serde_json::to_value(service.status()).expect("workspace snapshot must serialize");
        let expected: serde_json::Value =
            serde_json::from_str(include_str!("../../../fixtures/project-workspace.json"))
                .expect("shared workspace fixture must parse");

        assert_eq!(actual, expected);
    }

    #[test]
    fn attaches_and_preflights_the_original_directory_in_place() {
        let directory = temporary_directory("attach");
        fs::write(directory.join("kept-in-place.txt"), "original").expect("marker must be written");
        let service = ProjectService::in_memory();

        let pending = service.prepare_attachment(directory.clone());
        assert!(pending.pending_attachment.is_some());
        let attached = service.confirm_pending();

        assert_eq!(attached.state, ProjectWorkspaceState::Ready);
        assert_eq!(attached.projects.len(), 1);
        let preflight = service.preflight(attached.projects[0].id.clone());
        assert!(preflight.cwd_ready);
        assert!(directory.join("kept-in-place.txt").is_file());
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn persists_project_metadata_across_service_restarts() {
        let root = temporary_directory("persistence");
        let directory = root.join("project");
        let database = root.join("app-data/metadata.sqlite3");
        fs::create_dir(&directory).expect("project directory must be created");
        let service = ProjectService::open(&database);
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();
        drop(service);

        let reopened = ProjectService::open(&database);
        let status = reopened.status();
        let preflight = reopened.preflight(project_id);

        assert_eq!(status.projects.len(), 1);
        assert!(preflight.cwd_ready);
        drop(reopened);
        fs::remove_dir_all(root).expect("temporary directory must be removed");
    }

    #[test]
    fn rejects_duplicate_resolved_directories() {
        let directory = temporary_directory("duplicate");
        let alias = directory.with_extension("alias");
        symlink(&directory, &alias).expect("alias must be created");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        service.confirm_pending();

        let duplicate = service.prepare_attachment(alias.clone());

        assert_eq!(
            duplicate.diagnostic_code,
            Some(ProjectDiagnosticCode::DuplicateDirectory)
        );
        fs::remove_file(alias).expect("alias must be removed");
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn fails_closed_when_a_symlink_changes_after_confirmation_preview() {
        let root = temporary_directory("retarget");
        let first = root.join("first");
        let second = root.join("second");
        let selected = root.join("selected");
        fs::create_dir(&first).expect("first target must exist");
        fs::create_dir(&second).expect("second target must exist");
        symlink(&first, &selected).expect("selected symlink must exist");
        let service = ProjectService::in_memory();
        service.prepare_attachment(selected.clone());
        fs::remove_file(&selected).expect("old symlink must be removed");
        symlink(&second, &selected).expect("new symlink must exist");

        let result = service.confirm_pending();

        assert_eq!(
            result.diagnostic_code,
            Some(ProjectDiagnosticCode::IdentityChanged)
        );
        assert!(result.projects.is_empty());
        fs::remove_dir_all(root).expect("temporary directory must be removed");
    }

    #[test]
    fn fails_closed_when_project_configuration_changes_after_preview() {
        let directory = temporary_directory("config-retarget");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        fs::create_dir(directory.join(".codex")).expect("configuration directory must be created");

        let result = service.confirm_pending();

        assert_eq!(
            result.diagnostic_code,
            Some(ProjectDiagnosticCode::IdentityChanged)
        );
        assert!(result.projects.is_empty());
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn relinks_an_existing_project_without_touching_either_directory() {
        let first = temporary_directory("relink-first");
        let second = temporary_directory("relink-second");
        fs::write(first.join("first.txt"), "first").expect("first marker must be written");
        fs::write(second.join("second.txt"), "second").expect("second marker must be written");
        let service = ProjectService::in_memory();
        service.prepare_attachment(first.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();

        let pending = service.prepare_relink(project_id.clone(), second.clone());
        assert!(pending.pending_attachment.is_some());
        let relinked = service.confirm_pending();
        let preflight = service.preflight(project_id);

        assert_eq!(relinked.projects.len(), 1);
        assert!(preflight.cwd_ready);
        assert_eq!(preflight.display_path, Some(second.display().to_string()));
        assert!(first.join("first.txt").is_file());
        assert!(second.join("second.txt").is_file());
        fs::remove_dir_all(first).expect("first directory must be removed");
        fs::remove_dir_all(second).expect("second directory must be removed");
    }

    #[test]
    fn attaches_read_only_directories_but_refuses_them_as_a_working_cwd() {
        let directory = temporary_directory("read-only");
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o555))
            .expect("directory must become read-only");
        let service = ProjectService::in_memory();

        let pending = service.prepare_attachment(directory.clone());
        assert_eq!(
            pending
                .pending_attachment
                .as_ref()
                .expect("attachment must be pending")
                .state,
            DirectoryAccessibilityState::ConnectedReadOnly
        );
        let attached = service.confirm_pending();
        let preflight = service.preflight(attached.projects[0].id.clone());

        assert!(!preflight.cwd_ready);
        assert_eq!(
            preflight.state,
            DirectoryAccessibilityState::ConnectedReadOnly
        );
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))
            .expect("directory permissions must be restored");
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn detach_and_archive_never_delete_source_content() {
        let directory = temporary_directory("detach");
        let marker = directory.join("source.txt");
        fs::write(&marker, "keep").expect("marker must be written");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();

        service.archive(project_id.clone());
        service.detach(project_id);

        assert_eq!(
            fs::read_to_string(&marker).expect("marker must remain readable"),
            "keep"
        );
        fs::remove_dir_all(directory).expect("temporary directory must be removed");
    }

    #[test]
    fn concurrent_status_reads_are_serialized_without_state_drift() {
        let service = Arc::new(ProjectService::in_memory());
        let readers: Vec<_> = (0..8)
            .map(|_| {
                let service = Arc::clone(&service);
                thread::spawn(move || service.status())
            })
            .collect();

        for reader in readers {
            assert_eq!(
                reader.join().expect("status reader must finish").state,
                ProjectWorkspaceState::Empty
            );
        }
    }

    #[test]
    fn missing_directory_preflight_never_falls_back() {
        let directory = temporary_directory("missing");
        let service = ProjectService::in_memory();
        service.prepare_attachment(directory.clone());
        let attached = service.confirm_pending();
        let project_id = attached.projects[0].id.clone();
        fs::remove_dir_all(directory).expect("temporary directory must be removed");

        let preflight = service.preflight(project_id);

        assert!(!preflight.cwd_ready);
        assert_eq!(preflight.state, DirectoryAccessibilityState::MissingOrMoved);
    }

    #[test]
    fn rejects_malformed_ids_and_distinguishes_unavailable_metadata() {
        let service = ProjectService::in_memory();

        let malformed = service.detach("not-an-opaque-id".to_owned());
        assert_eq!(
            malformed.diagnostic_code,
            Some(ProjectDiagnosticCode::ProjectNotFound)
        );
        let malformed_preflight = service.preflight("not-an-opaque-id".to_owned());
        assert_eq!(
            malformed_preflight.diagnostic_code,
            Some(ProjectDiagnosticCode::ProjectNotFound)
        );

        let unavailable = ProjectService::unavailable().preflight(Uuid::now_v7().to_string());
        assert_eq!(
            unavailable.diagnostic_code,
            Some(ProjectDiagnosticCode::MetadataUnavailable)
        );
        assert!(!unavailable.cwd_ready);
    }
}
