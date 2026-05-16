---
name: rust-fix-review
description: Use when modifying Rust code after review.
---

# Coding Workflow

## Coding rules

see @docs/coding/rust.md

## Step 1: Fix

1. Carefully review the feedback you received from the reviewer.
2. Make revisions as appropriate.
3. Continue implementing until the tests pass. (`cargo test`) * DON'T Change test.
4. cargo fmt
5. Run `cargo check` and `cargo clippy` to verify that there are no errors.
6. Git commit: (Commit Message: `[fix review] description`)
    - Don't write the task/bug ID in commit message

## Spec Change Required

If implementation reveals that the spec must change (e.g., an interface is wrong, a constraint is impossible):

1. Insert `todo!("reason")` at the affected location so the code still compiles.
2. Stop immediately — do not continue implementing.
3. Clearly tell the user what spec change is needed and why.

Do not work around spec issues silently. Do not proceed past a `todo!` without user approval.



