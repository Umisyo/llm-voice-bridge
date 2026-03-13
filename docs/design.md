# llm-voice-bridge 設計ドキュメント

## 概要

テキスト入力 → LLM(OpenAI/Anthropic) → テキスト正規化 → VOICEVOX → WAV音声 を一気通貫で行う軽量Rustライブラリ。

## ディレクトリ構成

```
llm-voice-bridge/
├── Cargo.toml
├── LICENSE
├── docs/
│   └── design.md
├── src/
│   ├── lib.rs                    # クレートルート: 公開API再エクスポート
│   ├── config.rs                 # PipelineConfig, LlmProviderConfig, VoiceVoxConfig
│   ├── error.rs                  # Error enum (thiserror)
│   ├── types.rs                  # SynthesisRequest, SynthesisResult
│   ├── normalize.rs              # テキスト正規化 (Markdown除去等)
│   ├── pipeline.rs               # Pipeline struct, run() オーケストレーション
│   └── provider/
│       ├── mod.rs                # LlmClient / TtsClient trait定義
│       ├── openai.rs             # OpenAI Chat Completions実装
│       ├── anthropic.rs          # Anthropic Messages API実装
│       └── voicevox.rs           # VOICEVOX audio_query + synthesis実装
├── tests/
│   ├── normalize_test.rs         # 正規化ユニットテスト
│   └── integration_test.rs       # wiremockによる統合テスト
└── examples/
    └── basic.rs                  # 最小動作サンプル
```

## 公開API

### 利用例

```rust
use llm_voice_bridge::{Pipeline, PipelineConfig, LlmProviderConfig, VoiceVoxConfig, SynthesisRequest};

#[tokio::main]
async fn main() -> Result<(), llm_voice_bridge::Error> {
    let pipeline = Pipeline::new(PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: std::env::var("OPENAI_API_KEY").unwrap(),
            model: "gpt-4o".into(),
            base_url: None,
        },
        voicevox: VoiceVoxConfig {
            base_url: "http://127.0.0.1:50021".into(),
            speaker: 1,
        },
    })?;

    let result = pipeline.run(SynthesisRequest {
        input: "この文章に返答して読み上げて".into(),
        system_prompt: Some("簡潔に答えて".into()),
    }).await?;

    std::fs::write("output.wav", &result.audio_bytes)?;
    Ok(())
}
```

### 公開型一覧

| 型 | 説明 |
|---|---|
| `PipelineConfig` | LLM設定 + VOICEVOX設定 |
| `LlmProviderConfig` | enum: OpenAi / Anthropic |
| `VoiceVoxConfig` | base_url + speaker |
| `SynthesisRequest` | input + system_prompt? |
| `SynthesisResult` | text + audio_bytes |
| `Pipeline` | new(config), run(request) |
| `Error` | 構造化エラー enum |

## 内部trait

```rust
#[async_trait]
pub(crate) trait LlmClient: Send + Sync {
    async fn chat(&self, user_message: &str, system_prompt: Option<&str>) -> Result<String, Error>;
}

#[async_trait]
pub(crate) trait TtsClient: Send + Sync {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, Error>;
}
```

## Provider設計

### OpenAI
- `POST {base_url}/v1/chat/completions`
- Headers: `Authorization: Bearer {api_key}`

### Anthropic
- `POST https://api.anthropic.com/v1/messages`
- Headers: `x-api-key`, `anthropic-version: 2023-06-01`

### VOICEVOX
- Step 1: `POST {base_url}/audio_query?text={text}&speaker={speaker}`
- Step 2: `POST {base_url}/synthesis?speaker={speaker}` (body = audio_query result)

## エラー設計

- `InvalidConfig` — 設定バリデーション失敗
- `LlmAuthError` — HTTP 401
- `LlmRateLimited` — HTTP 429
- `LlmApiError` — その他のLLM APIエラー
- `LlmEmptyResponse` — 空レスポンス
- `VoiceVoxUnreachable` — VOICEVOX接続不可
- `VoiceVoxApiError` — VOICEVOX APIエラー
- `HttpError`, `JsonError`, `IoError` — 汎用エラー

## テキスト正規化

`normalize_for_tts(text) -> String`:
1. コードブロック除去
2. インラインコード除去
3. Markdown見出し除去
4. 太字/斜体マーカー除去
5. 箇条書き変換
6. URL除去
7. 連続空白の正規化 + trim

## パイプラインフロー

```
SynthesisRequest.input
    → LlmClient::chat() → LLMレスポンス(text)
    → normalize_for_tts() → 正規化テキスト
    → TtsClient::synthesize() → WAVバイト列
    → SynthesisResult { text: 元のLLMレスポンス, audio_bytes }
```
