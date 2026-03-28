use async_trait::async_trait;
use kittypaw_core::error::Result;
use kittypaw_core::types::LlmMessage;
use std::sync::Arc;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[LlmMessage]) -> Result<String>;

    /// Optional streaming generation. Default implementation collects full response.
    async fn generate_stream(
        &self,
        messages: &[LlmMessage],
        on_token: Arc<dyn Fn(String) + Send + Sync>,
    ) -> Result<String> {
        // Default: call generate() and pass full result as single token
        let result = self.generate(messages).await?;
        on_token(result.clone());
        Ok(result)
    }
}
