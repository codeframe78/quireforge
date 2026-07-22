use std::{process::Stdio, time::Duration};

use tokio::{process::Command, sync::Mutex, time::timeout};

use super::{
    app_server::{AppServerCommand, AppServerProcess},
    backend::CodexBackend,
    error::CodexAdapterError,
    types::CodexRuntimeSnapshot,
};

const CLI_TIMEOUT: Duration = Duration::from_secs(3);
const MAX_VERSION_OUTPUT_BYTES: usize = 128;

pub struct CodexRuntimeService {
    program: String,
    snapshot: Mutex<Option<CodexRuntimeSnapshot>>,
}

impl Default for CodexRuntimeService {
    fn default() -> Self {
        Self {
            program: "codex".to_owned(),
            snapshot: Mutex::new(None),
        }
    }
}

impl CodexRuntimeService {
    pub async fn snapshot(&self) -> CodexRuntimeSnapshot {
        let mut cached = self.snapshot.lock().await;
        if let Some(snapshot) = cached.as_ref() {
            return snapshot.clone();
        }

        let snapshot = SystemCodexBackend::new(&self.program)
            .snapshot()
            .await
            .unwrap_or_else(|error| CodexRuntimeSnapshot::unavailable(error.diagnostic_code()));
        *cached = Some(snapshot.clone());
        snapshot
    }
}

struct SystemCodexBackend {
    program: String,
}

impl SystemCodexBackend {
    fn new(program: &str) -> Self {
        Self {
            program: program.to_owned(),
        }
    }
}

impl CodexBackend for SystemCodexBackend {
    async fn snapshot(&self) -> Result<CodexRuntimeSnapshot, CodexAdapterError> {
        Ok(probe_with_program(&self.program).await)
    }
}

async fn probe_with_program(program: &str) -> CodexRuntimeSnapshot {
    let cli_version = match probe_cli_version(program).await {
        Ok(version) => version,
        Err(error) => return CodexRuntimeSnapshot::unavailable(error.diagnostic_code()),
    };

    let command = AppServerCommand::codex(program);
    let mut process = match AppServerProcess::spawn(command) {
        Ok(process) => process,
        Err(error) => {
            return CodexRuntimeSnapshot::degraded(cli_version, error.diagnostic_code());
        }
    };

    let discovery = process.discover_models().await;
    let shutdown = process.shutdown().await;

    match (discovery, shutdown) {
        (Ok((models, _events)), Ok(())) => CodexRuntimeSnapshot::ready(cli_version, models),
        (Err(error), _) | (_, Err(error)) => {
            CodexRuntimeSnapshot::degraded(cli_version, error.diagnostic_code())
        }
    }
}

pub(crate) async fn probe_cli_version(program: &str) -> Result<String, CodexAdapterError> {
    let output = timeout(
        CLI_TIMEOUT,
        Command::new(program)
            .arg("--version")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .map_err(|_| CodexAdapterError::TransportTimeout)?
    .map_err(|_| CodexAdapterError::CliNotFound)?;

    if !output.status.success() || output.stdout.len() > MAX_VERSION_OUTPUT_BYTES {
        return Err(CodexAdapterError::CliVersionInvalid);
    }

    parse_cli_version(&output.stdout)
}

fn parse_cli_version(output: &[u8]) -> Result<String, CodexAdapterError> {
    let output = std::str::from_utf8(output)
        .map_err(|_| CodexAdapterError::CliVersionInvalid)?
        .trim();
    let version = output
        .strip_prefix("codex-cli ")
        .ok_or(CodexAdapterError::CliVersionInvalid)?;

    let core = version
        .split_once(['-', '+'])
        .map_or(version, |(core, _suffix)| core);
    let segments = core.split('.').collect::<Vec<_>>();

    if version.is_empty()
        || version.len() > 32
        || !version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
        || segments.len() != 3
        || segments
            .iter()
            .any(|segment| segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(CodexAdapterError::CliVersionInvalid);
    }

    Ok(version.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_documented_cli_version_shape() {
        assert_eq!(
            parse_cli_version(b"codex-cli 0.144.6\n").expect("version must parse"),
            "0.144.6"
        );
    }

    #[test]
    fn rejects_unbounded_or_unexpected_version_output() {
        assert!(parse_cli_version(b"codex 0.144.6\n").is_err());
        assert!(parse_cli_version(b"codex-cli ../../private\n").is_err());
    }

    #[tokio::test]
    async fn returns_an_unavailable_snapshot_for_a_missing_cli() {
        let snapshot = probe_with_program("quireforge-definitely-missing-codex").await;

        assert_eq!(
            snapshot.availability,
            super::super::types::RuntimeAvailability::Unavailable
        );
        assert!(snapshot.models.is_empty());
    }

    #[tokio::test]
    async fn runtime_service_caches_one_normalized_snapshot() {
        let service = CodexRuntimeService {
            program: "quireforge-definitely-missing-codex".to_owned(),
            snapshot: Mutex::new(None),
        };

        let first = service.snapshot().await;
        let second = service.snapshot().await;

        assert_eq!(first, second);
        assert!(service.snapshot.lock().await.is_some());
    }

    #[tokio::test]
    #[ignore = "manual non-billable probe requiring the installed Codex CLI and account catalog"]
    async fn live_probe_uses_the_supported_local_app_server() {
        let snapshot = probe_with_program("codex").await;

        assert_eq!(
            snapshot.availability,
            super::super::types::RuntimeAvailability::Ready
        );
        assert_eq!(
            snapshot.backend,
            super::super::types::BackendKind::AppServerStdio
        );
        assert!(snapshot.cli_version.is_some());
        assert!(!snapshot.models.is_empty());
        assert!(snapshot.diagnostic_code.is_none());
    }
}
