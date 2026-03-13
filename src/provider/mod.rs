pub(crate) mod anthropic;
pub(crate) mod openai;
pub(crate) mod voicevox;

use async_trait::async_trait;

use crate::error::Error;

#[async_trait]
pub(crate) trait LlmClient: Send + Sync {
    async fn chat(&self, user_message: &str, system_prompt: Option<&str>) -> Result<String, Error>;
}

#[async_trait]
pub(crate) trait TtsClient: Send + Sync {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, Error>;
}
