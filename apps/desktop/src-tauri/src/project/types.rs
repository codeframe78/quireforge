use serde::Serialize;

pub const PROJECT_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectWorkspaceState {
    Empty,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DirectoryAccessibilityState {
    ConnectedAccessible,
    ConnectedReadOnly,
    MissingOrMoved,
    PermissionDenied,
    RemovableDisconnected,
    NetworkUnavailable,
    GitInvalid,
    SandboxRestricted,
    IdentityChanged,
    VerificationUnknown,
}

impl DirectoryAccessibilityState {
    pub(crate) const fn as_storage_value(self) -> &'static str {
        match self {
            Self::ConnectedAccessible => "connected-accessible",
            Self::ConnectedReadOnly => "connected-read-only",
            Self::MissingOrMoved => "missing-or-moved",
            Self::PermissionDenied => "permission-denied",
            Self::RemovableDisconnected => "removable-disconnected",
            Self::NetworkUnavailable => "network-unavailable",
            Self::GitInvalid => "git-invalid",
            Self::SandboxRestricted => "sandbox-restricted",
            Self::IdentityChanged => "identity-changed",
            Self::VerificationUnknown => "verification-unknown",
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        Some(match value {
            "connected-accessible" => Self::ConnectedAccessible,
            "connected-read-only" => Self::ConnectedReadOnly,
            "missing-or-moved" => Self::MissingOrMoved,
            "permission-denied" => Self::PermissionDenied,
            "removable-disconnected" => Self::RemovableDisconnected,
            "network-unavailable" => Self::NetworkUnavailable,
            "git-invalid" => Self::GitInvalid,
            "sandbox-restricted" => Self::SandboxRestricted,
            "identity-changed" => Self::IdentityChanged,
            "verification-unknown" => Self::VerificationUnknown,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExpectedAccess {
    ReadWrite,
}

impl ExpectedAccess {
    pub(crate) const fn as_storage_value(self) -> &'static str {
        match self {
            Self::ReadWrite => "read-write",
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        match value {
            "read-write" => Some(Self::ReadWrite),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PendingAttachmentKind {
    Attach,
    Relink,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectDiagnosticCode {
    MetadataUnavailable,
    PickerUnavailable,
    DirectoryUnavailable,
    DuplicateDirectory,
    ProjectNotFound,
    AttachmentNotPending,
    IdentityChanged,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitSummary {
    pub is_repository: bool,
    pub is_linked_worktree: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectorySummary {
    pub association_id: String,
    pub display_path: String,
    pub resolved_display_path: Option<String>,
    pub state: DirectoryAccessibilityState,
    pub expected_access: ExpectedAccess,
    pub is_primary: bool,
    pub git: GitSummary,
    pub has_agents_guidance: bool,
    pub has_codex_config: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub display_name: String,
    pub archived: bool,
    pub directory: Option<DirectorySummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingAttachmentPreview {
    pub operation: PendingAttachmentKind,
    pub project_id: Option<String>,
    pub display_name: String,
    pub selected_display_path: String,
    pub resolved_display_path: String,
    pub state: DirectoryAccessibilityState,
    pub git: GitSummary,
    pub has_agents_guidance: bool,
    pub has_codex_config: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectWorkspaceSnapshot {
    pub schema_version: u16,
    pub state: ProjectWorkspaceState,
    pub projects: Vec<ProjectSummary>,
    pub pending_attachment: Option<PendingAttachmentPreview>,
    pub diagnostic_code: Option<ProjectDiagnosticCode>,
}

impl ProjectWorkspaceSnapshot {
    pub(crate) fn unavailable(diagnostic_code: ProjectDiagnosticCode) -> Self {
        Self {
            schema_version: PROJECT_SCHEMA_VERSION,
            state: ProjectWorkspaceState::Unavailable,
            projects: Vec::new(),
            pending_attachment: None,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPreflightSnapshot {
    pub schema_version: u16,
    pub project_id: String,
    pub cwd_ready: bool,
    pub display_path: Option<String>,
    pub state: DirectoryAccessibilityState,
    pub diagnostic_code: Option<ProjectDiagnosticCode>,
}
