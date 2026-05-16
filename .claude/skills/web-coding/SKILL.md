---
name: web-coding
description: TypeScript coding. Use this skill to change tchart-editor folder.
---

# Coding Workflow

## Coding rules

see @docs/coding/ts.md

## Step 1: Implement

1. Implement the tests first. Do not implement the functionality first.
2. Verify that the tests fail.
3. Git commit (Commit message: `[TEST modify/add/impl] description`)
    - DON'T include task IDs or bug IDs in commit messages
4. Implement the functionality. Don't change test code.
5. Continue implementing until the tests pass. (`pnpm test`) * DON'T Change test.
6. `pnpm fmt`
7. Run `pnpm check` and `pnpm lint` to verify that there are no errors.
8. Git commit (Commit message: `[SRC impl] description`)
    - DON'T include task IDs or bug IDs in commit messages
## Spec Change Required

If implementation reveals that the spec must change (e.g., an interface is wrong, a constraint is impossible):

1. Insert `throw new Error("TODO: reason")` at the affected location so the code still compiles.
2. Stop immediately — do not continue implementing.
3. Clearly tell the user what spec change is needed and why.

Do not work around spec issues silently. Do not proceed past a `throw new Error("TODO: reason")` without user approval.
