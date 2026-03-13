use futures::StreamExt;
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

fn build_openai_sse(chunks: &[&str]) -> String {
    let mut body = String::new();
    for chunk in chunks {
        let json = serde_json::json!({
            "choices": [{
                "delta": { "content": chunk }
            }]
        });
        body.push_str(&format!("data: {}\n\n", json));
    }
    body.push_str("data: [DONE]\n\n");
    body
}

fn build_anthropic_sse(chunks: &[&str]) -> String {
    let mut body = String::new();
    // message_start event
    body.push_str("data: {\"type\": \"message_start\", \"message\": {}}\n\n");
    // content_block_start
    body.push_str("data: {\"type\": \"content_block_start\", \"index\": 0}\n\n");
    for chunk in chunks {
        let json = serde_json::json!({
            "type": "content_block_delta",
            "delta": { "type": "text_delta", "text": chunk }
        });
        body.push_str(&format!("data: {}\n\n", json));
    }
    // message_stop
    body.push_str("data: {\"type\": \"message_stop\"}\n\n");
    body
}

#[tokio::test]
async fn test_openai_streaming_pipeline() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    let sse_body = build_openai_sse(&["こんにちは。", "元気ですか？", "今日もいい天気ですね。"]);

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
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
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    })
    .unwrap();

    let mut stream = pipeline
        .run_stream(SynthesisRequest {
            input: "挨拶して".into(),
            system_prompt: None,
        })
        .await
        .unwrap();

    let mut chunks = Vec::new();
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        assert!(!chunk.audio_bytes.is_empty());
        assert!(!chunk.normalized_text.is_empty());
        chunks.push(chunk.text);
    }

    assert!(!chunks.is_empty());
    // All text should be present across chunks
    let full_text: String = chunks.join("");
    assert!(full_text.contains("こんにちは。"));
    assert!(full_text.contains("元気ですか？"));
}

#[tokio::test]
async fn test_anthropic_streaming_pipeline() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    let sse_body = build_anthropic_sse(&["はい、", "お手伝いします。", "何でも聞いてください！"]);

    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("x-api-key", "test-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
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
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    })
    .unwrap();

    let mut stream = pipeline
        .run_stream(SynthesisRequest {
            input: "助けて".into(),
            system_prompt: Some("丁寧に答えて".into()),
        })
        .await
        .unwrap();

    let mut chunks = Vec::new();
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        assert!(!chunk.audio_bytes.is_empty());
        chunks.push(chunk.text);
    }

    assert!(!chunks.is_empty());
    let full_text: String = chunks.join("");
    assert!(full_text.contains("お手伝いします。"));
}

#[tokio::test]
async fn test_streaming_auth_error() {
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
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    })
    .unwrap();

    let result = pipeline
        .run_stream(SynthesisRequest {
            input: "test".into(),
            system_prompt: None,
        })
        .await;

    assert!(matches!(result, Err(Error::LlmAuthError)));
}

#[tokio::test]
async fn test_streaming_incremental_chunks() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    // Simulate token-by-token streaming that builds up sentences
    let sse_body = build_openai_sse(&["Rust", "は", "素晴らしい。", "安全", "で高速です！"]);

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
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
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    })
    .unwrap();

    let mut stream = pipeline
        .run_stream(SynthesisRequest {
            input: "Rustについて".into(),
            system_prompt: None,
        })
        .await
        .unwrap();

    let mut chunks = Vec::new();
    while let Some(result) = stream.next().await {
        chunks.push(result.unwrap().text);
    }

    // Should get 2 chunks: "Rustは素晴らしい。" and "安全で高速です！"
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], "Rustは素晴らしい。");
    assert_eq!(chunks[1], "安全で高速です！");
}

#[tokio::test]
async fn test_streaming_multi_sentence_in_single_delta() {
    let llm_server = MockServer::start().await;
    let tts_server = MockServer::start().await;

    // Single delta contains multiple sentences — tests ordering correctness
    let sse_body = build_openai_sse(&["最初の文。二番目の文。三番目の文。"]);

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(sse_body),
        )
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
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    })
    .unwrap();

    let mut stream = pipeline
        .run_stream(SynthesisRequest {
            input: "テスト".into(),
            system_prompt: None,
        })
        .await
        .unwrap();

    let mut chunks = Vec::new();
    while let Some(result) = stream.next().await {
        chunks.push(result.unwrap().text);
    }

    // Verify correct ordering
    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], "最初の文。");
    assert_eq!(chunks[1], "二番目の文。");
    assert_eq!(chunks[2], "三番目の文。");
}
