---
name: impl-flow
description: Manage the development workflow for creating and revising specifications, fixing bugs, adding tests, and implementing features. This skill does not involve actual coding.
---

# Important

Development must always be documentation-driven.
Implementation must not proceed without first revising the documentation.
Under no circumstances is implementation permitted without first revising the documentation.

You must clearly indicate which step has been completed, specify the status of each step, and always state which step will be performed next.

You AI guys always try to jump straight to Step 4, but that won’t fly.

# Status update rule

"Update status" means APPEND, not overwrite.

Structure:

```
**状態:** <one of: 未着手 / 修正中 / [WIP] / 完了承認待ち / 修正済み>
**進捗:**
- <stage> done (<commit hash>)
- <stage> done (<commit hash>)
```

- Never overwrite an existing 進捗 line. Add one new line per step.
- Never invent new 状態 labels (no "spec 更新済み" etc.). Use only the listed ones.
- Don't stop after the status commit. Continue the step's real work.

# Step 1: Record tasks and bugs

**Reread the description of the impl-flow skill.**

Always record reported bugs and requests first.

Do not proceed to the next task without first making a record.

Git commit: Message `[Task/Bug update/add/fix] descritpion`.

# Step 2: Update Spec

**Reread the description of the impl-flow skill.**

Consider the changes to be made to the documentation and seek the user’s approval.

Do not change multiple specifications at once. Users cannot make informed decisions when presented with too many changes at once.

Additionally, summarize the key points clearly and concisely.

Git commit: Message `[Spec update/add/fix] descritpion`.

After that, you MUST update task/bug status, and commit. `[Task/Bug update spec] description`

# Step 3: Update test document

**Reread the description of the impl-flow skill.**

Require subagents to use the `test-planner` skill to devise test cases that define test requirements for the modified features

- Clearly specify which features in the specifications the tests should cover.
- Do not provide any other unnecessary instructions.

Git commit: Message `[TestSpec update/add/fix] descritpion`.

After that, you MUST update task/bug status, and commit. `[Task/Bug update test-doc] description`

# Step 4: Implements test and code

**Reread the description of the impl-flow skill.**

- Rust: Subagent/ skill `rust-coding`
- Web: Subagent/ skill `web-coding`

After that, you MUST update task/bug status, and commit. `[Task/Bug update impl] description`

# Step 5: Review code and fix loop

**Reread the description of the impl-flow skill.**

You may perform the review and correction loop up to three times.

Tell the Review agent which range to review and which file to output the review results to.
All the Review Agent needs is “the range of files to review, the file to output the review results, and the iteration count (1st / 2nd / 3rd review on the same task)”; do not make any other unnecessary requests or give it any other commands. The skill uses the iteration count to relax the criteria for low-value findings during follow-up reviews.

Tell the Fix agent which file contains the review results.

As the controller, it is your responsibility to filter out any comments that contradict what was said previously.

Do not consider skipping review for any reason.
- The fact that the changes are minor is not a valid reason to skip the review.
- The fact that the scope of impact is minor is not a valid reason to skip the review.

Skipping the review or proposing to skip the review is strictly prohibited.


- Rust
  - review: Subagent/ skill `rust-review`
  - fix code: Agent: `rust-review-fix`
    - **DON'T use `rust-coding` skill**
    - **DON'T use `rust-coder` agent**
- Web
  - review: Subagent/ skill `web-review`
  - fix code: Agent: `web-review-fix`
    - **DON'T use `web-coding` skill**
    - **DON'T use `web-coder` agent**

After done loop, you MUST update task/bug status, and commit. `[Task/Bug update review-fix-loop] description`

# Step 7: Run test

**Reread the description of the impl-flow skill.**

Verify that all of the following are executed successfully

1. cargo test
2. cargo check
3. scripts/regen-samples.sh
4. python3 help/build.py
5. build tchart-editor

Update the bug and task state `完了承認待ち`.

Git commit: Message `[Task/Bug update 完了承認待ち] descritpion`.

# Step 9: Completion Report

If there are still implementation tasks to complete, return to Step 2 and resume the process.

