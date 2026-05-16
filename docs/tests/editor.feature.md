# Web エディタ

ブラウザ上の TCML エディタと SVG プレビューのテスト仕様。

---

## @implemented @smoke
### Scenario: 初期表示でサンプル TCML がプレビューされる
- Given エディタページを開く
- When WASM モジュールの初期化が完了する
- Then 左ペインにサンプル TCML テキストが表示される
- And 右ペインにサンプル TCML の SVG プレビューが表示される

## @implemented
### Scenario: TCML 編集でプレビューが自動更新される
- Given エディタが初期化済みである
- When 左ペインのテキストを `Clock _~_~` に変更する
- Then 300ms のデバウンス後に右ペインの SVG が更新される
- And 更新後の SVG に "Clock" のテキスト要素が含まれる

## @implemented
### Scenario: デバウンスにより連続入力で再レンダリングが抑制される
- Given エディタが初期化済みである
- When 100ms 間隔で 5 回テキストを変更する
- Then `render_tcml` の呼び出しは最後の変更から 300ms 後の 1 回のみ実行される

## @implemented @negative
### Scenario: パースエラー時はステータス行に表示しプレビューは直前 SVG を残す
- Given 有効な SVG がプレビューに表示されている
- When 左ペインに不正な TCML テキストを入力する
- Then ステータス行 (`#status`) に "Parse error: ..." が赤字で表示される
- And 右ペインのプレビューは直前の有効な SVG をそのまま保持する

## @implemented
### Scenario: パースエラーからの復帰でステータス行のエラーがクリアされる
- Given パースエラー状態でステータス行にエラーが表示されている
- When 左ペインを有効な TCML テキストに修正する
- Then ステータス行のエラー表示がクリアされる
- And 右ペインに新しい SVG プレビューが表示される

## @implemented
### Scenario: SVG ダウンロード
- Given 有効な SVG がプレビューに表示されている
- When "Save SVG" ボタンをクリックする
- Then `tchart.svg` というファイル名で SVG ファイルがダウンロードされる
- And ダウンロードされたファイルの内容がプレビューの SVG と一致する

## @implemented
### Scenario: PNG ダウンロード
- Given 有効な SVG がプレビューに表示されている
- When "Save PNG" ボタンをクリックする
- Then `tchart.png` というファイル名で PNG ファイルがダウンロードされる
- And PNG の解像度は SVG の幅・高さの 4 倍である

## @implemented @negative
### Scenario: 一度も成功レンダリングしていない初期状態ではダウンロードボタンが無効
- Given 初回レンダリング前で `currentSvg` が一度もセットされていない
- When "Save SVG" ボタンの状態を確認する
- Then "Save SVG" ボタンが無効 (disabled) である
- And "Save PNG" ボタンが無効 (disabled) である

## @implemented
### Scenario: パースエラー中でも前回の有効 SVG が残っていれば Save SVG/PNG は有効
- Given 有効な SVG がプレビューに表示されている
- When 左ペインに不正な TCML テキストを入力する
- Then "Save SVG" ボタンは有効 (enabled) のままである
- And "Save PNG" ボタンは有効 (enabled) のままである

## @implemented
### Scenario: 2 分割レイアウトが正しく表示される
- Given エディタページを開く
- When ウィンドウ幅が 1024px 以上である
- Then 左ペイン (エディタ) と右ペイン (プレビュー) が横並びで表示される
- And 各ペインがウィンドウの約半分の幅を占める

## @implemented
### Scenario: textarea でタブキー入力ができる
- Given エディタの textarea にフォーカスがある
- When Tab キーを押す
- Then フォーカスが移動せず、カーソル位置にタブ文字が挿入される

## @implemented @edge-case
### Scenario: 空文字入力でプレビューが適切に処理される
- Given エディタが初期化済みで、直前に有効な SVG が表示されている
- When 左ペインのテキストをすべて消す
- Then ステータス行にパースエラーが表示されるか、空の SVG が右ペインに表示される
- And 直前の有効な SVG が残っている場合は右ペインのプレビューは破壊されない
- And アプリケーションがクラッシュしない

## @not-implemented
### Scenario: WaveDrom ボタンで JSON ダウンロード
- Given 有効な TCML がエディタに入力されている
- When "WaveDrom" ボタンをクリックする
- Then `to_wavejson` 呼び出しが行われる
- And `tchart.json` というファイル名で JSON ファイルがダウンロードされる
- And ダウンロードされた内容が `to_wavejson` の戻り値 `json` と一致する

## @not-implemented @negative
### Scenario: 現在のテキストがパースエラーの間は WaveDrom ボタンが無効
- Given 現在のエディタ内容がパースに失敗している
- When "WaveDrom" ボタンの状態を確認する
- Then "WaveDrom" ボタンが無効 (disabled) である
- And 直前の有効な SVG が右ペインに残っていても無効状態である

## @not-implemented
### Scenario: WaveDrom 変換時の警告がステータス行に表示される
- Given アンカーが 53 個以上ある TCML がエディタに入力されている
- When "WaveDrom" ボタンをクリックする
- Then ダウンロードは正常に行われる
- And ステータス行に警告メッセージ (件数または本文) が表示される

## @not-implemented
### Scenario: Help ボタンでモーダルが開く
- Given エディタが初期化済みである
- When "Help" ボタンをクリックする
- Then モーダルダイアログが表示される
- And モーダル内の `<iframe>` の `src` 属性が `./help/tcml-format.html` で終わる

## @not-implemented
### Scenario: Help モーダルが Esc キー / 背景クリック / × ボタンで閉じる
- Given Help モーダルが開いている
- When Esc キーを押す、または背景をクリックする、または `×` ボタンを押す
- Then モーダルが閉じ、`<iframe>` が DOM から取り除かれるかまたは hidden 状態になる

## @not-implemented
### Scenario: Help モーダルが多重に開かれない
- Given Help モーダルが既に開いている
- When 再度 "Help" ボタンをクリックする
- Then モーダル DOM はそのまま 1 つだけ存在する

## @not-implemented
### Scenario: Load ボタンで SVG を選ぶと中身判定で wasm の SVG 抽出が呼ばれる
- Given `<tchart:source>` を含む SVG ファイルが Load ボタン経由で選択された
- When editor がファイル先頭 8 バイトを検査する
- Then PNG シグネチャと一致しないので SVG として扱われ、wasm の `extract_tcml_source(svg)` が呼ばれる
- And textarea の内容が SVG から抽出された TCML に置換される
- And status 行が "Loaded <ファイル名>" になる
- And プレビューが新しい TCML で再描画される

## @not-implemented
### Scenario: Load ボタンで PNG を選ぶと中身判定で wasm の PNG 抽出が呼ばれる
- Given `tchart-source` iTXt を含む PNG ファイルが Load ボタン経由で選択された
- When editor がファイル先頭 8 バイトを検査する
- Then PNG シグネチャと一致するので PNG として扱われ、wasm の `extract_tcml_source_from_png(bytes)` が呼ばれる
- And textarea の内容が PNG iTXt から取り出した TCML に置換される
- And プレビューが再描画される

## @not-implemented @negative
### Scenario: 抽出できないファイルでは textarea が変更されない
- Given `<tchart:source>` も `tchart-source` iTXt も含まないファイルが Load ボタン経由で選択された
- When editor が wasm の対応関数を呼ぶ
- Then wasm 関数が `undefined` を返す
- And textarea は変更されない
- And status 行に "Load failed: ..." とエラーが表示される

## @not-implemented
### Scenario: Save PNG が tchart-source iTXt チャンクを埋め込む
- Given 有効な TCML "Clock _~_~" がエディタに入力されている
- When "Save PNG" ボタンをクリックする
- Then editor は Canvas が生成した PNG バイト列を wasm の `embed_tcml_source_in_png(bytes, source)` に渡し、戻り値の Uint8Array をダウンロードする
- And その Uint8Array を `extract_tcml_source_from_png` に渡すと "Clock _~_~" が返る

## @not-implemented
### Scenario: Save PNG → Load PNG のラウンドトリップ
- Given 有効な TCML "@title 日本語\nclk _~" がエディタに入力されている
- When Save PNG で得たファイルを Load PNG で読み込む
- Then textarea の内容が元の TCML と一致する

## @not-implemented
### Scenario: toolbar 右端にプライバシー注記が表示される
- Given エディタが初期化済みである
- When `navigator.language` が `ja*` の場合
- Then `#privacy-note` のテキストが「このページは外部に情報を送信しません」になる
- And それ以外の場合は "No data is sent externally" になる

---

## 観点A/B 補強: editor 単独 + 組合せ

### Scenario: 初期サンプル TCML が表示される (内容一致)
- Given エディタページを開く
- Then 左ペインの初期値が `docs/spec/editor.md` §「初期サンプル」のコードブロックと一致

### Scenario: textarea が等幅フォントで表示される
- Given エディタページを開く
- Then `<textarea>` の computed style の `font-family` が monospace を含む

### Scenario: 大量行のスクロール
- Given 1000 行の TCML をペースト
- Then エディタが応答性を維持し、プレビューが 300ms デバウンス後に更新される

### Scenario: あふれていない状態では左ペインにスクロールバーが出ない
- Given エディタページを開き、初期サンプル TCML が表示された直後の状態
- Then `.editor-pane` の `scrollWidth <= clientWidth` かつ `scrollHeight <= clientHeight`
- And ブラウザがネイティブのスクロールバー UI を `.editor-pane` 上に描画しない

### Scenario: 内容が pane をあふれた場合は左ペインまたは内部でスクロール可能
- Given 行数とカラム幅の両方が pane を超える長い TCML を流し込んだ状態
- Then ユーザーはエディタ内で縦・横にスクロールしてあふれた部分を閲覧できる
- And スクロール後もシンタックスハイライト・エラー下線の表示位置は textarea のテキスト位置とずれない

### Scenario: WaveDrom ボタンで警告がない場合は status 行に何も表示されない
- Given 警告なしの有効 TCML
- When WaveDrom ボタンをクリック
- Then ダウンロード成功、status 行は空 or "OK" 等の中立メッセージ

### Scenario: WaveDrom 1 件だけ警告がある場合は全文表示
- Given アンカー 53 個の TCML (1 件警告)
- When WaveDrom ボタンクリック
- Then status 行に該当警告本文が全文表示

### Scenario: WaveDrom 複数警告がある場合は "N warnings" 要約
- Given 複数警告 (例: アンカー 53 + step round)
- Then status 行に "2 warnings" 等の要約

### Scenario: パースエラーで WaveDrom ボタンが無効化される
- Given 不正な TCML (パースエラー)
- Then WaveDrom ボタンが disabled

### Scenario: パースエラー復帰で WaveDrom ボタンが再度有効
- Given パースエラー → 修正
- Then WaveDrom ボタンが enabled に戻る

### Scenario: Help モーダル iframe srcdoc に help HTML が含まれる
- Given Help ボタンをクリック
- Then モーダル内 iframe の `srcdoc` 属性に `help/output/tcml-format.html` の内容が埋め込まれる
- And 外部 fetch は発生しない

### Scenario: Help モーダル JA/EN トグル切替
- Given Help モーダルが日本語表示
- When JA/EN トグルクリック
- Then iframe srcdoc が英語版に切り替わる

### Scenario: Help モーダル初期表示言語が navigator.language に従う
- Given navigator.language === "ja-JP"
- When Help モーダル open
- Then 初期表示が日本語

### Scenario: privacy-note が言語に応じて切り替わる
- Given navigator.language === "en-US"
- Then `#privacy-note` の textContent が "No data is sent externally"

### Scenario: Load PNG で iTXt なしのファイルは textarea を変更しない
- Given PNG (`tchart-source` iTXt なし) を Load 経由で選択
- Then wasm が undefined を返し、textarea は変更されず status に "Load failed: ..."

### Scenario: Load SVG で `<tchart:source>` なしのファイル
- Given SVG (`<tchart:source>` 要素なし) を Load 経由
- Then 同様に textarea 変更なし、status エラー

### Scenario: Load 中に大ファイル (10MB) の PNG/SVG を読み込む
- Given 10MB の PNG (`tchart-source` 含む)
- Then ブラウザがハングせず正常に textarea 更新

### Scenario: 連続して Load → Save PNG → Load (ラウンドトリップ)
- Given 任意の有効 TCML
- When Load 手動入力 → Save PNG → 同じ PNG を Load
- Then textarea が元 TCML と一致

### Scenario: 編集中に WaveDrom ボタンを押した直後の編集
- Given WaveDrom ボタンクリックの直後 (デバウンス未達) に編集
- Then ダウンロードは押下時点の TCML、その後の編集は次回 WaveDrom クリックで反映

### Scenario: 「途中 `@step` × `@clock(auto)` × WaveDrom」E2E (editor 経由)
- Given editor 左ペインに以下を入力:
  ```
  @step 10
  @clock(pos) clk
  @step 20
  data ====
  ```
- When 右ペインプレビュー更新後、WaveDrom ボタンクリック
- Then ダウンロードファイルの JSON が `clk.wave="p..."` `data.wave="=..."` `data.period=2` を含む

### Scenario: editor 右ペインで `@signal(overline)` の overline `<line>` が描画される
- Given editor に `@signal(overline) nReset _~` を入力
- Then 右ペイン SVG に独立 `<line>` 要素 (信号名上線) が含まれる

## ユーザ承認反映シナリオ (2026-05-10)

### Scenario: editor が `.tc` プレーンテキストを Load 可能
- Given ローカル `.tc` ファイル `chart.tc` (UTF-8 テキスト)
- When ユーザが Load ボタンで `chart.tc` を選択
- Then textarea にファイル全文がそのまま流し込まれる
- And status 行に `Loaded chart.tc`
- And プレビューが即時更新される

### Scenario: `.tc` 判定は先頭バイトでテキスト判定 (PNG/SVG 以外)
- Given 拡張子が `.txt` でも中身が TCML テキスト
- When Load
- Then プレーンテキストとして扱われ、textarea に流し込まれる

### Scenario: file input の accept 属性に `.tc,text/plain` を含む
- Given editor の Load ボタン
- Then 内部 `<input type="file">` の accept に `.tc,text/plain,.svg,...,.png,...` が含まれる

### Scenario: 空 `.tc` をロード
- Given 空の `.tc` ファイル
- When Load
- Then textarea が空文字に置き換えられる
- And status 行は `Loaded empty.tc`

### Scenario: Help iframe は srcdoc のみ、URL fetch しない
- Given Help モーダルを開く
- Then モーダル内 `<iframe>` は `srcdoc` 属性を持つ
- And `src` 属性を持たない
- And ブラウザの DevTools Network タブで help 関連の追加 fetch が一切発生しない
- And `dist/index.html` 1 ファイルのみで完結する (外部 CSS/JS/wasm/HTML を含む追加 fetch なし)

---

## シンタックスハイライト

### @not-implemented @smoke
### Scenario: `@` ディレクティブが keyword 色で表示される
- Given エディタが初期化済みである
- When 左ペインに `@step 25` と `@slant 5` と `@clock` を含むテキストを入力する
- Then code-input overlay 内で `@step` / `@slant` / `@clock` が Prism の `token keyword` クラスを持つ要素として描画される
- And 通常の信号名や波形文字とは異なる色で表示される

### @not-implemented
### Scenario: `@->` が 1 個の keyword として扱われる
- Given エディタが初期化済みである
- When 左ペインに `@-> label` を入力する
- Then `@->` 全体が 1 個の `token keyword` 要素として描画される
- And `@-` で 2 トークンに分割されない

### @not-implemented
### Scenario: 文字列リテラルが string 色で表示される
- Given エディタが初期化済みである
- When 左ペインに `@title "hello world"` を入力する
- Then `"hello world"` (両端のクォート含む) が Prism の `token string` クラスを持つ要素として描画される

### @not-implemented
### Scenario: 信号名が variable 色で表示される
- Given エディタが初期化済みである
- When 左ペインに `Clock _~_~` を含むタイミング記述行を入力する
- Then 行頭の `Clock` が Prism の `token variable` クラスを持つ要素として描画される
- And 後続の波形文字 `_~_~` は variable 色ではない

### @not-implemented
### Scenario: `//` から行末までが comment 色で表示される
- Given エディタが初期化済みである
- When 左ペインに `// this is a comment` を含む行を入力する
- Then `// this is a comment` 全体が Prism の `token comment` クラスを持つ要素として描画される

### @not-implemented
### Scenario: コメントは当該行の末尾までで打ち切られる
- Given エディタが初期化済みである
- When 1 行目に `Clock _~ // tail` 、2 行目に `Data ==` を入力する
- Then 1 行目の `// tail` のみが comment 色で表示される
- And 2 行目の `Data ==` は comment 色にならない

### @not-implemented @negative
### Scenario: クォート内の `//` はコメント扱いされない
- Given エディタが初期化済みである
- When 左ペインに `@title "foo // bar"` を入力する
- Then `"foo // bar"` 全体が 1 個の `token string` 要素として描画される
- And `// bar` 部分が comment 色にならない

### @not-implemented @negative
### Scenario: クォート内の信号名 `"foo // bar"` がコメント扱いされない
- Given エディタが初期化済みである
- When 左ペインに `"foo // bar" _~_~` をタイミング記述行として入力する
- Then `"foo // bar"` は string 色のままで comment 色にならない
- And 後続の波形文字 `_~_~` は通常色のままになる

### @not-implemented
### Scenario: 数値リテラルが number 色で表示される
- Given エディタが初期化済みである
- When 左ペインに `@step 25` `@slant 1.5` `@font-size 12px` を含むテキストを入力する
- Then `25` `1.5` `12` がそれぞれ Prism の `token number` クラスを持つ要素として描画される
- And `px` は number 色にならない

### @not-implemented
### Scenario: 波形文字は通常色のままになる
- Given エディタが初期化済みである
- When 左ペインに `Clock _~=XO?` を入力する
- Then 波形文字 `_` `~` `=` `X` `O` `?` のいずれにも `token keyword` / `token string` / `token variable` / `token comment` / `token number` クラスが付与されない
- And 通常の前景色 (overlay のデフォルト色) で描画される

### @not-implemented @edge-case
### Scenario: 空ファイルでもハイライトが破綻しない
- Given エディタが初期化済みである
- When 左ペインの内容をすべて消す
- Then code-input overlay が空の状態で正しく表示される
- And ハイライト処理が例外を投げない

### @not-implemented @edge-case
### Scenario: 複数行にまたがる string がクォートを閉じるまで継続しない (単一行 string)
- Given エディタが初期化済みである
- When 1 行目に `@title "unterminated` 、2 行目に `Data ==` を入力する (クォート未閉じ)
- Then 2 行目の `Data ==` は string 色にならない
- And パースエラー扱いになるかは別途 §エラー位置の下線表示 で検証する

### @not-implemented
### Scenario: code-input ラップ後も IME 入力が動作する
- Given エディタの textarea にフォーカスがある
- When IME で日本語 (例: `日本語信号`) を入力する
- Then 確定後の文字列が textarea の value に反映される
- And ハイライトが再計算され、信号名 `日本語信号` が variable 色で描画される

### @not-implemented
### Scenario: code-input ラップ後も Ctrl+S が既定の保存ダイアログ抑制等の標準動作を維持する
- Given エディタの textarea にフォーカスがある
- When Ctrl+S を押す
- Then code-input がブラウザ標準の keydown 配信を妨げない
- And アプリケーション側のキーバインド (もしあれば) が動作する

### @not-implemented @regression
### Scenario: 既存の Save SVG / Save PNG / Load / WaveDrom / Help がハイライト導入後も動作する
- Given エディタが初期化済みで有効な SVG がプレビューされている
- When 順に Save SVG / Save PNG / Load (任意の `.tc`) / WaveDrom / Help の各ボタンを操作する
- Then いずれもハイライト導入前と同じ結果が得られる
- And ハイライト処理がボタン押下のイベント配信を妨げない

---

## エラー位置の下線表示

### @not-implemented @smoke
### Scenario: パースエラー時に該当範囲に赤の波線下線が表示される
- Given 有効な SVG がプレビューに表示されている
- When 左ペインを `renderTcml` がパースエラーを返す状態に変更する
- Then code-input overlay 上に `<div class="tcml-error-underline">` が 1 個だけ描画される
- And その内側の要素の `text-decoration-line` が `underline`、`text-decoration-style` が `wavy`、`text-decoration-color` が `--error-color` (status 行と同色) として算出される

### @not-implemented
### Scenario: 下線の位置が error の line / column / length に対応する
- Given パースエラー位置が `line=2 column=3 length=4` で返る TCML をエディタに入力
- Then `<div class="tcml-error-underline">` の y オフセットが 2 行目に相当する
- And x オフセットが行頭から「2 文字ぶん」(column - 1) に相当する
- And 幅が「4 文字ぶん」に相当する

### @not-implemented @edge-case
### Scenario: `length=0` (挿入点エラー) でも下線が最低 1 文字幅で表示される
- Given パースエラー位置が `length=0` で返る TCML (例: 開いた `"` のまま EOF) をエディタに入力
- Then `<div class="tcml-error-underline">` の幅が「1 文字ぶん」(`max(length, 1)` の結果) に相当する
- And 下線が消えずに視認できる

### @not-implemented @edge-case
### Scenario: マルチバイト UTF-8 のあとのエラーで column が文字単位で算出される
- Given タイミング記述行 `日本語 _~_~_~_X` で `X` の位置にパースエラーがあるとする (column は core 側で文字単位)
- When `renderTcml` が `{ error: { line, column, length, message } }` を返す
- Then エディタの下線 x オフセットが「行頭から 7 文字ぶん」(信号名 3 文字 + 空白 1 + 波形 3 = column 8 → x = 7 文字幅) に対応する
- And バイト数 (UTF-8 で 3 バイト × 3 + ASCII) ベースで計算した位置にはならない

### @not-implemented
### Scenario: パースエラーが解消すると下線が消える
- Given パースエラー状態で下線がエディタ内に表示されている
- When 左ペインを有効な TCML に修正する
- Then `renderTcml` が `{ svg }` を返す
- And `<div class="tcml-error-underline">` が DOM から消える (または display:none になる)
- And ステータス行のエラー表示もクリアされる

### @not-implemented
### Scenario: 下線の `title` 属性にエラーメッセージが入る
- Given パースエラーが下線として表示されている
- When 下線要素の `title` 属性を取得する
- Then `renderTcml` が返した `error.message` (英語固定本文) と完全一致する
- And マウスホバーでブラウザ標準の tooltip にその文言が表示される

### @not-implemented
### Scenario: 連続したパースエラーで下線が前回のものから差し替わる
- Given パースエラー位置 `line=1 column=5 length=2` の状態
- When エディタを編集して別のパースエラー位置 `line=3 column=1 length=8` の状態にする
- Then `<div class="tcml-error-underline">` が 1 個のまま (複数残らない) で、新しい位置・幅に更新される

### @not-implemented @edge-case
### Scenario: パースエラーが複数返るケースでも先頭 1 件のみが下線表示される
- Given パースエラーを複数含み得る TCML を入力
- When `renderTcml` が `{ error }` を返す (現状の仕様では先頭 1 件のみ)
- Then `<div class="tcml-error-underline">` は 1 個のみ描画される

### @not-implemented @regression
### Scenario: 下線描画が SVG プレビューの保持を妨げない
- Given 有効な SVG が右ペインに表示されている
- When 左ペインを編集してパースエラー状態にする
- Then 下線がエディタ内に出る
- And 右ペインの SVG プレビューは直前の有効な SVG をそのまま保持する

### @not-implemented @edge-case
### Scenario: ウィンドウリサイズや editor スクロールでも下線位置が崩れない
- Given パースエラー状態で下線が表示されている
- When エディタ内でスクロールする、またはウィンドウ幅を変更する
- Then 下線の x / y オフセットが新しい overlay 上の文字位置に追従する
- And 下線が overlay の外側にずれない
