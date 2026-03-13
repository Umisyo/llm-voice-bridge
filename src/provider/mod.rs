pub(crate) mod anthropic;
pub(crate) mod openai;
pub(crate) mod sse;
pub(crate) mod voicevox;

use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::error::Error;

#[async_trait]
pub(crate) trait LlmClient: Send + Sync {
    async fn chat(&self, user_message: &str, system_prompt: Option<&str>)
        -> Result<String, Error>;

    async fn chat_stream(
        &self,
        user_message: &str,
        system_prompt: Option<&str>,
    ) -> Result<BoxStream<'_, Result<String, Error>>, Error>;
}

#[async_trait]
pub(crate) trait TtsClient: Send + Sync {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, Error>;
}
