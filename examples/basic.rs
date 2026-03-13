use llm_voice_bridge::{
    LlmProviderConfig, Pipeline, PipelineConfig, SynthesisRequest, VoiceVoxConfig,
};

#[tokio::main]
async fn main() -> Result<(), llm_voice_bridge::Error> {
    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set"),
            model: "gpt-4o".into(),
            base_url: None,
        },
        voicevox: VoiceVoxConfig {
            base_url: "http://127.0.0.1:50021".into(),
            speaker: 1,
        },
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    })?;

    let result = pipeline
        .run(SynthesisRequest {
            input: "この文章に返答して読み上げて".into(),
            system_prompt: Some("簡潔に答えて".into()),
        })
        .await?;

    std::fs::write("output.wav", &result.audio_bytes)?;
    println!("LLM response: {}", result.text);
    println!(
        "Audio written to output.wav ({} bytes)",
        result.audio_bytes.len()
    );

    Ok(())
}
