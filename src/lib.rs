//! # llm-voice-bridge
//!
//! テキスト入力 → LLM（OpenAI / Anthropic） → テキスト正規化 → VOICEVOX → WAV音声
//! を一気通貫で行う軽量ライブラリです。
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use llm_voice_bridge::{Pipeline, PipelineConfig, LlmProviderConfig, VoiceVoxConfig, SynthesisRequest};
//!
//! # async fn example() -> Result<(), llm_voice_bridge::Error> {
//! let config = PipelineConfig {
//!     llm: LlmProviderConfig::OpenAi {
//!         api_key: "sk-...".to_string(),
//!         model: "gpt-4o-mini".to_string(),
//!         base_url: None,
//!     },
//!     voicevox: VoiceVoxConfig {
//!         base_url: "http://localhost:50021".to_string(),
//!         speaker: 1,
//!     },
//!     timeout_secs: None,
//! };
//!
//! let pipeline = Pipeline::new(config)?;
//! let result = pipeline.run(SynthesisRequest {
//!     input: "Rustの魅力を一言で".to_string(),
//!     system_prompt: None,
//! }).await?;
//!
//! std::fs::write("output.wav", &result.audio_bytes)?;
//! # Ok(())
//! # }
//! ```

mod config;
mod error;
mod normalize;
mod pipeline;
mod provider;
mod types;

pub use config::{LlmProviderConfig, PipelineConfig, VoiceVoxConfig};
pub use error::Error;
pub use normalize::normalize_for_tts;
pub use pipeline::Pipeline;
pub use types::{SynthesisRequest, SynthesisResult};
