use serde::{Deserialize, Serialize};

pub const ADAPTER_VERSION: &str = "codex-app-server-v2";
pub const RUNTIME_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexRuntimeSnapshot {
    pub schema_version: u16,
    pub adapter_version: String,
    pub availability: RuntimeAvailability,
    pub backend: BackendKind,
    pub cli_version: Option<String>,
    pub capabilities: Vec<CodexCapability>,
    pub models: Vec<CodexModel>,
    pub diagnostic_code: Option<DiagnosticCode>,
}

impl CodexRuntimeSnapshot {
    pub fn unavailable(code: DiagnosticCode) -> Self {
        Self {
            schema_version: RUNTIME_SCHEMA_VERSION,
            adapter_version: ADAPTER_VERSION.to_owned(),
            availability: RuntimeAvailability::Unavailable,
            backend: BackendKind::Unavailable,
            cli_version: None,
            capabilities: base_capabilities(CapabilityState::Unavailable),
            models: Vec::new(),
            diagnostic_code: Some(code),
        }
    }

    pub fn degraded(cli_version: String, code: DiagnosticCode) -> Self {
        Self {
            schema_version: RUNTIME_SCHEMA_VERSION,
            adapter_version: ADAPTER_VERSION.to_owned(),
            availability: RuntimeAvailability::Degraded,
            backend: BackendKind::CliFallback,
            cli_version: Some(cli_version),
            capabilities: vec![
                CodexCapability::ready("runtime-probe", CapabilityRoute::Cli),
                CodexCapability::unavailable("app-server-stdio", CapabilityRoute::AppServer),
                CodexCapability::unavailable("model-discovery", CapabilityRoute::AppServer),
                CodexCapability::unavailable("conversation-runtime", CapabilityRoute::AppServer),
                CodexCapability::ready("normalized-events", CapabilityRoute::Native),
            ],
            models: Vec::new(),
            diagnostic_code: Some(code),
        }
    }

    pub fn ready(cli_version: String, models: Vec<CodexModel>) -> Self {
        Self {
            schema_version: RUNTIME_SCHEMA_VERSION,
            adapter_version: ADAPTER_VERSION.to_owned(),
            availability: RuntimeAvailability::Ready,
            backend: BackendKind::AppServerStdio,
            cli_version: Some(cli_version),
            capabilities: base_capabilities(CapabilityState::Ready),
            models,
            diagnostic_code: None,
        }
    }
}

fn base_capabilities(state: CapabilityState) -> Vec<CodexCapability> {
    vec![
        CodexCapability {
            id: "runtime-probe".to_owned(),
            state,
            route: CapabilityRoute::Cli,
        },
        CodexCapability {
            id: "app-server-stdio".to_owned(),
            state,
            route: CapabilityRoute::AppServer,
        },
        CodexCapability {
            id: "model-discovery".to_owned(),
            state,
            route: CapabilityRoute::AppServer,
        },
        CodexCapability {
            id: "normalized-events".to_owned(),
            state: CapabilityState::Ready,
            route: CapabilityRoute::Native,
        },
        CodexCapability {
            id: "conversation-runtime".to_owned(),
            state,
            route: CapabilityRoute::AppServer,
        },
    ]
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeAvailability {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendKind {
    AppServerStdio,
    CliFallback,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexCapability {
    pub id: String,
    pub state: CapabilityState,
    pub route: CapabilityRoute,
}

impl CodexCapability {
    fn ready(id: &str, route: CapabilityRoute) -> Self {
        Self {
            id: id.to_owned(),
            state: CapabilityState::Ready,
            route,
        }
    }

    fn unavailable(id: &str, route: CapabilityRoute) -> Self {
        Self {
            id: id.to_owned(),
            state: CapabilityState::Unavailable,
            route,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityState {
    Ready,
    Unavailable,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityRoute {
    AppServer,
    Cli,
    Native,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModel {
    pub id: String,
    pub display_name: String,
    pub is_default: bool,
    pub default_reasoning_effort: String,
    pub supported_reasoning_efforts: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticCode {
    CliNotFound,
    CliVersionInvalid,
    ProcessSpawnFailed,
    ProcessExited,
    TransportTimeout,
    TransportClosed,
    MessageTooLarge,
    InvalidProtocolMessage,
    RpcRejected,
    UnexpectedServerRequest,
    InvalidModelCatalog,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum NormalizedCodexEvent {
    ProtocolReady,
    ModelCatalog { model_count: usize },
    ProcessExited { success: bool },
    TransportFailed { code: DiagnosticCode },
}
