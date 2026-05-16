# tchart-editor

English | [日本語](README.ja.md)

Web editor for TCML (Timing Chart Markup Language).
The released artifact is a single self-contained HTML file (`dist/index.html`).
CSS, JS, the wasm module, and the help content are all inlined; no Node, Rust, or network access is required at runtime.

For hosted usage, see the [repository root README](../README.md).
This document covers building from source.

## Prerequisites

- Rust toolchain (stable) with the `wasm32-unknown-unknown` target

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- `wasm-bindgen` CLI matching `Cargo.lock` (currently `0.2.121`)

  ```bash
  cargo install wasm-bindgen-cli --version 0.2.121 --locked
  ```

- Node.js (>=22) and pnpm (>=10)

## Build

```bash
pnpm install
pnpm build           # produces dist/index.html (single file)
```

`pnpm build` runs, in order:

1. `node scripts/build-wasm-pkg.mjs` — reads `workspace.package.version` from `Cargo.toml`, generates `../tchart-web/pkg/package.json`, and emits the wasm via `cargo build` + `wasm-bindgen`.
2. `tsc -p tsconfig.build.json` — type-checking TypeScript build.
3. `vite build` — produces `dist/index.html` (inlined via vite-plugin-singlefile).

## Development

```bash
pnpm dev             # local dev server with HMR
pnpm test            # vitest
pnpm check           # typecheck + lint + format check
pnpm fmt             # oxfmt
```

## Clean

```bash
pnpm clean           # removes node_modules, dist, and ../tchart-web/pkg
```

To verify the CI-equivalent clean-build flow locally:

```bash
pnpm verify:clean-build
```

This runs `clean` → wasm regen → `install --frozen-lockfile` → `build` in sequence.

## Specs

- Web editor spec: [`../docs/spec/editor.md`](../docs/spec/editor.md) (Japanese)
- Web (WASM) spec: [`../docs/spec/web.md`](../docs/spec/web.md) (Japanese)
