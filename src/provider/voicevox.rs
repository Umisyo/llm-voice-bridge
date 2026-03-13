use async_trait::async_trait;
use reqwest::Client;

use crate::error::Error;
use crate::provider::TtsClient;

pub(crate) struct VoiceVoxClient {
    http: Client,
    base_url: String,
    speaker: u32,
}

impl VoiceVoxClient {
    pub(crate) fn new(base_url: String, speaker: u32) -> Self {
        Self {
            http: Client::new(),
            base_url,
            speaker,
        }
    }
}

#[async_trait]
impl TtsClient for VoiceVoxClient {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, Error> {
        let audio_query_url = format!(
            "{}/audio_query?text={}&speaker={}",
            self.base_url,
            urlencoding(text),
            self.speaker
        );

        let query_response = self
            .http
            .post(&audio_query_url)
            .send()
            .await
            .map_err(|e| Error::VoiceVoxUnreachable {
                url: self.base_url.clone(),
                source: e,
            })?;

        let query_status = query_response.status();
        if !query_status.is_success() {
            let body = query_response.text().await.unwrap_or_default();
            return Err(Error::VoiceVoxApiError {
                status: query_status.as_u16(),
                body,
            });
        }

        let audio_query: serde_json::Value = query_response.json().await?;

        let synthesis_url = format!("{}/synthesis?speaker={}", self.base_url, self.speaker);

        let synth_response = self
            .http
            .post(&synthesis_url)
            .json(&audio_query)
            .send()
            .await
            .map_err(|e| Error::VoiceVoxUnreachable {
                url: self.base_url.clone(),
                source: e,
            })?;

        let synth_status = synth_response.status();
        if !synth_status.is_success() {
            let body = synth_response.text().await.unwrap_or_default();
            return Err(Error::VoiceVoxApiError {
                status: synth_status.as_u16(),
                body,
            });
        }

        let audio_bytes = synth_response.bytes().await?.to_vec();
        Ok(audio_bytes)
    }
}

fn urlencoding(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
