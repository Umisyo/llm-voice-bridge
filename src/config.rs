use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub llm: LlmProviderConfig,
    pub voicevox: VoiceVoxConfig,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum LlmProviderConfig {
    #[serde(rename = "openai")]
    OpenAi {
        api_key: String,
        model: String,
        base_url: Option<String>,
    },
    #[serde(rename = "anthropic")]
    Anthropic {
        api_key: String,
        model: String,
        max_tokens: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceVoxConfig {
    pub base_url: String,
    pub speaker: u32,
}
