# CLI

`tchart` コマンドのインターフェース・ファイル入出力・フォント解決のテスト仕様。
仕様: `docs/spec/cli.md`。

---

## `svg` サブコマンド (単一)

## @not-implemented @smoke
### Scenario: `svg` のデフォルト出力 (入力隣に `<STEM>.svg`)
- Given 有効な TCML ファイル `chart.tc` がある
- When `tchart svg chart.tc` を実行する
- Then `chart.svg` ファイルが生成される
- And 終了コードが `0` である

## @not-implemented @smoke
### Scenario: `svg -o` で出力ファイル指定
- Given 有効な TCML ファイル `chart.tc` がある
- When `tchart svg chart.tc -o output.svg` を実行する
- Then `output.svg` ファイルが生成される
- And 終了コードが `0` である

## @not-implemented
### Scenario: `svg --output` 長フォーム
- Given 有効な TCML ファイル `chart.tc` がある
- When `tchart svg chart.tc --output output.svg` を実行する
- Then `output.svg` ファイルが生成される

## @not-implemented @negative
### Scenario: `svg` に複数入力を指定するとエラー
- Given 2 つの有効な TCML ファイル `a.tc` `b.tc` がある
- When `tchart svg a.tc b.tc` を実行する
- Then 使用方法のエラーメッセージが stderr に出力される
- And 終了コードが `1` である

---

## `png` サブコマンド (単一)

## @not-implemented @smoke
### Scenario: `png` のデフォルト出力 (入力隣に `<STEM>.png`)
- Given 有効な TCML ファイル `chart.tc` がある
- When `tchart png chart.tc` を実行する
- Then `chart.png` ファイルが生成される
- And 終了コードが `0` である

## @not-implemented
### Scenario: `png -o` で出力ファイル指定
- Given 有効な TCML ファイル `chart.tc` がある
- When `tchart png chart.tc -o out.png` を実行する
- Then `out.png` ファイルが生成される
- And 終了コードが `0` である

## @not-implemented @regression
### Scenario: PNG にテキスト (信号名) が描画される
- Given 有効な TCML ファイル `chart.tc` (Clock, Data 信号を含む) がある
- And 有効なフォント (DejaVuSans 等) を `--font` で指定する
- When `tchart png chart.tc --font /path/to/DejaVuSans.ttf -o out.png` を実行する
- Then `out.png` の左帯 (信号名描画領域) に十分な数のダーク画素が存在する
- And `Clock` / `Data` ラベル位置に文字描画がある (resvg/usvg のフォントエイリアス未解決による無音テキストの退行を検出)

## @not-implemented @negative
### Scenario: `png` に複数入力を指定するとエラー
- Given 2 つの有効な TCML ファイル `a.tc` `b.tc` がある
- When `tchart png a.tc b.tc` を実行する
- Then 使用方法のエラーメッセージが stderr に出力される
- And 終了コードが `1` である

---

## `src` サブコマンド (単一、SVG/PNG → TCML 復元)

## @not-implemented @smoke
### Scenario: `src` のデフォルト出力 (SVG → 入力隣に `<STEM>.tc`)
- Given TCML ソースが埋め込まれた SVG ファイル `chart.svg` がある
- When `tchart src chart.svg` を実行する
- Then `chart.tc` ファイルが生成される
- And `chart.tc` の内容が元の TCML テキストと一致する
- And 終了コードが `0` である

## @not-implemented
### Scenario: `src` のデフォルト出力 (PNG → 入力隣に `<STEM>.tc`)
- Given TCML ソースが iTXt チャンクに埋め込まれた PNG ファイル `chart.png` がある
- When `tchart src chart.png` を実行する
- Then `chart.tc` ファイルが生成される
- And `chart.tc` の内容が元の TCML テキストと一致する

## @not-implemented
### Scenario: `src -o` で出力ファイル指定
- Given TCML ソースが埋め込まれた SVG ファイル `chart.svg` がある
- When `tchart src chart.svg -o restored.tc` を実行する
- Then `restored.tc` ファイルが生成される

## @not-implemented
### Scenario: `source` は `src` のエイリアス
- Given TCML ソースが埋め込まれた SVG ファイル `chart.svg` がある
- When `tchart source chart.svg` を実行する
- Then `chart.tc` が生成され、`tchart src` と同じ動作になる

## @not-implemented
### Scenario: XML エスケープされたソースが正しく復元される
- Given 元の TCML に `"<request>" _~_~` が含まれる SVG ファイルがある (信号名中の `<` `>` は文字)
- When `tchart src` で抽出する
- Then 出力に `"<request>" _~_~` がそのまま含まれる (`&lt;` `&gt;` ではない)

## @not-implemented @negative
### Scenario: `src` で TCML ソースが埋め込まれていない SVG
- Given tchart で生成されていない (`<tchart:source>` がない) SVG ファイルがある
- When `tchart src nosource.svg` を実行する
- Then エラーメッセージが stderr に出力される
- And 終了コードが `5` である

## @not-implemented @negative
### Scenario: `src` で不正なファイル形式
- Given テキストファイル `readme.txt` がある
- When `tchart src readme.txt` を実行する
- Then エラーメッセージが stderr に出力される
- And 終了コードが `5` である

## @not-implemented @negative
### Scenario: `src` に複数入力を指定するとエラー
- Given 2 つの埋込済 SVG ファイル `a.svg` `b.svg` がある
- When `tchart src a.svg b.svg` を実行する
- Then 使用方法のエラーメッセージが stderr に出力される
- And 終了コードが `1` である

---

## `batch` サブコマンド (複数入力)

## @not-implemented @smoke
### Scenario: `batch svg` で複数入力を出力ディレクトリにレンダリング
- Given 3 つの有効な TCML ファイル `a.tc` `b.tc` `c.tc` がある
- And ディレクトリ `out/` が存在する
- When `tchart batch svg a.tc b.tc c.tc -o out/` を実行する
- Then `out/a.svg` `out/b.svg` `out/c.svg` の 3 ファイルが生成される
- And 終了コードが `0` である

## @not-implemented @smoke
### Scenario: `batch png` で複数入力を出力ディレクトリにレンダリング
- Given 2 つの有効な TCML ファイル `a.tc` `b.tc` がある
- When `tchart batch png a.tc b.tc -o build/` を実行する
- Then `build/a.png` `build/b.png` の 2 ファイルが生成される
- And 終了コードが `0` である

## @not-implemented
### Scenario: `batch` の出力ディレクトリが存在しない場合は作成する
- Given 有効な TCML ファイル `a.tc` がある
- And ディレクトリ `new_out/` は存在しない
- When `tchart batch svg a.tc -o new_out/` を実行する
- Then `new_out/` が作成される
- And `new_out/a.svg` が生成される

## @not-implemented @negative
### Scenario: `batch` で `-o` 未指定はエラー
- Given 有効な TCML ファイル `a.tc` がある
- When `tchart batch svg a.tc` を実行する
- Then 使用方法のエラーメッセージが stderr に出力される
- And 終了コードが `1` である

## @not-implemented @negative
### Scenario: `batch` で出力 STEM が衝突するとエラー
- Given `dir1/chart.tc` と `dir2/chart.tc` がある (STEM が同じ `chart`)
- When `tchart batch svg dir1/chart.tc dir2/chart.tc -o out/` を実行する
- Then 出力ファイル名衝突のエラーメッセージが stderr に出力される
- And 終了コードが `3` である

## @not-implemented @negative
### Scenario: `batch` のフォーマット引数が不正
- Given 有効な TCML ファイル `a.tc` がある
- When `tchart batch jpeg a.tc -o out/` を実行する
- Then 使用方法のエラーメッセージが stderr に出力される
- And 終了コードが `1` である

## @not-implemented
### Scenario: `batch` で 1 入力でも動作する
- Given 有効な TCML ファイル `a.tc` がある
- When `tchart batch svg a.tc -o out/` を実行する
- Then `out/a.svg` が生成される
- And 終了コードが `0` である

---

## ヘルプ・バージョン

## @not-implemented
### Scenario: ヘルプ表示
- When `tchart --help` を実行する
- Then 使用方法と各サブコマンド (`svg` / `png` / `src` / `batch`) の概要が標準出力に表示される
- And 終了コードが `0` である

## @not-implemented
### Scenario: バージョン表示
- When `tchart --version` を実行する
- Then バージョン文字列が標準出力に表示される
- And 終了コードが `0` である

## @not-implemented @negative
### Scenario: 引数なしで実行
- When `tchart` を引数なしで実行する
- Then 使用方法のヘルプが表示される
- And ゼロ以外の終了コードで終了する

---

## フォント解決 (基本)

詳細なフォント挙動 (multi-CSV / 並列ロード / `batch` キャッシュ共有) は `cli-font.feature.md` を参照。

## @not-implemented
### Scenario: `--font` オプションでフォントファイルを指定
- Given 有効な TCML ファイルと有効なフォントファイルがある
- When `tchart svg chart.tc --font /path/to/font.ttf` を実行する
- Then 指定されたフォントを使用して SVG が生成される
- And 終了コードが `0` である

## @not-implemented
### Scenario: `--font-size` オプションでフォントサイズを指定
- Given 有効な TCML ファイルがある
- When `tchart svg chart.tc --font-size 16.0` を実行する
- Then フォントサイズ 16.0px で SVG が生成される
- And 終了コードが `0` である

## @not-implemented
### Scenario: フォント解決の優先順位 - `--font` オプション優先
- Given `TCHART_FONT` 環境変数が設定されている
- And `--font` オプションでも別のフォントを指定する
- When `tchart svg chart.tc --font /opt/font.ttf` を実行する
- Then `--font` オプションで指定したフォントが使用される

## @not-implemented
### Scenario: フォント解決の優先順位 - 環境変数 `TCHART_FONT`
- Given `TCHART_FONT=/path/to/font.ttf` 環境変数が設定されている
- And `--font` オプションは指定されていない
- When `tchart svg chart.tc` を実行する
- Then `TCHART_FONT` で指定されたフォントが使用される

---

## エラー処理

## @not-implemented @negative
### Scenario: 入力ファイルが存在しない
- Given `notfound.tc` というファイルが存在しない
- When `tchart svg notfound.tc` を実行する
- Then エラーメッセージが stderr に出力される
- And 終了コードが `1` である

## @not-implemented @negative
### Scenario: TCML パースエラー
- Given 不正な TCML 構文を含むファイル `invalid.tc` がある
- When `tchart svg invalid.tc` を実行する
- Then パースエラーのメッセージが stderr に出力される
- And 終了コードが `2` である

## @not-implemented @negative
### Scenario: 出力先ディレクトリが書き込めない
- Given 有効な TCML ファイル `chart.tc` がある
- And 書き込み権限のないディレクトリ `/readonly/` がある
- When `tchart svg chart.tc -o /readonly/out.svg` を実行する
- Then 出力エラーのメッセージが stderr に出力される
- And 終了コードが `3` である

## @not-implemented @negative
### Scenario: フォントが見つからない
- Given `--font` オプションに存在しないパスを指定する
- When `tchart svg chart.tc --font /nonexistent.ttf` を実行する
- Then フォントエラーのメッセージが stderr に出力される
- And 終了コードが `4` である

## @not-implemented @edge-case
### Scenario: 入力ファイルと出力ファイルが同じパス
- Given `chart.tc` という TCML ファイルがある
- When `tchart svg chart.tc -o chart.tc` を実行する
- Then エラーまたは警告が出力される

---

## 観点A 補強: CLI 単独仕様

### Scenario: `svg` でフォント未指定 + OS 自動検出成功
- Given 有効な TCML、`--font` `TCHART_FONT` 共に未指定、OS にフォントあり
- When `tchart svg chart.tc` 実行
- Then 自動検出フォントで描画、終了コード 0

### Scenario: `svg --font-size 24`
- Given 有効な TCML
- When `tchart svg chart.tc --font-size 24`
- Then SVG 内の `<text font-size>` が 24 (チャート全体)

### Scenario: `--font-size 0` はエラー
- Given 有効な TCML
- When `tchart svg chart.tc --font-size 0`
- Then 終了コード 1 (使用方法エラー)

### Scenario: `--font-size` 負値はエラー
- Given 有効な TCML
- When `tchart svg chart.tc --font-size -1`
- Then 終了コード 1

### Scenario: `--font` 存在しないファイル
- Given 有効な TCML
- When `tchart svg chart.tc --font /nonexistent.ttf`
- Then 終了コード 4 (フォントエラー)

### Scenario: 入力ファイル拡張子が `.tc` でない (例 `.txt`) でも受理
- Given `chart.txt` (中身は有効 TCML)
- When `tchart svg chart.txt`
- Then 終了コード 0、`chart.svg` 生成

### Scenario: 入力 STDIN 経由 (パイプ) は対応するか? (要仕様)
- Given `cat chart.tc | tchart svg -`
- Then 仕様で受理されるか拒否されるかを固定 (現仕様明記なし → 拒否が現状?)

### Scenario: `-o` 出力先が既存ファイル (上書き)
- Given `chart.tc` と既存 `out.svg`
- When `tchart svg chart.tc -o out.svg`
- Then `out.svg` が上書きされる、終了コード 0

### Scenario: `-o` 出力先がディレクトリ (svg は単一入力なのでエラー)
- Given `chart.tc` と既存ディレクトリ `out_dir/`
- When `tchart svg chart.tc -o out_dir/`
- Then 終了コード 3 (出力エラー、ファイルでなくディレクトリ)

### Scenario: `batch` のディレクトリが存在しない場合は作成
- Given `a.tc` `b.tc` と存在しない `out/`
- When `tchart batch svg a.tc b.tc -o out/`
- Then `out/` が新規作成され、`out/a.svg` `out/b.svg` が出力される

### Scenario: `batch` で同名 STEM 入力衝突 (`a.tc` × 2 回)
- Given 同じパスを 2 回指定 `tchart batch svg a.tc a.tc -o out/`
- Then 終了コード 3 (衝突)

### Scenario: `batch` で異なるディレクトリの同名 STEM (`dir1/a.tc` `dir2/a.tc`)
- Given `dir1/a.tc` と `dir2/a.tc`
- When `tchart batch svg dir1/a.tc dir2/a.tc -o out/`
- Then 終了コード 3 (両者の STEM が衝突)

### Scenario: `batch png` と `batch svg` の混在不可
- Given `tchart batch svg a.tc b.tc png c.tc -o out/`
- Then 終了コード 1 (使用方法エラー、第 1 引数のフォーマットは 1 つ)

### Scenario: `wavedrom` で `--font-size` 指定はエラー
- Given 有効 TCML
- When `tchart wavedrom chart.tc --font-size 16`
- Then 終了コード 1 (wavedrom はフォント関連オプションを受け付けない)

### Scenario: `src` で SVG 中に `<tchart:source>` が複数ある場合
- Given `<tchart:source>` を 2 個含む不正 SVG
- Then 最初の 1 個を抽出するか、エラーにするか — 仕様で固定

### Scenario: `src` で PNG の `tchart-source` iTXt が複数チャンクある場合
- Given iTXt を 2 個含む PNG
- Then 最初の 1 個を抽出 (PNG 仕様: テキストチャンクは複数許可)

---

## 観点B 補強: CLI 組合せ

### Scenario: `svg` 入力で TCML 中に `@font` を含み解決失敗 → 警告 + 出力
- Given TCML に `@font UnknownFamily`、`tchart svg`
- Then SVG 出力、stderr 警告 1 行、終了コード 0

### Scenario: `png` 入力で `@font` を含み警告 + 出力
- Given TCML に `@font UnknownFamily`、`tchart png`
- Then PNG 出力、stderr 警告 1 行、終了コード 0、PNG iTXt に元 TCML 埋込

### Scenario: `batch svg` 1 件パースエラーで他は成功
- Given `good.tc` と `broken.tc` を `batch svg good.tc broken.tc -o out/`
- Then `good.svg` は生成、`broken.svg` は生成されない
- And 終了コードは 2 (1 件以上のパースエラー) または 0 (個別失敗を許容) — 要仕様

### Scenario: `batch` で並列度が論理コア数を超えない
- Given 100 件入力、論理コア数 N
- Then 並列実行ワーカ数 ≤ N

### Scenario: `wavedrom` 入力に途中 `@step` × `@clock(auto)` を含む (回帰)
- Given TCML:
  ```
  @step 10
  @clock(pos) clk
  @step 20
  data ====
  ```
- When `tchart wavedrom chart.tc -o out.json`
- Then `out.json` が strict JSON、`clk.wave == "p..."`、`data.wave == "=..."`
- And gcd 計算で `data.period = 2`
- And 終了コード 0

## ユーザ承認反映シナリオ (2026-05-10)

### Scenario: `tchart wavedrom --quiet` で警告を抑制
- Given `step` round 警告が発生する入力 (例: `@step 2.4`)
- When `tchart wavedrom in.tc -q -o out.json`
- Then stderr が空
- And out.json は警告なし時と同一
- And 終了コード 0

### Scenario: `tchart wavedrom -q` 短縮形
- Given アンカー数 53 個 (52 超過警告対象) の入力
- When `tchart wavedrom in.tc -q -o out.json`
- Then stderr が空 (警告抑制)

### Scenario: `tchart wavedrom` (--quiet なし) は警告を出す (B-17 回帰)
- Given `step` round 警告が発生する入力
- When `tchart wavedrom in.tc -o out.json`
- Then stderr に警告 1 行以上

### Scenario: `tchart src` データなしは終了コード 1
- Given 入力 SVG に `<tchart:source>` 要素がない
- When `tchart src in.svg`
- Then 終了コード 1
- And stderr に「埋め込み TCML が見つかりません」等のメッセージ

### Scenario: `tchart src` 不正ファイルも終了コード 1
- Given 入力ファイルが SVG/PNG として不正な形式
- When `tchart src in.bin`
- Then 終了コード 1

---

## 観点E 補強: CLI / Web 跨ぎ round-trip / 優先順位

### Scenario: SVG round-trip 完全一致 (`tchart svg` → `tchart src`)
- Given 任意の有効 TCML 入力
- When `tchart svg in.tc -o out.svg` 後 `tchart src out.svg -o restored.tc`
- Then `restored.tc == in.tc` (byte 一致)

### Scenario: PNG round-trip で UTF-8 信号名復元
- Given 信号名 `日本語` を含む TCML
- When `tchart png in.tc -o out.png` 後 `tchart src out.png -o restored.tc`
- Then 復元 TCML 中で信号名が `日本語` で復元される (mojibake なし)

### Scenario: PNG round-trip で `<` `>` `&` を含む TCML が復元される
- Given TCML 中に信号名 `"<a>&<b>"` などを持つ
- When PNG round-trip
- Then 復元 TCML が元と一致

### Scenario: CLI `--font` と TCML 内 `@font` の優先順位 (CLI 優先)
- Given TCML に `@font Helvetica` を含み、CLI で `--font /tmp/Roboto.ttf` を指定
- When `tchart svg --font /tmp/Roboto.ttf in.tc`
- Then 信号名描画に Roboto が使われる (Helvetica は使われない)

### Scenario: CLI `--font` 未指定で TCML 内 `@font` のみ
- Given TCML に `@font Helvetica`、CLI で `--font` 指定なし
- When `tchart svg in.tc`
- Then Helvetica が family 解決経由で使われる

### Scenario: TCHART_FONT 環境変数より TCML `@font` が優先
- Given 環境変数 `TCHART_FONT=/tmp/Sans.ttf`、TCML に `@font Helvetica`
- When `tchart svg in.tc`
- Then Helvetica が使われる (環境変数フォントは default fallback 用途)

### Scenario: CLI の `--font-size` と TCML `@fontsize` の優先順位
- Given TCML `@fontsize 12`、CLI `--font-size 24`
- Then CLI 値 24 が使われる (CLI 優先)

### Scenario: round-trip 中に `@scale` も保持される
- Given `@scale 2.0\nA _~`
- When SVG round-trip
- Then 復元 TCML に `@scale 2.0` が含まれる

### Scenario: PNG iTXt 埋め込みが連続 round-trip で劣化しない
- Given TCML 入力
- When PNG round-trip を 3 回繰り返す
- Then 3 回目の復元 TCML も初回と一致

---

## 観点K: Help / SVG 抽出 (`tchart src`) の負ケース・大入力 round-trip

## @not-implemented
### Scenario: SVG 中に `<tchart:source>` が複数 (重複定義) → 先頭採用 or エラー
- Given SVG ファイル中に `<tchart:source>` 要素が 2 個 (異なる本文)
- When `tchart src dup.svg`
- Then 先頭要素本文を採用 or エラー終了 (仕様準拠)
- And stderr に "multiple <tchart:source> elements" 等の警告 (採用採択時)

## @not-implemented
### Scenario: PNG iTXt が複数チャンク (`tchart-source` 重複) → 先頭採用 or エラー
- Given PNG ファイル中に `tchart-source` iTXt チャンクが 2 個
- When `tchart src dup.png`
- Then 先頭チャンク本文を採用 or エラー終了 (仕様準拠)
- And 警告メッセージで重複チャンクが報告される

## @not-implemented
### Scenario: 巨大 TCML (10KB) の SVG round-trip
- Given 10KB の TCML 入力 (信号 100 行 × 各 100 unit)
- When `tchart svg in.tc -o out.svg && tchart src out.svg -o restored.tc`
- Then `restored.tc` が `in.tc` と byte 単位で一致

## @not-implemented
### Scenario: 巨大 TCML (100KB) の SVG round-trip
- Given 100KB の TCML 入力 (信号 1000 行 × 各 100 unit)
- When SVG round-trip
- Then 復元一致、かつ実行時間が現実的 (タイムアウトしない)

## @not-implemented
### Scenario: 巨大 TCML (10KB) の PNG round-trip で iTXt 完全保持
- Given 10KB TCML 入力
- When `tchart png` → `tchart src`
- Then 復元 TCML が一致

## @not-implemented
### Scenario: 不正な base64 / PNG 破損 を含む iTXt 拒否
- Given PNG ファイルの `tchart-source` iTXt チャンクが圧縮失敗 / 不正バイト列
- When `tchart src corrupt.png`
- Then 終了コード 1 (またはエラー)、stderr に "failed to decode tchart-source" 等

## @not-implemented
### Scenario: `<tchart:source>` の namespace `xmlns:tchart` が間違っている SVG
- Given SVG に `<tchart:source xmlns:tchart="http://wrong">` (公式 URI と異なる)
- When `tchart src wrongns.svg`
- Then 終了コード 1 (namespace 不一致でソース未検出扱い)

## @not-implemented
### Scenario: `<tchart:source>` が空要素 (`<tchart:source/>`)
- Given SVG に `<tchart:source/>` (子テキストなし)
- When `tchart src empty.svg`
- Then 復元 TCML が空文字列 (出力 0 byte) または "empty source" エラー

### Scenario: `tchart src` で stdout 出力をサポートする (`-o -`)
- Given `src` サブコマンドに `-o -` を指定する (単一ハイフン)
- When `tchart src in.svg -o -`
- Then 復元 TCML が stdout に書き出される (ファイルには書き込まない)
- And 終了コード 0
- Note: 本仕様で stdout モードを規定するのは `src` のみ (`docs/spec/cli.md` §`-` (単一ハイフン) で標準出力を指定する)

## @not-implemented
### Scenario: SVG round-trip で `<tchart:source>` の文字参照が二重エスケープされない
- Given TCML に `&amp;` を含む文字列
- When SVG round-trip
- Then 復元 TCML 中の `&amp;` は元と同じ (`&amp;amp;` に二重エスケープされない)

## @not-implemented
### Scenario: PNG round-trip で 改行 LF / CRLF が保持される
- Given TCML 改行が CRLF 混在
- When PNG round-trip
- Then 復元 TCML の改行コードは元と byte 一致

---

## 観点Q 補強: CLI batch 並列性

## @not-implemented
### Scenario: 1000 ファイル入力で出力順序保持
- Given 1000 個の TCML ファイル `f0001.tc` .. `f1000.tc`
- When `tchart svg f0001.tc f0002.tc ... f1000.tc -o out/`
- Then 出力 `out/f0001.svg` .. `out/f1000.svg` が全て生成
- And ログ順序 (stdout/stderr) が入力順と一致 (並列実行でも順序保持)

## @not-implemented
### Scenario: フォントキャッシュが複数入力で共有される
- Given 100 ファイル全てに `@font Roboto`
- When 1 コマンドで一括処理
- Then `Roboto` の load は 1 回のみ (fontdb / cache hit)
- And 警告 "font 'Roboto' not found" も 1 回のみ (もし出るなら)

## @not-implemented
### Scenario: 1 ファイル失敗が他に伝播しない (per-input failure 集約)
- Given 10 ファイル中 3 番目だけパースエラー
- When 一括処理
- Then 3 番目のみ失敗、残り 9 ファイルは成功して SVG 出力
- And exit code が non-zero (集約失敗) だが他出力は正常

## @not-implemented
### Scenario: 100 ファイル並列実行でも fontdb 再構築 1 回
- Given 100 ファイル全てに同じ `@font` 指定
- When 並列度 N で実行 (環境変数 / オプション)
- Then fontdb が thread-safe に共有され、再構築は 1 回
- Note: 仕様で並列度を定義していなければ spec gap

## @not-implemented
### Scenario: 入力ファイル数 0 でエラー
- Given `tchart svg` (引数なし)
- Then exit code != 0、"no input" 系メッセージ

## @not-implemented
### Scenario: 同一入力ファイルを 2 回指定
- Given `tchart svg in.tc in.tc -o out.svg out2.svg`
- Then 2 回処理して 2 つ生成 (重複排除しない)
- Note: 仕様確認

## @not-implemented
### Scenario: 1000 ファイル中 全失敗で exit code が non-zero
- Given 1000 ファイル全てパースエラー
- Then exit code != 0、stderr に 1000 件のエラーログ

---

## 観点S 補強: CLI round-trip (詳細)

## @not-implemented
### Scenario: SVG round-trip 3 周で初回 SVG と byte 一致
- Given TCML X
- When `tchart svg` → `tchart src` → `tchart svg` → `tchart src` → `tchart svg`
- Then 3 回目 SVG が 1 回目と byte 単位で一致
- And TCML 復元結果も常に X と一致

## @not-implemented
### Scenario: PNG round-trip 3 周で byte 一致
- Given TCML X (UTF-8)
- When PNG round-trip 3 回
- Then 3 回目 PNG が 1 回目と byte 単位で一致

## @not-implemented
### Scenario: CRLF 改行 TCML を SVG round-trip で改行コード保持
- Given TCML X が CRLF 改行
- When SVG round-trip
- Then 復元 TCML が CRLF 改行のまま (LF に正規化しない)

## @not-implemented
### Scenario: CR のみ (旧 Mac) 改行 TCML の round-trip
- Given TCML X が CR 改行
- When SVG round-trip
- Then 仕様で CR を許可するなら byte 一致、許可しないなら parser エラー
- Note: 仕様未定義なら spec gap

## @not-implemented
### Scenario: TCML 末尾改行ありの round-trip
- Given TCML X 末尾 LF あり
- When SVG round-trip
- Then 復元 TCML 末尾 LF が保持される

## @not-implemented
### Scenario: TCML 末尾改行なしの round-trip
- Given TCML X 末尾 LF なし
- When SVG round-trip
- Then 復元 TCML 末尾 LF が付与されない (byte 一致)

## @not-implemented
### Scenario: BOM 付き TCML の round-trip
- Given TCML X 先頭 BOM (U+FEFF) あり
- When SVG round-trip
- Then BOM 保持するか除去するかは仕様準拠
- Note: 仕様未定義なら spec gap

---

## パースエラー出力形式 (rustc 風)

`docs/spec/cli.md` §パースエラー出力形式 の検証。`svg` / `png` / `wavedrom` / `batch` で TCML パースエラー (終了コード 2) を stderr に出すときのフォーマットを規定する。

### 1 件のエラーの基本フォーマット

## @not-implemented @smoke
### Scenario: 単一エラーが rustc 風 4 構成要素で出力される
- Given `@step xyz` のみを含む TCML ファイル `sample.tc` (`xyz` は line=1, col=7 から 3 文字)
- When `tchart svg sample.tc` を実行する
- Then stderr の出力に以下の構成要素がこの順で 1 つずつ現れる
  - 1 行目: `error: ` で始まる error 本文行 (英語、文末にピリオド `.` を含まない)
  - 2 行目: ` --> sample.tc:1:7` の位置行
  - 3 行目: `1 | @step xyz` のスニペット行 (行番号と `|` と 該当行の本文)
  - 4 行目: `  |       ^^^` で始まるキャレット行 (`^` が 3 文字、length=3 と一致)
- And 終了コードが `2` である

## @not-implemented
### Scenario: error 行がピリオドで終わらない
- Given パースエラーを起こす TCML ファイル
- When `tchart svg` でレンダリングする
- Then stderr の最初の error 行が `.` で終わっていない

### 位置情報 (LINE / COL)

## @not-implemented
### Scenario: LINE / COL は 1 始まりで出力される
- Given TCML の 3 行目 7 桁目から始まるトークンがパースエラーになるファイル `sample.tc`
- When `tchart svg sample.tc` を実行する
- Then 位置行が ` --> sample.tc:3:7` である (line も col も 1 始まり)
- And スニペット行の左カラムに `3 | ` が出る (LINE と一致)

## @not-implemented
### Scenario: 1 行目 1 桁目のエラーで line=1, col=1
- Given TCML の冒頭 (line=1, col=1) からエラーになるファイル `sample.tc`
- When `tchart svg sample.tc` を実行する
- Then 位置行が ` --> sample.tc:1:1` である
- And スニペット行の左カラムに `1 | ` が出る

## @not-implemented
### Scenario: 行番号の桁が変わると左カラム `|` の桁位置も合う
- Given 100 行以上の TCML で、line=5 と line=120 のエラーが同一実行内で出るケース
- When `tchart svg` を実行する
- Then 各エラーエントリ内では行番号の桁数に応じて左カラム `|` の桁位置が揃う (line=5 のエントリ内では `5 |`、line=120 のエントリ内では `120 |` とパディング)
- Note: 仕様 §パースエラー出力形式「左カラムの `|` の桁位置は揃える」の検証。スニペット行とキャレット行は同一エントリ内で `|` 桁位置が一致する

### キャレット長

## @not-implemented @smoke
### Scenario: キャレット長 = length (length >= 1)
- Given `@step xyz` (`xyz` が length=3 のエラー範囲) のファイル
- When `tchart svg` を実行する
- Then キャレット行に `^` が 3 個連続で出る (length=3 と一致)

## @not-implemented
### Scenario: キャレット長 = 1 文字のとき `^` 1 個
- Given パースエラーの範囲が 1 文字のケース (例: `?` を信号行先頭で使う `DontCareWithoutAnchor`、length=1)
- When `tchart svg` を実行する
- Then キャレット行に `^` が 1 個だけ出る

## @not-implemented @edge-case
### Scenario: length == 0 (挿入位置エラー) は `^` 1 文字のみ
- Given `SigA "hello` のように `"` が行末まで閉じない TCML (`UnclosedQuote` で length=0、col は行末の挿入点を指す)
- When `tchart svg` を実行する
- Then キャレット行に `^` が 1 文字だけ出る (length=0 でも 1 文字で挿入点を可視化)
- And 終了コードが `2` である

### スニペット行

## @not-implemented
### Scenario: スニペットは該当行をそのまま 1 行出す
- Given エラー行が `  @step xyz` (先頭にスペース 2 個含む) の TCML
- When `tchart svg` を実行する
- Then スニペット行に `  @step xyz` がそのまま (先頭スペース込みで) 含まれる

## @not-implemented
### Scenario: タブを含む行は 4 スペースに展開して出す
- Given エラー行に `\t` を含む TCML (例: `\t@step xyz`)
- When `tchart svg` を実行する
- Then スニペット行ではタブ 1 個が空白 4 個に展開されて出力される
- And 位置行の `<COL>` および キャレット列もタブ展開後の桁を指す

### 標準入力経由

## @not-implemented
### Scenario: 標準入力経由のとき、ファイル名は `<stdin>`
- Given 不正な TCML を標準入力にパイプして `tchart` を起動するシナリオ (stdin 入力をサポートする場合)
- When stdin 経由でパースエラーが起きる
- Then 位置行のファイル名部分が `<stdin>` (山括弧込み 7 文字) と出る
- And 終了コードが `2` である
- Note: stdin 入力サポート自体が未実装の場合はこのシナリオは保留 (spec gap)

### 複数エラー

## @not-implemented
### Scenario: 複数エラー連結 (全件 rustc 風で出力)
- Given 3 件のパースエラーを含む TCML
- When `tchart svg` を実行する
- Then stderr に 3 件のエラーがいずれも rustc 風 4 構成要素フォーマットで出力される
- And 各エラーエントリは空行で区切られる
- And 終了コードが `2` である

## @not-implemented
### Scenario: 複数エラー要約 (最初の 1 件 + 件数行)
- Given 5 件のパースエラーを含む TCML
- When `tchart svg` を実行する (実装が要約モードを選んだ場合)
- Then 最初の 1 件は rustc 風フォーマットで出る
- And 最後に `error: aborting due to 5 previous errors` のような件数行が 1 行で添えられる (rustc 風、英語固定)
- And 終了コードが `2` である
- Note: 連結 / 要約のどちらを採るかは実装裁量。最低保証として「最初の 1 件はフォーマット」「件数が何らかの形で分かる」

## @not-implemented @smoke
### Scenario: 複数エラーでも最初の 1 件は必ず rustc 風フォーマット
- Given 任意件数 (2 件以上) のパースエラーを含む TCML
- When `tchart svg` を実行する
- Then stderr の先頭エントリは error 行 / `-->` 位置行 / スニペット / キャレット の 4 構成要素を満たす
- And 終了コードが `2` である

## @not-implemented
### Scenario: 単一エラーでも全体エラー件数が 1 件であることが分かる出力にする必要はない
- Given 1 件のみのパースエラー TCML
- When `tchart svg` を実行する
- Then stderr に件数行 (`error: aborting due to 1 previous error` 等) は出ても出なくてもよい
- Note: 件数行は複数エラー要約モードのときの最低保証であり、単件時の付与は実装裁量

### 対象サブコマンド

## @not-implemented
### Scenario: `png` でもパースエラーは同フォーマット
- Given パースエラー TCML
- When `tchart png sample.tc -o out.png` を実行する
- Then stderr に rustc 風フォーマットでエラーが出る
- And 終了コードが `2` である
- And `out.png` は生成されない

## @not-implemented
### Scenario: `wavedrom` でもパースエラーは同フォーマット
- Given パースエラー TCML
- When `tchart wavedrom sample.tc` を実行する
- Then stderr に rustc 風フォーマットでエラーが出る
- And 終了コードが `2` である

## @not-implemented
### Scenario: `batch` の入力 1 件でパースエラー
- Given 3 つの TCML ファイル (うち `bad.tc` だけパースエラー)
- When `tchart batch svg good1.tc bad.tc good2.tc -o out/` を実行する
- Then stderr の `bad.tc` 起因のエラーエントリで位置行が ` --> bad.tc:<LINE>:<COL>` と出る
- And `good1.svg` / `good2.svg` は生成される (per-input failure 集約)
- And 終了コードが `2` である

## @not-implemented
### Scenario: `batch` の複数入力で別々にパースエラー
- Given 3 つの TCML ファイル `a.tc` / `b.tc` / `c.tc` が全てパースエラー
- When `tchart batch svg a.tc b.tc c.tc -o out/` を実行する
- Then stderr に 3 つのエラーエントリが出て、それぞれの位置行に対応するファイル名 (`a.tc` / `b.tc` / `c.tc`) が出る
- And 終了コードが `2` である

### このフォーマットの対象外 (現状維持)

## @not-implemented @negative
### Scenario: ファイル不存在 (終了コード 1) は rustc 風フォーマットの対象外
- Given `notfound.tc` というファイルが存在しない
- When `tchart svg notfound.tc` を実行する
- Then stderr に rustc 風フォーマット (`error:` / ` --> ` / スニペット / キャレット) は出ない
- And 従来の 1 行メッセージ形式が出る
- And 終了コードが `1` である

## @not-implemented @negative
### Scenario: 出力エラー (終了コード 3) は rustc 風フォーマットの対象外
- Given 有効な TCML と書き込めない出力先 `/readonly/out.svg`
- When `tchart svg chart.tc -o /readonly/out.svg` を実行する
- Then stderr に rustc 風フォーマットは出ない
- And 従来の 1 行メッセージ形式が出る
- And 終了コードが `3` である

## @not-implemented @negative
### Scenario: フォントエラー (終了コード 4) は rustc 風フォーマットの対象外
- Given デフォルトフォントの解決にすべて失敗する環境
- When `tchart svg chart.tc` を実行する (フォント解決が必須な経路)
- Then stderr に rustc 風フォーマットは出ない
- And 従来の 1 行メッセージ形式が出る
- And 終了コードが `4` である

### 終了コード

## @not-implemented @smoke
### Scenario: パースエラー時の終了コードは 2 (rustc 風フォーマット導入後も変わらない)
- Given パースエラー TCML
- When `tchart svg sample.tc` を実行する
- Then 終了コードが `2` である (1 / 3 / 4 ではない)

### 組み合わせ / エッジケース

## @not-implemented @edge-case
### Scenario: TCML が空ファイルでフォーマット影響なし
- Given 空ファイル `empty.tc` (パース成功で空ドキュメント)
- When `tchart svg empty.tc` を実行する
- Then stderr に rustc 風フォーマットのエラーは出ない
- And 終了コードが `0` である (空はエラーではない)

## @not-implemented @edge-case
### Scenario: TCML の最終行末尾エラー (末尾 LF なし)
- Given 末尾改行なしの TCML で最終行末尾の `"` が閉じない `UnclosedQuote`
- When `tchart svg` を実行する
- Then 位置行の `<LINE>` は最終行の番号、`<COL>` は行末の挿入点を指す
- And キャレット行は length=0 で `^` 1 文字のみ
- And 終了コードが `2` である

## @not-implemented @edge-case
### Scenario: CRLF 改行の TCML でもパースエラーフォーマットが壊れない
- Given CRLF 改行で 3 行目にパースエラーがある TCML
- When `tchart svg` を実行する
- Then 位置行が ` --> sample.tc:3:<COL>` と出る (CRLF でも line 番号が正しい)
- And スニペット行に CR が混入しない (改行コード正規化済みで表示される)

## @not-implemented @edge-case
### Scenario: マルチバイト信号名でも col / length が文字単位で揃う
- Given 日本語の信号名を含む TCML (例: `同期信号 _~_~`) で、その行のエラートークンがマルチバイト文字内にある
- When `tchart svg` を実行する
- Then 位置行 `<COL>` は文字単位の桁を指す (UTF-8 byte offset ではない)
- And キャレット行の `^` 個数は length (文字単位) と一致
- And スニペット中のマルチバイト文字はそのまま (壊れずに) 表示される
