---
name: release
description: >
  llm-voice-bridge のバージョンリリース手順。
  Cargo.toml のバージョン更新、CHANGELOG.md の更新、git tag 作成、push によるリリースを一気通貫で行う。
  Triggers: "リリース", "release", "バージョンアップ", "publish", "crates.io に公開"
---

# Release Skill

llm-voice-bridge のバージョンリリースを対話的に実行する。

## 前提条件

- `main` ブランチにいること
- ワーキングツリーがクリーンであること（未コミットの変更がないこと）
- CI が通っていること

## リリースフロー

### Step 1: 事前チェック

現在の状態を確認する。

```bash
git status
git branch --show-current
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```

- ブランチが `main` でなければ警告し、ユーザーに確認する
- 未コミットの変更があれば中断する
- fmt/clippy/test に失敗したら中断する

### Step 2: バージョン決定

現在のバージョンを `Cargo.toml` から読み取り、ユーザーに表示する。

```
現在のバージョン: 0.1.2
次のバージョンを選んでください:
  1. patch (0.1.3)
  2. minor (0.2.0)
  3. major (1.0.0)
  4. カスタム指定
```

ユーザーが指定しない場合は **patch** をデフォルトとする。
ユーザーが引数でバージョンを指定している場合（例: `/release 0.2.0`）はそのまま使う。

### Step 3: CHANGELOG.md 更新

`CHANGELOG.md` を以下のルールで更新する。

1. `## [Unreleased]` セクションの内容を確認する
2. Unreleased が空の場合、`git log` から前回タグ以降のコミットを取得し、変更内容を自動生成してユーザーに提示する
3. `## [Unreleased]` の直下に新しいバージョンセクションを挿入する:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

4. ファイル末尾のリンク参照も更新する:
   - `[Unreleased]` リンクの比較先を新バージョンタグに変更
   - 新バージョンのリンクを追加

Keep a Changelog 形式を厳守すること。カテゴリは Added / Changed / Deprecated / Removed / Fixed / Security のみ使用する。

### Step 4: Cargo.toml 更新

`Cargo.toml` の `version` フィールドを新バージョンに更新する。

### Step 5: コミット & タグ作成

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "$(cat <<'EOF'
release: vX.Y.Z

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>
EOF
)"
git tag vX.Y.Z
```

### Step 6: push（ユーザー確認必須）

**必ずユーザーに確認してから push する。**

```
以下をpushします:
  - コミット: release: vX.Y.Z
  - タグ: vX.Y.Z

push すると GitHub Actions により以下が自動実行されます:
  - cargo test
  - crates.io への publish
  - GitHub Release の作成

続行しますか？ (y/n)
```

確認後:

```bash
git push origin main --follow-tags
```

### Step 7: リリース確認

push 後、GitHub Actions のステータスを確認する。

```bash
gh run list --limit 3
```

リリースのURLを表示して完了:

```
リリース完了！
  - GitHub Release: https://github.com/Umisyo/llm-voice-bridge/releases/tag/vX.Y.Z
  - crates.io: https://crates.io/crates/llm-voice-bridge/X.Y.Z
```

## エラーハンドリング

- **fmt/clippy/test 失敗**: 修正を促して中断
- **push 失敗**: コミットとタグはローカルに残っているため、問題解決後に再度 push を案内
- **CI 失敗（push 後）**: `gh run view` で詳細を確認し、必要に応じてタグ削除・コミット revert を案内

## 引数

- バージョン番号を直接指定可能（例: `/release 0.2.0`）
- `patch`, `minor`, `major` キーワードでも指定可能（例: `/release minor`）
