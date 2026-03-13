pub struct SynthesisRequest {
    pub input: String,
    pub system_prompt: Option<String>,
}

pub struct SynthesisResult {
    pub text: String,
    pub audio_bytes: Vec<u8>,
}

/// A single chunk emitted by the streaming pipeline.
pub struct StreamChunk {
    /// Raw LLM text for this chunk.
    pub text: String,
    /// Text after TTS normalization.
    pub normalized_text: String,
    /// VOICEVOX synthesized audio bytes (WAV).
    pub audio_bytes: Vec<u8>,
}
