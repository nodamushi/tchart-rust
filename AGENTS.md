# General Rules

- *Don't read source code you don't need.*

- **Documentation comes first — always, without exception. Before touching any code or file, check and update the relevant documents (spec, bugs, tasks, etc.). Never start work first and write docs later.**

- You must talk in Japanese.
- tmp/user_message.md is a user message to AI. Since it is cumbersome for users to write commands in the CLI, they should list their requests and requirements in this file.Delete the contents when the message is no longer needed.
- tmp/tmp_*.md are memo. DON'T see these file. DON'T edit these file.
- Do not write Japanese in source code.
  - Exception: Documents should be written in Japanese. Writing Japanese in code is prohibited.

# No Local IDs (ABSOLUTE)

`docs/bugs.md`, `docs/bugs/`, `docs/tasks.md`, `docs/tasks/` and review scratch files own local IDs that are scheduled for deletion. These IDs MUST NOT appear in source, public docs, commit messages, or PR / issue bodies. Only GitHub / GitLab `#123` is allowed.

## ID prefix convention

Local files MUST assign IDs using these prefixes (so the detection grep catches every leak):

- bugs → `tcml-bug-<n>` (e.g. `tcml-bug-001`)
- tasks → `tcml-task-<n>`
- audits → `tcml-audit-<n>`
- reviews → `tcml-review-<n>`

`tcml-` is project-unique; it will not collide with normal English / 日本語 prose. Old short prefixes (`BUG-`, `T-N`, `A-N`, `R-N`) are forbidden everywhere, including local files.

## Detection

Hook `scripts/precommit-check.sh` runs on every `git commit` and blocks on any hit. Manual run:

```
grep -rnE "(\btcml-(bug|task|audit|review)-[0-9]+|BUG-|\bT-[0-9]+\b|\bA-[0-9]+\b|\bR-[0-9]+\b)" \
    docs/spec/ docs/tests/ docs/spec.md docs/tests.md README.md help/ \
    tchart-core/src/ tchart-cli/src/ tchart-web/src/ tchart-editor/src/
```

The regex covers both the new prefix (`tcml-*`) leaking into public files and the legacy short prefixes that are still being purged.

`docs/bugs.md`, `docs/bugs/`, `docs/tasks.md`, `docs/tasks/` are the authoritative homes and are excluded from the search. `docs/bugs.md` is a small index; each individual bug detail lives in `docs/bugs/<slug>.md` (split since v0.1.1 for size reasons — the original single-file form held ~2000 lines). Sentences in public files that depend on a past bug number become meaningless when the local file is deleted — write design rationale in the present tense. Fix every match in one cycle.

# Spec files (docs/spec/*.md, docs/tests/*.feature.md, docs/coding/*.md)

- **NEVER modify any spec, test, or coding doc without first explaining to the user exactly what you intend to change and why, and receiving explicit approval.**
- This applies even when a discrepancy between spec and implementation is found. Report the finding, propose the fix, wait for "yes", then edit.
- No exceptions for "obvious fixes", "typos", "just updating to match reality", or any other reason.

# Sample files (docs/images/*.tc)

- **NEVER reduce the content of `docs/images/*.tc` files for ANY reason without explicit user approval.** This includes deleting lines, removing test cases, simplifying patterns, or any change that decreases the test surface. "Spec compliance", "parser rejects it", "the new implementation does not support this", or any other technical reason is NOT a valid excuse to bypass user approval.
- Adding new lines or new sample files is allowed without prior approval, but reducing existing content always requires confirmation.
- If a sample file fails the current parser/spec, leave it as-is and report it as a BUG. Do not "fix" it by editing the sample.

# Coding

- Follow documentation-driven development. Always create and update documents first
  - Don't implemente or modification without create/modify documents.
  - It is prohibited to request user permission for implementation without first creating or modifying the documentation.
  - See `/doc-spec`, `/doc-tests`, `/doc-tasks`, `/doc-readme` skills for each document format.
- If implementation cannot satisfy the spec or test plan, ask the user for permission before proceeding.
- Always implement tests first. Start coding only after tests fail.
- When fixing bugs, fix tests first and start from a failing state.

## Document priority order (ABSOLUTE)

- When a higher-priority document conflicts with a lower one, the lower must be changed to match the higher. NEVER the other way around.
- Implementation MUST NOT contradict the spec. If contradiction is found, fix the implementation, not the spec.
- "The implementation already does X, so let's update the spec to match" is a VIOLATION. Spec change requires explicit user approval, always.

## Authoring order rules (HARD CONSTRAINTS — NO EXCEPTIONS)

These rules govern when files may be created or modified. Git commit timing is a SECONDARY enforcement and does NOT replace these rules. "Write implementation first, then write spec before committing them together" is a VIOLATION even if the final commit order looks clean.

1. **No implementation file may be edited without a corresponding spec already in place.**
   - Before touching any file under `tchart-core/src/`, `tchart-cli/src/`, `tchart-web/src/`, or `tchart-editor/src/`, the relevant `docs/spec/*.md` content MUST already exist (in working tree or committed) and accurately describe what the implementation will do. If the spec is silent or contradicts the intended change, STOP and ask the user.
   - "I'll write the spec after I see what works" is a VIOLATION.
   - "I'll prototype quickly then write the spec" is a VIOLATION.
   - "The spec is in my head and I'll write it down before committing" is a VIOLATION.

2. **No implementation file may be edited without corresponding test scenarios already in place.**
   - Before writing implementation code, the relevant `docs/tests/*.feature.md` MUST already contain the scenarios that the new code will satisfy. The Rust test code (`#[test]`) MUST be written and FAILING before the implementation is written.
   - "I'll add the test cases after the impl works" is a VIOLATION.

3. **Spec changes require explicit user approval — at the moment of change, not at commit time.**
   - You may NOT edit `docs/spec/*.md` based on what you discovered while implementing. The implementation does not authorize spec changes.
   - To change a spec: (a) describe the proposed change to the user, (b) wait for explicit approval, (c) edit the spec, (d) then update implementation if needed.
   - "I'll show the user the spec diff at commit time" is a VIOLATION — the spec was already edited by then.

4. **Mid-implementation spec gaps halt work.**
   - If you start implementing and discover the spec is ambiguous, incomplete, or contradicts what's needed, STOP IMMEDIATELY. Do not "fill in the gap" by writing code that interprets the spec one way. Report the gap to the user and wait.

5. **Self-report on violation.**
   - If you realize you've already violated rules 1–4 mid-task (e.g., you wrote impl code before the spec covered it), STOP, do NOT commit, and report to the user with the file paths involved. Do not try to "catch up" silently by writing the spec retroactively.

# Git

- **NEVER commit directly to `develop` or `main`.** All work commits go on a branch derived from `local`. Before staging any change, verify `git rev-parse --abbrev-ref HEAD` is NOT `develop` and NOT `main`. If you are on `develop` or `main`, switch to a branch derived from `local` first. `develop` and `main` are produced solely by the `branch-sync` skill flow.
- Be sure to commit to Git before launching the rust-coder agent. Always commit, no matter what.
   - If your code isn't working, ignore the following command, and commit with message like "WIP: Explanation".
- Commit at appropriate intervals with appropriate granularity.
  - Always run `cargo fmt` and `cargo check` before committing.
  - Always run `pnpm fmt` and `pnpm check` before committing in tchart-editor.
  - Always check secirity. (`cargo audit`, `pnpm audito` in tchart-editor)
  - Always check `docs/tasks.md`, `docs/tests.md` and `README.md` before committing.
  - Help regeneration. If a commit stages any of `tchart-core/src/**`, `tchart-cli/src/**`, `docs/spec/tcml-format.md`, or `help/source.py`, then `python3 help/build.py` must be run and `help/output/tcml-format.html` staged in the same commit. Additional pairing rule: a staged `docs/spec/tcml-format.md` requires `help/source.py` to be staged in the same commit (TCML syntax changes mean the help template must also be updated). Hook `scripts/precommit-check.sh` enforces this on `git commit` invoked through Claude. WIP commits (`-m "WIP: ..."`) skip the regen check (the program may not be runnable in WIP states).

# Important

- *Don't read source code you don't need.*
    - Before reading any source code, always tell yourself: “I almost certainly don’t need to read this code. Do I absolutely, 100% have to read it? I believe I don’t need to, but do I still have to? I definitely don’t need to read every single file.”

# How to Call a Skill

All the Review Agent needs is “the range of files to review, the file to output the review results, and the iteration count (1st / 2nd / 3rd review on the same task)”; do not make any other unnecessary requests or give it any other commands. The skill consults the iteration count to relax low-value findings on follow-up reviews.

Coding Do not impose unnecessary rules when calling the agent (such as termination conditions or review-related requirements). The skill handles everything. Simply communicate the purpose, scope, and the fact that the sub-agent is a sub-agent (not a parent, and cannot create its own sub-agents). Do not let the parent meddle by assigning unnecessary tasks or issuing commands.

Do not run the review and revision loop more than about three times. Get confirmation from the user.

Don’t let coding agents edit the specifications.

# If you modify tchart-cli or tchart-core

Be sure to run `python3 help/build.py` and `./scripts/regen-samples.sh` to update the sample images and help documentation.

# DON'T use memory

Every time the container starts up, your memory will be deleted. Memory is not persisted. Do not use memory.

Write it in the documentation, specifications, files in `docs/coding`, and your skills.

