use std::{
    collections::{HashSet, VecDeque},
    ffi::OsString,
    process::Stdio,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::timeout,
};
use uuid::Uuid;

use super::{
    error::CodexAdapterError,
    integration::IntegrationRefreshReason,
    types::{CodexModel, NormalizedCodexEvent},
};

const MAX_PROTOCOL_LINE_BYTES: usize = 1024 * 1024;
const MAX_MODELS: usize = 256;
const MAX_MODEL_ID_BYTES: usize = 128;
const MAX_DISPLAY_NAME_BYTES: usize = 128;
const MAX_REASONING_EFFORTS: usize = 12;
const MAX_REASONING_EFFORT_BYTES: usize = 32;
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub(crate) struct AppServerCommand {
    program: OsString,
    args: Vec<OsString>,
}

impl AppServerCommand {
    pub(crate) fn codex(program: &str) -> Self {
        Self {
            program: program.into(),
            args: vec!["app-server".into(), "--listen".into(), "stdio://".into()],
        }
    }

    #[cfg(test)]
    pub(crate) fn test(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(OsString::from).collect(),
        }
    }
}

pub(crate) struct AppServerProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Lines<BufReader<ChildStdout>>,
    next_request_id: u64,
    request_timeout: Duration,
    notifications: VecDeque<AppServerNotification>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AppServerNotification {
    AccountLoginCompleted {
        login_id: Option<String>,
        success: bool,
    },
    AccountUpdated,
    McpOauthLoginCompleted {
        name: String,
        success: bool,
    },
    Conversation(ConversationNotification),
    ConversationRequest(ConversationServerRequest),
    IntegrationRefresh(IntegrationRefreshReason),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub(crate) enum ServerRequestId {
    String(String),
    Integer(i64),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ConversationServerDecision {
    Accept,
    Decline,
    Cancel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConversationServerRequest {
    DynamicTool {
        request_id: ServerRequestId,
        thread_id: String,
        turn_id: String,
        call_id: String,
        namespace: Option<String>,
        tool: String,
        arguments: Value,
    },
    CommandExecution {
        request_id: ServerRequestId,
        thread_id: String,
        turn_id: String,
        item_id: String,
        command: Option<String>,
        cwd: Option<String>,
        reason: Option<String>,
        additional_permissions: Option<Value>,
        network_host: Option<String>,
        network_protocol: Option<String>,
        available_decisions: Vec<ConversationServerDecision>,
    },
    FileChange {
        request_id: ServerRequestId,
        thread_id: String,
        turn_id: String,
        item_id: String,
        grant_root: Option<String>,
        reason: Option<String>,
    },
    Permissions {
        request_id: ServerRequestId,
        thread_id: String,
        turn_id: String,
        item_id: String,
        cwd: String,
        permissions: Value,
        reason: Option<String>,
    },
}

impl ConversationServerRequest {
    pub(crate) fn request_id(&self) -> &ServerRequestId {
        match self {
            Self::DynamicTool { request_id, .. }
            | Self::CommandExecution { request_id, .. }
            | Self::FileChange { request_id, .. }
            | Self::Permissions { request_id, .. } => request_id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConversationNotification {
    ThreadStarted {
        thread_id: String,
    },
    ThreadArchived {
        thread_id: String,
    },
    ThreadUnarchived {
        thread_id: String,
    },
    TurnStarted {
        thread_id: String,
        turn_id: String,
    },
    AgentMessageDelta {
        thread_id: String,
        turn_id: String,
        delta: String,
    },
    ReasoningSummaryDelta {
        thread_id: String,
        turn_id: String,
        delta: String,
    },
    PlanUpdated {
        thread_id: String,
        turn_id: String,
        explanation: Option<String>,
        steps: Vec<ConversationPlanStep>,
    },
    ItemLifecycle {
        thread_id: String,
        turn_id: String,
        item: ConversationItem,
    },
    ActivityDelta {
        thread_id: String,
        turn_id: String,
        item_id: String,
        kind: ConversationActivityDeltaKind,
        delta: String,
    },
    ServerRequestResolved {
        thread_id: String,
        request_id: ServerRequestId,
    },
    TurnCompleted {
        thread_id: String,
        turn_id: String,
        status: ConversationTurnStatus,
    },
    Error {
        thread_id: String,
        turn_id: String,
        code: ConversationErrorCode,
        will_retry: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationActivityDeltaKind {
    CommandOutput,
    ToolProgress,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConversationItem {
    pub item_id: String,
    pub kind: ConversationItemKind,
    pub status: ConversationItemStatus,
    pub detail: ConversationItemDetail,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ConversationItemDetail {
    CommandExecution {
        command: String,
        cwd: String,
        exit_code: Option<i32>,
    },
    FileChange {
        paths: Vec<String>,
    },
    ToolCall {
        server: Option<String>,
        tool: String,
        app_name: Option<String>,
        action_name: Option<String>,
    },
    WebSearch {
        query: String,
    },
    None,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConversationPlanStep {
    pub step: String,
    pub status: ConversationPlanStepStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationPlanStepStatus {
    Pending,
    InProgress,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationItemStatus {
    Started,
    Completed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationItemKind {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationTurnStatus {
    Completed,
    Interrupted,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ConversationErrorCode {
    ContextWindowExceeded,
    UsageLimitExceeded,
    Unauthorized,
    Sandbox,
    Server,
    Other,
}

impl AppServerProcess {
    pub(crate) fn spawn(command: AppServerCommand) -> Result<Self, CodexAdapterError> {
        Self::spawn_with_timeout(command, Duration::from_secs(5))
    }

    pub(crate) fn spawn_with_timeout(
        command: AppServerCommand,
        request_timeout: Duration,
    ) -> Result<Self, CodexAdapterError> {
        let mut child = Command::new(&command.program)
            .args(&command.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|_| CodexAdapterError::ProcessSpawnFailed)?;

        let stdin = child
            .stdin
            .take()
            .ok_or(CodexAdapterError::ProcessSpawnFailed)?;
        let stdout = child
            .stdout
            .take()
            .ok_or(CodexAdapterError::ProcessSpawnFailed)?;

        Ok(Self {
            child,
            stdin: Some(stdin),
            lines: BufReader::new(stdout).lines(),
            next_request_id: 1,
            request_timeout,
            notifications: VecDeque::new(),
        })
    }

    pub(crate) async fn initialize(&mut self) -> Result<(), CodexAdapterError> {
        let result = self
            .request(
                "initialize",
                json!({
                    "clientInfo": {
                        "name": "quireforge",
                        "title": "QuireForge",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }),
            )
            .await?;

        if !result.is_object() {
            return Err(CodexAdapterError::InvalidProtocolMessage);
        }

        Ok(())
    }

    pub(crate) async fn discover_models(
        &mut self,
    ) -> Result<(Vec<CodexModel>, Vec<NormalizedCodexEvent>), CodexAdapterError> {
        let mut events = Vec::new();
        self.initialize().await?;
        events.push(NormalizedCodexEvent::ProtocolReady);

        let model_result = self.request("model/list", json!({})).await?;
        let models = parse_model_catalog(model_result)?;
        events.push(NormalizedCodexEvent::ModelCatalog {
            model_count: models.len(),
        });

        Ok((models, events))
    }

    pub(crate) async fn request(
        &mut self,
        method: &str,
        params: Value,
    ) -> Result<Value, CodexAdapterError> {
        let request_id = self.next_request_id;
        self.next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(CodexAdapterError::InvalidProtocolMessage)?;

        let encoded = serde_json::to_vec(&json!({
            "method": method,
            "id": request_id,
            "params": params
        }))
        .map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;

        if encoded.len() > MAX_PROTOCOL_LINE_BYTES {
            return Err(CodexAdapterError::MessageTooLarge);
        }

        let stdin = self
            .stdin
            .as_mut()
            .ok_or(CodexAdapterError::TransportClosed)?;
        stdin
            .write_all(&encoded)
            .await
            .map_err(|_| CodexAdapterError::TransportClosed)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|_| CodexAdapterError::TransportClosed)?;
        stdin
            .flush()
            .await
            .map_err(|_| CodexAdapterError::TransportClosed)?;

        loop {
            let line = timeout(self.request_timeout, self.lines.next_line())
                .await
                .map_err(|_| CodexAdapterError::TransportTimeout)?
                .map_err(|_| CodexAdapterError::TransportClosed)?
                .ok_or(CodexAdapterError::ProcessExited)?;

            if line.len() > MAX_PROTOCOL_LINE_BYTES {
                return Err(CodexAdapterError::MessageTooLarge);
            }

            let message: Value = serde_json::from_str(&line)
                .map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;

            if message.get("method").and_then(Value::as_str).is_some() {
                if let Some(notification) = parse_notification(&message)? {
                    self.notifications.push_back(notification);
                }
                continue;
            }

            if message.get("id").and_then(Value::as_u64) == Some(request_id) {
                if message.get("error").is_some_and(|error| !error.is_null()) {
                    return Err(CodexAdapterError::RpcRejected);
                }

                return message
                    .get("result")
                    .cloned()
                    .ok_or(CodexAdapterError::InvalidProtocolMessage);
            }

            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
    }

    pub(crate) async fn respond_server_request(
        &mut self,
        request_id: &ServerRequestId,
        result: Value,
    ) -> Result<(), CodexAdapterError> {
        let encoded = serde_json::to_vec(&json!({
            "id": request_id,
            "result": result,
        }))
        .map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;
        if encoded.len() > MAX_PROTOCOL_LINE_BYTES {
            return Err(CodexAdapterError::MessageTooLarge);
        }
        let stdin = self
            .stdin
            .as_mut()
            .ok_or(CodexAdapterError::TransportClosed)?;
        stdin
            .write_all(&encoded)
            .await
            .map_err(|_| CodexAdapterError::TransportClosed)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|_| CodexAdapterError::TransportClosed)?;
        stdin
            .flush()
            .await
            .map_err(|_| CodexAdapterError::TransportClosed)
    }

    pub(crate) fn take_notification(&mut self) -> Option<AppServerNotification> {
        self.notifications.pop_front()
    }

    pub(crate) async fn next_notification(
        &mut self,
    ) -> Result<AppServerNotification, CodexAdapterError> {
        if let Some(notification) = self.take_notification() {
            return Ok(notification);
        }

        loop {
            let line = self
                .lines
                .next_line()
                .await
                .map_err(|_| CodexAdapterError::TransportClosed)?
                .ok_or(CodexAdapterError::ProcessExited)?;

            if line.len() > MAX_PROTOCOL_LINE_BYTES {
                return Err(CodexAdapterError::MessageTooLarge);
            }

            let message: Value = serde_json::from_str(&line)
                .map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;
            if let Some(notification) = parse_notification(&message)? {
                return Ok(notification);
            }

            if message.get("method").and_then(Value::as_str).is_some() {
                continue;
            }

            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
    }

    pub(crate) async fn next_notification_with_timeout(
        &mut self,
        wait: Duration,
    ) -> Result<Option<AppServerNotification>, CodexAdapterError> {
        if let Some(notification) = self.take_notification() {
            return Ok(Some(notification));
        }
        match timeout(wait, self.next_notification()).await {
            Ok(result) => result.map(Some),
            Err(_) => Ok(None),
        }
    }

    pub(crate) async fn shutdown(&mut self) -> Result<(), CodexAdapterError> {
        self.stdin.take();

        match timeout(SHUTDOWN_TIMEOUT, self.child.wait()).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) => Err(CodexAdapterError::ProcessExited),
            Err(_) => {
                self.child
                    .kill()
                    .await
                    .map_err(|_| CodexAdapterError::ProcessExited)?;
                self.child
                    .wait()
                    .await
                    .map_err(|_| CodexAdapterError::ProcessExited)?;
                Ok(())
            }
        }
    }
}

fn parse_notification(message: &Value) -> Result<Option<AppServerNotification>, CodexAdapterError> {
    let Some(method) = message.get("method").and_then(Value::as_str) else {
        return Ok(None);
    };
    if message.get("id").is_some_and(|id| !id.is_null()) {
        return parse_server_request(message, method)
            .map(AppServerNotification::ConversationRequest)
            .map(Some);
    }

    match method {
        "account/login/completed" => {
            #[derive(Deserialize)]
            #[serde(deny_unknown_fields, rename_all = "camelCase")]
            struct LoginCompleted {
                login_id: Option<String>,
                success: bool,
                #[serde(default)]
                error: Option<Value>,
            }

            let params: LoginCompleted = serde_json::from_value(
                message
                    .get("params")
                    .cloned()
                    .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
            )
            .map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;
            if let Some(login_id) = params.login_id.as_deref() {
                validate_protocol_identifier(login_id, 128)?;
            }
            // The error payload is intentionally observed only for shape validation and
            // immediately discarded. Frontend diagnostics use stable local codes.
            if params
                .error
                .as_ref()
                .is_some_and(|error| !error.is_null() && !error.is_string())
            {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }

            Ok(Some(AppServerNotification::AccountLoginCompleted {
                login_id: params.login_id,
                success: params.success,
            }))
        }
        "account/updated" => Ok(Some(AppServerNotification::AccountUpdated)),
        "mcpServer/oauthLogin/completed" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct McpOauthCompleted {
                name: String,
                success: bool,
                #[serde(default)]
                error: Option<Value>,
                #[serde(default)]
                thread_id: Option<Value>,
            }

            let params: McpOauthCompleted = notification_params(message)?;
            validate_protocol_identifier(&params.name, 128)?;
            if params
                .error
                .as_ref()
                .is_some_and(|error| !error.is_null() && !error.is_string())
                || params
                    .thread_id
                    .as_ref()
                    .is_some_and(|thread_id| !thread_id.is_null() && !thread_id.is_string())
            {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            Ok(Some(AppServerNotification::McpOauthLoginCompleted {
                name: params.name,
                success: params.success,
            }))
        }
        "app/list/updated" => {
            let params = message
                .get("params")
                .and_then(Value::as_object)
                .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
            if params.len() != 1
                || !params
                    .get("data")
                    .is_some_and(|data| data.as_array().is_some_and(|items| items.len() <= 512))
            {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            Ok(Some(AppServerNotification::IntegrationRefresh(
                IntegrationRefreshReason::AppListUpdated,
            )))
        }
        "skills/changed" => {
            if !message
                .get("params")
                .is_some_and(|params| params.as_object().is_some_and(|params| params.is_empty()))
            {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            Ok(Some(AppServerNotification::IntegrationRefresh(
                IntegrationRefreshReason::SkillsChanged,
            )))
        }
        "mcpServer/startupStatus/updated" => {
            let params = message
                .get("params")
                .and_then(Value::as_object)
                .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
            if !params.get("name").is_some_and(Value::is_string)
                || !params.get("status").is_some_and(Value::is_string)
                || params.keys().any(|key| {
                    !matches!(
                        key.as_str(),
                        "name" | "status" | "error" | "failureReason" | "threadId"
                    )
                })
            {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            Ok(Some(AppServerNotification::IntegrationRefresh(
                IntegrationRefreshReason::McpStatusUpdated,
            )))
        }
        "config/warning" => {
            let params = message
                .get("params")
                .and_then(Value::as_object)
                .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
            if !params.get("summary").is_some_and(Value::is_string)
                || params
                    .keys()
                    .any(|key| !matches!(key.as_str(), "summary" | "details" | "path" | "range"))
            {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            Ok(Some(AppServerNotification::IntegrationRefresh(
                IntegrationRefreshReason::ConfigWarning,
            )))
        }
        "thread/started" => {
            #[derive(Deserialize)]
            struct Thread {
                id: String,
            }
            #[derive(Deserialize)]
            struct Params {
                thread: Thread,
            }
            let params: Params = notification_params(message)?;
            validate_uuid_v7(&params.thread.id)?;
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::ThreadStarted {
                    thread_id: params.thread.id,
                },
            )))
        }
        "thread/archived" | "thread/unarchived" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
            }
            let params: Params = notification_params(message)?;
            validate_uuid_v7(&params.thread_id)?;
            let notification = if method == "thread/archived" {
                ConversationNotification::ThreadArchived {
                    thread_id: params.thread_id,
                }
            } else {
                ConversationNotification::ThreadUnarchived {
                    thread_id: params.thread_id,
                }
            };
            Ok(Some(AppServerNotification::Conversation(notification)))
        }
        "turn/started" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Turn {
                id: String,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                turn: Turn,
            }
            let params: Params = notification_params(message)?;
            validate_uuid_v7(&params.thread_id)?;
            validate_uuid_v7(&params.turn.id)?;
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::TurnStarted {
                    thread_id: params.thread_id,
                    turn_id: params.turn.id,
                },
            )))
        }
        "item/agentMessage/delta" | "item/reasoning/summaryTextDelta" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                turn_id: String,
                delta: String,
            }
            let params: Params = notification_params(message)?;
            validate_conversation_ids(&params.thread_id, &params.turn_id)?;
            validate_stream_text(&params.delta, 64 * 1024)?;
            let notification = if method == "item/agentMessage/delta" {
                ConversationNotification::AgentMessageDelta {
                    thread_id: params.thread_id,
                    turn_id: params.turn_id,
                    delta: params.delta,
                }
            } else {
                ConversationNotification::ReasoningSummaryDelta {
                    thread_id: params.thread_id,
                    turn_id: params.turn_id,
                    delta: params.delta,
                }
            };
            Ok(Some(AppServerNotification::Conversation(notification)))
        }
        "turn/plan/updated" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct WireStep {
                step: String,
                status: String,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                turn_id: String,
                explanation: Option<String>,
                plan: Vec<WireStep>,
            }
            let params: Params = notification_params(message)?;
            validate_conversation_ids(&params.thread_id, &params.turn_id)?;
            if params.plan.len() > 128 {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            if let Some(explanation) = params.explanation.as_deref() {
                validate_stream_text(explanation, 4096)?;
            }
            let steps = params
                .plan
                .into_iter()
                .map(|step| {
                    validate_stream_text(&step.step, 4096)?;
                    let status = match step.status.as_str() {
                        "pending" => ConversationPlanStepStatus::Pending,
                        "inProgress" => ConversationPlanStepStatus::InProgress,
                        "completed" => ConversationPlanStepStatus::Completed,
                        _ => return Err(CodexAdapterError::InvalidProtocolMessage),
                    };
                    Ok(ConversationPlanStep {
                        step: step.step,
                        status,
                    })
                })
                .collect::<Result<Vec<_>, CodexAdapterError>>()?;
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::PlanUpdated {
                    thread_id: params.thread_id,
                    turn_id: params.turn_id,
                    explanation: params.explanation,
                    steps,
                },
            )))
        }
        "item/started" | "item/completed" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                turn_id: String,
                item: Value,
            }
            let params: Params = notification_params(message)?;
            validate_conversation_ids(&params.thread_id, &params.turn_id)?;
            let status = if method == "item/started" {
                ConversationItemStatus::Started
            } else {
                ConversationItemStatus::Completed
            };
            let item = parse_conversation_item(params.item, status)?;
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::ItemLifecycle {
                    thread_id: params.thread_id,
                    turn_id: params.turn_id,
                    item,
                },
            )))
        }
        "item/commandExecution/outputDelta" | "item/mcpToolCall/progress" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                turn_id: String,
                item_id: String,
                #[serde(default)]
                delta: Option<String>,
                #[serde(default)]
                message: Option<String>,
            }
            let params: Params = notification_params(message)?;
            validate_conversation_ids(&params.thread_id, &params.turn_id)?;
            validate_protocol_identifier(&params.item_id, 128)?;
            let (kind, delta) = if method == "item/commandExecution/outputDelta" {
                (
                    ConversationActivityDeltaKind::CommandOutput,
                    params
                        .delta
                        .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
                )
            } else {
                (
                    ConversationActivityDeltaKind::ToolProgress,
                    params
                        .message
                        .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
                )
            };
            validate_bounded_protocol_text(&delta, 64 * 1024)?;
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::ActivityDelta {
                    thread_id: params.thread_id,
                    turn_id: params.turn_id,
                    item_id: params.item_id,
                    kind,
                    delta,
                },
            )))
        }
        "serverRequest/resolved" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Params {
                thread_id: String,
                request_id: Value,
            }
            let params: Params = notification_params(message)?;
            validate_uuid_v7(&params.thread_id)?;
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::ServerRequestResolved {
                    thread_id: params.thread_id,
                    request_id: parse_server_request_id(&params.request_id)?,
                },
            )))
        }
        "turn/completed" => {
            #[derive(Deserialize)]
            struct Turn {
                id: String,
                status: String,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                turn: Turn,
            }
            let params: Params = notification_params(message)?;
            validate_conversation_ids(&params.thread_id, &params.turn.id)?;
            let status = match params.turn.status.as_str() {
                "completed" => ConversationTurnStatus::Completed,
                "interrupted" => ConversationTurnStatus::Interrupted,
                "failed" => ConversationTurnStatus::Failed,
                _ => return Err(CodexAdapterError::InvalidProtocolMessage),
            };
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::TurnCompleted {
                    thread_id: params.thread_id,
                    turn_id: params.turn.id,
                    status,
                },
            )))
        }
        "error" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct WireError {
                codex_error_info: Option<Value>,
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                thread_id: String,
                turn_id: String,
                error: WireError,
                will_retry: bool,
            }
            let params: Params = notification_params(message)?;
            validate_conversation_ids(&params.thread_id, &params.turn_id)?;
            let code = match params
                .error
                .codex_error_info
                .as_ref()
                .and_then(Value::as_str)
            {
                Some("contextWindowExceeded") => ConversationErrorCode::ContextWindowExceeded,
                Some("sessionBudgetExceeded" | "usageLimitExceeded") => {
                    ConversationErrorCode::UsageLimitExceeded
                }
                Some("unauthorized") => ConversationErrorCode::Unauthorized,
                Some("sandboxError") => ConversationErrorCode::Sandbox,
                Some("serverOverloaded" | "internalServerError") => ConversationErrorCode::Server,
                _ => ConversationErrorCode::Other,
            };
            Ok(Some(AppServerNotification::Conversation(
                ConversationNotification::Error {
                    thread_id: params.thread_id,
                    turn_id: params.turn_id,
                    code,
                    will_retry: params.will_retry,
                },
            )))
        }
        _ => Ok(None),
    }
}

fn parse_server_request(
    message: &Value,
    method: &str,
) -> Result<ConversationServerRequest, CodexAdapterError> {
    let request_id = parse_server_request_id(
        message
            .get("id")
            .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
    )?;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct CommonApprovalParams {
        thread_id: String,
        turn_id: String,
        item_id: String,
        started_at_ms: i64,
        #[serde(default)]
        reason: Option<String>,
    }

    fn validate_common(params: &CommonApprovalParams) -> Result<(), CodexAdapterError> {
        validate_conversation_ids(&params.thread_id, &params.turn_id)?;
        validate_protocol_identifier(&params.item_id, 128)?;
        if params.started_at_ms < 0 {
            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
        if let Some(reason) = params.reason.as_deref() {
            validate_bounded_protocol_text(reason, 4096)?;
        }
        Ok(())
    }

    match method {
        "item/tool/call" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase", deny_unknown_fields)]
            struct Params {
                thread_id: String,
                turn_id: String,
                call_id: String,
                #[serde(default)]
                namespace: Option<String>,
                tool: String,
                arguments: Value,
            }
            let params: Params = notification_params(message)?;
            validate_conversation_ids(&params.thread_id, &params.turn_id)?;
            validate_protocol_identifier(&params.call_id, 128)?;
            validate_protocol_identifier(&params.tool, 128)?;
            if let Some(namespace) = params.namespace.as_deref() {
                validate_protocol_identifier(namespace, 128)?;
            }
            if !params.arguments.is_object() {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            Ok(ConversationServerRequest::DynamicTool {
                request_id,
                thread_id: params.thread_id,
                turn_id: params.turn_id,
                call_id: params.call_id,
                namespace: params.namespace,
                tool: params.tool,
                arguments: params.arguments,
            })
        }
        "item/commandExecution/requestApproval" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(flatten)]
                common: CommonApprovalParams,
                #[serde(default)]
                command: Option<String>,
                #[serde(default)]
                cwd: Option<String>,
                #[serde(default)]
                available_decisions: Option<Vec<Value>>,
                #[serde(default)]
                additional_permissions: Option<RequestPermissionProfile>,
                #[serde(default)]
                network_approval_context: Option<NetworkApprovalContext>,
            }
            let params: Params = notification_params(message)?;
            validate_common(&params.common)?;
            if let Some(command) = params.command.as_deref() {
                validate_bounded_protocol_text(command, 64 * 1024)?;
            }
            if let Some(cwd) = params.cwd.as_deref() {
                validate_absolute_path(cwd)?;
            }
            if let Some(permissions) = params.additional_permissions.as_ref() {
                permissions.validate()?;
            }
            if let Some(context) = params.network_approval_context.as_ref() {
                context.validate()?;
            }
            let additional_permissions = params
                .additional_permissions
                .map(serde_json::to_value)
                .transpose()
                .map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;
            let available_decisions = parse_available_decisions(params.available_decisions)?;
            Ok(ConversationServerRequest::CommandExecution {
                request_id,
                thread_id: params.common.thread_id,
                turn_id: params.common.turn_id,
                item_id: params.common.item_id,
                command: params.command,
                cwd: params.cwd,
                reason: params.common.reason,
                additional_permissions,
                network_host: params
                    .network_approval_context
                    .as_ref()
                    .map(|context| context.host.clone()),
                network_protocol: params
                    .network_approval_context
                    .map(|context| context.protocol.as_str().to_owned()),
                available_decisions,
            })
        }
        "item/fileChange/requestApproval" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(flatten)]
                common: CommonApprovalParams,
                #[serde(default)]
                grant_root: Option<String>,
            }
            let params: Params = notification_params(message)?;
            validate_common(&params.common)?;
            if let Some(grant_root) = params.grant_root.as_deref() {
                validate_absolute_path(grant_root)?;
            }
            Ok(ConversationServerRequest::FileChange {
                request_id,
                thread_id: params.common.thread_id,
                turn_id: params.common.turn_id,
                item_id: params.common.item_id,
                grant_root: params.grant_root,
                reason: params.common.reason,
            })
        }
        "item/permissions/requestApproval" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Params {
                #[serde(flatten)]
                common: CommonApprovalParams,
                cwd: String,
                permissions: RequestPermissionProfile,
            }
            let params: Params = notification_params(message)?;
            validate_common(&params.common)?;
            validate_absolute_path(&params.cwd)?;
            params.permissions.validate()?;
            let permissions = serde_json::to_value(params.permissions)
                .map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;
            Ok(ConversationServerRequest::Permissions {
                request_id,
                thread_id: params.common.thread_id,
                turn_id: params.common.turn_id,
                item_id: params.common.item_id,
                cwd: params.cwd,
                permissions,
                reason: params.common.reason,
            })
        }
        _ => Err(CodexAdapterError::UnexpectedServerRequest),
    }
}

fn parse_server_request_id(value: &Value) -> Result<ServerRequestId, CodexAdapterError> {
    if let Some(value) = value.as_str() {
        validate_protocol_identifier(value, 128)?;
        return Ok(ServerRequestId::String(value.to_owned()));
    }
    value
        .as_i64()
        .map(ServerRequestId::Integer)
        .ok_or(CodexAdapterError::InvalidProtocolMessage)
}

fn parse_available_decisions(
    values: Option<Vec<Value>>,
) -> Result<Vec<ConversationServerDecision>, CodexAdapterError> {
    let Some(values) = values else {
        return Ok(vec![
            ConversationServerDecision::Accept,
            ConversationServerDecision::Decline,
            ConversationServerDecision::Cancel,
        ]);
    };
    if values.len() > 16 {
        return Err(CodexAdapterError::InvalidProtocolMessage);
    }
    let mut decisions = Vec::new();
    for value in values {
        let decision = match &value {
            Value::String(value) if value == "accept" => Some(ConversationServerDecision::Accept),
            Value::String(value) if value == "decline" => Some(ConversationServerDecision::Decline),
            Value::String(value) if value == "cancel" => Some(ConversationServerDecision::Cancel),
            Value::String(value) if value == "acceptForSession" => None,
            Value::Object(_) => None,
            _ => return Err(CodexAdapterError::InvalidProtocolMessage),
        };
        if let Some(decision) = decision {
            if !decisions.contains(&decision) {
                decisions.push(decision);
            }
        }
    }
    if decisions.is_empty() {
        return Err(CodexAdapterError::InvalidProtocolMessage);
    }
    Ok(decisions)
}

fn parse_conversation_item(
    item: Value,
    status: ConversationItemStatus,
) -> Result<ConversationItem, CodexAdapterError> {
    let object = item
        .as_object()
        .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
    let item_id = required_string(object.get("id"), 128)?;
    validate_protocol_identifier(&item_id, 128)?;
    let item_type = required_string(object.get("type"), 64)?;

    let (kind, detail) = match item_type.as_str() {
        "userMessage" => (
            ConversationItemKind::UserMessage,
            ConversationItemDetail::None,
        ),
        "agentMessage" => (
            ConversationItemKind::AgentMessage,
            ConversationItemDetail::None,
        ),
        "plan" => (ConversationItemKind::Plan, ConversationItemDetail::None),
        "reasoning" => (
            ConversationItemKind::Reasoning,
            ConversationItemDetail::None,
        ),
        "commandExecution" => {
            let command = required_bounded_text(object.get("command"), 64 * 1024)?;
            let cwd = required_bounded_text(object.get("cwd"), 4096)?;
            validate_absolute_path(&cwd)?;
            validate_enum_string(
                object.get("status"),
                &["inProgress", "completed", "failed", "declined"],
            )?;
            let command_actions = object
                .get("commandActions")
                .and_then(Value::as_array)
                .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
            if command_actions.len() > 128 {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            let exit_code = object
                .get("exitCode")
                .filter(|value| !value.is_null())
                .map(|value| {
                    value
                        .as_i64()
                        .and_then(|value| i32::try_from(value).ok())
                        .ok_or(CodexAdapterError::InvalidProtocolMessage)
                })
                .transpose()?;
            (
                ConversationItemKind::CommandExecution,
                ConversationItemDetail::CommandExecution {
                    command,
                    cwd,
                    exit_code,
                },
            )
        }
        "fileChange" => {
            validate_enum_string(
                object.get("status"),
                &["inProgress", "completed", "failed", "declined"],
            )?;
            let changes = object
                .get("changes")
                .and_then(Value::as_array)
                .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
            if changes.len() > 128 {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            let paths = changes
                .iter()
                .map(|change| {
                    let change = change
                        .as_object()
                        .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
                    let path = required_bounded_text(change.get("path"), 4096)?;
                    validate_bounded_protocol_text(
                        change
                            .get("diff")
                            .and_then(Value::as_str)
                            .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
                        256 * 1024,
                    )?;
                    validate_patch_kind(
                        change
                            .get("kind")
                            .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
                    )?;
                    Ok(path)
                })
                .collect::<Result<Vec<_>, CodexAdapterError>>()?;
            (
                ConversationItemKind::FileChange,
                ConversationItemDetail::FileChange { paths },
            )
        }
        "mcpToolCall" => {
            if !object.contains_key("arguments") {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            validate_enum_string(object.get("status"), &["inProgress", "completed", "failed"])?;
            let server = required_string(object.get("server"), 128)?;
            validate_protocol_identifier(&server, 128)?;
            let tool = required_string(object.get("tool"), 128)?;
            validate_protocol_identifier(&tool, 128)?;
            let (app_name, action_name) = parse_app_context(object.get("appContext"))?;
            (
                ConversationItemKind::ToolCall,
                ConversationItemDetail::ToolCall {
                    server: Some(server),
                    tool,
                    app_name,
                    action_name,
                },
            )
        }
        "dynamicToolCall" => {
            if !object.contains_key("arguments") {
                return Err(CodexAdapterError::InvalidProtocolMessage);
            }
            validate_enum_string(object.get("status"), &["inProgress", "completed", "failed"])?;
            let tool = required_string(object.get("tool"), 128)?;
            validate_protocol_identifier(&tool, 128)?;
            (
                ConversationItemKind::ToolCall,
                ConversationItemDetail::ToolCall {
                    server: None,
                    tool,
                    app_name: None,
                    action_name: None,
                },
            )
        }
        "collabAgentToolCall" | "subAgentActivity" => (
            ConversationItemKind::ToolCall,
            ConversationItemDetail::ToolCall {
                server: None,
                tool: "collaboration".to_owned(),
                app_name: None,
                action_name: None,
            },
        ),
        "webSearch" => {
            let query = required_bounded_text(object.get("query"), 4096)?;
            (
                ConversationItemKind::WebSearch,
                ConversationItemDetail::WebSearch { query },
            )
        }
        "imageView" | "imageGeneration" => {
            (ConversationItemKind::Image, ConversationItemDetail::None)
        }
        _ => (ConversationItemKind::Other, ConversationItemDetail::None),
    };

    Ok(ConversationItem {
        item_id,
        kind,
        status,
        detail,
    })
}

fn validate_patch_kind(value: &Value) -> Result<(), CodexAdapterError> {
    let object = value
        .as_object()
        .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
    match object.get("type").and_then(Value::as_str) {
        Some("add" | "delete") if object.len() == 1 => Ok(()),
        Some("update") if object.keys().all(|key| key == "type" || key == "movePath") => {
            if let Some(path) = object.get("movePath").filter(|path| !path.is_null()) {
                validate_bounded_protocol_text(
                    path.as_str()
                        .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
                    4096,
                )?;
            }
            Ok(())
        }
        _ => Err(CodexAdapterError::InvalidProtocolMessage),
    }
}

fn parse_app_context(
    value: Option<&Value>,
) -> Result<(Option<String>, Option<String>), CodexAdapterError> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok((None, None));
    };
    let object = value
        .as_object()
        .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
    let app_name = object
        .get("appName")
        .and_then(Value::as_str)
        .map(|value| {
            validate_bounded_protocol_text(value, 128)?;
            Ok(value.to_owned())
        })
        .transpose()?;
    let action_name = object
        .get("actionName")
        .and_then(Value::as_str)
        .map(|value| {
            validate_bounded_protocol_text(value, 128)?;
            Ok(value.to_owned())
        })
        .transpose()?;
    Ok((app_name, action_name))
}

fn required_string(value: Option<&Value>, max_bytes: usize) -> Result<String, CodexAdapterError> {
    let value = value
        .and_then(Value::as_str)
        .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
    validate_bounded_protocol_text(value, max_bytes)?;
    Ok(value.to_owned())
}

fn required_bounded_text(
    value: Option<&Value>,
    max_bytes: usize,
) -> Result<String, CodexAdapterError> {
    required_string(value, max_bytes)
}

fn validate_enum_string(value: Option<&Value>, allowed: &[&str]) -> Result<(), CodexAdapterError> {
    let value = value
        .and_then(Value::as_str)
        .ok_or(CodexAdapterError::InvalidProtocolMessage)?;
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(CodexAdapterError::InvalidProtocolMessage)
    }
}

fn validate_absolute_path(value: &str) -> Result<(), CodexAdapterError> {
    validate_bounded_protocol_text(value, 4096)?;
    if !std::path::Path::new(value).is_absolute() {
        return Err(CodexAdapterError::InvalidProtocolMessage);
    }
    Ok(())
}

fn validate_bounded_protocol_text(value: &str, max_bytes: usize) -> Result<(), CodexAdapterError> {
    if value.len() > max_bytes || value.contains('\0') {
        return Err(CodexAdapterError::InvalidProtocolMessage);
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RequestPermissionProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file_system: Option<AdditionalFileSystemPermissions>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    network: Option<AdditionalNetworkPermissions>,
}

impl RequestPermissionProfile {
    fn validate(&self) -> Result<(), CodexAdapterError> {
        if let Some(file_system) = &self.file_system {
            file_system.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AdditionalFileSystemPermissions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    entries: Option<Vec<FileSystemSandboxEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    glob_scan_max_depth: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    read: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    write: Option<Vec<String>>,
}

impl AdditionalFileSystemPermissions {
    fn validate(&self) -> Result<(), CodexAdapterError> {
        if self.glob_scan_max_depth == Some(0) {
            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
        if self
            .entries
            .as_ref()
            .is_some_and(|entries| entries.len() > 64)
            || self.read.as_ref().is_some_and(|paths| paths.len() > 64)
            || self.write.as_ref().is_some_and(|paths| paths.len() > 64)
        {
            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
        for entry in self.entries.iter().flatten() {
            entry.validate()?;
        }
        for path in self
            .read
            .iter()
            .flatten()
            .chain(self.write.iter().flatten())
        {
            validate_bounded_protocol_text(path, 4096)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AdditionalNetworkPermissions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NetworkApprovalContext {
    host: String,
    protocol: NetworkApprovalProtocol,
}

impl NetworkApprovalContext {
    fn validate(&self) -> Result<(), CodexAdapterError> {
        if self.host.is_empty() {
            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
        validate_bounded_protocol_text(&self.host, 253)
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum NetworkApprovalProtocol {
    Http,
    Https,
    Socks5Tcp,
    Socks5Udp,
}

impl NetworkApprovalProtocol {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Socks5Tcp => "socks5 TCP",
            Self::Socks5Udp => "socks5 UDP",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileSystemSandboxEntry {
    access: FileSystemAccessMode,
    path: FileSystemPath,
}

impl FileSystemSandboxEntry {
    fn validate(&self) -> Result<(), CodexAdapterError> {
        self.path.validate()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum FileSystemAccessMode {
    Read,
    Write,
    Deny,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum FileSystemPath {
    Path { path: String },
    GlobPattern { pattern: String },
    Special { value: FileSystemSpecialPath },
}

impl FileSystemPath {
    fn validate(&self) -> Result<(), CodexAdapterError> {
        match self {
            Self::Path { path } => validate_bounded_protocol_text(path, 4096),
            Self::GlobPattern { pattern } => validate_bounded_protocol_text(pattern, 4096),
            Self::Special { value } => value.validate(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum FileSystemSpecialPath {
    Root,
    Minimal,
    ProjectRoots {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subpath: Option<String>,
    },
    Tmpdir,
    SlashTmp,
    Unknown {
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subpath: Option<String>,
    },
}

impl FileSystemSpecialPath {
    fn validate(&self) -> Result<(), CodexAdapterError> {
        match self {
            Self::ProjectRoots {
                subpath: Some(path),
            }
            | Self::Unknown {
                path,
                subpath: None,
            } => validate_bounded_protocol_text(path, 4096),
            Self::Unknown {
                path,
                subpath: Some(subpath),
            } => {
                validate_bounded_protocol_text(path, 4096)?;
                validate_bounded_protocol_text(subpath, 4096)
            }
            Self::Root
            | Self::Minimal
            | Self::ProjectRoots { subpath: None }
            | Self::Tmpdir
            | Self::SlashTmp => Ok(()),
        }
    }
}

fn notification_params<T: for<'de> Deserialize<'de>>(
    message: &Value,
) -> Result<T, CodexAdapterError> {
    serde_json::from_value(
        message
            .get("params")
            .cloned()
            .ok_or(CodexAdapterError::InvalidProtocolMessage)?,
    )
    .map_err(|_| CodexAdapterError::InvalidProtocolMessage)
}

fn validate_conversation_ids(thread_id: &str, turn_id: &str) -> Result<(), CodexAdapterError> {
    validate_uuid_v7(thread_id)?;
    validate_uuid_v7(turn_id)
}

pub(crate) fn validate_uuid_v7(value: &str) -> Result<(), CodexAdapterError> {
    let parsed = Uuid::parse_str(value).map_err(|_| CodexAdapterError::InvalidProtocolMessage)?;
    if parsed.get_version_num() != 7 || parsed.to_string() != value {
        return Err(CodexAdapterError::InvalidProtocolMessage);
    }
    Ok(())
}

fn validate_stream_text(value: &str, max_bytes: usize) -> Result<(), CodexAdapterError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.chars().any(|character| {
            (character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
                || matches!(
                    character,
                    '\u{200B}'..='\u{200F}'
                        | '\u{202A}'..='\u{202E}'
                        | '\u{2060}'..='\u{206F}'
                        | '\u{FEFF}'
                )
        })
    {
        return Err(CodexAdapterError::InvalidProtocolMessage);
    }
    Ok(())
}

pub(crate) fn validate_protocol_identifier(
    value: &str,
    max_bytes: usize,
) -> Result<(), CodexAdapterError> {
    if value.is_empty()
        || value.len() > max_bytes
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
    {
        return Err(CodexAdapterError::InvalidProtocolMessage);
    }

    Ok(())
}

impl Drop for AppServerProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModelListResult {
    data: Vec<WireModel>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireModel {
    model: String,
    display_name: String,
    #[serde(default)]
    is_default: bool,
    default_reasoning_effort: String,
    supported_reasoning_efforts: Vec<WireReasoningEffort>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireReasoningEffort {
    reasoning_effort: String,
}

fn parse_model_catalog(value: Value) -> Result<Vec<CodexModel>, CodexAdapterError> {
    let result: ModelListResult =
        serde_json::from_value(value).map_err(|_| CodexAdapterError::InvalidModelCatalog)?;

    if result.data.len() > MAX_MODELS {
        return Err(CodexAdapterError::InvalidModelCatalog);
    }

    let mut seen_models = HashSet::with_capacity(result.data.len());
    let mut default_models = 0_usize;
    let models = result
        .data
        .into_iter()
        .map(|model| {
            validate_identifier(&model.model, MAX_MODEL_ID_BYTES)?;
            validate_display_text(&model.display_name, MAX_DISPLAY_NAME_BYTES)?;
            validate_identifier(&model.default_reasoning_effort, MAX_REASONING_EFFORT_BYTES)?;

            if !seen_models.insert(model.model.clone()) {
                return Err(CodexAdapterError::InvalidModelCatalog);
            }
            if model.is_default {
                default_models += 1;
            }

            if model.supported_reasoning_efforts.len() > MAX_REASONING_EFFORTS {
                return Err(CodexAdapterError::InvalidModelCatalog);
            }

            let efforts = model
                .supported_reasoning_efforts
                .into_iter()
                .map(|effort| {
                    validate_identifier(&effort.reasoning_effort, MAX_REASONING_EFFORT_BYTES)?;
                    Ok(effort.reasoning_effort)
                })
                .collect::<Result<Vec<_>, CodexAdapterError>>()?;

            if !efforts.contains(&model.default_reasoning_effort) {
                return Err(CodexAdapterError::InvalidModelCatalog);
            }

            Ok(CodexModel {
                id: model.model,
                display_name: model.display_name,
                is_default: model.is_default,
                default_reasoning_effort: model.default_reasoning_effort,
                supported_reasoning_efforts: efforts,
            })
        })
        .collect::<Result<Vec<_>, CodexAdapterError>>()?;

    if default_models > 1 {
        return Err(CodexAdapterError::InvalidModelCatalog);
    }

    Ok(models)
}

fn validate_identifier(value: &str, max_bytes: usize) -> Result<(), CodexAdapterError> {
    if value.is_empty()
        || value.len() > max_bytes
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':' | b'/')
        })
    {
        return Err(CodexAdapterError::InvalidModelCatalog);
    }

    Ok(())
}

fn validate_display_text(value: &str, max_bytes: usize) -> Result<(), CodexAdapterError> {
    if value.is_empty()
        || value.len() > max_bytes
        || value.chars().any(|character| {
            character.is_control()
                || matches!(
                    character,
                    '\u{200B}'..='\u{200F}'
                        | '\u{202A}'..='\u{202E}'
                        | '\u{2060}'..='\u{206F}'
                        | '\u{FEFF}'
                )
        })
    {
        return Err(CodexAdapterError::InvalidModelCatalog);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_sanitized_model_catalog_fixture() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../../../fixtures/codex-model-list-response.json"
        ))
        .expect("fixture must be JSON");
        let models = parse_model_catalog(fixture).expect("fixture must normalize");

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].id, "gpt-5.6-sol");
        assert!(models[0]
            .supported_reasoning_efforts
            .contains(&"high".to_owned()));
    }

    #[test]
    fn rejects_a_default_effort_missing_from_the_supported_set() {
        let fixture = json!({
            "data": [{
                "model": "safe-model",
                "displayName": "Safe model",
                "defaultReasoningEffort": "high",
                "supportedReasoningEfforts": [{"reasoningEffort": "low"}]
            }]
        });

        assert!(matches!(
            parse_model_catalog(fixture),
            Err(CodexAdapterError::InvalidModelCatalog)
        ));
    }

    #[test]
    fn rejects_duplicate_models_and_multiple_defaults() {
        let duplicate = json!({
            "data": [
                {
                    "model": "same-model",
                    "displayName": "First",
                    "isDefault": false,
                    "defaultReasoningEffort": "medium",
                    "supportedReasoningEfforts": [{"reasoningEffort": "medium"}]
                },
                {
                    "model": "same-model",
                    "displayName": "Second",
                    "isDefault": false,
                    "defaultReasoningEffort": "medium",
                    "supportedReasoningEfforts": [{"reasoningEffort": "medium"}]
                }
            ]
        });
        let multiple_defaults = json!({
            "data": [
                {
                    "model": "first-model",
                    "displayName": "First",
                    "isDefault": true,
                    "defaultReasoningEffort": "medium",
                    "supportedReasoningEfforts": [{"reasoningEffort": "medium"}]
                },
                {
                    "model": "second-model",
                    "displayName": "Second",
                    "isDefault": true,
                    "defaultReasoningEffort": "medium",
                    "supportedReasoningEfforts": [{"reasoningEffort": "medium"}]
                }
            ]
        });

        assert!(parse_model_catalog(duplicate).is_err());
        assert!(parse_model_catalog(multiple_defaults).is_err());
    }

    #[test]
    fn rejects_directional_controls_in_display_metadata() {
        let fixture = json!({
            "data": [{
                "model": "safe-model",
                "displayName": "Safe\u{202e}spoofed",
                "isDefault": true,
                "defaultReasoningEffort": "medium",
                "supportedReasoningEfforts": [{"reasoningEffort": "medium"}]
            }]
        });

        assert!(parse_model_catalog(fixture).is_err());
    }

    #[test]
    fn rejects_unsafe_or_uncorrelatable_conversation_notifications() {
        let unsafe_delta = json!({
            "method": "item/agentMessage/delta",
            "params": {
                "threadId": "018f0000-0000-7000-8000-000000000020",
                "turnId": "018f0000-0000-7000-8000-000000000030",
                "itemId": "item-1",
                "delta": "safe\u{202e}spoofed"
            }
        });
        let invalid_thread = json!({
            "method": "turn/completed",
            "params": {
                "threadId": "not-a-thread-id",
                "turn": {
                    "id": "018f0000-0000-7000-8000-000000000030",
                    "status": "completed"
                }
            }
        });

        assert!(parse_notification(&unsafe_delta).is_err());
        assert!(parse_notification(&invalid_thread).is_err());
    }

    #[test]
    fn normalizes_integration_invalidations_without_retaining_payloads() {
        let notifications = [
            (
                json!({
                    "method": "app/list/updated",
                    "params": {"data": [{"installUrl": "private-app-url"}]}
                }),
                IntegrationRefreshReason::AppListUpdated,
            ),
            (
                json!({"method": "skills/changed", "params": {}}),
                IntegrationRefreshReason::SkillsChanged,
            ),
            (
                json!({
                    "method": "mcpServer/startupStatus/updated",
                    "params": {
                        "name": "private-server-name",
                        "status": "failed",
                        "failureReason": "private failure"
                    }
                }),
                IntegrationRefreshReason::McpStatusUpdated,
            ),
            (
                json!({
                    "method": "config/warning",
                    "params": {
                        "summary": "private summary",
                        "details": "private details",
                        "path": "/private/config.toml"
                    }
                }),
                IntegrationRefreshReason::ConfigWarning,
            ),
        ];

        for (notification, expected) in notifications {
            let parsed = parse_notification(&notification).expect("invalidation must parse");
            assert_eq!(
                parsed,
                Some(AppServerNotification::IntegrationRefresh(expected))
            );
            let debug = format!("{parsed:?}");
            assert!(!debug.contains("private"));
        }
    }

    #[test]
    fn parses_only_reviewed_approval_fields_and_safe_one_turn_decisions() {
        let request = json!({
            "id": "approval-1",
            "method": "item/commandExecution/requestApproval",
            "params": {
                "threadId": "018f0000-0000-7000-8000-000000000020",
                "turnId": "018f0000-0000-7000-8000-000000000030",
                "itemId": "item-1",
                "startedAtMs": 1,
                "command": "cargo check --token private",
                "cwd": "/workspace/project",
                "reason": "Run a check",
                "availableDecisions": [
                    "accept",
                    "acceptForSession",
                    {"acceptWithExecpolicyAmendment": {"execpolicy_amendment": ["cargo"]}},
                    "decline",
                    "cancel"
                ],
                "proposedExecpolicyAmendment": ["private value is discarded"]
            }
        });

        let parsed = parse_notification(&request).expect("approval must parse");
        let debug = format!("{parsed:?}");
        let Some(AppServerNotification::ConversationRequest(
            ConversationServerRequest::CommandExecution {
                request_id,
                available_decisions,
                ..
            },
        )) = parsed
        else {
            panic!("command approval must normalize");
        };
        assert_eq!(request_id, ServerRequestId::String("approval-1".to_owned()));
        assert_eq!(
            available_decisions,
            vec![
                ConversationServerDecision::Accept,
                ConversationServerDecision::Decline,
                ConversationServerDecision::Cancel,
            ]
        );
        assert!(!debug.contains("execpolicy_amendment"));
        assert!(!debug.contains("private value is discarded"));
    }

    #[test]
    fn parses_only_correlated_dynamic_selector_calls() {
        let request = json!({
            "id": "selector-request-1",
            "method": "item/tool/call",
            "params": {
                "threadId": "018f0000-0000-7000-8000-000000000020",
                "turnId": "018f0000-0000-7000-8000-000000000030",
                "callId": "selector-call-1",
                "namespace": null,
                "tool": "quireforge_model_selector",
                "arguments": {
                    "action": "request",
                    "modelId": "fixture-model",
                    "reasoningEffort": "medium",
                    "rationale": "Use the bounded next-turn choice."
                }
            }
        });
        let parsed = parse_notification(&request).expect("dynamic tool call must parse");
        let Some(AppServerNotification::ConversationRequest(
            ConversationServerRequest::DynamicTool {
                request_id,
                thread_id,
                turn_id,
                call_id,
                namespace,
                tool,
                arguments,
            },
        )) = parsed
        else {
            panic!("dynamic tool call must normalize");
        };
        assert_eq!(
            request_id,
            ServerRequestId::String("selector-request-1".to_owned())
        );
        assert_eq!(
            (
                thread_id.as_str(),
                turn_id.as_str(),
                call_id.as_str(),
                namespace,
                tool.as_str()
            ),
            (
                "018f0000-0000-7000-8000-000000000020",
                "018f0000-0000-7000-8000-000000000030",
                "selector-call-1",
                None,
                "quireforge_model_selector"
            )
        );
        assert_eq!(arguments["action"], "request");

        let mut uncorrelated = request;
        uncorrelated["params"]["callId"] = json!("bad id with spaces");
        assert!(parse_notification(&uncorrelated).is_err());
        uncorrelated["params"]["callId"] = json!("selector-call-1");
        uncorrelated["params"]["arguments"] = json!("raw payload");
        assert!(parse_notification(&uncorrelated).is_err());
    }

    #[test]
    fn strictly_validates_permission_profiles_before_retaining_them() {
        let valid = json!({
            "id": 42,
            "method": "item/permissions/requestApproval",
            "params": {
                "threadId": "018f0000-0000-7000-8000-000000000020",
                "turnId": "018f0000-0000-7000-8000-000000000030",
                "itemId": "item-1",
                "startedAtMs": 1,
                "cwd": "/workspace/project",
                "permissions": {
                    "fileSystem": {
                        "entries": [{
                            "access": "read",
                            "path": {"type": "special", "value": {"kind": "project_roots"}}
                        }]
                    },
                    "network": {"enabled": false}
                }
            }
        });
        let mut invalid = valid.clone();
        invalid["params"]["permissions"]["credential"] = json!("private");

        assert!(matches!(
            parse_notification(&valid),
            Ok(Some(AppServerNotification::ConversationRequest(
                ConversationServerRequest::Permissions { .. }
            )))
        ));
        assert!(matches!(
            parse_notification(&invalid),
            Err(CodexAdapterError::InvalidProtocolMessage)
        ));
    }

    #[test]
    fn discards_raw_tool_arguments_and_file_diffs_from_activity_events() {
        let tool = json!({
            "method": "item/started",
            "params": {
                "threadId": "018f0000-0000-7000-8000-000000000020",
                "turnId": "018f0000-0000-7000-8000-000000000030",
                "item": {
                    "id": "item-1",
                    "type": "mcpToolCall",
                    "server": "github",
                    "tool": "get_issue",
                    "status": "inProgress",
                    "arguments": {"token": "private argument"}
                }
            }
        });
        let file = json!({
            "method": "item/completed",
            "params": {
                "threadId": "018f0000-0000-7000-8000-000000000020",
                "turnId": "018f0000-0000-7000-8000-000000000030",
                "item": {
                    "id": "item-2",
                    "type": "fileChange",
                    "status": "completed",
                    "changes": [{
                        "path": "/workspace/project/src/lib.rs",
                        "kind": {"type": "update", "movePath": null},
                        "diff": "private diff body"
                    }]
                }
            }
        });

        let tool = parse_notification(&tool).expect("tool must parse");
        let file = parse_notification(&file).expect("file must parse");
        assert!(!format!("{tool:?}").contains("private argument"));
        assert!(!format!("{file:?}").contains("private diff body"));
    }

    #[tokio::test]
    async fn correlates_responses_and_discards_notification_payloads() {
        let script = r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{"userAgent":"private value is discarded"}}'
read -r _models
printf '%s\n' '{"method":"remoteControl/status/changed","params":{"installationId":"private value is discarded"}}'
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
"#;
        let command = AppServerCommand::test("sh", &["-c", script]);
        let mut process = AppServerProcess::spawn_with_timeout(command, Duration::from_secs(1))
            .expect("fixture process must start");

        let (models, events) = process
            .discover_models()
            .await
            .expect("fixture protocol must succeed");
        process.shutdown().await.expect("fixture must stop");

        assert_eq!(models[0].id, "fixture-model");
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn queues_a_server_request_even_when_its_id_matches_a_client_request() {
        let script = r#"
read -r _initialize
printf '%s\n' '{"id":1,"method":"item/commandExecution/requestApproval","params":{"threadId":"018f0000-0000-7000-8000-000000000020","turnId":"018f0000-0000-7000-8000-000000000030","itemId":"item-1","startedAtMs":1,"command":"cargo check","cwd":"/workspace/project"}}'
printf '%s\n' '{"id":1,"result":{}}'
read -r _models
printf '%s\n' '{"id":2,"result":{"data":[{"model":"fixture-model","displayName":"Fixture model","isDefault":true,"defaultReasoningEffort":"medium","supportedReasoningEfforts":[{"reasoningEffort":"medium"}]}]}}'
"#;
        let command = AppServerCommand::test("sh", &["-c", script]);
        let mut process = AppServerProcess::spawn_with_timeout(command, Duration::from_secs(1))
            .expect("fixture process must start");

        process
            .discover_models()
            .await
            .expect("client responses must remain correlated");
        assert!(matches!(
            process.take_notification(),
            Some(AppServerNotification::ConversationRequest(
                ConversationServerRequest::CommandExecution {
                    request_id: ServerRequestId::Integer(1),
                    ..
                }
            ))
        ));
        process.shutdown().await.expect("fixture must stop");
    }

    #[tokio::test]
    async fn reports_an_unexpected_process_exit_without_raw_output() {
        let command = AppServerCommand::test("sh", &["-c", "exit 0"]);
        let mut process = AppServerProcess::spawn_with_timeout(command, Duration::from_millis(250))
            .expect("fixture process must start");

        assert!(matches!(
            process.discover_models().await,
            Err(CodexAdapterError::ProcessExited | CodexAdapterError::TransportClosed)
        ));
        process
            .shutdown()
            .await
            .expect("exited process can be reaped");
    }

    #[tokio::test]
    async fn fails_closed_on_an_unexpected_server_request() {
        let script = r#"
read -r _initialize
printf '%s\n' '{"id":99,"method":"item/tool/requestUserInput","params":{"private":"discarded"}}'
"#;
        let command = AppServerCommand::test("sh", &["-c", script]);
        let mut process = AppServerProcess::spawn_with_timeout(command, Duration::from_secs(1))
            .expect("fixture process must start");

        assert!(matches!(
            process.discover_models().await,
            Err(CodexAdapterError::UnexpectedServerRequest)
        ));
        process.shutdown().await.expect("fixture must stop");
    }

    #[tokio::test]
    async fn times_out_and_reaps_an_unresponsive_process() {
        let command =
            AppServerCommand::test("sh", &["-c", "read -r _request; read -r _never_respond"]);
        let mut process = AppServerProcess::spawn_with_timeout(command, Duration::from_millis(25))
            .expect("fixture process must start");

        assert!(matches!(
            process.discover_models().await,
            Err(CodexAdapterError::TransportTimeout)
        ));
        process
            .shutdown()
            .await
            .expect("timed-out process must be reaped");
    }
}
