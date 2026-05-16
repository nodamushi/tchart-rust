# tchart-editor

[English](README.md) | 日本語

タイミングチャート (TCML) 用の Web エディタ。
スタンドアロン HTML 1 ファイル (`dist/index.html`) として配布されます。
CSS / JS / wasm / help はすべて inline 済みで、実行時に Node / Rust / ネット接続は不要です。

ホスト版の使い方は [リポジトリルートの README](../README.ja.md) を参照。
ここではソースからのビルド方法を記載します。

## 前提

- Rust toolchain (stable) + `wasm32-unknown-unknown` target

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

- `wasm-bindgen` CLI (`Cargo.lock` と同じ `0.2.121` を推奨)

  ```bash
  cargo install wasm-bindgen-cli --version 0.2.121 --locked
  ```

- Node.js (>=22) + pnpm (>=10)

## ビルド

```bash
pnpm install
pnpm build           # dist/index.html (1 ファイル) を生成
```

`pnpm build` は内部で:

1. `node scripts/build-wasm-pkg.mjs` — `Cargo.toml` の `workspace.package.version` を読んで `../tchart-web/pkg/package.json` を生成し、`cargo build` + `wasm-bindgen` で wasm を出力
2. `tsc -p tsconfig.build.json` — TypeScript 型チェック付きビルド
3. `vite build` — `dist/index.html` を出力 (vite-plugin-singlefile で 1 ファイル化)

を順に実行します。

## 開発

```bash
pnpm dev             # ローカル開発サーバ (HMR あり)
pnpm test            # vitest
pnpm check           # typecheck + lint + format check
pnpm fmt             # oxfmt 整形
```

## クリーン

```bash
pnpm clean           # node_modules / dist / ../tchart-web/pkg を削除
```

CI と同じクリーン状態でビルドできるか検証する場合:

```bash
pnpm verify:clean-build
```

`clean` → wasm 再生成 → `install --frozen-lockfile` → `build` を一気に実行します。

## 仕様

- Web エディタ仕様: [`../docs/spec/editor.md`](../docs/spec/editor.md)
- Web (WASM) 仕様: [`../docs/spec/web.md`](../docs/spec/web.md)
