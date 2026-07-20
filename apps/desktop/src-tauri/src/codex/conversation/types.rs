use serde::{Deserialize, Serialize};

pub const CONVERSATION_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationStartRequest {
    pub project_id: String,
    pub prompt: String,
    pub model_id: String,
    pub reasoning_effort: String,
    pub sandbox_mode: ConversationSandboxMode,
    pub approval_policy: ConversationApprovalPolicy,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationSandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl ConversationSandboxMode {
    pub(crate) const fn as_protocol_value(self) -> &'static str {
        match self {
            Self::ReadOnly => "read-only",
            Self::WorkspaceWrite => "workspace-write",
            Self::DangerFullAccess => "danger-full-access",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalPolicy {
    Untrusted,
    OnRequest,
    Never,
}

impl ConversationApprovalPolicy {
    pub(crate) const fn as_protocol_value(self) -> &'static str {
        match self {
            Self::Untrusted => "untrusted",
            Self::OnRequest => "on-request",
            Self::Never => "never",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationState {
    Empty,
    Running,
    Stopping,
    Completed,
    Interrupted,
    Blocked,
    Failed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationDiagnosticCode {
    ConversationActive,
    ConversationNotFound,
    InvalidRequest,
    ProjectUnavailable,
    ProjectIdentityChanged,
    ProjectNotWritable,
    ProjectBusy,
    RuntimeUnavailable,
    ModelUnavailable,
    ReasoningUnavailable,
    MetadataUnavailable,
    ApprovalRequired,
    ProcessExited,
    TransportFailed,
    ProtocolInvalid,
    RpcRejected,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationSnapshot {
    pub schema_version: u16,
    pub state: ConversationState,
    pub conversation_id: Option<String>,
    pub project_id: Option<String>,
    pub model_id: Option<String>,
    pub reasoning_effort: Option<String>,
    pub sandbox_mode: Option<ConversationSandboxMode>,
    pub approval_policy: Option<ConversationApprovalPolicy>,
    pub events: Vec<ConversationEvent>,
    pub diagnostic_code: Option<ConversationDiagnosticCode>,
}

impl ConversationSnapshot {
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: CONVERSATION_SCHEMA_VERSION,
            state: ConversationState::Empty,
            conversation_id: None,
            project_id: None,
            model_id: None,
            reasoning_effort: None,
            sandbox_mode: None,
            approval_policy: None,
            events: Vec::new(),
            diagnostic_code: None,
        }
    }

    pub(crate) fn unavailable(diagnostic_code: ConversationDiagnosticCode) -> Self {
        Self {
            state: ConversationState::Unavailable,
            diagnostic_code: Some(diagnostic_code),
            ..Self::empty()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ConversationEvent {
    Lifecycle {
        sequence: u64,
        phase: ConversationLifecyclePhase,
    },
    AgentMessageDelta {
        sequence: u64,
        delta: String,
    },
    ReasoningSummaryDelta {
        sequence: u64,
        delta: String,
    },
    PlanUpdated {
        sequence: u64,
        explanation: Option<String>,
        steps: Vec<ConversationPlanStep>,
    },
    Activity {
        sequence: u64,
        kind: ConversationActivityKind,
        status: ConversationActivityStatus,
    },
    Error {
        sequence: u64,
        code: ConversationStreamErrorCode,
        will_retry: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationLifecyclePhase {
    Starting,
    Running,
    Stopping,
    Completed,
    Interrupted,
    Blocked,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationPlanStep {
    pub step: String,
    pub status: ConversationPlanStepStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationPlanStepStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationActivityKind {
    UserMessage,
    AgentMessage,
    Plan,
    Reasoning,
    CommandExecution,
    FileChange,
    ToolCall,
    WebSearch,
    Image,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationActivityStatus {
    Started,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationStreamErrorCode {
    ContextWindowExceeded,
    UsageLimitExceeded,
    Unauthorized,
    Sandbox,
    Server,
    Other,
}
