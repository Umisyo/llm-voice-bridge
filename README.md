# llm-voice-bridge

テキスト入力 → LLM（OpenAI / Anthropic） → テキスト正規化 → VOICEVOX → WAV音声 を一気通貫で行う軽量Rustライブラリです。

## 特徴

- **LLMプロバイダー切り替え** — OpenAI・Anthropic を設定だけで切り替え可能
- **ストリーミング合成** — `run_stream()` でLLM応答を文単位で逐次音声合成し、低遅延で再生可能
- **TTS向けテキスト正規化** — Markdown記法（見出し・太字・コードブロック・リンク・テーブル・URLなど）を自動除去し、音声合成に適したテキストへ変換
- **VOICEVOX連携** — VOICEVOX Engine の audio_query + synthesis API を使い WAV 音声を生成
- **リトライ機構** — 一時的なエラー（429 / 5xx）に対する指数バックオフ付きリトライ
- **構造化ロギング** — `tracing` クレートによるリクエスト/レスポンスのトレーシング
- **シンプルなAPI** — `Pipeline::new()` で構築し `Pipeline::run()` または `Pipeline::run_stream()` を呼ぶだけ

## インストール

```bash
cargo add llm-voice-bridge
```

または `Cargo.toml` に直接追加：

```toml
[dependencies]
llm-voice-bridge = "0.1"
```

## 使い方

### 基本的な使用例（一括合成）

```rust
use llm_voice_bridge::{
    LlmProviderConfig, Pipeline, PipelineConfig, SynthesisRequest, VoiceVoxConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: std::env::var("OPENAI_API_KEY")?,
            model: "gpt-4o-mini".to_string(),
            base_url: None,
        },
        voicevox: VoiceVoxConfig {
            base_url: "http://localhost:50021".to_string(),
            speaker: 1,
        },
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    };

    let pipeline = Pipeline::new(config)?;

    let result = pipeline
        .run(SynthesisRequest {
            input: "Rustの魅力を一言で教えて".to_string(),
            system_prompt: Some("簡潔に日本語で回答してください".to_string()),
        })
        .await?;

    println!("LLM応答: {}", result.text);
    std::fs::write("output.wav", &result.audio_bytes)?;
    println!("音声を output.wav に保存しました（{} bytes）", result.audio_bytes.len());

    Ok(())
}
```

### ストリーミング合成

LLMの応答を文単位でリアルタイムに音声合成します。長い応答でも最初の文が完成次第、即座に音声を受け取れます。

```rust
use futures::StreamExt;
use llm_voice_bridge::{
    LlmProviderConfig, Pipeline, PipelineConfig, SynthesisRequest, VoiceVoxConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PipelineConfig {
        llm: LlmProviderConfig::OpenAi {
            api_key: std::env::var("OPENAI_API_KEY")?,
            model: "gpt-4o-mini".to_string(),
            base_url: None,
        },
        voicevox: VoiceVoxConfig {
            base_url: "http://localhost:50021".to_string(),
            speaker: 1,
        },
        timeout_secs: None,
        max_retries: None,
        system_prompt: None,
    };

    let pipeline = Pipeline::new(config)?;

    let mut stream = pipeline
        .run_stream(SynthesisRequest {
            input: "日本の四季について教えて".to_string(),
            system_prompt: Some("簡潔に日本語で回答してください".to_string()),
        })
        .await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        println!("テキスト: {}", chunk.normalized_text);
        println!("音声: {} bytes", chunk.audio_bytes.len());
        // chunk.audio_bytes を逐次再生可能
    }

    Ok(())
}
```

### Anthropic を使う場合

```rust
let config = PipelineConfig {
    llm: LlmProviderConfig::Anthropic {
        api_key: std::env::var("ANTHROPIC_API_KEY")?,
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: Some(1024),
        base_url: None,
    },
    voicevox: VoiceVoxConfig {
        base_url: "http://localhost:50021".to_string(),
        speaker: 1,
    },
    timeout_secs: Some(60),
    max_retries: Some(3),
    system_prompt: Some("あなたは親切なアシスタントです".to_string()),
};
```

### テキスト正規化のみ使う場合

LLM応答に含まれるMarkdown記法を除去して、TTS向けのプレーンテキストに変換できます。

```rust
use llm_voice_bridge::normalize_for_tts;

let markdown = "# タイトル\n\n**太字**のテキスト\n\n- 項目1\n- 項目2\n\n```\ncode block\n```\n\n終わり";
let plain = normalize_for_tts(markdown);
// => "タイトル\n太字のテキスト\n項目1。\n項目2。\n終わり"
```

正規化で処理される内容:

- Markdown見出し（`#`〜`######`）の除去
- 太字・斜体（`**`, `__`, `*`）マーカーの除去
- インラインコード（`` ` ``）の除去
- コードブロック（` ``` `）全体の除去
- Markdownリンク（`[text](url)`）→ テキスト部分のみ抽出
- URL（`http://`, `https://`）の除去
- 箇条書き（`-`, `*`）・番号付きリスト（`1.`）を文末「。」付きのテキストへ変換
- HTMLテーブル（`|`区切り）の除去
- 連続する空白・改行の正規化

## 設定リファレンス

### PipelineConfig

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `llm` | `LlmProviderConfig` | Yes | LLMプロバイダーの設定 |
| `voicevox` | `VoiceVoxConfig` | Yes | VOICEVOX Engineの設定 |
| `timeout_secs` | `Option<u64>` | No | HTTPタイムアウト秒数（デフォルト: 30） |
| `max_retries` | `Option<u32>` | No | 一時的エラー時の最大リトライ回数（デフォルト: 0） |
| `system_prompt` | `Option<String>` | No | デフォルトのシステムプロンプト（リクエスト側で上書き可能） |

### LlmProviderConfig

タグ付きenum。`provider` フィールドで切り替えます。

**OpenAI**

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `api_key` | `String` | Yes | OpenAI APIキー |
| `model` | `String` | Yes | モデル名（例: `gpt-4o-mini`） |
| `base_url` | `Option<String>` | No | カスタムエンドポイント（互換API向け） |

**Anthropic**

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `api_key` | `String` | Yes | Anthropic APIキー |
| `model` | `String` | Yes | モデル名（例: `claude-sonnet-4-20250514`） |
| `max_tokens` | `Option<u32>` | No | 最大トークン数（デフォルト: 1024） |
| `base_url` | `Option<String>` | No | カスタムエンドポイント（互換API向け） |

### VoiceVoxConfig

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `base_url` | `String` | Yes | VOICEVOX EngineのURL（例: `http://localhost:50021`） |
| `speaker` | `u32` | Yes | 話者ID |

### SynthesisRequest

| フィールド | 型 | 必須 | 説明 |
|---|---|---|---|
| `input` | `String` | Yes | LLMに送るテキスト入力 |
| `system_prompt` | `Option<String>` | No | LLMへのシステムプロンプト（設定のデフォルトを上書き） |

### SynthesisResult

| フィールド | 型 | 説明 |
|---|---|---|
| `text` | `String` | LLMの応答テキスト（正規化前） |
| `audio_bytes` | `Vec<u8>` | VOICEVOX が生成したWAV音声データ |

### StreamChunk

ストリーミングモードで返されるチャンク。

| フィールド | 型 | 説明 |
|---|---|---|
| `text` | `String` | LLMの生テキスト |
| `normalized_text` | `String` | TTS向けに正規化されたテキスト |
| `audio_bytes` | `Vec<u8>` | VOICEVOX が生成したWAV音声データ |

## エラーハンドリング

`llm_voice_bridge::Error` enumで構造化されたエラーを返します。

| バリアント | 説明 | リトライ対象 |
|---|---|---|
| `InvalidConfig` | 設定値が不正（APIキーやURLが空など） | No |
| `LlmAuthError` | LLM API認証失敗（HTTP 401） | No |
| `LlmRateLimited` | LLM APIレート制限（HTTP 429） | Yes |
| `LlmApiError` | その他のLLM APIエラー（5xxはリトライ対象） | 5xxのみ |
| `LlmEmptyResponse` | LLMが空の応答を返した | No |
| `VoiceVoxUnreachable` | VOICEVOX Engineに接続できない | Yes |
| `VoiceVoxApiError` | VOICEVOX APIエラー | No |
| `HttpError` | HTTP通信エラー | Yes |
| `JsonError` | JSONシリアライズ/デシリアライズエラー | No |
| `IoError` | I/Oエラー | No |

リトライは指数バックオフ（500ms × 2^回数、上限30秒）で行われます。

## 前提条件

- [VOICEVOX Engine](https://github.com/VOICEVOX/voicevox_engine) が起動していること
- OpenAI または Anthropic の APIキーを取得済みであること

## ビルド・テスト

```bash
cargo build              # ライブラリビルド
cargo test               # 全テスト実行（ユニット + 統合）
cargo build --examples   # サンプルビルド
cargo fmt -- --check     # フォーマットチェック
```

## ライセンス

[MIT](LICENSE)
