---
name: doc-tests
description: Generate Gherkin-style test cases as .feature.md files. Use when creating or updating test scenarios for docs/tests.md and docs/tests/ directory.
---

# Gherkin Test Case Generator

Generate test cases as `.feature.md` files.
This format renders as clean Markdown on GitHub, and is also parseable by Cucumber for future automation.

---

## Output Format

Always output as `.feature.md` files (Markdown with Gherkin).
Never output as a bullet list. Never output as a plain Markdown table.

### File structure

```
docs/
├── tests.md                          ← Root index file (fixed, always maintain this)
└── tests/
    ├── login.feature.md
    ├── user-profile.feature.md
    └── ...
```

### Root index file: `docs/tests.md`

This file is fixed at `docs/tests.md`. Always keep it up to date.
When adding or removing a feature test file, update this index accordingly.

```markdown
# Test Cases

| Feature | File | Status Summary |
|---|---|---|
| ユーザーログイン | [login.feature.md](tests/login.feature.md) | ✅ 3 / ⬜ 2 / 🚧 1 |
| ユーザープロフィール | [user-profile.feature.md](tests/user-profile.feature.md) | ✅ 1 / ⬜ 4 / 🚧 0 |
```

Status summary legend (see Status Tags for full definitions):

| Symbol | Tag |
|---|---|
| ✅ | `@implemented` |
| ⬜ | `@not-implemented` |
| 🚧 | `@wip` |
| ⏭ | `@skip` |

### Individual test files

```
docs/tests/<feature-name>.feature.md
```

### Template

```markdown
# <Feature name>

<One-line description of what this feature does.>

---

## @<status> [@<extra-tag> ...]
### Scenario: <scenario title>
- Given <precondition>
- When <action>
- Then <expected result>
```

---

## Status Tags (Required)

Every Scenario **must** have exactly one status tag on the line above it.

| Tag | Meaning |
|---|---|
| `@implemented` | Implemented and manually verified |
| `@not-implemented` | Not yet implemented |
| `@wip` | Work in progress |
| `@skip` | Temporarily excluded from testing |

---

## Optional Extra Tags

Add these alongside the status tag as needed.

| Tag | Meaning |
|---|---|
| `@smoke` | Smoke test (run on every build) |
| `@regression` | Regression test |
| `@edge-case` | Edge case or boundary value |
| `@negative` | Negative test (invalid input, error path) |

---

## Coverage Requirements

For each feature, always include all of the following unless explicitly told otherwise:

1. **Happy path** — Normal successful operation
2. **Negative cases** — Invalid input, unauthorized access, missing data
3. **Edge cases** — Boundary values, empty input, extremely large input

---

## Full Example

````markdown
# ユーザーログイン

メールアドレスとパスワードによる認証フローのテスト仕様。

---

## @implemented @smoke
### Scenario: 正常ログイン
- Given ユーザーが登録済みである
- When 正しいメールアドレスとパスワードを入力してログインボタンを押す
- Then ダッシュボードページに遷移する

## @implemented @negative
### Scenario: パスワード誤りでログイン失敗
- Given ユーザーが登録済みである
- When 正しいメールアドレスと誤ったパスワードを入力してログインボタンを押す
- Then 「メールアドレスまたはパスワードが正しくありません」エラーが表示される
- And ログインページに留まる

## @not-implemented @negative
### Scenario: 存在しないメールアドレスでログイン失敗
- Given ユーザーが登録されていない
- When 未登録のメールアドレスとパスワードを入力してログインボタンを押す
- Then 「メールアドレスまたはパスワードが正しくありません」エラーが表示される

## @not-implemented @edge-case
### Scenario: 連続ログイン失敗でアカウントロック
- Given ユーザーが登録済みである
- When 誤ったパスワードで5回連続ログインを試みる
- Then アカウントがロックされる
- And 「アカウントがロックされました」メッセージが表示される

## @wip @negative
### Scenario: 空欄のままログイン試行
- Given ログインページを開いている
- When メールアドレスとパスワードを入力せずにログインボタンを押す
- Then 各入力欄にバリデーションエラーが表示される
````

---

## Notes

- Write step text in Japanese. `Given`/`When`/`Then`/`And` keywords stay in English.
- Do not add implementation details (no function names, no class names) in step text.
