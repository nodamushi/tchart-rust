# Web (WASM) 仕様

## 概要

`tchart-web` クレートは WebAssembly モジュールとしてビルドされ、ブラウザ上で
TCML → SVG 変換を行う。

## ビルド

```bash
wasm-pack build tchart-web --target web
```

出力先: `tchart-web/pkg/`

## JavaScript API

TS 側に露出する関数名は wasm-bindgen の `#[wasm_bindgen(js_name = ...)]`
属性で camelCase に揃えられる (Rust 側関数名は `render_tcml` 等の snake_case
のまま)。以下の API シグネチャは全て TS 側から見た名前。

```typescript
import init, { renderTcml } from './tchart_web.js';

await init();

const svg: string = renderTcml(tcmlText, options);
```

### `renderTcml(input: string, options?: RenderOptions): RenderResult`

| 引数 | 型 | 説明 |
|------|----|------|
| `input` | `string` | TCML テキスト |
| `options` | `RenderOptions?` | レンダリングオプション |

**戻り値**: `RenderResult` オブジェクト。成功時は `svg` フィールドに SVG 文字列、失敗時は `error` フィールドにパースエラーの位置情報と英語固定の error 本文が入る。どちらか片方のみ存在する (両方同時には存在しない)。

```typescript
interface RenderResult {
  svg?: string;
  error?: ParseErrorInfo;
}

interface ParseErrorInfo {
  line: number;     // 1 始まり
  column: number;   // 1 始まり、文字単位
  length: number;   // エラー範囲の文字数 (0 は挿入点エラー = 開いた `"` 等)
  message: string;  // 英語固定の error 本文 (cli.md §パースエラー出力形式 と同じ文言)
}
```

パースエラーは例外ではなく `error` フィールドで返す (UI 側で位置情報を活用するため)。フォントエラー / レイアウトエラーなど内部処理失敗は引き続き例外をスローする。

### `RenderOptions`

```typescript
interface RenderOptions {
  fontSize?: number;   // フォントサイズ (px), デフォルト 12.0。`> 0` の有限値のみ受理する (0 / 負値 / NaN / Infinity は例外をスロー)
}
```

### `toWaveJson(input: string): { json: string, warnings: string[] }`

TCML を WaveDrom の WaveJSON 形式に変換する。`tchart-core::wavedrom::to_wavejson`
の wasm ラッパー。

| 戻り値キー | 型 | 説明 |
|------------|----|------|
| `json` | `string` | WaveJSON テキスト (UTF-8 整形済み) |
| `warnings` | `string[]` | 変換時に落とした要素についての警告 (アンカー名の通し番号化、52 個超過、未対応スタイル等)。ない場合は空配列 |

パースエラー時は例外をスローする (戻り値の `warnings` には混入させない)。
`renderTcml` と独立してフォントメトリクスは利用しない (WaveJSON 化はレイアウト計算を伴わない)。

```typescript
import { toWaveJson } from './tchart_web.js';

const { json, warnings } = toWaveJson(tcmlText);
if (warnings.length > 0) {
  console.warn(warnings.join('\n'));
}
```

### `extractTcmlSource(svg: string): string | undefined`

SVG 文字列から `<tchart:source>...</tchart:source>` 要素を検索し、XML アンエスケープ
(`&lt;` → `<`, `&gt;` → `>`, `&amp;` → `&`) した TCML テキストを返す。
要素が見つからない場合は `undefined` (Rust 側 `None`) を返す。

```typescript
import { extractTcmlSource } from './tchart_web.js';

const source = extractTcmlSource(svgString);
if (source !== undefined) {
  console.log(source);
}
```

### `extractTcmlSourceFromPng(bytes: Uint8Array): string | undefined`

PNG バイト列を Rust の `png` クレートで decode し、`tchart-source` keyword の
`iTXt` チャンクの本文 (UTF-8) を返す。圧縮されていればクレートが復元する。
シグネチャ不正・該当 iTXt 不在等で取り出せない場合は `undefined` を返す。
JS 側は iTXt 構造を意識しない。

```typescript
import { extractTcmlSourceFromPng } from './tchart_web.js';

const tcml = extractTcmlSourceFromPng(new Uint8Array(await file.arrayBuffer()));
if (tcml !== undefined) {
  editor.value = tcml;
}
```

### `embedTcmlSourceInPng(bytes: Uint8Array, source: string): Uint8Array`

PNG バイト列を Rust の `png` クレートで decode し、`tchart-source` keyword の
`iTXt` チャンクに `source` を埋め込んだ新しい PNG バイト列を返す。
既存 iTXt が同 keyword で存在した場合の扱い (上書きか追加か) は `png` クレートの
`add_itxt_chunk` 仕様に従う (現状は追加)。元の画像データ (IHDR / IDAT / IEND) は保持する。
入力が PNG として不正な場合は例外をスローする。

```typescript
import { embedTcmlSourceInPng } from './tchart_web.js';

const withSource = embedTcmlSourceInPng(rawPng, "Clock _~_~");
const blob = new Blob([withSource], { type: "image/png" });
```

これら 2 関数は tchart-web 内部で `png` クレートを依存に持ち、その
encode/decode を直接利用する。同等の処理は `tchart-cli/src/render.rs` にも
存在するが、core を経由しないため両者は独立した実装である。

## フォントメトリクスの取得

ブラウザ環境では Canvas API を使ってテキスト幅を計測する。

```typescript
function measureText(text: string, fontSize: number): number {
  const canvas = document.createElement('canvas');
  const ctx = canvas.getContext('2d')!;
  ctx.font = `${fontSize}px sans-serif`;
  return ctx.measureText(text).width;
}
```

Rust 側から `wasm-bindgen` 経由でこの関数を呼び出す。

## 対応ブラウザ

- Chrome / Chromium 最新版
- Firefox 最新版
- Edge 最新版

Safari は動作確認対象外。
