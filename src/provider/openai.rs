use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::error::Error;
use crate::provider::sse::parse_sse_stream;
use crate::provider::LlmClient;

pub(crate) struct OpenAiClient {
    http: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiClient {
    pub(crate) fn new(
        http: Client,
        api_key: String,
        model: String,
        base_url: Option<String>,
    ) -> Self {
        Self {
            http,
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".into()),
        }
    }

    fn build_messages(&self, user_message: &str, system_prompt: Option<&str>) -> Vec<Message> {
        let mut messages = Vec::new();
        if let Some(system) = system_prompt {
            messages.push(Message {
                role: "system".into(),
                content: system.into(),
            });
        }
        messages.push(Message {
            role: "user".into(),
            content: user_message.into(),
        });
        messages
    }

    fn check_error_status(status: reqwest::StatusCode) -> Option<Error> {
        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Some(Error::LlmAuthError);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Some(Error::LlmRateLimited);
        }
        None
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: Option<String>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
    content: Option<String>,
}

#[derive(Deserialize)]
struct StreamChunkResponse {
    choices: Vec<StreamChoice>,
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn chat(&self, user_message: &str, system_prompt: Option<&str>) -> Result<String, Error> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: self.build_messages(user_message, system_prompt),
            stream: None,
        };

        let url = format!("{}/v1/chat/completions", self.base_url);

        debug!(url = %url, model = %self.model, "sending OpenAI request");

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if let Some(err) = Self::check_error_status(status) {
            warn!(status = status.as_u16(), "OpenAI error status detected");
            return Err(err);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            warn!(status = status.as_u16(), "OpenAI API error");
            return Err(Error::LlmApiError {
                status: status.as_u16(),
                body,
            });
        }

        let chat_response: ChatResponse = response.json().await?;

        let text = chat_response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or(Error::LlmEmptyResponse)?;

        if text.is_empty() {
            return Err(Error::LlmEmptyResponse);
        }

        Ok(text)
    }

    async fn chat_stream(
        &self,
        user_message: &str,
        system_prompt: Option<&str>,
    ) -> Result<BoxStream<'_, Result<String, Error>>, Error> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: self.build_messages(user_message, system_prompt),
            stream: Some(true),
        };

        let url = format!("{}/v1/chat/completions", self.base_url);

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if let Some(err) = Self::check_error_status(status) {
            return Err(err);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::LlmApiError {
                status: status.as_u16(),
                body,
            });
        }

        let byte_stream = response.bytes_stream();
        let sse_stream = parse_sse_stream(byte_stream);

        let text_stream = sse_stream.filter_map(|data| async move {
            let parsed: Result<StreamChunkResponse, _> = serde_json::from_str(&data);
            match parsed {
                Ok(chunk) => chunk
                    .choices
                    .into_iter()
                    .next()
                    .and_then(|c| c.delta.content)
                    .filter(|s| !s.is_empty())
                    .map(Ok),
                Err(e) => Some(Err(Error::JsonError(e))),
            }
        });

        Ok(Box::pin(text_stream))
    }
}
