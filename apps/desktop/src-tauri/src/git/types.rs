use serde::{Deserialize, Serialize};

pub const GIT_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitWorkspaceState {
    Clean,
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitDiagnosticCode {
    ProjectNotFound,
    DirectoryUnavailable,
    IdentityChanged,
    NotRepository,
    GitUnavailable,
    GitFailed,
    OutputTooLarge,
    InvalidPath,
    DiffUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitChangeKind {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unmerged,
    Untracked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchSummary {
    pub head: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub detached: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitFileChange {
    pub path: String,
    pub previous_path: Option<String>,
    pub staged: Option<GitChangeKind>,
    pub worktree: Option<GitChangeKind>,
    pub conflict: bool,
    pub submodule: bool,
    pub reviewable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitWorkspaceSnapshot {
    pub schema_version: u16,
    pub state: GitWorkspaceState,
    pub project_id: Option<String>,
    pub branch: Option<GitBranchSummary>,
    pub changes: Vec<GitFileChange>,
    pub truncated: bool,
    pub diagnostic_code: Option<GitDiagnosticCode>,
}

impl GitWorkspaceSnapshot {
    pub fn unavailable(project_id: Option<String>, diagnostic_code: GitDiagnosticCode) -> Self {
        Self {
            schema_version: GIT_SCHEMA_VERSION,
            state: GitWorkspaceState::Unavailable,
            project_id,
            branch: None,
            changes: Vec::new(),
            truncated: false,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitDiffArea {
    Staged,
    Worktree,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitDiffRequest {
    pub project_id: String,
    pub path: String,
    pub area: GitDiffArea,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitOpenFileRequest {
    pub project_id: String,
    pub path: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitDiffState {
    Ready,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitDiffKind {
    Text,
    Binary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GitDiffLineKind {
    Hunk,
    Context,
    Addition,
    Deletion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffLine {
    pub kind: GitDiffLineKind,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDiffSnapshot {
    pub schema_version: u16,
    pub state: GitDiffState,
    pub project_id: String,
    pub path: String,
    pub area: GitDiffArea,
    pub kind: Option<GitDiffKind>,
    pub lines: Vec<GitDiffLine>,
    pub truncated: bool,
    pub diagnostic_code: Option<GitDiagnosticCode>,
}

impl GitDiffSnapshot {
    pub fn unavailable(request: GitDiffRequest, diagnostic_code: GitDiagnosticCode) -> Self {
        Self {
            schema_version: GIT_SCHEMA_VERSION,
            state: GitDiffState::Unavailable,
            project_id: request.project_id,
            path: request.path,
            area: request.area,
            kind: None,
            lines: Vec::new(),
            truncated: false,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}
