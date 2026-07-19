use std::time::Duration;

use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{
    sync::{mpsc, oneshot, watch, Mutex},
    task::JoinHandle,
    time::timeout,
};
use url::Url;

use super::{
    app_server::{
        validate_protocol_identifier, AppServerCommand, AppServerNotification, AppServerProcess,
    },
    error::CodexAdapterError,
};

pub mod types;

use types::{
    AuthAccountKind, AuthDiagnosticCode, AuthHandoff, AuthLoginMethod, AuthState, CodexAuthSnapshot,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const CONTROL_TIMEOUT: Duration = Duration::from_secs(7);
const MAX_URL_BYTES: usize = 2048;
const MAX_USER_CODE_BYTES: usize = 32;

pub struct CodexAuthService {
    command: AppServerCommand,
    request_timeout: Duration,
    state: Mutex<ServiceState>,
}

#[derive(Default)]
struct ServiceState {
    cached: Option<CodexAuthSnapshot>,
    login: Option<LoginTask>,
}

struct LoginTask {
    state: watch::Receiver<CodexAuthSnapshot>,
    control: mpsc::Sender<LoginControl>,
    join: Option<JoinHandle<()>>,
}

enum LoginControl {
    Cancel(oneshot::Sender<CodexAuthSnapshot>),
}

impl Default for CodexAuthService {
    fn default() -> Self {
        Self {
            command: AppServerCommand::codex("codex"),
            request_timeout: REQUEST_TIMEOUT,
            state: Mutex::new(ServiceState::default()),
        }
    }
}

impl CodexAuthService {
    pub async fn status(&self) -> CodexAuthSnapshot {
        let mut state = self.state.lock().await;
        if let Some(snapshot) = settle_login_if_terminal(&mut state).await {
            return snapshot;
        }
        if let Some(snapshot) = state.cached.as_ref() {
            return snapshot.clone();
        }

        let snapshot = self.read_account().await;
        state.cached = Some(snapshot.clone());
        snapshot
    }

    pub async fn refresh(&self) -> CodexAuthSnapshot {
        let mut state = self.state.lock().await;
        if let Some(snapshot) = settle_login_if_terminal(&mut state).await {
            return snapshot;
        }

        let snapshot = self.read_account().await;
        state.cached = Some(snapshot.clone());
        snapshot
    }

    pub async fn start_login(&self, method: AuthLoginMethod) -> CodexAuthSnapshot {
        let mut state = self.state.lock().await;
        if let Some(login) = state.login.as_mut() {
            let snapshot = login.state.borrow().clone();
            if snapshot.state == AuthState::LoginPending {
                return snapshot;
            }
            if let Some(join) = login.join.take() {
                let _ = join.await;
            }
            state.login = None;
        }

        match self.begin_login(method).await {
            Ok((process, login_id, snapshot)) => {
                let (state_tx, state_rx) = watch::channel(snapshot.clone());
                let (control_tx, control_rx) = mpsc::channel(1);
                let join = tokio::spawn(run_login_owner(process, login_id, state_tx, control_rx));
                state.cached = None;
                state.login = Some(LoginTask {
                    state: state_rx,
                    control: control_tx,
                    join: Some(join),
                });
                snapshot
            }
            Err(code) => {
                let snapshot = CodexAuthSnapshot::unavailable(code);
                state.cached = Some(snapshot.clone());
                snapshot
            }
        }
    }

    pub async fn cancel_login(&self) -> CodexAuthSnapshot {
        let mut state = self.state.lock().await;
        let Some(login) = state.login.as_mut() else {
            return state
                .cached
                .clone()
                .unwrap_or_else(CodexAuthSnapshot::unauthenticated);
        };

        let (result_tx, result_rx) = oneshot::channel();
        if login
            .control
            .send(LoginControl::Cancel(result_tx))
            .await
            .is_err()
        {
            return CodexAuthSnapshot::unavailable(AuthDiagnosticCode::RuntimeUnavailable);
        }

        let snapshot = timeout(CONTROL_TIMEOUT, result_rx)
            .await
            .ok()
            .and_then(Result::ok)
            .unwrap_or_else(|| CodexAuthSnapshot::unavailable(AuthDiagnosticCode::Timeout));
        if let Some(join) = login.join.take() {
            let _ = join.await;
        }
        state.login = None;
        state.cached = Some(snapshot.clone());
        snapshot
    }

    pub async fn logout(&self) -> CodexAuthSnapshot {
        if self.status().await.state == AuthState::LoginPending {
            let _ = self.cancel_login().await;
        }

        let snapshot = self.logout_account().await;
        let mut state = self.state.lock().await;
        state.login = None;
        state.cached = Some(snapshot.clone());
        snapshot
    }

    pub async fn handoff_url(&self) -> Option<String> {
        let state = self.state.lock().await;
        state
            .login
            .as_ref()
            .and_then(|login| login.state.borrow().handoff.clone())
            .map(|handoff| handoff.verification_url)
    }

    async fn begin_login(
        &self,
        method: AuthLoginMethod,
    ) -> Result<(AppServerProcess, String, CodexAuthSnapshot), AuthDiagnosticCode> {
        let mut process = self.spawn_initialized().await?;
        let params = match method {
            AuthLoginMethod::Browser => json!({"type": "chatgpt"}),
            AuthLoginMethod::DeviceCode => json!({"type": "chatgptDeviceCode"}),
        };
        let response = match process.request("account/login/start", params).await {
            Ok(response) => response,
            Err(error) => {
                let code = auth_diagnostic(&error);
                let _ = process.shutdown().await;
                return Err(code);
            }
        };
        let (login_id, handoff) = match parse_login_response(response, method) {
            Ok(parsed) => parsed,
            Err(code) => {
                let _ = process.shutdown().await;
                return Err(code);
            }
        };
        let snapshot = CodexAuthSnapshot::pending(method, handoff);
        Ok((process, login_id, snapshot))
    }

    async fn read_account(&self) -> CodexAuthSnapshot {
        let mut process = match self.spawn_initialized().await {
            Ok(process) => process,
            Err(code) => return CodexAuthSnapshot::unavailable(code),
        };
        let result = process
            .request("account/read", json!({"refreshToken": false}))
            .await
            .map_err(|error| auth_diagnostic(&error))
            .and_then(parse_account_response);
        let shutdown = process.shutdown().await;

        match (result, shutdown) {
            (Ok(snapshot), Ok(())) => snapshot,
            (Err(code), _) => CodexAuthSnapshot::unavailable(code),
            (_, Err(error)) => CodexAuthSnapshot::unavailable(auth_diagnostic(&error)),
        }
    }

    async fn logout_account(&self) -> CodexAuthSnapshot {
        let mut process = match self.spawn_initialized().await {
            Ok(process) => process,
            Err(code) => return CodexAuthSnapshot::unavailable(code),
        };
        let result = process
            .request("account/logout", json!({}))
            .await
            .map_err(|error| auth_diagnostic(&error))
            .and_then(|value| {
                value
                    .as_object()
                    .map(|_| CodexAuthSnapshot::unauthenticated())
                    .ok_or(AuthDiagnosticCode::ProtocolInvalid)
            });
        let shutdown = process.shutdown().await;

        match (result, shutdown) {
            (Ok(snapshot), Ok(())) => snapshot,
            (Err(code), _) => CodexAuthSnapshot::unavailable(code),
            (_, Err(error)) => CodexAuthSnapshot::unavailable(auth_diagnostic(&error)),
        }
    }

    async fn spawn_initialized(&self) -> Result<AppServerProcess, AuthDiagnosticCode> {
        let mut process =
            AppServerProcess::spawn_with_timeout(self.command.clone(), self.request_timeout)
                .map_err(|error| auth_diagnostic(&error))?;
        if let Err(error) = process.initialize().await {
            let code = auth_diagnostic(&error);
            let _ = process.shutdown().await;
            return Err(code);
        }
        Ok(process)
    }

    #[cfg(test)]
    fn fixture(script: &str) -> Self {
        Self {
            command: AppServerCommand::test("sh", &["-c", script]),
            request_timeout: Duration::from_secs(1),
            state: Mutex::new(ServiceState::default()),
        }
    }
}

async fn settle_login_if_terminal(state: &mut ServiceState) -> Option<CodexAuthSnapshot> {
    let snapshot = state.login.as_ref()?.state.borrow().clone();
    if snapshot.state == AuthState::LoginPending {
        return Some(snapshot);
    }

    let mut login = state.login.take().expect("login state was present");
    if let Some(join) = login.join.take() {
        let _ = join.await;
    }
    state.cached = Some(snapshot.clone());
    Some(snapshot)
}

async fn run_login_owner(
    mut process: AppServerProcess,
    login_id: String,
    state: watch::Sender<CodexAuthSnapshot>,
    mut control: mpsc::Receiver<LoginControl>,
) {
    loop {
        tokio::select! {
            command = control.recv() => {
                let Some(LoginControl::Cancel(result)) = command else {
                    let _ = process.shutdown().await;
                    return;
                };
                let snapshot = cancel_active_login(&mut process, &login_id).await;
                let _ = process.shutdown().await;
                state.send_replace(snapshot.clone());
                let _ = result.send(snapshot);
                return;
            }
            notification = process.next_notification() => {
                match notification {
                    Ok(AppServerNotification::AccountUpdated) => continue,
                    Ok(AppServerNotification::AccountLoginCompleted { login_id: completed_id, success }) => {
                        let snapshot = complete_login(&mut process, &login_id, completed_id, success).await;
                        let _ = process.shutdown().await;
                        state.send_replace(snapshot);
                        return;
                    }
                    Err(error) => {
                        let _ = process.shutdown().await;
                        state.send_replace(CodexAuthSnapshot::unavailable(auth_diagnostic(&error)));
                        return;
                    }
                }
            }
        }
    }
}

async fn cancel_active_login(process: &mut AppServerProcess, login_id: &str) -> CodexAuthSnapshot {
    let response = process
        .request("account/login/cancel", json!({"loginId": login_id}))
        .await;

    while let Some(notification) = process.take_notification() {
        if let AppServerNotification::AccountLoginCompleted {
            login_id: completed_id,
            success,
        } = notification
        {
            return complete_login(process, login_id, completed_id, success).await;
        }
    }

    match response {
        Ok(value) => {
            #[derive(Deserialize)]
            struct CancelResponse {
                status: String,
            }
            match serde_json::from_value::<CancelResponse>(value)
                .map(|response| response.status)
                .as_deref()
            {
                Ok("canceled") => CodexAuthSnapshot::unauthenticated(),
                Ok("notFound") => {
                    CodexAuthSnapshot::unauthenticated_with(AuthDiagnosticCode::CancelNotFound)
                }
                _ => CodexAuthSnapshot::unavailable(AuthDiagnosticCode::ProtocolInvalid),
            }
        }
        Err(error) => CodexAuthSnapshot::unavailable(auth_diagnostic(&error)),
    }
}

async fn complete_login(
    process: &mut AppServerProcess,
    expected_login_id: &str,
    completed_login_id: Option<String>,
    success: bool,
) -> CodexAuthSnapshot {
    if completed_login_id.as_deref() != Some(expected_login_id) {
        return CodexAuthSnapshot::unavailable(AuthDiagnosticCode::ProtocolInvalid);
    }
    if !success {
        return CodexAuthSnapshot::unauthenticated_with(AuthDiagnosticCode::LoginFailed);
    }

    match process
        .request("account/read", json!({"refreshToken": false}))
        .await
    {
        Ok(value) => parse_account_response(value).unwrap_or_else(CodexAuthSnapshot::unavailable),
        Err(error) => CodexAuthSnapshot::unavailable(auth_diagnostic(&error)),
    }
}

fn parse_login_response(
    value: Value,
    expected_method: AuthLoginMethod,
) -> Result<(String, AuthHandoff), AuthDiagnosticCode> {
    #[derive(Deserialize)]
    #[serde(tag = "type")]
    enum LoginResponse {
        #[serde(rename = "chatgpt")]
        Browser {
            #[serde(rename = "authUrl")]
            auth_url: String,
            #[serde(rename = "loginId")]
            login_id: String,
        },
        #[serde(rename = "chatgptDeviceCode")]
        DeviceCode {
            #[serde(rename = "verificationUrl")]
            verification_url: String,
            #[serde(rename = "userCode")]
            user_code: String,
            #[serde(rename = "loginId")]
            login_id: String,
        },
    }

    let response: LoginResponse =
        serde_json::from_value(value).map_err(|_| AuthDiagnosticCode::ProtocolInvalid)?;
    let (login_id, handoff) = match (expected_method, response) {
        (AuthLoginMethod::Browser, LoginResponse::Browser { auth_url, login_id }) => (
            login_id,
            AuthHandoff {
                verification_url: validate_auth_url(&auth_url)?,
                user_code: None,
            },
        ),
        (
            AuthLoginMethod::DeviceCode,
            LoginResponse::DeviceCode {
                verification_url,
                user_code,
                login_id,
            },
        ) => {
            validate_user_code(&user_code)?;
            (
                login_id,
                AuthHandoff {
                    verification_url: validate_auth_url(&verification_url)?,
                    user_code: Some(user_code),
                },
            )
        }
        _ => return Err(AuthDiagnosticCode::ProtocolInvalid),
    };
    validate_protocol_identifier(&login_id, 128)
        .map_err(|_| AuthDiagnosticCode::ProtocolInvalid)?;
    Ok((login_id, handoff))
}

fn parse_account_response(value: Value) -> Result<CodexAuthSnapshot, AuthDiagnosticCode> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct AccountResponse {
        requires_openai_auth: bool,
        account: Option<AccountType>,
    }
    #[derive(Deserialize)]
    struct AccountType {
        #[serde(rename = "type")]
        kind: String,
    }

    let response: AccountResponse =
        serde_json::from_value(value).map_err(|_| AuthDiagnosticCode::ProtocolInvalid)?;
    match response.account {
        Some(account) => {
            if account.kind.len() > 32 || account.kind.chars().any(char::is_control) {
                return Err(AuthDiagnosticCode::ProtocolInvalid);
            }
            let kind = match account.kind.as_str() {
                "chatgpt" => AuthAccountKind::Chatgpt,
                "apiKey" => AuthAccountKind::ApiKey,
                "amazonBedrock" => AuthAccountKind::ManagedProvider,
                _ => AuthAccountKind::Unknown,
            };
            Ok(CodexAuthSnapshot::authenticated(kind))
        }
        None if response.requires_openai_auth => Ok(CodexAuthSnapshot::unauthenticated()),
        None => Ok(CodexAuthSnapshot::not_required()),
    }
}

fn validate_auth_url(value: &str) -> Result<String, AuthDiagnosticCode> {
    if value.is_empty() || value.len() > MAX_URL_BYTES {
        return Err(AuthDiagnosticCode::ProtocolInvalid);
    }
    let url = Url::parse(value).map_err(|_| AuthDiagnosticCode::ProtocolInvalid)?;
    let host = url
        .host_str()
        .map(str::to_ascii_lowercase)
        .ok_or(AuthDiagnosticCode::ProtocolInvalid)?;
    let allowed_host = host == "openai.com"
        || host.ends_with(".openai.com")
        || host == "chatgpt.com"
        || host.ends_with(".chatgpt.com");
    if url.scheme() != "https"
        || !allowed_host
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(AuthDiagnosticCode::ProtocolInvalid);
    }
    Ok(url.into())
}

fn validate_user_code(value: &str) -> Result<(), AuthDiagnosticCode> {
    if value.is_empty()
        || value.len() > MAX_USER_CODE_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(AuthDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

fn auth_diagnostic(error: &CodexAdapterError) -> AuthDiagnosticCode {
    match error {
        CodexAdapterError::TransportTimeout => AuthDiagnosticCode::Timeout,
        CodexAdapterError::RpcRejected => AuthDiagnosticCode::RpcRejected,
        CodexAdapterError::InvalidProtocolMessage
        | CodexAdapterError::MessageTooLarge
        | CodexAdapterError::UnexpectedServerRequest => AuthDiagnosticCode::ProtocolInvalid,
        _ => AuthDiagnosticCode::RuntimeUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reads_account_status_without_retaining_email_or_plan() {
        let service = CodexAuthService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _account
printf '%s\n' '{"id":2,"result":{"account":{"type":"chatgpt","email":"private@example.test","planType":"pro"},"requiresOpenaiAuth":true}}'
"#,
        );

        let snapshot = service.status().await;
        let serialized = serde_json::to_string(&snapshot).expect("snapshot must serialize");
        assert_eq!(snapshot.state, AuthState::Authenticated);
        assert_eq!(snapshot.account_kind, Some(AuthAccountKind::Chatgpt));
        assert!(!serialized.contains("private@example.test"));
        assert!(!serialized.contains("planType"));
    }

    #[tokio::test]
    async fn completes_the_exact_browser_login() {
        let service = CodexAuthService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _login
printf '%s\n' '{"id":2,"result":{"type":"chatgpt","loginId":"login-1","authUrl":"https://auth.openai.com/authorize?state=sanitized"}}'
printf '%s\n' '{"method":"account/login/completed","params":{"loginId":"login-1","success":true,"error":null}}'
read -r _account
printf '%s\n' '{"id":3,"result":{"account":{"type":"chatgpt","email":"private@example.test","planType":"plus"},"requiresOpenaiAuth":true}}'
"#,
        );

        let pending = service.start_login(AuthLoginMethod::Browser).await;
        assert_eq!(pending.state, AuthState::LoginPending);
        let terminal = timeout(Duration::from_secs(1), async {
            loop {
                let snapshot = service.status().await;
                if snapshot.state != AuthState::LoginPending {
                    break snapshot;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("fixture login must finish");
        assert_eq!(terminal.state, AuthState::Authenticated);
        assert_eq!(terminal.handoff, None);
    }

    #[tokio::test]
    async fn reduces_a_raw_completion_error_to_a_stable_code() {
        let service = CodexAuthService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _login
printf '%s\n' '{"id":2,"result":{"type":"chatgpt","loginId":"login-1","authUrl":"https://auth.openai.com/authorize"}}'
printf '%s\n' '{"method":"account/login/completed","params":{"loginId":"login-1","success":false,"error":"private@example.test token-secret"}}'
"#,
        );

        let _ = service.start_login(AuthLoginMethod::Browser).await;
        let terminal = timeout(Duration::from_secs(1), async {
            loop {
                let snapshot = service.status().await;
                if snapshot.state != AuthState::LoginPending {
                    break snapshot;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("fixture login must finish");
        let serialized = serde_json::to_string(&terminal).expect("snapshot must serialize");
        assert_eq!(
            terminal.diagnostic_code,
            Some(AuthDiagnosticCode::LoginFailed)
        );
        assert!(!serialized.contains("private@example.test"));
        assert!(!serialized.contains("token-secret"));
    }

    #[tokio::test]
    async fn cancels_the_owned_login_without_exposing_the_login_id() {
        let service = CodexAuthService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _login
printf '%s\n' '{"id":2,"result":{"type":"chatgptDeviceCode","loginId":"private-login-id","userCode":"SAFE-CODE","verificationUrl":"https://auth.openai.com/device"}}'
read -r _cancel
printf '%s\n' '{"id":3,"result":{"status":"canceled"}}'
"#,
        );

        let pending = service.start_login(AuthLoginMethod::DeviceCode).await;
        assert_eq!(
            pending
                .handoff
                .as_ref()
                .and_then(|value| value.user_code.as_deref()),
            Some("SAFE-CODE")
        );
        let canceled = service.cancel_login().await;
        assert_eq!(canceled, CodexAuthSnapshot::unauthenticated());
        assert!(!serde_json::to_string(&pending)
            .expect("snapshot must serialize")
            .contains("private-login-id"));
    }

    #[tokio::test]
    async fn fails_closed_for_a_stale_completion_or_untrusted_url() {
        let stale = CodexAuthService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _login
printf '%s\n' '{"id":2,"result":{"type":"chatgpt","loginId":"login-1","authUrl":"https://auth.openai.com/authorize"}}'
printf '%s\n' '{"method":"account/login/completed","params":{"loginId":"login-2","success":true,"error":null}}'
"#,
        );
        let _ = stale.start_login(AuthLoginMethod::Browser).await;
        let terminal = timeout(Duration::from_secs(1), async {
            loop {
                let snapshot = stale.status().await;
                if snapshot.state != AuthState::LoginPending {
                    break snapshot;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("fixture login must finish");
        assert_eq!(
            terminal.diagnostic_code,
            Some(AuthDiagnosticCode::ProtocolInvalid)
        );

        let untrusted = CodexAuthService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _login
printf '%s\n' '{"id":2,"result":{"type":"chatgpt","loginId":"login-1","authUrl":"https://openai.com.attacker.test/authorize"}}'
"#,
        );
        let rejected = untrusted.start_login(AuthLoginMethod::Browser).await;
        assert_eq!(
            rejected.diagnostic_code,
            Some(AuthDiagnosticCode::ProtocolInvalid)
        );
    }

    #[test]
    fn rejects_embedded_credentials_and_malformed_device_codes() {
        assert!(validate_auth_url("https://user:secret@openai.com/login").is_err());
        assert!(validate_auth_url("javascript:alert(1)").is_err());
        assert!(validate_user_code("unsafe code").is_err());
        assert!(validate_user_code("SAFE-CODE").is_ok());
    }

    #[tokio::test]
    #[ignore = "manual non-mutating probe requiring the installed Codex CLI"]
    async fn live_status_returns_only_normalized_account_state() {
        let snapshot = CodexAuthService::default().refresh().await;
        assert_ne!(snapshot.state, AuthState::Unavailable);
        let serialized = serde_json::to_string(&snapshot).expect("snapshot must serialize");
        for forbidden in ["email", "accountId", "loginId", "accessToken", "apiKey"] {
            assert!(!serialized.contains(forbidden));
        }
    }
}
