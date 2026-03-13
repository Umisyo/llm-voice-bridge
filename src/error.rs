use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid configuration: {message}")]
    InvalidConfig { message: String },

    #[error("LLM authentication failed (HTTP 401)")]
    LlmAuthError,

    #[error("LLM rate limited (HTTP 429)")]
    LlmRateLimited,

    #[error("LLM API error (HTTP {status}): {body}")]
    LlmApiError { status: u16, body: String },

    #[error("LLM returned empty response")]
    LlmEmptyResponse,

    #[error("VOICEVOX engine unreachable at {url}: {source}")]
    VoiceVoxUnreachable { url: String, source: reqwest::Error },

    #[error("VOICEVOX API error (HTTP {status}): {body}")]
    VoiceVoxApiError { status: u16, body: String },

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
