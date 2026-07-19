use serde::{Deserialize, Serialize};

pub const AUTH_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthLoginMethod {
    Browser,
    DeviceCode,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthState {
    Authenticated,
    Unauthenticated,
    LoginPending,
    NotRequired,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthAccountKind {
    Chatgpt,
    ApiKey,
    ManagedProvider,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthDiagnosticCode {
    RuntimeUnavailable,
    ProtocolInvalid,
    RpcRejected,
    Timeout,
    LoginFailed,
    CancelNotFound,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthHandoff {
    pub verification_url: String,
    pub user_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexAuthSnapshot {
    pub schema_version: u16,
    pub state: AuthState,
    pub account_kind: Option<AuthAccountKind>,
    pub pending_method: Option<AuthLoginMethod>,
    pub handoff: Option<AuthHandoff>,
    pub diagnostic_code: Option<AuthDiagnosticCode>,
}

impl CodexAuthSnapshot {
    pub fn authenticated(kind: AuthAccountKind) -> Self {
        Self::new(AuthState::Authenticated, Some(kind), None, None, None)
    }

    pub fn unauthenticated() -> Self {
        Self::new(AuthState::Unauthenticated, None, None, None, None)
    }

    pub fn not_required() -> Self {
        Self::new(AuthState::NotRequired, None, None, None, None)
    }

    pub fn pending(method: AuthLoginMethod, handoff: AuthHandoff) -> Self {
        Self::new(
            AuthState::LoginPending,
            None,
            Some(method),
            Some(handoff),
            None,
        )
    }

    pub fn unavailable(code: AuthDiagnosticCode) -> Self {
        Self::new(AuthState::Unavailable, None, None, None, Some(code))
    }

    pub fn unauthenticated_with(code: AuthDiagnosticCode) -> Self {
        Self::new(AuthState::Unauthenticated, None, None, None, Some(code))
    }

    fn new(
        state: AuthState,
        account_kind: Option<AuthAccountKind>,
        pending_method: Option<AuthLoginMethod>,
        handoff: Option<AuthHandoff>,
        diagnostic_code: Option<AuthDiagnosticCode>,
    ) -> Self {
        Self {
            schema_version: AUTH_SCHEMA_VERSION,
            state,
            account_kind,
            pending_method,
            handoff,
            diagnostic_code,
        }
    }
}
