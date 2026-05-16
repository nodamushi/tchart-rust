#!/usr/bin/env bash
# Claude Code PreToolUse hook (Bash matcher).
# Reads tool input JSON from stdin, acts only on `git commit`.
# Enforces AGENTS.md "# No Local IDs" and the help-regeneration rule.
#
# Exit 0: allow.
# Exit 2: block (stderr is shown to Claude).
# Limitation: only fires when Claude invokes `git commit` via the Bash tool.
# Direct `git commit` from a terminal / IDE bypasses this hook.

set -euo pipefail

input=$(cat)
command=$(printf '%s' "$input" | jq -r '.tool_input.command // ""')

# Match `git commit` as an actual command (start of line, or after a shell
# separator). Global options before the subcommand (`git -C path commit`,
# `git --no-pager commit`, etc.) are allowed via the optional middle group,
# which excludes shell separators so it cannot cross command boundaries.
# Skip if the literal text appears only inside quoted strings, echo
# arguments, etc. — `git commit-tree` does not trigger either (boundary
# requires space or end after `commit`).
if ! printf '%s' "$command" | grep -qE '(^|;|&&|\|\|)[[:space:]]*git[[:space:]]+([^;&|]*[[:space:]])?commit([[:space:]]|$)'; then
  exit 0
fi

# Refuse compound commands that mutate staging in the same Bash call. The
# PreToolUse hook fires before any of the chained commands run, so `git add
# X && git commit -m ...` would slip through (X is not yet in `git diff
# --cached` when this script inspects it). Force the caller to run staging
# operations as a separate Bash call.
if printf '%s' "$command" | grep -qE '(^|;|&&|\|\|)[[:space:]]*git[[:space:]]+(add|stage|rm|mv|restore|reset|stash)\b'; then
  printf 'Refuse compound git command. Run staging operations (git add / rm / mv / stash / etc.) as a separate Bash call so this hook can inspect the actual staged state.\n' >&2
  exit 2
fi

forbidden_re='(\btcml-(bug|task|audit|review)-[0-9]+|BUG-|\bT-[0-9]+\b|\bA-[0-9]+\b|\bR-[0-9]+\b)'

# Inspect the commit-message portion of the bash command. Catches `-m`,
# `-m=`, and `--message` forms with either quote style.
# A `git commit` invocation is already fully matched, so any forbidden ID
# appearing anywhere in the command text — including heredoc bodies that span
# multiple lines and would slip past line-by-line `-o` matching — is a leak.
# `grep -z` treats the whole input as a single record, so newlines inside
# `"$(cat <<EOF ... EOF)"` style messages no longer break the search.
if printf '%s' "$command" | grep -zqE "$forbidden_re"; then
  printf 'Commit command contains a local ID. Block.\n' >&2
  printf '%s' "$command" | grep -nE "$forbidden_re" >&2 || true
  exit 2
fi

# WIP commits skip the help-regen check (the program may not be runnable).
wip_skip=0
if printf '%s' "$command" | grep -qE -- "-m[[:space:]]+['\"]?WIP[: ]"; then
  wip_skip=1
fi

repo_root=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$repo_root"

staged=$(git diff --cached --name-only 2>/dev/null || true)
[ -z "$staged" ] && exit 0

# ---------- Check 1: no local IDs in staged public files ----------
# Excluded from the scan:
# - docs/bugs.md / docs/tasks.md / docs/tasks/: authoritative homes
# - tmp/: gitignored anyway
# - AGENTS.md: defines the rule itself (mentions forbidden patterns as examples)
# - scripts/precommit-check.sh: implements the regex
excluded_re='^(docs/bugs\.md|docs/tasks\.md|docs/tasks/|tmp/|AGENTS\.md|scripts/precommit-check\.sh)'
public_files=$(printf '%s\n' "$staged" | grep -vE "$excluded_re" || true)

violations=""
if [ -n "$public_files" ]; then
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    [ ! -f "$f" ] && continue
    if hits=$(grep -nE "$forbidden_re" "$f" 2>/dev/null); then
      violations="${violations}--- ${f} ---"$'\n'"${hits}"$'\n'
    fi
  done <<< "$public_files"
fi

if [ -n "$violations" ]; then
  printf 'AGENTS.md "# No Local IDs" violation in staged files. Commit blocked.\n%s' "$violations" >&2
  exit 2
fi

# ---------- Check 2: tcml-format.md change requires source.py update ----------
if printf '%s\n' "$staged" | grep -qx 'docs/spec/tcml-format.md'; then
  if ! printf '%s\n' "$staged" | grep -qx 'help/source.py'; then
    printf 'docs/spec/tcml-format.md is staged but help/source.py is not. TCML syntax changes require updating the help template in the same commit.\n' >&2
    exit 2
  fi
fi

# ---------- Check 3: help regen ----------
[ "$wip_skip" -eq 1 ] && exit 0

# ---------- Check 4: wasm32 build sanity ----------
# `tchart-web/src/wasm_api/**` is gated by `#[cfg(target_arch = "wasm32")]`,
# so a host-target `cargo check` will silently skip it. Run an explicit
# wasm32 check whenever the staged set could affect the wasm build.
wasm_trigger_re='^(tchart-core/src/|tchart-core/Cargo\.toml$|tchart-web/|Cargo\.lock$)'
if printf '%s\n' "$staged" | grep -qE "$wasm_trigger_re"; then
  if rustup target list --installed 2>/dev/null | grep -qx 'wasm32-unknown-unknown'; then
    if ! cargo check --target wasm32-unknown-unknown -p tchart-web --quiet >/tmp/precommit-wasm.log 2>&1; then
      printf 'cargo check --target wasm32-unknown-unknown -p tchart-web failed. Commit blocked.\n' >&2
      cat /tmp/precommit-wasm.log >&2
      exit 2
    fi
  else
    printf 'WARNING: wasm32-unknown-unknown target not installed; skipping wasm build sanity check. Run `rustup target add wasm32-unknown-unknown` to enable.\n' >&2
  fi
fi

# ---------- Check 5: editor clean-build sanity ----------
# CI workflow / editor build script / editor package config への変更は、
# ローカルでは古い生成物 (tchart-web/pkg/) が残骸として残っているため
# 気付けず、CI のクリーンチェックアウトで初めて落ちることがある。
# 該当ファイルが staged にあるときは CI と同じ「クリーン → wasm 生成 →
# install → build」フローを走らせて事前検知する。
clean_build_trigger_re='^(\.github/workflows/pages\.yml$|tchart-editor/scripts/build-wasm-pkg\.mjs$|tchart-editor/package\.json$|tchart-editor/pnpm-lock\.yaml$|tchart-editor/pnpm-workspace\.yaml$)'
if printf '%s\n' "$staged" | grep -qE "$clean_build_trigger_re"; then
  log=$(mktemp)
  if ! (cd tchart-editor && pnpm verify:clean-build) >"$log" 2>&1; then
    printf 'pnpm verify:clean-build failed. CI would fail with the same change. Commit blocked.\n' >&2
    cat "$log" >&2
    rm -f "$log"
    exit 2
  fi
  rm -f "$log"
fi

trigger_re='^(tchart-core/src/|tchart-cli/src/|docs/spec/tcml-format\.md$|help/source\.py$)'
if printf '%s\n' "$staged" | grep -qE "$trigger_re"; then
  if [ ! -f "help/build.py" ]; then
    exit 0
  fi
  log=$(mktemp)
  if python3 help/build.py >"$log" 2>&1; then
    if ! git diff --quiet -- help/output/tcml-format.html help/output/tcml-format.en.html 2>/dev/null; then
      printf 'help/build.py produced changes that are not staged. Run "python3 help/build.py" and stage help/output/tcml-format.html and help/output/tcml-format.en.html.\n' >&2
      rm -f "$log"
      exit 2
    fi
  else
    printf 'WARNING: help/build.py failed; help-regen check skipped. Log: %s\n' "$log" >&2
  fi
  rm -f "$log"
fi

exit 0
