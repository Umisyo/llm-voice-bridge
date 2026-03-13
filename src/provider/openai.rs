use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::provider::LlmClient;

pub(crate) struct OpenAiClient {
    http: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiClient {
    pub(crate) fn new(api_key: String, model: String, base_url: Option<String>) -> Self {
        Self {
            http: Client::new(),
            api_key,
            model,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com".into()),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
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

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn chat(&self, user_message: &str, system_prompt: Option<&str>) -> Result<String, Error> {
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

        let request = ChatRequest {
            model: self.model.clone(),
            messages,
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
}
