use serde::{Deserialize, Serialize};

pub const WORKTREE_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeWorkspaceState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeEntryState {
    Ready,
    Missing,
    Archived,
    Locked,
    Prunable,
    Detached,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeOwnership {
    Source,
    Managed,
    Attached,
    External,
}

impl WorktreeOwnership {
    pub(crate) fn as_storage_value(self) -> Option<&'static str> {
        match self {
            Self::Managed => Some("managed"),
            Self::Attached => Some("attached"),
            Self::Source | Self::External => None,
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "managed" => Some(Self::Managed),
            "attached" => Some(Self::Attached),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeDiagnosticCode {
    MetadataUnavailable,
    ProjectNotFound,
    ProjectBusy,
    NotRepository,
    DirectoryUnavailable,
    IdentityChanged,
    PickerUnavailable,
    InvalidBranch,
    BranchExists,
    DuplicateDirectory,
    NotLinkedWorktree,
    DifferentRepository,
    GitUnavailable,
    GitFailed,
    OutputTooLarge,
    ConfirmationExpired,
    StalePreview,
    WorktreeRemains,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeOperation {
    Create,
    Attach,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreePreviewState {
    Ready,
    Cancelled,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorktreeResultState {
    Applied,
    Unavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeEntry {
    pub project_id: Option<String>,
    pub display_name: String,
    pub display_path: String,
    pub branch_name: Option<String>,
    pub ownership: WorktreeOwnership,
    pub state: WorktreeEntryState,
    pub current: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeWorkspaceSnapshot {
    pub schema_version: u16,
    pub state: WorktreeWorkspaceState,
    pub source_project_id: Option<String>,
    pub worktrees: Vec<WorktreeEntry>,
    pub truncated: bool,
    pub diagnostic_code: Option<WorktreeDiagnosticCode>,
}

impl WorktreeWorkspaceSnapshot {
    pub(crate) fn unavailable(
        source_project_id: Option<String>,
        diagnostic_code: WorktreeDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreeWorkspaceState::Unavailable,
            source_project_id,
            worktrees: Vec::new(),
            truncated: false,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorktreeCreatePreviewRequest {
    pub project_id: String,
    pub branch_name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorktreeConfirmRequest {
    pub confirmation_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorktreeCancelRequest {
    pub confirmation_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreePreviewSnapshot {
    pub schema_version: u16,
    pub state: WorktreePreviewState,
    pub source_project_id: String,
    pub operation: WorktreeOperation,
    pub branch_name: Option<String>,
    pub display_path: Option<String>,
    pub ownership: Option<WorktreeOwnership>,
    pub destructive: bool,
    pub confirmation_id: Option<String>,
    pub diagnostic_code: Option<WorktreeDiagnosticCode>,
}

impl WorktreePreviewSnapshot {
    pub(crate) fn unavailable(
        source_project_id: String,
        operation: WorktreeOperation,
        diagnostic_code: WorktreeDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreePreviewState::Unavailable,
            source_project_id,
            operation,
            branch_name: None,
            display_path: None,
            ownership: None,
            destructive: false,
            confirmation_id: None,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeResultSnapshot {
    pub schema_version: u16,
    pub state: WorktreeResultState,
    pub source_project_id: Option<String>,
    pub project_id: Option<String>,
    pub workspace: Option<WorktreeWorkspaceSnapshot>,
    pub recoverable_display_path: Option<String>,
    pub diagnostic_code: Option<WorktreeDiagnosticCode>,
}

impl WorktreeResultSnapshot {
    pub(crate) fn unavailable(
        source_project_id: Option<String>,
        diagnostic_code: WorktreeDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: WORKTREE_SCHEMA_VERSION,
            state: WorktreeResultState::Unavailable,
            source_project_id,
            project_id: None,
            workspace: None,
            recoverable_display_path: None,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}
