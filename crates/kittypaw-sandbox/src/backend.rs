use async_trait::async_trait;
use kittypaw_core::error::Result;
use kittypaw_core::types::ExecutionResult;

/// Configuration for sandbox execution
pub struct SandboxExecConfig {
    pub code: String,
    pub context_json: String,
    pub timeout_ms: u64,
    pub max_memory_bytes: usize,
}

#[async_trait]
pub trait SandboxBackend: Send + Sync {
    async fn execute(&self, config: SandboxExecConfig) -> Result<ExecutionResult>;
}
