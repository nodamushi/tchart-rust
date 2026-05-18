---
name: branch-sync
description: Sync local→develop or develop→main.
---

# Branch Sync

- First, always ask in Japanese: 「進めてよろしいですか?」
- Do nothing until the user replies Yes (「はい」 etc.). Yes is required every time.
- All user-facing messages (questions, plans, status reports) must be written in Japanese.
- Forbidden: push / reset / force / `--no-verify` / direct commits to `main`.
- `develop` and `main` must always be green. On failure, stop and report.

## Commit message (develop / main)

Format: `[<prefix> #N]`. No issue: `[<prefix>]`. Multi issue: `[<prefix> #1 #2]`. Multi prefix: `[a,b #1]`. Always include `#N` for GitHub-issue-driven commits.

Prefixes:

- `doc/spec` — `docs/spec/*.md`
- `doc/test` — `docs/tests/*.feature.md`
- `doc/sample` — `docs/images/*.tc` + regenerated svg/png
- `doc/help` — `help/source.py` + `help/output/*`
- `code` — product source (non-test under `tchart-*/src/`)
- `test` — test code (`tchart-*/tests/`, `*/tests.rs`, `tchart-editor/src/__tests__/`)
- `claude` — `.claude/`, `AGENTS.md`, `scripts/`, `help/build.py`
- `version` — version bump only (`Cargo.toml`, `Cargo.lock`, `tchart-editor/package.json`)

## develop / main rules

- When the prefix list contains `code`, the following must pass before commit (enforced by `scripts/precommit-check.sh` Check 6):
  - `cargo test --workspace`
  - `pnpm --dir tchart-editor test`
- Other prefixes (`test`, `doc/*`, `claude`, `version`) skip the test-pass requirement:
  - `test` alone: TDD intermediate; tests are expected to fail before the paired `code` commit.
  - `doc/*`, `claude`, `version`: no product-source change.
- If the commit message does not parse as a known `[<prefix>...]`, default to running both test suites (safe fallback).
- Never include local-only files: `docs/bugs.md`, `docs/bugs/`, `docs/tasks.md`, `docs/tasks/`.

## local → develop

- Reorganize `git log develop..local` into clean commits on `develop`.
- Exclude `docs/tasks.md`, `docs/tasks/`, `docs/bugs.md`, `docs/bugs/`.
- Split: spec / tests / impl / refactor.
- Generated files go with their source change.
- Present plan, wait for approval, commit one at a time.
- Before each commit: fmt / check, audit if deps added.
- Then regrow `local`:
```
  git branch -m local local-old
  git checkout -b local develop
  git checkout local-old -- docs/tasks.md docs/tasks docs/bugs.md docs/bugs
  git commit -m "restore task files"
  git branch -d local-old   # only after approval
```
- **End-of-sync: HEAD MUST end on `local`.** The regrow step above already leaves you on the recreated `local`; do not switch away before reporting completion.

## develop → main

- Pre-check: if `Cargo.toml` and `package.json` versions disagree, abort. Align major on `local` (do not commit).
- Pre-check: if current version matches the latest tag `vX.Y.Z`, ask user whether to proceed without bumping. If declined, abort and bump patch on `local` (do not commit).
- Steps: `git checkout main` → `git merge --no-ff develop`.
- On conflict, do not touch it; report.
- **End-of-sync: HEAD MUST end on `local`.** After the merge (and any conflict resolution), run `git checkout local` before reporting completion.
