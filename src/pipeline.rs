use crate::config::{LlmProviderConfig, PipelineConfig};
use crate::error::Error;
use crate::normalize::normalize_for_tts;
use crate::provider::anthropic::AnthropicClient;
use crate::provider::openai::OpenAiClient;
use crate::provider::voicevox::VoiceVoxClient;
use crate::provider::{LlmClient, TtsClient};
use crate::types::{SynthesisRequest, SynthesisResult};

pub struct Pipeline {
    llm: Box<dyn LlmClient>,
    tts: Box<dyn TtsClient>,
}

impl Pipeline {
    pub fn new(config: PipelineConfig) -> Result<Self, Error> {
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
                Box::new(OpenAiClient::new(api_key, model, base_url))
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
                Box::new(AnthropicClient::new(api_key, model, max_tokens, base_url))
            }
        };

        if config.voicevox.base_url.is_empty() {
            return Err(Error::InvalidConfig {
                message: "VOICEVOX base_url is empty".into(),
            });
        }

        let tts = Box::new(VoiceVoxClient::new(
            config.voicevox.base_url,
            config.voicevox.speaker,
        ));

        Ok(Self { llm, tts })
    }

    pub async fn run(&self, request: SynthesisRequest) -> Result<SynthesisResult, Error> {
        let llm_response = self
            .llm
            .chat(&request.input, request.system_prompt.as_deref())
            .await?;

        let normalized_text = normalize_for_tts(&llm_response);

        let audio_bytes = self.tts.synthesize(&normalized_text).await?;

        Ok(SynthesisResult {
            text: llm_response,
            audio_bytes,
        })
    }
}
