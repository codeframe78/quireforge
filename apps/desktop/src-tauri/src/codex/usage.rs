use std::{collections::HashSet, time::Duration};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::Mutex;

use super::{
    app_server::{AppServerCommand, AppServerProcess},
    error::CodexAdapterError,
};

const USAGE_SCHEMA_VERSION: u16 = 1;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_METERS: usize = 8;
const MAX_IDENTIFIER_BYTES: usize = 64;
const MAX_LABEL_BYTES: usize = 80;
const MAX_WINDOW_MINUTES: u64 = 525_600;
const MAX_RESET_TIMESTAMP: i64 = 32_503_680_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodexUsageState {
    Ready,
    NotMetered,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodexUsageDiagnosticCode {
    RuntimeUnavailable,
    ProtocolInvalid,
    RpcRejected,
    Timeout,
    NoUsageWindows,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CodexUsageWindowKind {
    Primary,
    Secondary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUsageWindow {
    pub kind: CodexUsageWindowKind,
    pub used_percent: u8,
    pub remaining_percent: u8,
    pub window_duration_minutes: Option<u64>,
    pub resets_at: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUsageMeter {
    pub label: String,
    pub limit_id: String,
    pub windows: Vec<CodexUsageWindow>,
    pub limited: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexUsageSnapshot {
    pub schema_version: u16,
    pub state: CodexUsageState,
    pub meters: Vec<CodexUsageMeter>,
    pub diagnostic_code: Option<CodexUsageDiagnosticCode>,
}

impl CodexUsageSnapshot {
    fn ready(meters: Vec<CodexUsageMeter>) -> Self {
        Self {
            schema_version: USAGE_SCHEMA_VERSION,
            state: CodexUsageState::Ready,
            meters,
            diagnostic_code: None,
        }
    }

    fn not_metered() -> Self {
        Self {
            schema_version: USAGE_SCHEMA_VERSION,
            state: CodexUsageState::NotMetered,
            meters: Vec::new(),
            diagnostic_code: Some(CodexUsageDiagnosticCode::NoUsageWindows),
        }
    }

    fn unavailable(code: CodexUsageDiagnosticCode) -> Self {
        Self {
            schema_version: USAGE_SCHEMA_VERSION,
            state: CodexUsageState::Unavailable,
            meters: Vec::new(),
            diagnostic_code: Some(code),
        }
    }
}

pub struct CodexUsageService {
    command: AppServerCommand,
    request_timeout: Duration,
    cached: Mutex<Option<CodexUsageSnapshot>>,
}

impl Default for CodexUsageService {
    fn default() -> Self {
        Self {
            command: AppServerCommand::codex("codex"),
            request_timeout: REQUEST_TIMEOUT,
            cached: Mutex::new(None),
        }
    }
}

impl CodexUsageService {
    pub async fn snapshot(&self) -> CodexUsageSnapshot {
        let mut cached = self.cached.lock().await;
        if let Some(snapshot) = cached.as_ref() {
            return snapshot.clone();
        }
        let snapshot = self.read().await;
        *cached = Some(snapshot.clone());
        snapshot
    }

    pub async fn refresh(&self) -> CodexUsageSnapshot {
        let snapshot = self.read().await;
        *self.cached.lock().await = Some(snapshot.clone());
        snapshot
    }

    async fn read(&self) -> CodexUsageSnapshot {
        let mut process = match AppServerProcess::spawn_with_timeout(
            self.command.clone(),
            self.request_timeout,
        ) {
            Ok(process) => process,
            Err(error) => return CodexUsageSnapshot::unavailable(usage_diagnostic(&error)),
        };
        if let Err(error) = process.initialize().await {
            let code = usage_diagnostic(&error);
            let _ = process.shutdown().await;
            return CodexUsageSnapshot::unavailable(code);
        }

        let result = process
            .request("account/rateLimits/read", json!({}))
            .await
            .map_err(|error| usage_diagnostic(&error))
            .and_then(parse_usage_response);
        let shutdown = process.shutdown().await;

        match (result, shutdown) {
            (Ok(snapshot), Ok(())) => snapshot,
            (Err(code), _) => CodexUsageSnapshot::unavailable(code),
            (_, Err(error)) => CodexUsageSnapshot::unavailable(usage_diagnostic(&error)),
        }
    }

    #[cfg(test)]
    fn fixture(script: &str) -> Self {
        Self {
            command: AppServerCommand::test("sh", &["-c", script]),
            request_timeout: Duration::from_secs(1),
            cached: Mutex::new(None),
        }
    }
}

fn parse_usage_response(value: Value) -> Result<CodexUsageSnapshot, CodexUsageDiagnosticCode> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct UsageResponse {
        rate_limits: RateLimitSnapshot,
        #[serde(default)]
        rate_limits_by_limit_id: Option<serde_json::Map<String, Value>>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RateLimitSnapshot {
        #[serde(default)]
        limit_id: Option<String>,
        #[serde(default)]
        limit_name: Option<String>,
        #[serde(default)]
        primary: Option<RateLimitWindow>,
        #[serde(default)]
        secondary: Option<RateLimitWindow>,
        #[serde(default)]
        rate_limit_reached_type: Option<RateLimitReachedType>,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum RateLimitReachedType {
        RateLimitReached,
        WorkspaceOwnerCreditsDepleted,
        WorkspaceMemberCreditsDepleted,
        WorkspaceOwnerUsageLimitReached,
        WorkspaceMemberUsageLimitReached,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RateLimitWindow {
        used_percent: i64,
        #[serde(default)]
        window_duration_mins: Option<i64>,
        #[serde(default)]
        resets_at: Option<i64>,
    }

    fn normalize_window(
        kind: CodexUsageWindowKind,
        window: RateLimitWindow,
    ) -> Result<CodexUsageWindow, CodexUsageDiagnosticCode> {
        let used_percent = u8::try_from(window.used_percent)
            .ok()
            .filter(|percent| *percent <= 100)
            .ok_or(CodexUsageDiagnosticCode::ProtocolInvalid)?;
        let window_duration_minutes = match window.window_duration_mins {
            Some(minutes) => Some(
                u64::try_from(minutes)
                    .ok()
                    .filter(|minutes| *minutes > 0 && *minutes <= MAX_WINDOW_MINUTES)
                    .ok_or(CodexUsageDiagnosticCode::ProtocolInvalid)?,
            ),
            None => None,
        };
        let resets_at = match window.resets_at {
            Some(timestamp) if (0..=MAX_RESET_TIMESTAMP).contains(&timestamp) => Some(timestamp),
            Some(_) => return Err(CodexUsageDiagnosticCode::ProtocolInvalid),
            None => None,
        };
        Ok(CodexUsageWindow {
            kind,
            used_percent,
            remaining_percent: 100 - used_percent,
            window_duration_minutes,
            resets_at,
        })
    }

    fn normalize_meter(
        fallback_id: &str,
        snapshot: RateLimitSnapshot,
    ) -> Result<Option<CodexUsageMeter>, CodexUsageDiagnosticCode> {
        let limit_id = snapshot.limit_id.as_deref().unwrap_or(fallback_id);
        validate_identifier(limit_id)?;
        let label = snapshot
            .limit_name
            .as_deref()
            .map(normalize_label)
            .transpose()?
            .unwrap_or_else(|| fallback_usage_label(limit_id));
        let mut windows = Vec::with_capacity(2);
        if let Some(primary) = snapshot.primary {
            windows.push(normalize_window(CodexUsageWindowKind::Primary, primary)?);
        }
        if let Some(secondary) = snapshot.secondary {
            windows.push(normalize_window(
                CodexUsageWindowKind::Secondary,
                secondary,
            )?);
        }
        if windows.is_empty() {
            return Ok(None);
        }
        Ok(Some(CodexUsageMeter {
            label,
            limit_id: limit_id.to_owned(),
            windows,
            limited: snapshot.rate_limit_reached_type.is_some(),
        }))
    }

    let response: UsageResponse =
        serde_json::from_value(value).map_err(|_| CodexUsageDiagnosticCode::ProtocolInvalid)?;
    let mut meters = Vec::new();
    let mut identifiers = HashSet::new();

    if let Some(by_id) = response.rate_limits_by_limit_id {
        if by_id.len() > MAX_METERS {
            return Err(CodexUsageDiagnosticCode::ProtocolInvalid);
        }
        let mut entries = by_id.into_iter().collect::<Vec<_>>();
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        for (limit_id, value) in entries {
            validate_identifier(&limit_id)?;
            let snapshot: RateLimitSnapshot = serde_json::from_value(value)
                .map_err(|_| CodexUsageDiagnosticCode::ProtocolInvalid)?;
            if let Some(meter) = normalize_meter(&limit_id, snapshot)? {
                if !identifiers.insert(meter.limit_id.clone()) {
                    return Err(CodexUsageDiagnosticCode::ProtocolInvalid);
                }
                meters.push(meter);
            }
        }
    } else if let Some(meter) = normalize_meter("codex", response.rate_limits)? {
        meters.push(meter);
    }

    if meters.is_empty() {
        Ok(CodexUsageSnapshot::not_metered())
    } else {
        Ok(CodexUsageSnapshot::ready(meters))
    }
}

fn validate_identifier(value: &str) -> Result<(), CodexUsageDiagnosticCode> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_BYTES
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(CodexUsageDiagnosticCode::ProtocolInvalid);
    }
    Ok(())
}

fn normalize_label(value: &str) -> Result<String, CodexUsageDiagnosticCode> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > MAX_LABEL_BYTES
        || value.chars().any(|character| {
            let code = character as u32;
            character.is_control()
                || matches!(
                    code,
                    0x200b..=0x200f | 0x202a..=0x202e | 0x2060..=0x206f | 0xfeff
                )
        })
    {
        return Err(CodexUsageDiagnosticCode::ProtocolInvalid);
    }
    Ok(value.to_owned())
}

fn fallback_usage_label(limit_id: &str) -> String {
    match limit_id {
        "codex" => "Codex".to_owned(),
        "codex_other" => "Other Codex activity".to_owned(),
        _ => "Codex usage".to_owned(),
    }
}

fn usage_diagnostic(error: &CodexAdapterError) -> CodexUsageDiagnosticCode {
    match error {
        CodexAdapterError::TransportTimeout => CodexUsageDiagnosticCode::Timeout,
        CodexAdapterError::RpcRejected => CodexUsageDiagnosticCode::RpcRejected,
        CodexAdapterError::InvalidProtocolMessage
        | CodexAdapterError::MessageTooLarge
        | CodexAdapterError::UnexpectedServerRequest => CodexUsageDiagnosticCode::ProtocolInvalid,
        _ => CodexUsageDiagnosticCode::RuntimeUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn normalizes_remaining_usage_without_retaining_account_metadata() {
        let service = CodexUsageService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _usage
printf '%s\n' '{"id":2,"result":{"rateLimits":{"limitId":"codex","limitName":null,"primary":{"usedPercent":25,"windowDurationMins":300,"resetsAt":1784808000},"secondary":{"usedPercent":58,"windowDurationMins":10080,"resetsAt":1785412800},"planType":"pro","credits":{"balance":"private-balance","hasCredits":true,"unlimited":false},"rateLimitReachedType":null},"rateLimitsByLimitId":null,"rateLimitResetCredits":{"availableCount":2,"credits":[{"id":"private-credit-id"}]}}}'
"#,
        );

        let snapshot = service.snapshot().await;
        assert_eq!(snapshot.state, CodexUsageState::Ready);
        assert_eq!(snapshot.meters[0].windows[0].remaining_percent, 75);
        assert_eq!(snapshot.meters[0].windows[1].remaining_percent, 42);
        let serialized = serde_json::to_string(&snapshot).expect("usage snapshot must serialize");
        for forbidden in [
            "private-balance",
            "private-credit-id",
            "planType",
            "credits",
        ] {
            assert!(!serialized.contains(forbidden));
        }
    }

    #[tokio::test]
    async fn sorts_and_bounds_multi_meter_usage() {
        let service = CodexUsageService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _usage
printf '%s\n' '{"id":2,"result":{"rateLimits":{"limitId":"codex","primary":{"usedPercent":1}},"rateLimitsByLimitId":{"codex_other":{"limitId":"codex_other","limitName":"Reviews","primary":{"usedPercent":40,"windowDurationMins":60,"resetsAt":1784808000}},"codex":{"limitId":"codex","limitName":null,"primary":{"usedPercent":20,"windowDurationMins":300,"resetsAt":1784808000}}}}}'
"#,
        );

        let snapshot = service.refresh().await;
        assert_eq!(
            snapshot
                .meters
                .iter()
                .map(|meter| meter.limit_id.as_str())
                .collect::<Vec<_>>(),
            vec!["codex", "codex_other"]
        );
        assert_eq!(snapshot.meters[1].label, "Reviews");
    }

    #[tokio::test]
    async fn rejects_invalid_percentages_timestamps_and_labels() {
        for response in [
            r#"{"rateLimits":{"limitId":"codex","primary":{"usedPercent":101}}}"#,
            r#"{"rateLimits":{"limitId":"codex","primary":{"usedPercent":10,"resetsAt":-1}}}"#,
            r#"{"rateLimits":{"limitId":"codex","limitName":"safe\u202eevil","primary":{"usedPercent":10}}}"#,
            r#"{"rateLimits":{"limitId":"codex","primary":{"usedPercent":10},"rateLimitReachedType":"future_unreviewed_value"}}"#,
        ] {
            let script = format!(
                "read -r _initialize\nprintf '%s\\n' '{{\"id\":1,\"result\":{{}}}}'\nread -r _usage\nprintf '%s\\n' '{{\"id\":2,\"result\":{response}}}'\n"
            );
            let snapshot = CodexUsageService::fixture(&script).refresh().await;
            assert_eq!(snapshot.state, CodexUsageState::Unavailable);
            assert_eq!(
                snapshot.diagnostic_code,
                Some(CodexUsageDiagnosticCode::ProtocolInvalid)
            );
        }
    }

    #[tokio::test]
    async fn reports_an_honest_not_metered_state() {
        let service = CodexUsageService::fixture(
            r#"
read -r _initialize
printf '%s\n' '{"id":1,"result":{}}'
read -r _usage
printf '%s\n' '{"id":2,"result":{"rateLimits":{"limitId":"codex","primary":null,"secondary":null}}}'
"#,
        );
        assert_eq!(service.refresh().await, CodexUsageSnapshot::not_metered());
    }
}
