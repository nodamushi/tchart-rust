#!/usr/bin/env bash
# Demo: show the rustc-style parse error format introduced for
# `docs/spec/cli.md` §パースエラー出力形式.
#
# Generates a handful of intentionally broken TCML samples in a temporary
# directory, runs `tchart svg` on each, and prints both the input and the
# resulting stderr so a human can visually verify the new error format.

set -u

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TCHART_BIN="${REPO_ROOT}/target/debug/tchart"
SAMPLE_DIR="$(mktemp -d -t tchart-parse-errors.XXXXXX)"
trap 'rm -rf "${SAMPLE_DIR}"' EXIT

if [ ! -x "${TCHART_BIN}" ]; then
    echo "==> building tchart (debug) ..."
    (cd "${REPO_ROOT}" && cargo build -p tchart-cli --bin tchart)
fi

write_sample() {
    local name="$1"
    local body="$2"
    printf '%s' "${body}" > "${SAMPLE_DIR}/${name}.tc"
}

write_sample "00_valid_baseline" '@step 25
SigA _~_~_~
SigB ====X===
'
write_sample "01_step_xyz"          '@step xyz
'
write_sample "02_dontcare_leading"  'SigA ?==
'
write_sample "03_unclosed_quote"    'SigA _"hello world
'
write_sample "04_error_in_line3"    'SigA _~_~
SigB ____
@step xyz
SigC ~~__
'
write_sample "05_invalid_color"     '@signal_color notacolor
SigA _~_~
'
write_sample "06_unknown_param"     '@notarealparam 42
SigA _~_~
'

run_one() {
    local tc_file="$1"
    local label
    label="$(basename "${tc_file}")"
    echo "================================================================"
    echo "  ${label}"
    echo "================================================================"
    echo "---- input ----"
    cat "${tc_file}"
    echo "---- stderr ----"
    "${TCHART_BIN}" svg "${tc_file}" -o /dev/null 2>&1 1>/dev/null
    local code=$?
    echo "---- exit code: ${code} ----"
    echo
}

for f in "${SAMPLE_DIR}"/*.tc; do
    run_one "${f}"
done
