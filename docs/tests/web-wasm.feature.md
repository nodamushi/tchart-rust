# Web (WASM)

`tchart-web` の JavaScript API のテスト仕様。ブラウザ環境での TCML → SVG 変換を検証する。

---

## @not-implemented @smoke
### Scenario: renderTcml の基本動作 (成功時は RenderResult.svg)
- Given WASM モジュールが初期化されている
- When `renderTcml("Clock _~_~")` を呼び出す
- Then 戻り値が `RenderResult` オブジェクトである
- And `result.svg` が `<svg` で始まる文字列である
- And `result.error` フィールドが存在しないか `undefined` である

## @not-implemented
### Scenario: オプションなしで renderTcml を呼び出す
- Given WASM モジュールが初期化されている
- When `renderTcml(tcmlText)` をオプション引数なしで呼び出す
- Then デフォルトのフォントサイズ (12.0) で SVG が生成される
- And 戻り値の `svg` が定義され `error` が存在しないか `undefined` である

## @not-implemented
### Scenario: fontSize オプションの指定
- Given WASM モジュールが初期化されている
- When `renderTcml(tcmlText, { fontSize: 16.0 })` を呼び出す
- Then フォントサイズ 16.0 で SVG が生成される
- And 戻り値の `svg` が定義されている

## @not-implemented
### Scenario: Canvas API によるテキスト幅計測の連携
- Given ブラウザの Canvas API が利用可能な環境がある
- When `renderTcml` でラベル付き信号を含む TCML をレンダリングする
- Then Canvas API の measureText を使用してテキスト幅が計測される
- And 計測結果をもとにラベルが正しく配置された SVG が `result.svg` として返る

## @not-implemented @negative
### Scenario: パースエラー時は例外をスローせず RenderResult.error を返す
- Given WASM モジュールが初期化されている
- When パースエラーになる TCML 文字列で `renderTcml` を呼び出す
- Then 例外がスローされない
- And 戻り値の `result.error` が `ParseErrorInfo` オブジェクトである
- And `result.svg` フィールドが存在しないか `undefined` である

## @not-implemented @negative
### Scenario: ParseErrorInfo が line / column / length / message を持つ
- Given パースエラーになる TCML 文字列がある
- When `renderTcml(text)` を呼び出して `result.error` を取得する
- Then `error.line` が 1 始まりの正の整数である
- And `error.column` が 1 始まりの正の整数 (文字単位) である
- And `error.length` が 0 以上の整数である
- And `error.message` が空でない文字列である (英語固定本文、`cli.md` §パースエラー出力形式 と同じ文言)

## @not-implemented @negative @edge-case
### Scenario: 挿入点エラー (開いた `"` 等) では length=0 が返る
- Given パース時に挿入点エラーとなる TCML (例: 文字列クォート未閉じ) がある
- When `renderTcml(text)` を呼び出す
- Then `result.error.length === 0` である
- And `result.error.line` と `result.error.column` が該当位置を指す
- And 例外はスローされない

## @not-implemented @negative @edge-case
### Scenario: マルチバイト UTF-8 のあとのエラーで column が文字単位で返る
- Given 信号名 `日本語` を含み、その直後の波形文字でパースエラーが起きる TCML がある
- When `renderTcml(text)` を呼び出す
- Then `result.error.column` がバイト位置ではなく文字位置 (例: 信号名 3 文字 + 空白などを文字単位で数えた値) である

## @not-implemented @negative
### Scenario: 不正な fontSize オプションは引き続き例外をスローする
- Given WASM モジュールが初期化されている
- When `renderTcml(text, { fontSize: 0 })` や `{ fontSize: -1 }` や `{ fontSize: NaN }` を呼び出す
- Then パースエラーではなく内部の検証エラーとして JavaScript 例外がスローされる
- And 戻り値経由の `RenderResult` は返らない

## @not-implemented @negative
### Scenario: フォント / レイアウト等の内部処理エラーは引き続き例外をスローする
- Given Canvas API が利用不可など、フォントメトリクスの取得に失敗する状況がある
- When `renderTcml(text)` を呼び出す
- Then JavaScript 例外がスローされる
- And `RenderResult` 経由ではエラーを返さない

## @not-implemented
### Scenario: 成功 → パースエラー → 成功 で `error` / `svg` フィールドが排他的に切り替わる
- Given WASM モジュールが初期化されている
- When 有効な TCML → 不正な TCML → 有効な TCML の順に `renderTcml` を 3 回呼び出す
- Then 1 回目の戻り値は `svg` を持ち `error` を持たない
- And 2 回目の戻り値は `error` を持ち `svg` を持たない
- And 3 回目の戻り値は再度 `svg` を持ち `error` を持たない
- And いずれの呼び出しも例外をスローしない

## @not-implemented @negative
### Scenario: 空文字列の入力
- Given WASM モジュールが初期化されている
- When `renderTcml("")` を呼び出す
- Then 戻り値が `RenderResult` であり、空のタイミングチャートを表す `svg` を持つか、もしくは `error` を持つ
- And どちらの場合も例外はスローされない

## @not-implemented @edge-case
### Scenario: 大量の信号を含む TCML のレンダリング
- Given 100 本の信号を含む TCML テキストがある
- When `render_tcml` を呼び出す
- Then タイムアウトせずに SVG が生成される

## @not-implemented
### Scenario: to_wavejson の基本動作
- Given WASM モジュールが初期化されている
- When `to_wavejson("Clock _~_~")` を呼び出す
- Then 戻り値 `{ json, warnings }` が返る
- And `json` が `{` で始まり `}` を含む有効な JSON である
- And `warnings` が配列である

## @not-implemented
### Scenario: to_wavejson は警告を抜き出す
- Given アンカーが 53 個以上含まれる TCML がある
- When `to_wavejson(text)` を呼び出す
- Then `warnings` が 1 件以上の警告を含む

## @not-implemented @negative
### Scenario: 不正な TCML で to_wavejson が例外をスローする
- Given WASM モジュールが初期化されている
- When 不正な TCML で `to_wavejson` を呼び出す
- Then JavaScript の例外がスローされる

## @not-implemented
### Scenario: extract_tcml_source_from_png が tchart-source iTXt 本文を返す
- Given `tchart-source` keyword の iTXt チャンクに "Clock _~_~" が入った PNG バイト列がある
- When `extract_tcml_source_from_png(bytes)` を呼び出す
- Then 戻り値が "Clock _~_~" と一致する

## @not-implemented
### Scenario: extract_tcml_source_from_png は UTF-8 を保つ
- Given `tchart-source` iTXt に "@title 日本語タイトル\nclk _~" が入った PNG がある
- When `extract_tcml_source_from_png(bytes)` を呼び出す
- Then 戻り値の TCML が元のマルチバイト文字列と一致する

## @not-implemented @negative
### Scenario: tchart-source を持たない PNG では undefined が返る
- Given `tchart-source` iTXt を持たない PNG バイト列がある
- When `extract_tcml_source_from_png(bytes)` を呼び出す
- Then 戻り値が `undefined` である

## @not-implemented @negative
### Scenario: PNG として不正なバイト列では undefined が返る
- Given PNG シグネチャを持たないバイト列がある
- When `extract_tcml_source_from_png(bytes)` を呼び出す
- Then 戻り値が `undefined` である

## @not-implemented
### Scenario: embed_tcml_source_in_png が iTXt を埋め込んだ PNG を返す
- Given 任意の PNG バイト列 (`tchart-source` iTXt を含まない) がある
- When `embed_tcml_source_in_png(bytes, "Clock _~_~")` を呼び出す
- Then 戻り値の `Uint8Array` を `extract_tcml_source_from_png` に渡すと "Clock _~_~" が返る
- And 戻り値の先頭 8 バイトは PNG シグネチャ (`89 50 4E 47 0D 0A 1A 0A`) である

## @not-implemented @negative
### Scenario: PNG として不正なバイト列で embed_tcml_source_in_png が例外をスローする
- Given PNG シグネチャを持たないバイト列がある
- When `embed_tcml_source_in_png(bytes, "Clock _~_~")` を呼び出す
- Then JavaScript の例外がスローされる

---

## 観点A/B 補強: WASM 単独 + 組合せ

### Scenario: render_tcml で SVG ルート要素が NS 付き
- Given WASM 初期化済み、有効 TCML
- When `render_tcml(text)`
- Then 戻り値が `xmlns="http://www.w3.org/2000/svg"` および `xmlns:tchart="http://tchart-rust/1.0"` を含む

### Scenario: render_tcml で fontSize オプション 0 は例外
- Given WASM 初期化済み
- When `render_tcml(text, { fontSize: 0 })`
- Then 例外スロー

### Scenario: render_tcml で fontSize オプション負値は例外
- When `render_tcml(text, { fontSize: -1 })`
- Then 例外

### Scenario: to_wavejson で警告ゼロは空配列
- Given 警告条件のない TCML
- When `to_wavejson(text)`
- Then `warnings.length === 0`

### Scenario: to_wavejson で複数警告 (アンカー超過 + step round)
- Given アンカー 53 個 + 非整数 step
- When `to_wavejson(text)`
- Then `warnings.length >= 2` (各条件 1 件)

### Scenario: to_wavejson 出力 JSON が `JSON.parse` 可能
- Given 任意の有効 TCML
- When `JSON.parse(to_wavejson(text).json)`
- Then 例外なし、object 型を返す

### Scenario: extract_tcml_source で `&lt;` `&gt;` `&amp;` をアンエスケープ
- Given SVG 文字列に `<tchart:source>a &amp;lt; b</tchart:source>`
- When `extract_tcml_source(svg)`
- Then 戻り値が `"a &lt; b"` (1 段アンエスケープ) — 仕様の二重エスケープ確認

### Scenario: embed_tcml_source_in_png で UTF-8 マルチバイト保持
- Given 元 PNG + `source = "@title 日本語"`
- When `embed_tcml_source_in_png(bytes, source)` の戻りを `extract_tcml_source_from_png`
- Then `"@title 日本語"` が完全一致で返る

### Scenario: embed_tcml_source_in_png で既存 iTXt がある場合の追記/上書き挙動
- Given 既に `tchart-source` iTXt を含む PNG
- When `embed_tcml_source_in_png(bytes, "new")`
- Then 仕様に従い追記または上書き (実装で固定)
- And `extract_tcml_source_from_png` の戻り値が新しい "new" になるか、既存値が優先されるか明示

### Scenario: render_tcml × 途中 `@step` × `@clock(pos)` (E2E、回帰)
- Given TCML:
  ```
  @step 10
  @clock(pos) clk
  @step 20
  data ====
  ```
- When `render_tcml(text)`
- Then 例外なく SVG 文字列を返す
- And SVG 中に `clk` 信号の polyline (step=10) と `data` の polyline (step=20) が個別の幅で含まれる

### Scenario: to_wavejson × 途中 `@step` × `@clock(pos)` (回帰)
- Given 同上 TCML
- When `to_wavejson(text)`
- Then `json` に `"wave":"p..."` と `"wave":"=..."` が含まれる
- And `data` 信号の `"period":2` が含まれる

### Scenario: render_tcml で空 TCML
- When `render_tcml("")`
- Then 例外をスローするか、空 SVG (`<svg width="..." height="..."/>`) を返すか — 実装で固定

### Scenario: render_tcml で `@font` 指定の TCML がブラウザ Canvas API でフォント計測される
- Given TCML `@font monospace` を含む
- When `render_tcml(text)` (Canvas API 利用環境)
- Then 出力 SVG の `<text font-family>` が `monospace` を含み、Canvas measureText を使った幅計算が反映される
