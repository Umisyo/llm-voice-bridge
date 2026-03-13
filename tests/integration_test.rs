use llm_voice_bridge::{
    Error, LlmProviderConfig, Pipeline, PipelineConfig, SynthesisRequest, VoiceVoxConfig,
};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn voicevox_config(base_url: &str) -> VoiceVoxConfig {
    VoiceVoxConfig {
        base_url: base_url.to_string(),
        speaker: 1,
    }
}

async fn setup_voicevox_mocks(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/audio_query"))
        .and(query_param("speaker", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"accent_phrases": [], "speedScale": 1.0})),
        )
        .mount(server)
        .await;

    Mock::given(method("POST"))
        .and(path("/synthesis"))
        .and(query_param("speaker", "1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_bytes(b"RIFF\x00\x00\x00\x00WAVEfmt " as &[u8]),
        )
        .mount(server)
        .await;
}

#[tokio::test]
async fn test_openai_pipeline() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": "こんにちは、元気ですか？"
                }
            }]
        })))
        .mount(&llm_server)
        .await;

    setup_voicevox_mocks(&tts_server).await;

    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "test-key".into(),
            model: "gpt-4o".into(),
            base_url: Some(llm_server.uri()),
        },
        voicevox: voicevox_config(&tts_server.uri()),
    })
    .unwrap();

    let result = pipeline
        .run(SynthesisRequest {
            input: "挨拶して".into(),
            system_prompt: None,
        })
        .await
        .unwrap();

    assert_eq!(result.text, "こんにちは、元気ですか？");
    assert!(!result.audio_bytes.is_empty());
}

#[tokio::test]
async fn test_anthropic_pipeline() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-key"))
        .and(header("anthropic-version", "2023-06-01"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "content": [{
                "type": "text",
                "text": "はい、お手伝いします。"
            }]
        })))
        .mount(&llm_server)
        .await;

    setup_voicevox_mocks(&tts_server).await;

    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::Anthropic {
            api_key: "test-key".into(),
            model: "claude-sonnet-4-20250514".into(),
            max_tokens: Some(1024),
            base_url: Some(llm_server.uri()),
        },
        voicevox: voicevox_config(&tts_server.uri()),
    })
    .unwrap();

    let result = pipeline
        .run(SynthesisRequest {
            input: "助けて".into(),
            system_prompt: Some("丁寧に答えて".into()),
        })
        .await
        .unwrap();

    assert_eq!(result.text, "はい、お手伝いします。");
    assert!(!result.audio_bytes.is_empty());
}

#[tokio::test]
async fn test_llm_auth_error() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&llm_server)
        .await;

    setup_voicevox_mocks(&tts_server).await;

    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "bad-key".into(),
            model: "gpt-4o".into(),
            base_url: Some(llm_server.uri()),
        },
        voicevox: voicevox_config(&tts_server.uri()),
    })
    .unwrap();

    let result = pipeline
        .run(SynthesisRequest {
            input: "test".into(),
            system_prompt: None,
        })
        .await;

    assert!(matches!(result, Err(Error::LlmAuthError)));
}

#[tokio::test]
async fn test_llm_rate_limited() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Rate limited"))
        .mount(&llm_server)
        .await;

    setup_voicevox_mocks(&tts_server).await;

    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "test-key".into(),
            model: "gpt-4o".into(),
            base_url: Some(llm_server.uri()),
        },
        voicevox: voicevox_config(&tts_server.uri()),
    })
    .unwrap();

    let result = pipeline
        .run(SynthesisRequest {
            input: "test".into(),
            system_prompt: None,
        })
        .await;

    assert!(matches!(result, Err(Error::LlmRateLimited)));
}

#[tokio::test]
async fn test_llm_empty_response() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": ""
                }
            }]
        })))
        .mount(&llm_server)
        .await;

    setup_voicevox_mocks(&tts_server).await;

    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "test-key".into(),
            model: "gpt-4o".into(),
            base_url: Some(llm_server.uri()),
        },
        voicevox: voicevox_config(&tts_server.uri()),
    })
    .unwrap();

    let result = pipeline
        .run(SynthesisRequest {
            input: "test".into(),
            system_prompt: None,
        })
        .await;

    assert!(matches!(result, Err(Error::LlmEmptyResponse)));
}

#[tokio::test]
async fn test_voicevox_api_error() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": "テスト応答"
                }
            }]
        })))
        .mount(&llm_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/audio_query"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&tts_server)
        .await;

    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "test-key".into(),
            model: "gpt-4o".into(),
            base_url: Some(llm_server.uri()),
        },
        voicevox: voicevox_config(&tts_server.uri()),
    })
    .unwrap();

    let result = pipeline
        .run(SynthesisRequest {
            input: "test".into(),
            system_prompt: None,
        })
        .await;

    assert!(matches!(
        result,
        Err(Error::VoiceVoxApiError { status: 500, .. })
    ));
}

#[tokio::test]
async fn test_invalid_config_empty_api_key() {
    let result = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "".into(),
            model: "gpt-4o".into(),
            base_url: None,
        },
        voicevox: VoiceVoxConfig {
            base_url: "http://localhost:50021".into(),
            speaker: 1,
        },
    });

    assert!(matches!(result, Err(Error::InvalidConfig { .. })));
}

#[tokio::test]
async fn test_invalid_config_empty_base_url() {
    let result = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "test-key".into(),
            model: "gpt-4o".into(),
            base_url: None,
        },
        voicevox: VoiceVoxConfig {
            base_url: "".into(),
            speaker: 1,
        },
    });

    assert!(matches!(result, Err(Error::InvalidConfig { .. })));
}

#[tokio::test]
async fn test_full_pipeline_with_markdown_normalization() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{
                "message": {
                    "content": "# 回答\n\n**重要**: これはテストです。\n\n- 項目1\n- 項目2\n\n```\ncode\n```"
                }
            }]
        })))
        .mount(&llm_server)
        .await;

    setup_voicevox_mocks(&tts_server).await;

    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: "test-key".into(),
            model: "gpt-4o".into(),
            base_url: Some(llm_server.uri()),
        },
        voicevox: voicevox_config(&tts_server.uri()),
    })
    .unwrap();

    let result = pipeline
        .run(SynthesisRequest {
            input: "テスト".into(),
            system_prompt: None,
        })
        .await
        .unwrap();

    // Original text preserved in result
    assert!(result.text.contains("# 回答"));
    assert!(result.text.contains("**重要**"));

    // Audio bytes returned
    assert!(!result.audio_bytes.is_empty());
}
