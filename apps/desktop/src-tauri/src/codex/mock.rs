use super::{backend::CodexBackend, error::CodexAdapterError, types::CodexRuntimeSnapshot};

#[derive(Clone)]
pub struct MockCodexBackend {
    snapshot: CodexRuntimeSnapshot,
}

impl MockCodexBackend {
    pub fn from_shared_fixture() -> Self {
        let snapshot = serde_json::from_str(include_str!("../../../fixtures/codex-runtime.json"))
            .expect("shared Codex runtime fixture must be valid");

        Self { snapshot }
    }
}

impl CodexBackend for MockCodexBackend {
    async fn snapshot(&self) -> Result<CodexRuntimeSnapshot, CodexAdapterError> {
        Ok(self.snapshot.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codex::types::{BackendKind, RuntimeAvailability};

    #[tokio::test]
    async fn returns_the_sanitized_shared_fixture() {
        let backend = MockCodexBackend::from_shared_fixture();
        let snapshot = backend.snapshot().await.expect("mock must succeed");

        assert_eq!(snapshot.availability, RuntimeAvailability::Ready);
        assert_eq!(snapshot.backend, BackendKind::AppServerStdio);
        assert_eq!(snapshot.models.len(), 1);
        assert_eq!(snapshot.models[0].id, "gpt-5.6-sol");
    }
}
