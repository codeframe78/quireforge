use std::{
    collections::{HashSet, VecDeque},
    ffi::OsString,
    process::Stdio,
    time::Duration,
};

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines},
    process::{Child, ChildStdin, ChildStdout, Command},
    time::timeout,
};

use super::{
    error::CodexAdapterError,
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

            if message.get("id").and_then(Value::as_u64) == Some(request_id) {
                if message.get("error").is_some_and(|error| !error.is_null()) {
                    return Err(CodexAdapterError::RpcRejected);
                }

                return message
                    .get("result")
                    .cloned()
                    .ok_or(CodexAdapterError::InvalidProtocolMessage);
            }

            if let Some(notification) = parse_notification(&message)? {
                self.notifications.push_back(notification);
                continue;
            }

            if message.get("method").and_then(Value::as_str).is_some() {
                // Unrelated notifications are deliberately discarded without retaining
                // account, installation, path, or remote-control metadata.
                continue;
            }

            return Err(CodexAdapterError::InvalidProtocolMessage);
        }
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
        return Err(CodexAdapterError::UnexpectedServerRequest);
    }

    match method {
        "account/login/completed" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
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
        _ => Ok(None),
    }
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
printf '%s\n' '{"id":99,"method":"item/permissions/requestApproval","params":{"private":"discarded"}}'
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
