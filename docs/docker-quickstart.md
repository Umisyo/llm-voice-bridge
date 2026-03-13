# Docker Quickstart

Docker Compose を使って VOICEVOX Engine を起動し、llm-voice-bridge を最低構成で動かす手順です。

## 前提条件

- [Docker](https://docs.docker.com/get-docker/) および Docker Compose がインストール済み
- Rust ツールチェイン（1.75 以上）がインストール済み
- OpenAI または Anthropic の API キーを取得済み

## 手順

### 1. 環境変数の設定

`.env.example` をコピーして API キーを設定します。

```bash
cp .env.example .env
```

`.env` を編集し、使用するプロバイダーの API キーを入力してください。

```bash
# OpenAI を使う場合
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxx

# Anthropic を使う場合
ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxxxxx
```

### 2. VOICEVOX Engine の起動

```bash
docker compose up -d
```

VOICEVOX Engine が `http://localhost:50021` で起動します。

起動確認:

```bash
curl http://localhost:50021/version
```

バージョン文字列が返れば OK です。

### 3. サンプルの実行

```bash
# .env を読み込んで実行
source .env
cargo run --example basic
```

`output.wav` が生成されれば成功です。

### 4. 停止

```bash
docker compose down
```

## GPU 版を使う場合

NVIDIA GPU を利用できる環境では、イメージタグを変更すると高速に音声合成できます。

```yaml
# docker-compose.yml
services:
  voicevox:
    image: voicevox/voicevox_engine:nvidia-ubuntu20.04-latest
    ports:
      - "50021:50021"
    deploy:
      resources:
        reservations:
          devices:
            - driver: nvidia
              count: 1
              capabilities: [gpu]
```

## トラブルシューティング

| 症状 | 対処 |
|------|------|
| `VoiceVoxUnreachable` エラー | `docker compose ps` で VOICEVOX コンテナが起動しているか確認 |
| コンテナが起動直後に落ちる | `docker compose logs voicevox` でログを確認。メモリ不足の可能性あり |
| 音声生成が遅い | CPU 版は初回推論に時間がかかります。GPU 版の利用を検討してください |
