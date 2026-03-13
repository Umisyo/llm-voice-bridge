use std::time::Duration;

use reqwest::Client;

use crate::config::{LlmProviderConfig, PipelineConfig};
use crate::error::Error;
use crate::normalize::normalize_for_tts;
use crate::provider::anthropic::AnthropicClient;
use crate::provider::openai::OpenAiClient;
use crate::provider::voicevox::VoiceVoxClient;
use crate::provider::{LlmClient, TtsClient};
use crate::retry::with_retry;
use crate::types::{SynthesisRequest, SynthesisResult};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

pub struct Pipeline {
    llm: Box<dyn LlmClient>,
    tts: Box<dyn TtsClient>,
    max_retries: u32,
}

impl Pipeline {
    pub fn new(config: PipelineConfig) -> Result<Self, Error> {
        let timeout = config.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS);
        let max_retries = config.max_retries.unwrap_or(0);
        let http_client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()?;

        let llm: Box<dyn LlmClient> = match config.llm {
            LlmProviderConfig::OpenAi {
                api_key,
                model,
                base_url,
            } => {
                if api_key.is_empty() {
                    return Err(Error::InvalidConfig {
                        message: "OpenAI API key is empty".into(),
                    });
                }
                Box::new(OpenAiClient::new(
                    http_client.clone(),
                    api_key,
                    model,
                    base_url,
                ))
            }
            LlmProviderConfig::Anthropic {
                api_key,
                model,
                max_tokens,
                base_url,
            } => {
                if api_key.is_empty() {
                    return Err(Error::InvalidConfig {
                        message: "Anthropic API key is empty".into(),
                    });
                }
                Box::new(AnthropicClient::new(
                    http_client.clone(),
                    api_key,
                    model,
                    max_tokens,
                    base_url,
                ))
            }
        };

        if config.voicevox.base_url.is_empty() {
            return Err(Error::InvalidConfig {
                message: "VOICEVOX base_url is empty".into(),
            });
        }

        let tts = Box::new(VoiceVoxClient::new(
            http_client,
            config.voicevox.base_url,
            config.voicevox.speaker,
        ));

        Ok(Self {
            llm,
            tts,
            max_retries,
        })
    }

    pub async fn run(&self, request: SynthesisRequest) -> Result<SynthesisResult, Error> {
        let llm_response = with_retry(self.max_retries, || {
            self.llm
                .chat(&request.input, request.system_prompt.as_deref())
        })
        .await?;

        let normalized_text = normalize_for_tts(&llm_response);

        let audio_bytes =
            with_retry(self.max_retries, || self.tts.synthesize(&normalized_text)).await?;

        Ok(SynthesisResult {
            text: llm_response,
            audio_bytes,
        })
    }
}
