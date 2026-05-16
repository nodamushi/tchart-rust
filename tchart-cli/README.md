# tchart-cli

English | [日本語](README.ja.md)

Command-line tool that renders TCML (Timing Chart Markup Language) to SVG / PNG / WaveDrom.

For prebuilt binary usage, see the [repository root README](../README.md).
This document covers building from source.

## Prerequisites

- Rust toolchain (stable)

## Build

Run from the repository root:

```bash
cargo build --release -p tchart-cli
```

Artifacts:
- Linux: `target/release/tchart`
- Windows: `target/release/tchart.exe`

To use the size-optimised `cli-release` profile:

```bash
cargo build --profile cli-release -p tchart-cli
```

Artifacts land in `target/cli-release/tchart` (Linux) / `target/cli-release/tchart.exe` (Windows).

## Smoke test

```bash
target/release/tchart svg ../docs/images/sample.tc -o /tmp/sample.svg
target/release/tchart png ../docs/images/sample.tc -o /tmp/sample.png
```

## Specs

- CLI spec: [`../docs/spec/cli.md`](../docs/spec/cli.md) (Japanese)
- TCML format: [`../docs/spec/tcml-format.md`](../docs/spec/tcml-format.md) (Japanese)
