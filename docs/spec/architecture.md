# アーキテクチャ設計

[rust.md](./coding/rust.md) を必ず一読し、本プロジェクトにおける Rust のコーディングルールを把握すること。

> 型定義の正本は実装ソース (`tchart-core/src/`)。実装構造とアルゴリズム概要は [`types.md`](types.md)。本ドキュメントはクレート分割・データフロー・各層の責務を扱う。

## Cargo ワークスペース構成

```
tchart_rust/
├── Cargo.toml          # ワークスペースルート
├── tchart-core/        # コアライブラリ
├── tchart-cli/         # CLI ツール
├── tchart-web/         # WASM モジュール
└── tchart-editor/      # Web エディタ (Vite + TypeScript)
```

## クレート一覧

### `tchart-core`

**役割**: TCML パーサー + レイアウト + SVG レンダラー + 抽出。プラットフォーム非依存。

**提供するもの**:

- TCML パーサー: テキスト → `ChartDocument` (parser 段階の AST)
- Clock 展開パス: `@clock` 装飾付き SignalRow の波形自動展開
- アンカー解決パス: `AnchorRegistry` 構築
- レイアウトエンジン: 行ベースのジオメトリ確定 (`Line.bbox`、`SignalRow.geometry` 解決)
- SVG レンダラー: `ChartDocument` → SVG 文字列
- 抽出: SVG / PNG → 元の TCML 文字列

#### モジュール構成 (`tchart-core/src/`)

```
src/
  units          // Length, Px
  geometry       // Point, Size, Rect
  color          // Color, ColorError
  text           // SignalName, UserText, FontFamily, FontSpec
  style          // ChartStyle / CanvasStyle / BackgroundStyle / SignalStyle / ...
  line           // Line, LineContent, SignalRow, SkipRow, TitleRow,
                 // SignalDecorations, SignalGeometry,
                 // Waveform, WaveformElement, LevelRun, SignalLevel,
                 // LevelShape, Transition, TransitionKind
  clock          // ClockSpec, ClockEdge, ClockPulse, ClockPhase
  anchor         // AnchorId, AnchorName, AnchorRegistry, ResolvedAnchor
  arrow          // Arrow, ArrowEnd, ArrowStyle, LineDashStyle, ArrowHead
  document       // ChartDocument, Annotations, TextOverlay, TcmlSource
  defaults       // 全デフォルト定数（マジックナンバー集約）
  errors         // ParseError, ColorError, NameError, TextError, ...
  parser         // TCML テキスト → ChartDocument (raw)
  layout         // 行積み上げ + signal_box / label_box ジオメトリ確定
  svg            // ChartDocument → SVG 文字列
  lib.rs
```

モジュール構成と各モジュールの主要型一覧は [`types.md`](types.md) §2 を参照。

#### 型による信頼境界 (NewType Pattern)

ユーザー入力に由来するすべての文字列は、用途別の NewType を経由する。
旧設計の汎用 `UserStr` は廃止し、検証ルールを型ごとに焼き込む。

| 型 | 用途 | 検証 |
|----|------|------|
| `SignalName` | 信号名 | UTF-8、改行 (`\n`) のみ許可、その他制御文字禁止、空不可 |
| `UserText` | ラベル・タイトル本文・overlay 文字列 | 制御文字 (`\n` `\t` を除く) 禁止 |
| `FontFamily` | フォント名 | 制御文字禁止、カンマ区切り複数指定対応 |
| `Color` | 色値 | `#rgb` / `#rrggbb` / `#rrggbbaa` / CSS 名前付き / `none` |
| `AnchorName` | アンカー名 | `[A-Za-z_][A-Za-z0-9_-]*` |

設計原則:

- 検証は **NewType の `parse` / `try_from` 内**で行う。後段で「文字列のまま検証関数を通す」設計は禁止。
- 関数 (`fn escape(&str) -> String` 等) を介して `String` を「解決済み」と見なすコードは禁止 (`docs/coding/rust.md`)。
- `ChartDocument` 出力以降で `String` をユーザー値として持つフィールドは存在しない。
- SVG 出力レイヤーは NewType だけを引数に取るエスケープ API を提供する。

### `tchart-cli`

**役割**: CLI インターフェース。Linux / Windows で動作。

**責任**:

- コマンドライン引数の解析
- 入力ファイルの読み込み
- フォントメトリクスの取得 (OS フォント / 埋め込みフォント)
- `tchart-core` を呼び出して SVG / PNG 生成
- `tchart extract` サブコマンド: SVG / PNG → TCML 抽出

### `tchart-web`

**役割**: WebAssembly モジュール。ブラウザ上で動作。

**責任**:

- WASM バインディング (`wasm-bindgen`)
- フォントメトリクスの取得 (Canvas API 経由)
- `tchart-core` を呼び出して SVG 生成
- JavaScript に SVG 文字列を返す
- SVG / PNG からの抽出関数 (`extract_tcml_source` / `extract_tcml_source_from_png`) と PNG への埋め込み関数 (`embed_tcml_source_in_png`) を JavaScript に公開
- PNG iTXt 処理は `png` クレートを `tchart-web` 自身の依存として持ち、`tchart-core` を経由しない (CLI 側の同等処理 (`tchart-cli/src/render.rs::embed_itxt`) とは独立した実装、コードは重複)

### `tchart-editor`

**役割**: ブラウザ上の TCML エディタ。Vite + TypeScript。

**責任**:

- 2 分割画面 (エディタ + SVG プレビュー) UI
- `tchart-web` (WASM) でリアルタイムプレビュー
- SVG / PNG ダウンロード
- Vite 開発サーバー

## データフロー (パイプライン)

```
[TCML テキスト (.tc)]
        |
        v
[Parser] (構文木構築 + 仮 LevelRun + DontCare 解決)
        |
        v
[Clock 展開パス] (@clock 装飾の SignalRow を chart_units まで展開、
                エッジ矢印を Annotations.arrows に追加)
        |
        v
[アンカー解決パス] (@{name} / @N の x/y 座標を確定し AnchorRegistry を構築、
                 @-> の ArrowEnd::Anchor を解決)
        |
        v
ChartDocument (parser-level: ジオメトリ未確定、要素列のみ)
        |
        v
[LayoutEngine] + FontMetrics
   (Line.bbox を縦に積み上げ、SignalRow.geometry の signal_box / label_box を確定、
    アンカー座標 (Px) を確定し AnchorRegistry を更新)
        |
        v
ChartDocument (layout-resolved: 全 Px 確定)
        |
        v
[SVG Renderer] (ChartDocument → SVG ノードツリー → 文字列)
        |
        v
[SVG 文字列 (TCML ソース埋め込み済み)]
        |
   +---------+
   |         |
[ファイル出力]  [PNG 変換 (CLI のみ)]
```

```
[SVG / PNG ファイル]
        |
        v
[Extractor] → TCML テキスト
```

### パイプラインの不変条件

- **Parser 出力**:
  - `Vec<WaveformElement>` に `DontCarePending` は残らない (全 `?` が `DontCareAlong*` のいずれかに確定済み)。
  - `WaveformElement::Anchor` の `AnchorId` は重複しない。
  - `@->` の `ArrowEnd::Anchor` 参照は `AnchorRegistry` で解決可能。
- **Clock 展開後**:
  - `clock` 装飾付き SignalRow の `waveform.elements` は `chart_units` 分の波形を完全に持つ。
  - クロックの三角形マーカーは `SignalRow.edge_marks` に注入済み (`Annotations.arrows` には混入させない、`types.md` §6.6 参照)。
- **Layout 後**:
  - `Line[i+1].bbox.origin.y == Line[i].bbox.origin.y + Line[i].bbox.size.height` (`debug_assert!`)
  - 全 `SignalRow` で `signal_box.origin.y == signal_gap / 2`、`bbox.size.height - signal_box.size.height == signal_gap`
  - 全 `SignalRow` で `signal_box.size.width == sum(elements.width())`
  - `AnchorRegistry` の全 `ResolvedAnchor.at` が確定 (`Px` 値、未解決なし)
- **SVG レンダラー**:
  - `ChartDocument` のみを入力に取り、外部状態を引かない。
  - `TransitionKind` の `match` は網羅的 (`_ =>` 禁止)。

## フォントメトリクスの抽象化

```rust
pub trait FontMetrics {
    /// 指定フォント・サイズでのテキスト幅を返す (Px)
    fn measure_text_width(&self, text: &str, font: &FontSpec) -> Px;
}
```

- CLI: OS のフォント情報または埋め込みフォント (例: `ab_glyph` / `fontdue`)。
- Web: `Canvas.measureText` を `wasm-bindgen` 経由で呼び出す。Canvas に渡す CSS `font` 短縮形 (`"<size>px <family>"`) は `FontSpec::to_canvas_css` で生成し、`FontFamily` の生 `String` をクロスクレートに露出させない。
- LayoutEngine の入力として注入する。`tchart-core` 内で具体実装を持たない。

## レイアウトエンジンの責務

レイアウトは **行を縦に積み上げるだけ**。詳細は [`types.md`](types.md) §3.1「行ジオメトリ規約 (対称ギャップ)」と §4.5「レイアウトアルゴリズム」、§6「過去バグ防止条項」参照。

1. `ChartStyle.canvas.line_height` を確定 (`fontsize × lineheight`)。
2. 各 `Line` の `bbox.size.height` を計算。
   - `Skip`: `amount.resolve(line_height)`
   - `Title`: `text 行数 * line_height + h_space` (Signal と同様に `h_space` を加算し、上下に `h_space/2` ずつ対称配分。詳細は `types.md` §4.5)
   - `Signal`: `max(name 行数 × name_font.line_height, waveform_height) + signal_gap`
3. 各 `SignalRow` で `signal_box.origin = (capwidth + namepad, signal_gap / 2)`、`signal_box.size = (sum(elements.width()), waveform_height)`。
4. `Line[i].bbox.origin.y` を上から `Line[i-1].bbox.size.height` の合計で確定。
5. `WaveformElement::Anchor` の x/y を **要素の累積位置 + 直前 LevelRun の線位置**で確定し、`AnchorRegistry` に登録。
6. `Arrow.from`/`to` を `AnchorRegistry` で解決し、確定座標に書き換え。

「直前のレベルから遷移を再構築する」「最終行は gap 半分」のような分岐は **禁止**。

## SVG レンダラーの責務

- `ChartDocument` を 1 つ受け取り、`SvgNode` ツリーを構築 → 文字列化。
- 描画契約は [`svg-rendering.md`](svg-rendering.md) で網羅。
  - `TransitionKind` ごとの線本数・始終点・接続先 polyline。
  - `?` (DontCare) polygon の塗り (y_h〜y_l 範囲、隣接遷移追従) + 内部水平線描画。
  - 矢印 (`Arrow`) の描画 (色・太さ・線種・矢印頭)。
  - エッジ矢印 (clock 由来) の配置。
  - 信号名上線 (`SignalDecorations.name_overline`)。

## 定数集約 (`defaults.rs`)

すべてのデフォルト値・固定スラント幅などは `defaults.rs` に集約。
コード中のリテラル長・色・フォント名は禁止。

実装は `tchart-core/src/defaults/` に集約。

## 抽出 (Extract)

- SVG: `<metadata>` 内の `<tchart:source>` カスタム要素から TCML 文字列を取り出す。
- PNG: `iTXt` チャンク (`tchart-source` キー) から TCML 文字列を取り出す。
- 抽出処理はパーサー / レンダラーから独立した薄いユーティリティ (`extract.rs`)。
