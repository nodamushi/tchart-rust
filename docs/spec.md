# tchart-rust 仕様書

## 目的と背景

tchart-rust は、タイミングチャート清書ツール「tchart」の Rust 再実装。

参考実装:
- [オリジナル tchart (東北学院大)](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/) — Perl/C++ 製、EPS/BMP 出力
- [tchart-coffee (筑波大)](https://github.com/osamutake/tchart-coffee) — CoffeeScript 製、SVG 出力。ハイライト・不定値・スタイルカスタマイズ等の拡張機能あり

本プロジェクトの目標:

- **Rust で実装**し、Node.js 等の外部ランタイム不要
- **SVG/PNG 出力**に対応
- **CLI** および **WebAssembly (ブラウザ)** の両方で動作
- 対応プラットフォーム: Linux / Windows
- tchart-coffee の拡張機能（ハイライト、不定値、スタイルカスタマイズ、データ埋め込み等）を取り込む
- **フォントサイズ基準のレイアウト**により、直感的なパラメータ設定を実現

---

## 概要

TCML (Timing Chart Markup Language) 形式のテキストファイルを入力として受け取り、
タイミングチャートを SVG (または PNG) として出力するツール。

出力 SVG/PNG には元の TCML ソースが埋め込まれており、出力ファイルから元データを復元できる。

**入力例:**

```tcml
# クロック同期回路
@fontsize 14
@step 5
@slant 0

Clock   _~_~_~_~_~_~
Data    =<D0>====X<D1>====
Enable  ____~~~~____
```

**出力:** 上記を可視化したタイミングチャートの SVG ファイル

---

## 使い方

### CLI

詳細: [docs/spec/cli.md](spec/cli.md)

```bash
# SVG 出力 (デフォルト)
tchart chart.tc -o output.svg

# PNG 出力 (SVG 実装後)
tchart chart.tc --format png -o output.png

# SVG/PNG から元の TCML ソースを抽出
tchart extract output.svg
tchart extract output.png
```

### Web (WASM)

詳細: [docs/spec/web.md](spec/web.md)

```typescript
import init, { render_tcml } from './tchart_web.js';
await init();
const svg = render_tcml(tcmlSource);
document.getElementById('chart').innerHTML = svg;
```

### Web エディタ

詳細: [docs/spec/editor.md](spec/editor.md)

ブラウザ上で TCML を編集し、リアルタイムに SVG プレビューを確認できる 2 分割画面エディタ。
`tchart-web` (WASM) を使用し、Vite で開発サーバーを起動する。
SVG / PNG のダウンロード機能付き。

```bash
cd tchart-editor && pnpm dev
```

### テスト実行

```bash
cargo test
```

---

## プロジェクト構成

[rust.md](./coding/rust.md) を必ず一読し、本プロジェクトにおける Rust のコーディングルールを把握すること。

詳細: [docs/spec/architecture.md](spec/architecture.md)

| クレート | 役割 |
|----------|------|
| `tchart-core` | TCML パーサー + SVG レンダラー + データ抽出。プラットフォーム非依存のコアロジック |
| `tchart-cli` | CLI ツール。フォント取得・ファイル I/O・TCML 抽出を担当 |
| `tchart-web` | WASM モジュール。ブラウザ上でのフォント計測と JS バインディングを担当 |
| `tchart-editor` | Web エディタ (Vite + TypeScript)。TCML 編集 + SVG プレビュー + エクスポート |

---

## 実装設計

[rust.md](../coding/rust.md) を必ず一読し、本プロジェクトにおける Rust のコーディングルールを把握すること。

### TCML フォーマット

詳細: [docs/spec/tcml-format.md](spec/tcml-format.md)

- 4種類の行: コメント (`#`), パラメータ (`@`), テキスト配置 (`%`), タイミング記述
- レベル記号: `_` (Low), `~` (High), `-` (Hi-Z), `=` (Bus), `?` (不定値)
- 補助記号: `:` (空白), `X` (バス遷移), `|` (縦線/ガイド線), `[` `]` (ハイライト)
- パラメータは **グローバル** (fontsize, lineheight 等) と **ローカル** (step, slant 等) に分類
- ローカルパラメータは途中変更可能（非同期クロック、信号ごとの色変更等に対応）

### アーキテクチャ

詳細: [docs/spec/architecture.md](spec/architecture.md)

```
[TCML テキスト]
     ↓ Parser
[TcmlDocument (AST)]
     ↓ LayoutEngine + FontMetrics (trait)
[LayoutDocument]
     ↓ SvgRenderer
[SVG 文字列 (TCML ソース埋め込み済み)]
```

フォントメトリクスは `FontMetrics` trait で抽象化し、CLI/Web それぞれが実装を提供する。

### SVG レンダリング

詳細: [docs/spec/svg-rendering.md](spec/svg-rendering.md)

- 外部 CSS 非依存の自己完結型 SVG を出力
- フォントサイズ基準のレイアウト: `fontsize` → `waveform_height` → `signal_spacing`
- エッジの傾斜は `slant` パラメータで制御
- 信号ごとの背景色 (`@bg`)、ハイライト (`[]`)、不定値 (`?`) を描画
- 縦線はガイド線スタイル（赤色・細線）で上下に飛び出して描画
- 元の TCML ソースを `<metadata>` に埋め込み

---

## 外部リンク

- [オリジナル tchart (東北学院大)](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/)
- [TCML 文法リファレンス](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/syntax.html)
- [TCML 記述例](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/examples.html)
- [tchart-coffee (筑波大)](https://github.com/osamutake/tchart-coffee)
- [tchart-coffee 文法](https://github.com/osamutake/tchart-coffee/blob/master/doc/syntax.html)
- [WaveDrom](https://wavedrom.com/) (類似ツール、Node.js 製)
