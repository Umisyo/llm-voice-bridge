# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-03-13

### Added

- ストリーミング合成モード `Pipeline::run_stream()` — LLM応答をチャンク単位で逐次音声合成
- 一時的なエラー（5xx, 429）に対する自動リトライ機構（指数バックオフ）
- `PipelineConfig` にデフォルトシステムプロンプト設定 (`system_prompt`) を追加
- `tracing` クレートによるロギング/トレーシング対応

### Changed

- README を現状の機能に合わせて最新化
- インストール手順を crates.io 経由に更新

## [0.1.2] - 2026-03-13

### Fixed

- `cargo publish` 時の Cargo.lock dirty エラーを修正（git管理から除外）

## [0.1.1] - 2026-03-13

### Changed

- Cargo.toml にcrates.io公開用メタデータ追加（repository, keywords, categories, readme, rust-version）
- クレートレベルのドキュメント追加
- CHANGELOG.md 追加
- リリース自動化ワークフロー追加

## [0.1.0] - 2026-03-13

### Added

- `Pipeline::new()` / `Pipeline::run()` による LLM → TTS パイプライン
- OpenAI Chat Completions プロバイダー（`base_url` によるカスタムエンドポイント対応）
- Anthropic Messages API プロバイダー
- VOICEVOX Engine 連携（audio_query + synthesis）
- `normalize_for_tts()` — Markdown記法の除去・テキスト正規化
  - 見出し、太字、斜体、インラインコード、コードブロック
  - URL除去、Markdownリンク変換、テーブル変換
  - 箇条書き（順序付き・順序なし）の変換
- APIキーのDebug/Serialize時マスキング
- 構造化エラー型 (`Error` enum)
- HTTP タイムアウト設定（`timeout_secs`）

[Unreleased]: https://github.com/Umisyo/llm-voice-bridge/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Umisyo/llm-voice-bridge/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/Umisyo/llm-voice-bridge/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/Umisyo/llm-voice-bridge/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/Umisyo/llm-voice-bridge/releases/tag/v0.1.0
