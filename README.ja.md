# tchart-rust

[English](README.md) | 日本語

本ツールは [タイミングチャート清書ツール](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html) (東北学院大学) と [tchart-coffee](https://dora.bk.tsukuba.ac.jp/~takeuchi/) (筑波大学) から大きな着想をいただいた、独立した Rust 実装です。詳細は [謝辞](#謝辞) を参照してください。

**デジタル回路のタイミングチャート (clock / data / bus / 制御信号 の波形) をテキストで描くツール**です。デジタル回路 / FPGA / HDL の設計、設計メモ、議論中に「この波形どうなるんだっけ？」をさっと描くといった用途を想定しています。

チャートは TCML (Timing Chart Markup Language) というテキストで書きます:

```
@title Request / Acknowledge handshake
@slant 0

@clock(pos)
clk

request      ___@{r}~~~~~~~~@1____
acknowledge  _____@{a}~~________
ready        _____~~~~~~~~@2__
data_bus     =?====X==DATA=====X?=

@-> (@{r}, @{a}) req → ack
@-> (@1, @2, red)
```

`tchart` がこれをレンダリングします:

![showcase](docs/images/showcase.png)

ブラウザですぐに試せます: <https://nodamushi.github.io/tchart-rust/>

CLI バイナリ (Linux / Windows) や、ブラウザで開くオフライン版のスタンドアロン HTML エディタとしても配布しています。Node.js / ランタイム / インストーラは不要です。

## 特徴

先行ツールから着想を得ているため文法レベルの違いは意図して小さくしています。本実装で意識的に追加・改善したのは以下です。

- **信号名カラムの自動レイアウト** — 信号名の幅指定が不要です。任意長の名前を書いてもそのまま揃って表示されます。
- **信号間の矢印 (`@->`)** — 波形内で `@{name}` (または `@1` のような短いアンカー名) で位置に名前を付け、別の信号上の位置との間に矢印を引けます。色 / 線種 / 矢頭の形 / ラベルが選べ、信号間の因果関係や時間的依存を可視化できます。
- **`@clock` 1 行宣言** — クロック信号を 1 行で宣言でき、波形本体と立ち上がり / 立ち下がり / 両方 / なし のエッジマーカーが自動展開されます。
- **WaveDrom 出力** — `tchart wavedrom` で同じ TCML を [WaveJSON](https://github.com/wavedrom/schema/blob/master/WaveJSON.md) に変換できます。粗書きした TCML を [`wavedrom-cli`](https://wavedrom.com/) に渡せばデータシート品質の図に仕上げられます (一部の TCML 機能は WaveDrom 側に対応物がなく落ちます)。

## その他のサンプル

![サンプル出力](docs/images/sample.png)
![矢印サンプル](docs/images/arrow.png)
![クロックマーカー](docs/images/clock_marks.png)

ソース TCML: [`docs/images/sample.tc`](docs/images/sample.tc) / [`docs/images/arrow.tc`](docs/images/arrow.tc) / [`docs/images/clock_marks.tc`](docs/images/clock_marks.tc)

## クイックスタート

### CLI

[Releases](https://github.com/nodamushi/tchart-rust/releases) から最新バイナリを取得して使います。

#### Linux (x86_64)

```bash
VER=v0.1.0
curl -fsSLO https://github.com/nodamushi/tchart-rust/releases/download/$VER/tchart-$VER-x86_64-unknown-linux-gnu.tar.gz
tar -xzf tchart-$VER-x86_64-unknown-linux-gnu.tar.gz
cd tchart-$VER-x86_64-unknown-linux-gnu
./tchart svg chart.tc
./tchart png chart.tc
```

#### Windows (x86_64)

[Releases](https://github.com/nodamushi/tchart-rust/releases) ページから `tchart-v0.1.0-x86_64-pc-windows-msvc.zip` をダウンロードして展開し、`tchart.exe` を実行します。

```powershell
.\tchart.exe svg chart.tc
.\tchart.exe png chart.tc
```

#### 使い方

```
tchart svg      <INPUT>                          # TCML を SVG にレンダリング
tchart png      <INPUT>                          # TCML を PNG にレンダリング
tchart src      <SVG_OR_PNG>                     # 埋め込み TCML を抽出
tchart wavedrom <INPUT>                          # TCML を WaveDrom JSON に変換
tchart batch    <svg|png> <INPUT>... -o <DIR>    # 複数ファイルの一括変換
```

実行例:

```bash
tchart svg chart.tc                    # → 入力の隣に chart.svg を出力
tchart png chart.tc -o out.png         # 出力先を明示
tchart svg chart.tc --font-size 14     # フォントサイズを変更
tchart src chart.png -o -              # 埋め込み TCML を標準出力へ
tchart batch svg samples/*.tc -o out/  # 一括変換
```

主なオプション:

| フラグ | 説明 | デフォルト |
|---|---|---|
| `-o, --output <PATH>` | 出力先パス (`batch` ではディレクトリ) | 入力ファイルの隣 |
| `--font <FILE>` | デフォルトフォントファイル | システムフォント自動検出 |
| `--font-size <SIZE>` | フォントサイズ (px, `> 0`) | `12.0` |
| `-h, --help` | ヘルプ表示 | |

詳細な CLI 仕様は [`docs/spec/cli.md`](docs/spec/cli.md)、ソースからのビルド方法は [`tchart-cli/README.ja.md`](tchart-cli/README.ja.md)。

### Web エディタ

- エディタ: <https://nodamushi.github.io/tchart-rust/>
- ヘルプ (日本語): <https://nodamushi.github.io/tchart-rust/help/tcml-format.html>
- ヘルプ (英語): <https://nodamushi.github.io/tchart-rust/help/tcml-format.en.html>

公開 URL をブラウザの「名前を付けて保存」で取得すれば、その HTML 1 ファイルだけでオフライン動作します (CSS / JS / wasm / help を inline 済み)。

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

以下の先行実装に深く感謝申し上げます。

- [タイミングチャート清書ツール](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html) (東北学院大学)
- [tchart-coffee](https://dora.bk.tsukuba.ac.jp/~takeuchi/?%E3%82%BD%E3%83%95%E3%83%88%E3%82%A6%E3%82%A7%E3%82%A2%2F%E3%82%BF%E3%82%A4%E3%83%9F%E3%83%B3%E3%82%B0%E3%83%81%E3%83%A3%E3%83%BC%E3%83%88%E6%B8%85%E6%9B%B8%E3%82%B5%E3%83%BC%E3%83%93%E3%82%B9) (筑波大学)

## 関連プロジェクト

- [WaveDrom](https://wavedrom.com/) — DataSheet 品質の本格的なタイミングチャート描画ツール
