# CLAUDE.md

## プロジェクト概要

llm-voice-bridge — テキスト入力 → LLM(OpenAI/Anthropic) → テキスト正規化 → VOICEVOX → WAV音声 を一気通貫で行う軽量Rustライブラリ。

## 技術スタック

- Rust (edition 2021)
- async runtime: tokio
- HTTP: reqwest
- エラー: thiserror
- テスト: wiremock

## ビルド・テスト

```bash
cargo build              # ライブラリビルド
cargo test               # 全テスト実行 (ユニット + 統合)
cargo build --examples   # サンプルビルド
cargo fmt -- --check     # フォーマットチェック
```

**push前に必ず `cargo fmt -- --check` を実行し、差分があれば `cargo fmt` で修正してからpushすること。** CIで `cargo fmt -- --check` が走るため、フォーマット未適用だとCIが落ちる。

## ディレクトリ構成

```
src/
├── lib.rs           # クレートルート (pub use 再エクスポート)
├── config.rs        # PipelineConfig, LlmProviderConfig, VoiceVoxConfig
├── error.rs         # Error enum
├── types.rs         # SynthesisRequest, SynthesisResult
├── normalize.rs     # normalize_for_tts() テキスト正規化
├── pipeline.rs      # Pipeline::new() / run()
└── provider/
    ├── mod.rs       # LlmClient / TtsClient trait (pub(crate))
    ├── openai.rs    # OpenAI Chat Completions
    ├── anthropic.rs # Anthropic Messages API
    └── voicevox.rs  # VOICEVOX audio_query + synthesis
```

## 開発ワークフロー

**実装作業は必ず Issue 駆動で行うこと。**

1. `/gh-issue-manager` でIssueを作成・選択する
2. `/dev-workflow` でIssue起点の実装フローを実行する（ブランチ作成 → 実装 → レビュー → PR）

Issue を先に作成してから実装に入る。実装してからIssueを作るのは禁止。

## コーディング規約

- Provider固有の型は非公開 (`pub(crate)` or private)
- `LlmClient` / `TtsClient` trait は `pub(crate)` — 利用者は `Pipeline` 経由でのみ使う
- `normalize_for_tts` は公開関数
- テストは `tests/` に統合テスト、各モジュール内に `#[cfg(test)]` でユニットテスト
