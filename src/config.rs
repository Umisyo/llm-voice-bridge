use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub llm: LlmProviderConfig,
    pub voicevox: VoiceVoxConfig,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default)]
    pub system_prompt: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "provider")]
pub enum LlmProviderConfig {
    #[serde(rename = "openai")]
    OpenAi {
        #[serde(skip_serializing)]
        api_key: String,
        model: String,
        base_url: Option<String>,
    },
    #[serde(rename = "anthropic")]
    Anthropic {
        #[serde(skip_serializing)]
        api_key: String,
        model: String,
        max_tokens: Option<u32>,
        base_url: Option<String>,
    },
}

impl fmt::Debug for LlmProviderConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmProviderConfig::OpenAi {
                model, base_url, ..
            } => f
                .debug_struct("OpenAi")
                .field("api_key", &"***")
                .field("model", model)
                .field("base_url", base_url)
                .finish(),
            LlmProviderConfig::Anthropic {
                model,
                max_tokens,
                base_url,
                ..
            } => f
                .debug_struct("Anthropic")
                .field("api_key", &"***")
                .field("model", model)
                .field("max_tokens", max_tokens)
                .field("base_url", base_url)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceVoxConfig {
    pub base_url: String,
    pub speaker: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_masks_api_key_openai() {
        let config = LlmProviderConfig::OpenAi {
            api_key: "sk-secret-key-12345".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };
        let debug_output = format!("{:?}", config);
        assert!(
            !debug_output.contains("sk-secret-key-12345"),
            "Debug output must not contain the raw API key"
        );
        assert!(
            debug_output.contains("***"),
            "Debug output must contain masked value"
        );
    }

    #[test]
    fn debug_masks_api_key_anthropic() {
        let config = LlmProviderConfig::Anthropic {
            api_key: "ant-secret-key-67890".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: Some(1024),
            base_url: None,
        };
        let debug_output = format!("{:?}", config);
        assert!(
            !debug_output.contains("ant-secret-key-67890"),
            "Debug output must not contain the raw API key"
        );
        assert!(
            debug_output.contains("***"),
            "Debug output must contain masked value"
        );
    }

    #[test]
    fn serialize_skips_api_key() {
        let config = LlmProviderConfig::OpenAi {
            api_key: "sk-secret-key-12345".to_string(),
            model: "gpt-4".to_string(),
            base_url: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(
            !json.contains("sk-secret-key-12345"),
            "Serialized output must not contain the raw API key"
        );
        assert!(
            !json.contains("api_key"),
            "Serialized output must not contain api_key field"
        );
    }
}
