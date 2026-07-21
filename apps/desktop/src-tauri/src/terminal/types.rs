use serde::{Deserialize, Serialize};

pub const TERMINAL_SCHEMA_VERSION: u16 = 1;
pub const TERMINAL_REGISTRY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalStartRequest {
    pub project_id: String,
    pub columns: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalPollRequest {
    pub terminal_id: String,
    pub after_sequence: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalWriteRequest {
    pub terminal_id: String,
    pub data_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalResizeRequest {
    pub terminal_id: String,
    pub columns: u16,
    pub rows: u16,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalCloseRequest {
    pub terminal_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalState {
    Running,
    Closing,
    Exited,
    Interrupted,
    Failed,
    Unavailable,
}

impl TerminalState {
    pub(crate) const fn storage_value(self) -> Option<&'static str> {
        match self {
            Self::Running => Some("running"),
            Self::Closing => Some("closing"),
            Self::Exited => Some("exited"),
            Self::Interrupted => Some("interrupted"),
            Self::Failed => Some("failed"),
            Self::Unavailable => None,
        }
    }

    pub(crate) fn from_storage_value(value: &str) -> Option<Self> {
        Some(match value {
            "running" => Self::Running,
            "closing" => Self::Closing,
            "exited" => Self::Exited,
            "interrupted" => Self::Interrupted,
            "failed" => Self::Failed,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalDiagnosticCode {
    InvalidRequest,
    CapacityReached,
    TerminalNotFound,
    ProjectUnavailable,
    ProjectIdentityChanged,
    ProjectNotWritable,
    ProjectBusy,
    MetadataUnavailable,
    PtyUnavailable,
    ShellUnavailable,
    InputTooLarge,
    InputUnavailable,
    ResizeUnavailable,
    OutputUnavailable,
    CleanupIncomplete,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalOutputChunk {
    pub sequence: u64,
    pub data_base64: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalSnapshot {
    pub schema_version: u16,
    pub state: TerminalState,
    pub terminal_id: Option<String>,
    pub project_id: Option<String>,
    pub title: Option<String>,
    pub live: bool,
    pub columns: u16,
    pub rows: u16,
    pub output: Vec<TerminalOutputChunk>,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub truncated: bool,
    pub has_more: bool,
    pub exit_code: Option<i32>,
    pub diagnostic_code: Option<TerminalDiagnosticCode>,
}

impl TerminalSnapshot {
    pub(crate) fn unavailable(
        project_id: Option<String>,
        diagnostic_code: TerminalDiagnosticCode,
    ) -> Self {
        Self {
            schema_version: TERMINAL_SCHEMA_VERSION,
            state: TerminalState::Unavailable,
            terminal_id: None,
            project_id,
            title: None,
            live: false,
            columns: 0,
            rows: 0,
            output: Vec::new(),
            first_sequence: 0,
            last_sequence: 0,
            truncated: false,
            has_more: false,
            exit_code: None,
            diagnostic_code: Some(diagnostic_code),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalRegistrySnapshot {
    pub schema_version: u16,
    pub capacity: u8,
    pub terminals: Vec<TerminalSnapshot>,
    pub diagnostic_code: Option<TerminalDiagnosticCode>,
}

impl TerminalRegistrySnapshot {
    pub(crate) fn unavailable(diagnostic_code: TerminalDiagnosticCode) -> Self {
        Self {
            schema_version: TERMINAL_REGISTRY_SCHEMA_VERSION,
            capacity: 8,
            terminals: Vec::new(),
            diagnostic_code: Some(diagnostic_code),
        }
    }
}
