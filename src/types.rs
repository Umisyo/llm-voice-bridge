pub struct SynthesisRequest {
    pub input: String,
    pub system_prompt: Option<String>,
}

pub struct SynthesisResult {
    pub text: String,
    pub audio_bytes: Vec<u8>,
}
