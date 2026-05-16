---
name: doc-tasks
description: Format and structure for docs/tasks.md. Use when creating or updating the implementation task list.
---

# `docs/tasks.md` Format

**Before starting any implementation**, list all tasks here first. Adding implementation without a corresponding task entry is prohibited, no matter how minor.

When a task needs an explicit ID, use the prefix `tcml-task-<n>`. The prefix and rationale are in AGENTS.md `# No Local IDs` — follow it; do not introduce any other prefix.

Use the value of `n` from section n, and increment the value of n in section n by 1.

## Section Structure

```markdown
# タスク

## n

number

## 重要

- 各タスクは impl-flow スキルに従って処理すること
- 各タスクに着手したら、必ず最初に[WIP]に変え、タスクファイル単体でGit Commit すること
    - メッセージは [Start Task] タスク名
    - メッセージに ID を含めることを禁ずる
- 各タスクは終了したら削除しタスクファイル単体でコミットすること
    - メッセージは [Done Task] タスク名
    - メッセージに ID を含めることを禁ずる

## tcml-task-<n> タスク名 [WIP/停止/未着手/完了承認待ち]

説明

```
