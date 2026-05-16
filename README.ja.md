# tchart-rust

[English](README.md) | 日本語

[オリジナル tchart](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html) の Rust 再実装。
TCML (Timing Chart Markup Language) テキストファイルを SVG タイミングチャートに変換します。

Node.js 等の外部ランタイム不要。Linux / Windows で動作。

## サンプル

![サンプル出力](docs/images/sample.png)
![矢印サンプル](docs/images/arrow.png)
![クロックマーカー](docs/images/clock_marks.png)

入力例: [`docs/images/sample.tc`](docs/images/sample.tc) / [`docs/images/arrow.tc`](docs/images/arrow.tc) / [`docs/images/clock_marks.tc`](docs/images/clock_marks.tc)

## クイックスタート

### CLI

```bash
cargo build --release -p tchart-cli
target/release/tchart svg chart.tc
target/release/tchart png chart.tc
```

詳細は [`docs/spec/cli.md`](docs/spec/cli.md)。

### Web エディタ

GitHub Pages で公開しています。インストール不要でブラウザから利用できます。

- エディタ: <https://nodamushi.github.io/tchart-rust/>
- ヘルプ (日本語): <https://nodamushi.github.io/tchart-rust/help/tcml-format.html>
- ヘルプ (英語): <https://nodamushi.github.io/tchart-rust/help/tcml-format.en.html>

リリース成果物としてスタンドアロン HTML 1 ファイル (`tchart-editor/dist/index.html`) も生成します。デスクトップ等に置いてダブルクリックすれば動きます。実行時に Node / Rust / ネット接続は不要。CSS / JS / wasm / help はすべて HTML 内に inline 済みです。

ソースからビルドする場合:

```bash
wasm-pack build tchart-web --target web
cd tchart-editor
pnpm install
pnpm build           # dist/index.html を生成 (1 ファイル)
pnpm dev             # ローカル開発サーバ
```

### WASM ライブラリ

```typescript
import init, { render_tcml } from './tchart-web/pkg/tchart_web.js';
await init();
const svg = render_tcml("Clock _~_~_~");
```

## ドキュメント

- [TCML フォーマット](docs/spec/tcml-format.md)
- [CLI 仕様](docs/spec/cli.md)
- [Web (WASM) 仕様](docs/spec/web.md)
- [Web エディタ仕様](docs/spec/editor.md)
- [WaveDrom 変換](docs/spec/wavedrom.md)

## 謝辞

本実装は以下の先行実装・定義を参考にさせていただいております。この場での謝辞を述べさせていただきます。ありがとうございます。

- [タイミングチャート清書ツール](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html) (東北学院大学)
- [tchart-coffee](https://dora.bk.tsukuba.ac.jp/~takeuchi/?%E3%82%BD%E3%83%95%E3%83%88%E3%82%A6%E3%82%A7%E3%82%A2%2F%E3%82%BF%E3%82%A4%E3%83%9F%E3%83%B3%E3%82%B0%E3%83%81%E3%83%A3%E3%83%BC%E3%83%88%E6%B8%85%E6%9B%B8%E3%82%B5%E3%83%BC%E3%83%93%E3%82%B9) (筑波大学)

## 関連プロジェクト

- [WaveDrom](https://wavedrom.com/) — DataSheet 品質の本格的なタイミングチャート描画ツール
