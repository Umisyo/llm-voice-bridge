---
name: gh-issue-manager
description: >
  Autonomous GitHub Issue management using gh CLI. Use when:
  (1) Creating GitHub Issues with proper labels and structure,
  (2) Fetching and selecting Issues to work on,
  (3) Picking up an Issue and implementing it end-to-end,
  (4) Creating PRs linked to Issues after implementation,
  (5) Managing AI workflow labels (ai-ready, ai-in-progress, ai-required-review) to coordinate between AI agents.
  Triggers: "issue作成", "issue取得", "issueを実装", "issueに取り組む", "create issue", "pick up issue", "work on issue", "list issues", "setup labels".
---

# GitHub Issue Manager

Manage GitHub Issues autonomously via `gh` CLI with AI coordination labels.

## Label Reference

See [references/labels.md](references/labels.md) for full label definitions and setup commands.

## Workflows

### 1. Setup Labels

Initialize labels on a repository. Read `references/labels.md` and run the label creation commands for the target repo:

```bash
gh label create "<name>" --description "<desc>" --color "<hex>" --repo <owner/repo> --force
```

### 2. Create Issue

```bash
gh issue create --repo <owner/repo> \
  --title "<concise title>" \
  --body "$(cat <<'EOF'
## Overview
<1-2 sentence summary>

## Tasks
- [ ] <task 1>
- [ ] <task 2>

## Acceptance Criteria
- <criterion 1>
- <criterion 2>
EOF
)" \
  --label "type:<type>,priority:<priority>,ai-ready"
```

Rules:
- Always assign exactly one `type:*` and one `priority:*` label.
- Add `scope:*` labels as appropriate.
- Add `ai-ready` if the issue is suitable for AI implementation.
- Write the body in Japanese unless instructed otherwise.

### 3. Fetch Issues

List open issues available for AI:

```bash
gh issue list --repo <owner/repo> --label "ai-ready" --state open --json number,title,labels,body --limit 20
```

Filter by type or priority:

```bash
gh issue list --repo <owner/repo> --label "ai-ready,type:bug" --state open --json number,title,labels,body
```

### 4. Pick Up and Implement an Issue

This is the core autonomous workflow. Follow these steps strictly.

**Step 1: Claim the issue (prevent other AIs from taking it)**

```bash
gh issue edit <number> --repo <owner/repo> --remove-label "ai-ready" --add-label "ai-in-progress"
```

> CRITICAL: Always claim BEFORE starting work. If `ai-in-progress` is already on the issue, DO NOT proceed — another AI is working on it.

**Step 2: Read the issue details**

```bash
gh issue view <number> --repo <owner/repo> --json title,body,labels,comments
```

**Step 3: Create a feature branch**

```bash
git checkout -b <issue-number>-<short-description> main
```

Branch naming: `<issue-number>-<kebab-case-description>` (e.g., `42-add-user-auth`)

**Step 4: Implement**

- Analyze the codebase, understand architecture, then implement.
- Follow existing code conventions.
- Run tests and linters before finishing.

**Step 5: If implementation fails or is abandoned**

Roll back the label:

```bash
gh issue edit <number> --repo <owner/repo> --remove-label "ai-in-progress" --add-label "ai-ready"
```

Add a comment explaining why:

```bash
gh issue comment <number> --repo <owner/repo> --body "AI実装を中断しました。理由: <reason>"
```

### 5. Create PR Linked to Issue

After successful implementation:

```bash
git push -u origin HEAD
gh pr create --repo <owner/repo> \
  --title "<PR title>" \
  --body "$(cat <<'EOF'
## Summary
<what was done>

## Changes
- <change 1>
- <change 2>

## Test Plan
- [ ] <test 1>
- [ ] <test 2>

Closes #<issue-number>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Then update the issue label:

```bash
gh issue edit <number> --repo <owner/repo> --remove-label "ai-in-progress" --add-label "ai-required-review"
```

Rules:
- Always include `Closes #<number>` in PR body to auto-link and auto-close.
- Transition label from `ai-in-progress` → `ai-required-review`.
- Write PR body in Japanese unless instructed otherwise.

## AI Label State Machine

```
ai-ready → (claim) → ai-in-progress → (PR created) → ai-required-review
                            ↓ (abort)
                         ai-ready
```

- `ai-ready`: No AI is working on it. Safe to pick up.
- `ai-in-progress`: An AI is actively working. DO NOT touch.
- `ai-required-review`: AI completed work. Human review needed.

## Concurrency Rules

1. ALWAYS check for `ai-in-progress` before claiming an issue.
2. NEVER work on an issue that already has `ai-in-progress`.
3. ALWAYS claim (`ai-ready` → `ai-in-progress`) before starting implementation.
4. If you crash or abort, roll back to `ai-ready` with a comment.
5. One AI agent = one issue at a time.
