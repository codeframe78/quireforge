use thiserror::Error;

use super::types::DiagnosticCode;

#[derive(Debug, Error)]
pub enum CodexAdapterError {
    #[error("Codex CLI is unavailable")]
    CliNotFound,
    #[error("Codex CLI returned an invalid version")]
    CliVersionInvalid,
    #[error("Codex app-server could not be started")]
    ProcessSpawnFailed,
    #[error("Codex app-server exited before completing the request")]
    ProcessExited,
    #[error("Codex app-server did not respond before the timeout")]
    TransportTimeout,
    #[error("Codex app-server closed its transport")]
    TransportClosed,
    #[error("Codex app-server returned a message above the safety limit")]
    MessageTooLarge,
    #[error("Codex app-server returned an invalid protocol message")]
    InvalidProtocolMessage,
    #[error("Codex app-server rejected a request")]
    RpcRejected,
    #[error("Codex app-server sent an unexpected server request")]
    UnexpectedServerRequest,
    #[error("Codex app-server returned an invalid model catalog")]
    InvalidModelCatalog,
}

impl CodexAdapterError {
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::CliNotFound => DiagnosticCode::CliNotFound,
            Self::CliVersionInvalid => DiagnosticCode::CliVersionInvalid,
            Self::ProcessSpawnFailed => DiagnosticCode::ProcessSpawnFailed,
            Self::ProcessExited => DiagnosticCode::ProcessExited,
            Self::TransportTimeout => DiagnosticCode::TransportTimeout,
            Self::TransportClosed => DiagnosticCode::TransportClosed,
            Self::MessageTooLarge => DiagnosticCode::MessageTooLarge,
            Self::InvalidProtocolMessage => DiagnosticCode::InvalidProtocolMessage,
            Self::RpcRejected => DiagnosticCode::RpcRejected,
            Self::UnexpectedServerRequest => DiagnosticCode::UnexpectedServerRequest,
            Self::InvalidModelCatalog => DiagnosticCode::InvalidModelCatalog,
        }
    }
}
