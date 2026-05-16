# tchart-cli

[English](README.md) | 日本語

タイミングチャート (TCML) を SVG / PNG / WaveDrom に変換するコマンドラインツール。

リリース済みバイナリの使い方は [リポジトリルートの README](../README.ja.md) を参照。
ここではソースからのビルド方法を記載します。

## 前提

- Rust toolchain (stable)

## ビルド

リポジトリのルートで実行:

```bash
cargo build --release -p tchart-cli
```

成果物:
- Linux: `target/release/tchart`
- Windows: `target/release/tchart.exe`

リリースプロファイル (`cli-release`) で最適化したい場合:

```bash
cargo build --profile cli-release -p tchart-cli
```

成果物は `target/cli-release/tchart` (Linux) / `target/cli-release/tchart.exe` (Windows)。

## 動作確認

```bash
target/release/tchart svg ../docs/images/sample.tc -o /tmp/sample.svg
target/release/tchart png ../docs/images/sample.tc -o /tmp/sample.png
```

## 仕様

- CLI 仕様: [`../docs/spec/cli.md`](../docs/spec/cli.md)
- TCML フォーマット: [`../docs/spec/tcml-format.md`](../docs/spec/tcml-format.md)
