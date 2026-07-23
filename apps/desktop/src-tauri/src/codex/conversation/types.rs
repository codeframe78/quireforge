use serde::{Deserialize, Serialize};

use crate::codex::model_selection::{
    ModelSelectionApplication, ModelSelectionChoice, ModelSelectionPolicy, ModelSelectionSnapshot,
};

pub const CONVERSATION_SCHEMA_VERSION: u16 = 3;
pub const CONVERSATION_REGISTRY_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationStartRequest {
    pub project_id: String,
    pub prompt: String,
    pub attachment_ids: Vec<String>,
    pub integration_entry_ids: Vec<String>,
    pub model_id: String,
    pub reasoning_effort: String,
    pub selection_policy: ModelSelectionPolicy,
    pub sandbox_mode: ConversationSandboxMode,
    pub approval_policy: ConversationApprovalPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConversationApprovalDecisionRequest {
    pub conversation_id: String,
    pub approval_id: String,
    pub decision: ConversationApprovalDecision,
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
    WaitingForApproval,
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
    ParallelCapacityReached,
    ConversationNotFound,
    InvalidRequest,
    ProjectUnavailable,
    ProjectIdentityChanged,
    ProjectNotWritable,
    ProjectBusy,
    RuntimeUnavailable,
    ModelUnavailable,
    ReasoningUnavailable,
    IntegrationUnavailable,
    AttachmentUnavailable,
    MetadataUnavailable,
    ApprovalRequired,
    ApprovalNotFound,
    ApprovalDecisionUnavailable,
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
    pub model_selection: Option<ModelSelectionSnapshot>,
    pub sandbox_mode: Option<ConversationSandboxMode>,
    pub approval_policy: Option<ConversationApprovalPolicy>,
    pub pending_approval: Option<ConversationApproval>,
    pub events: Vec<ConversationEvent>,
    pub diagnostic_code: Option<ConversationDiagnosticCode>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationRegistrySnapshot {
    pub schema_version: u16,
    pub capacity: u8,
    pub conversations: Vec<ConversationSnapshot>,
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
            model_selection: None,
            sandbox_mode: None,
            approval_policy: None,
            pending_approval: None,
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

    pub(crate) fn turn_in_flight(&self) -> bool {
        matches!(
            self.state,
            ConversationState::Running
                | ConversationState::WaitingForApproval
                | ConversationState::Stopping
        )
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
        activity_id: String,
        kind: ConversationActivityKind,
        status: ConversationActivityStatus,
        title: String,
        detail: Option<String>,
        exit_code: Option<i32>,
    },
    ActivityOutputDelta {
        sequence: u64,
        activity_id: String,
        delta: String,
    },
    ApprovalRequested {
        sequence: u64,
        approval_id: String,
        activity_id: String,
        kind: ConversationApprovalKind,
    },
    ApprovalResolved {
        sequence: u64,
        approval_id: String,
        resolution: ConversationApprovalResolution,
    },
    ModelSelectionRequested {
        sequence: u64,
        choice: ModelSelectionChoice,
        application: ModelSelectionApplication,
        rationale: String,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalDecision {
    Approve,
    Decline,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalResolution {
    Approved,
    Declined,
    Canceled,
    ResolvedExternally,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConversationApprovalKind {
    CommandExecution,
    FileChange,
    Permissions,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationApprovalDetail {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationApproval {
    pub approval_id: String,
    pub activity_id: String,
    pub kind: ConversationApprovalKind,
    pub title: String,
    pub reason: Option<String>,
    pub details: Vec<ConversationApprovalDetail>,
    pub decisions: Vec<ConversationApprovalDecision>,
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
