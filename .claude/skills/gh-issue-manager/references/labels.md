# Issue Label Definitions

## Label Setup Command

Run once per repository to initialize all labels:

```bash
# Type labels
gh label create "type:feature" --description "新機能" --color "0E8A16" --force
gh label create "type:bug" --description "バグ" --color "D93F0B" --force
gh label create "type:docs" --description "ドキュメント" --color "0075CA" --force
gh label create "type:chore" --description "雑務・メンテナンス" --color "BFDADC" --force
gh label create "type:refactor" --description "リファクタリング" --color "D4C5F9" --force

# Priority labels
gh label create "priority:high" --description "優先度高" --color "B60205" --force
gh label create "priority:medium" --description "優先度中" --color "FBCA04" --force
gh label create "priority:low" --description "優先度低" --color "C2E0C6" --force

# Scope labels
gh label create "scope:frontend" --description "フロントエンド" --color "1D76DB" --force
gh label create "scope:backend" --description "バックエンド" --color "5319E7" --force
gh label create "scope:infra" --description "インフラ" --color "F9D0C4" --force

# Status labels
gh label create "needs-triage" --description "トリアージ待ち" --color "E4E669" --force
gh label create "blocked" --description "ブロック中" --color "B60205" --force
gh label create "security" --description "セキュリティ関連" --color "D93F0B" --force

# AI workflow labels
gh label create "ai-ready" --description "AIが着手可能" --color "0E8A16" --force
gh label create "ai-in-progress" --description "AIが作業中" --color "FBCA04" --force
gh label create "ai-required-review" --description "AIの作業完了・レビュー待ち" --color "1D76DB" --force
```

## Label Categories

### Type (`type:*`)
Issue の種類を分類する。1つのIssueに1つだけ付与。

| Label | Description |
|---|---|
| `type:feature` | 新機能 |
| `type:bug` | バグ |
| `type:docs` | ドキュメント |
| `type:chore` | 雑務・メンテナンス |
| `type:refactor` | リファクタリング |

### Priority (`priority:*`)
優先度。1つのIssueに1つだけ付与。

| Label | Description |
|---|---|
| `priority:high` | 優先度高 |
| `priority:medium` | 優先度中 |
| `priority:low` | 優先度低 |

### Scope (`scope:*`)
影響範囲。複数付与可。

| Label | Description |
|---|---|
| `scope:frontend` | フロントエンド |
| `scope:backend` | バックエンド |
| `scope:infra` | インフラ |

### Status
作業状態を示す。

| Label | Description |
|---|---|
| `needs-triage` | トリアージ待ち |
| `blocked` | ブロック中 |
| `security` | セキュリティ関連 |

### AI Workflow Labels
AI エージェント間の作業バッティングを防ぐためのステータスラベル。

| Label | Description | State |
|---|---|---|
| `ai-ready` | AIが着手可能 | 未着手・着手可 |
| `ai-in-progress` | AIが作業中 | 作業中(他AIは着手禁止) |
| `ai-required-review` | AIの作業完了・レビュー待ち | 人間のレビュー待ち |

## AI Label State Machine

```
[ai-ready] → AI picks up → [ai-in-progress] → AI completes & opens PR → [ai-required-review]
                                    ↓ (failure/abort)
                              [ai-ready] (rollback)
```
