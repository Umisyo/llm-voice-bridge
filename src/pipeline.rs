use std::collections::VecDeque;
use std::time::{Duration, Instant};

use futures::stream::BoxStream;
use futures::StreamExt;
use reqwest::Client;
use tracing::{debug, error, info, instrument};

use crate::chunker::StreamChunker;
use crate::config::{LlmProviderConfig, PipelineConfig};
use crate::error::Error;
use crate::normalize::normalize_for_tts;
use crate::provider::anthropic::AnthropicClient;
use crate::provider::openai::OpenAiClient;
use crate::provider::voicevox::VoiceVoxClient;
use crate::provider::{LlmClient, TtsClient};
use crate::retry::with_retry;
use crate::types::{StreamChunk, SynthesisRequest, SynthesisResult};

const DEFAULT_TIMEOUT_SECS: u64 = 30;

pub struct Pipeline {
    llm: Box<dyn LlmClient>,
    tts: Box<dyn TtsClient>,
    max_retries: u32,
    system_prompt: Option<String>,
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
                debug!(provider = "openai", model = %model, "LLM provider configured");
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
                debug!(provider = "anthropic", model = %model, "LLM provider configured");
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

        debug!(base_url = %config.voicevox.base_url, speaker = config.voicevox.speaker, "VOICEVOX configured");

        let tts = Box::new(VoiceVoxClient::new(
            http_client,
            config.voicevox.base_url,
            config.voicevox.speaker,
        ));

        Ok(Self {
            llm,
            tts,
            max_retries,
            system_prompt: config.system_prompt,
        })
    }

    #[instrument(skip(self, request), fields(input_len = request.input.len()))]
    pub async fn run(&self, request: SynthesisRequest) -> Result<SynthesisResult, Error> {
        info!("pipeline started");

        let system_prompt = request
            .system_prompt
            .as_deref()
            .or(self.system_prompt.as_deref());

        let start = Instant::now();
        let llm_response = with_retry(self.max_retries, || {
            self.llm.chat(&request.input, system_prompt)
        })
        .await
        .map_err(|e| {
            error!(error = %e, "LLM request failed");
            e
        })?;
        let llm_elapsed = start.elapsed();
        debug!(
            response_len = llm_response.len(),
            elapsed_ms = llm_elapsed.as_millis() as u64,
            "LLM response received"
        );

        let before_len = llm_response.len();
        let normalized_text = normalize_for_tts(&llm_response);
        debug!(
            before_len,
            after_len = normalized_text.len(),
            "text normalized"
        );

        let start = Instant::now();
        let audio_bytes = with_retry(self.max_retries, || self.tts.synthesize(&normalized_text))
            .await
            .map_err(|e| {
                error!(error = %e, "TTS synthesis failed");
                e
            })?;
        let tts_elapsed = start.elapsed();
        debug!(
            audio_bytes = audio_bytes.len(),
            elapsed_ms = tts_elapsed.as_millis() as u64,
            "TTS synthesis completed"
        );

        info!(
            total_elapsed_ms = (llm_elapsed + tts_elapsed).as_millis() as u64,
            "pipeline completed"
        );

        Ok(SynthesisResult {
            text: llm_response,
            audio_bytes,
        })
    }

    pub async fn run_stream(
        &self,
        request: SynthesisRequest,
    ) -> Result<BoxStream<'_, Result<StreamChunk, Error>>, Error> {
        let llm_stream = self
            .llm
            .chat_stream(&request.input, request.system_prompt.as_deref())
            .await?;

        let tts = &self.tts;

        let stream = futures::stream::unfold(
            StreamState::Streaming {
                llm_stream,
                chunker: StreamChunker::new(),
                pending_sentences: VecDeque::new(),
            },
            move |state| async move {
                match state {
                    StreamState::Streaming {
                        mut llm_stream,
                        mut chunker,
                        mut pending_sentences,
                    } => {
                        // First, drain any pending sentences in order
                        if let Some(sentence) = pending_sentences.pop_front() {
                            let result = synthesize_sentence(tts.as_ref(), &sentence).await;
                            return Some((
                                result,
                                StreamState::Streaming {
                                    llm_stream,
                                    chunker,
                                    pending_sentences,
                                },
                            ));
                        }

                        // Pull from LLM stream
                        loop {
                            match llm_stream.next().await {
                                Some(Ok(delta)) => {
                                    let sentences = chunker.push(&delta);
                                    if !sentences.is_empty() {
                                        let mut iter = sentences.into_iter();
                                        let first = iter.next().unwrap();
                                        let pending: VecDeque<String> = iter.collect();
                                        let result =
                                            synthesize_sentence(tts.as_ref(), &first).await;
                                        return Some((
                                            result,
                                            StreamState::Streaming {
                                                llm_stream,
                                                chunker,
                                                pending_sentences: pending,
                                            },
                                        ));
                                    }
                                    // No complete sentence yet, continue pulling
                                }
                                Some(Err(e)) => {
                                    return Some((Err(e), StreamState::Done));
                                }
                                None => {
                                    // Stream ended, flush remaining
                                    if let Some(remaining) = chunker.flush() {
                                        let result =
                                            synthesize_sentence(tts.as_ref(), &remaining).await;
                                        return Some((result, StreamState::Done));
                                    }
                                    return None;
                                }
                            }
                        }
                    }
                    StreamState::Done => None,
                }
            },
        );

        Ok(Box::pin(stream))
    }
}

enum StreamState<'a> {
    Streaming {
        llm_stream: BoxStream<'a, Result<String, Error>>,
        chunker: StreamChunker,
        pending_sentences: VecDeque<String>,
    },
    Done,
}

async fn synthesize_sentence(tts: &dyn TtsClient, sentence: &str) -> Result<StreamChunk, Error> {
    let normalized = normalize_for_tts(sentence);
    let audio_bytes = tts.synthesize(&normalized).await?;
    Ok(StreamChunk {
        text: sentence.to_string(),
        normalized_text: normalized,
        audio_bytes,
    })
}
