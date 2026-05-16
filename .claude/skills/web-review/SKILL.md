---
name: web-review
description: TypeScript code review skill.
---

# Review Workflow

Your primary objective is to evaluate the readability of the code and verify that it adheres to coding standards.

Review IDs use the prefix `tcml-review-<n>` (sequential within a review file). The prefix and rationale are in AGENTS.md `# No Local IDs` — follow it; do not introduce any other prefix.


You do not need to know the project’s objectives, nor do you need to understand the program’s algorithms. You do not need to know how the code works, nor should you try to understand it.

You do not need to assess the program’s validity.

Judge from a third-party perspective whether the code follows the rules and maintains readability.

It doesn’t matter if it works or if it follows the specs. Do not tolerate any messy code.


Whenever you find something, write it all to a file immediately; summarizing is not permitted.

If a destination file is specified, write the output there.
If no destination is specified, write the output to a file with a unique name in the ./tmp/review/ directory and include the path of the output file in the output.

Summarizing the results is not permitted. Always write everything to a file as soon as you find it.

Write all reviews to a file immediately. This rule must be followed regardless of any other requirements.

Do not summarize the reviews.

You must not point out the good points. There is absolutely no need for that.

## Output format

```
# Title

## File Name1

### tcml-review-<n> (Log/Low/Middle/High/Critical) line:Line-Number Title
Description

### tcml-review-<n> (Log/Low/Middle/High/Critical) line:Line-Number Title
Description

## File Name2

### tcml-review-<n> (Log/Low/Middle/High/Critical) line:Line-Number Title
Description

### tcml-review-<n> (Log/Low/Middle/High/Critical) line:Line-Number Title
Description
```

Do not output all lines at once.

Write the following content to the file in the order listed below.

```
### tcml-review-<n> (Log/Low/Middle/High/Critical) line:Line-Number Title
Description
```

## Step 1: Check by tools

- `pnpm fmt` (oxfmt)
- `pnpm check` (tsc no-emit)
- `pnpm lint` (oxlint)

You must not allow the use of `// oxlint-disable`, `// eslint-disable`, `@ts-ignore`, or `@ts-expect-error` without a written justification. Under no circumstances should you allow it. The implementation model tends to grant this permission arbitrarily and without authorization.

Prohibit this absolutely.

However, the following cases are permitted as exceptions:

- `@ts-expect-error` accompanied by a same-line written reason that explains the unavoidable external constraint (e.g. an upstream library type bug).
- `// oxlint-disable-next-line <rule>` with a same-line written reason that the user has approved.

No other exceptions are permitted. Things like a long chain of `if` statements are absolutely not permitted.

## Step 2: Read Coding rules

see @docs/coding/ts.md.

DON'T read any other documents. It is most important that you know nothing.

It doesn’t matter if it works or if it follows the specs. Do not tolerate any messy code.

Therefore, you must not be distracted by irrelevant information such as the specs or how other code is written.

## Step 3: Check git diff and Code review

Unless you are explicitly instructed to review the entire project, try to identify the scope of changes as narrowly as possible from the Git diff.

By “narrowly,” I mean you should generally identify changes at the level of individual functions. Don’t waste tokens by trying to analyze the entire file.

For newly created functions, use `rg -t ts "function "` (and `rg -t ts "=>"` for arrow function declarations) to retrieve a list of functions and check whether any similar functions already exist.

In addition to coding conventions, you must pay close attention to whether file names, module names, function names, class names, type names, variable names, and field names are appropriate; whether the structure of files and program is sound; and whether the flow of the code can be understood as a coherent narrative.

AI tends to take shortcuts and often places functions in contexts where they don’t belong. Always ask yourself: “If someone with no prior knowledge saw a function with this name placed here, what would they think?”—to determine whether it’s in the right place and whether it makes sense.

## Step 4: Verify Compliance with Specifications

After completing the review from a “blank slate” perspective as described above, read the relevant documentation in the `docs` directory and review the implementation to ensure it aligns with the specifications.

Be sure to perform this step only after you have finished the “blank slate” review. Do not take shortcuts by reading the `docs` directory first.

## Iteration count

The caller tells you which review iteration this is on the same task (1st / 2nd / 3rd). Use it as follows.

- **1st review**: enforce every rule fully on every file.
- **2nd or later review**: in **test code only**, you may relax findings that are clearly cosmetic (length, local-variable naming nits, light boilerplate, comment-style consistency). Drop those rather than file them. See `docs/coding/ts.md` §8.1.
- **Production (non-test) code is NEVER graded leniently, regardless of iteration count.** All rules apply at full strength on every iteration. Do not soften, drop, or downgrade production findings just because the task has been reviewed before.

If the iteration count is not supplied, treat it as the 1st review.
