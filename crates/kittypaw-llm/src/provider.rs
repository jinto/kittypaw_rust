use async_trait::async_trait;
use kittypaw_core::error::Result;
use kittypaw_core::types::LlmMessage;
use std::sync::Arc;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate(&self, messages: &[LlmMessage]) -> Result<String>;

    /// Maximum context window in tokens. Used for prompt budget calculation.
    /// Override in concrete providers to return model-specific values.
    fn context_window(&self) -> usize {
        8_192
    }

    /// Maximum output tokens reserved for the model's response.
    fn max_tokens(&self) -> usize {
        4_096
    }

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
