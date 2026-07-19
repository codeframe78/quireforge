use std::future::Future;

use super::{error::CodexAdapterError, types::CodexRuntimeSnapshot};

pub trait CodexBackend: Send + Sync {
    fn snapshot(
        &self,
    ) -> impl Future<Output = Result<CodexRuntimeSnapshot, CodexAdapterError>> + Send;
}
