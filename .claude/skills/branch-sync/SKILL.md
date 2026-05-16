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

## local → develop

- Reorganize `git log develop..local` into clean commits on `develop`.
- Exclude `docs/tasks.md`, `docs/tasks`, `docs/bugs.md`.
- Split: spec / tests / impl / refactor.
- Generated files go with their source change.
- Present plan, wait for approval, commit one at a time.
- Before each commit: fmt / check, audit if deps added.
- Then regrow `local`:
```
  git branch -m local local-old
  git checkout -b local develop
  git checkout local-old -- docs/tasks.md docs/tasks docs/bugs.md
  git commit -m "restore task files"
  git branch -d local-old   # only after approval
```

## develop → main

- Pre-check: if `Cargo.toml` and `package.json` versions disagree, abort. Align major on `local` (do not commit).
- Pre-check: if current version matches the latest tag `vX.Y.Z`, ask user whether to proceed without bumping. If declined, abort and bump patch on `local` (do not commit).
- Steps: `git checkout main` → `git merge --no-ff develop`.
- On conflict, do not touch it; report.
