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
        let audio_query_url = format!("{}/audio_query", self.base_url);

        let query_response = self
            .http
            .post(&audio_query_url)
            .query(&[("text", text), ("speaker", &self.speaker.to_string())])
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

        let synthesis_url = format!("{}/synthesis", self.base_url);

        let synth_response = self
            .http
            .post(&synthesis_url)
            .query(&[("speaker", &self.speaker.to_string())])
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
