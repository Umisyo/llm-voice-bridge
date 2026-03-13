use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::provider::LlmClient;

pub(crate) struct AnthropicClient {
    http: Client,
    api_key: String,
    model: String,
    max_tokens: u32,
    base_url: String,
}

impl AnthropicClient {
    pub(crate) fn new(
        http: Client,
        api_key: String,
        model: String,
        max_tokens: Option<u32>,
        base_url: Option<String>,
    ) -> Self {
        Self {
            http,
            api_key,
            model,
            max_tokens: max_tokens.unwrap_or(1024),
            base_url: base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
        }
    }
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    text: Option<String>,
}

#[async_trait]
impl LlmClient for AnthropicClient {
    async fn chat(&self, user_message: &str, system_prompt: Option<&str>) -> Result<String, Error> {
        let request = MessagesRequest {
            model: self.model.clone(),
            max_tokens: self.max_tokens,
            system: system_prompt.map(|s| s.to_string()),
            messages: vec![Message {
                role: "user".into(),
                content: user_message.into(),
            }],
        };

        let response = self
            .http
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED {
            return Err(Error::LlmAuthError);
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(Error::LlmRateLimited);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::LlmApiError {
                status: status.as_u16(),
                body,
            });
        }

        let messages_response: MessagesResponse = response.json().await?;

        let text = messages_response
            .content
            .into_iter()
            .find(|block| block.block_type == "text")
            .and_then(|block| block.text)
            .ok_or(Error::LlmEmptyResponse)?;

        if text.is_empty() {
            return Err(Error::LlmEmptyResponse);
        }

        Ok(text)
    }
}
