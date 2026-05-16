---
name: web-fix-review
description: Use when modifying TypeScript code after review.
---

# Coding Workflow

## Coding rules

see @docs/coding/ts.md

## Step 1: Fix

1. Carefully review the feedback you received from the reviewer.
2. Make revisions as appropriate.
3. Continue implementing until the tests pass. (`pnpm test`) * DON'T Change test.
4. `pnpm fmt`
5. Run `pnpm check` and `pnpm lint` to verify that there are no errors.
6. Git commit: (Commit Message: `[fix review] description`)
    - DON'T include task IDs or bug IDs in commit messages

## Spec Change Required

If implementation reveals that the spec must change (e.g., an interface is wrong, a constraint is impossible):

1. Insert `throw new Error("TODO: reason")` at the affected location so the code still compiles.
2. Stop immediately — do not continue implementing.
3. Clearly tell the user what spec change is needed and why.

Do not work around spec issues silently. Do not proceed past a `throw new Error("TODO: reason")` without user approval.
