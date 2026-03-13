---
name: dev-workflow
description: Issue-driven development workflow orchestrator. Use when starting work on a GitHub Issue, implementing a feature or bugfix tied to an issue, or when the user says "start working on issue", "implement issue #N", "pick up an issue", "/dev-workflow". Covers the full cycle from issue pickup through branch/worktree setup, implementation, ADR creation, code review via codex CLI, and PR creation.
---

# Dev Workflow

Issue-driven development workflow. All implementation is tied to a GitHub Issue and follows a structured pipeline from issue pickup to PR creation.

## Workflow

### Phase 1: Issue Selection

Use `/gh-issue-manager` to fetch and select the target issue. Confirm the issue with the user before proceeding.

### Phase 2: Branch & Worktree Setup

```
git fetch origin
git switch main && git pull origin main
```

Create a branch named `<type>/issue-<number>-<short-description>` from the latest `origin/main`. Type is one of: `feat`, `fix`, `refactor`, `docs`, `chore`.

Then create a worktree and enter it:

```
git worktree add ../<branch-name> -b <branch-name> origin/main
cd ../<branch-name>
```

All subsequent work happens inside this worktree.

### Phase 3: Complexity Assessment

Assess issue complexity before implementation:

| Signal | Low | High |
|--------|-----|------|
| Files touched | 1-3 | 4+ |
| Cross-cutting concerns | No | Yes |
| New abstractions needed | No | Yes |
| Unclear spec areas | None | Multiple |

- **Low complexity** → implement directly in the main conversation
- **High complexity** → use Agent tool with multiple parallel agents (agent-team pattern): split work into independent sub-tasks, launch agents concurrently, integrate results

If any spec is unclear, use `AskUserQuestion` to clarify before writing code.

### Phase 4: Implementation

Implement the changes according to the issue requirements.

**During implementation, continuously evaluate whether an ADR is warranted.** See [ADR Guidelines](#adr-guidelines) below.

### Phase 5: Code Review via Codex CLI

After implementation is complete, run codex CLI for review:

```bash
codex --approval-mode suggest "Review the changes in this worktree. Focus on: correctness, edge cases, security, and maintainability. Suggest improvements."
```

Address any issues raised by the review before proceeding.

### Phase 6: Commit, Push & PR

Use `/commit-push-pr` to:

1. Commit all changes with a descriptive message
2. Push the branch
3. Create a PR linked to the issue (include `Closes #<issue-number>` in the PR body)

### Phase 7: Cleanup

After PR is created, clean up the worktree:

```bash
cd <original-repo-path>
git worktree remove ../<branch-name>
```

## ADR Guidelines

Create an ADR when the implementation meets ANY of these criteria:

- Introduces a new architectural pattern or abstraction
- Makes a non-obvious technical choice between viable alternatives
- Changes data flow, state management, or system boundaries
- Would take a new maintainer (human or LLM) >30 minutes to understand the "why"

### ADR Placement

Place ADRs under `docs/adr/` mirroring the source directory structure:

```
src/auth/oauth.ts       → docs/adr/src/auth/NNNN-oauth-provider-selection.md
lib/cache/strategy.ts   → docs/adr/lib/cache/NNNN-cache-invalidation-strategy.md
```

`NNNN` is a zero-padded sequential number. Check existing ADRs to determine the next number.

### ADR Format

Follow the Cognitect/Nygard format:

```markdown
# <NNNN>. <Title>

Date: <YYYY-MM-DD>

## Status

Proposed | Accepted | Deprecated | Superseded by [NNNN](link)

## Context

What is the issue that we're seeing that is motivating this decision or change?

## Decision

What is the change that we're proposing and/or doing?

## Consequences

What becomes easier or more difficult to do because of this change?
```

Keep each section concise. The goal is to capture the *why*, not to document the code itself.
