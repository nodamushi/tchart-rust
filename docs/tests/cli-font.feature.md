# CLI フォント管理

`docs/spec/cli.md` §フォント管理 のテスト仕様。
プロセス内グローバル管理・多重読込み禁止・family 解決・PNG `fontdb` 統合をカバー。
基本的な `--font` / `--font-size` / 環境変数優先順は `cli.feature.md` §フォント解決
を参照。本ファイルは §フォント管理 で新規に規定された挙動のみを扱う。

---

## 絶対条件 (MUST)

## @not-implemented @smoke
### Scenario: 同一 family を複数行で参照しても load は 1 回
- Given `@font` で同じフォントファミリーを 100 行以上で参照する TCML がある
- When `tchart svg` でレンダリングする
- Then 同じフォントファイルの読み込みは 1 回しか発生しない

## @not-implemented
### Scenario: レイアウトと PNG の fontdb で同一フォント実体を参照する
- Given `@font` で別フォントファミリーを指定した TCML がある
- And family 解決が成功する環境である
- When `tchart png` でレンダリングする
- Then PNG ラスタライズで使われたフォントが、レイアウト測定で使われたフォントと同一実体である

## @not-implemented @negative
### Scenario: 解決できない family は同名で再 load されない
- Given `@font NoSuchFont` を複数行で繰り返す TCML がある
- When `tchart svg` でレンダリングする
- Then OS への解決問い合わせは family につき 1 回しか発生しない
- And 警告メッセージは family につき 1 行しか stderr に出ない

---

## デフォルトフォント

## @not-implemented
### Scenario: `--font` で指定したフォントがデフォルトとして使われる
- Given `@font` 未指定の TCML がある
- When `tchart svg chart.tc --font /path/to/Default.ttf` を実行する
- Then チャート内のすべてのテキストが指定されたフォントで描画される

## @not-implemented @negative
### Scenario: デフォルトフォント解決全段失敗で終了コード 4
- Given `--font` / `TCHART_FONT` 未指定
- And OS 自動検出も失敗する環境である
- When `tchart svg chart.tc` を実行する
- Then エラーメッセージが stderr に出力される
- And 終了コードが `4` である

---

## family 解決

## @not-implemented @smoke
### Scenario: 単一 family 指定で別フォントが使われる
- Given `@font "Liberation Sans"` を持つ TCML がある
- And `Liberation Sans` が OS で解決可能である
- When `tchart svg` でレンダリングする
- Then 該当行のテキストは `Liberation Sans` で描画される
- And デフォルトフォントは別 family で別箇所に使われる

## @not-implemented
### Scenario: CSV 指定で左から順に試行し最初に解決できたものを使う
- Given `@font "NoSuchFont, Liberation Sans, sans-serif"` を持つ TCML がある
- And `Liberation Sans` が OS で解決可能である
- When `tchart svg` でレンダリングする
- Then 該当行のテキストは `Liberation Sans` で描画される

## @not-implemented @negative
### Scenario: family 解決失敗時はデフォルトにフォールバックし警告を出す
- Given `@font NoSuchFont` を持つ TCML がある
- When `tchart svg` でレンダリングする
- Then 該当行のテキストはデフォルトフォントで描画される
- And stderr に「font family not found」相当の警告が 1 行出力される
- And 終了コードは `0` である

## @not-implemented
### Scenario: ジェネリック (`sans-serif`) が OS 解決経由で実フォントに繋がる
- Given `@font sans-serif` を持つ TCML がある
- When `tchart svg` でレンダリングする
- Then `sans-serif` が OS で解決した実フォントで描画される

---

## PNG `fontdb` 統合

## @not-implemented
### Scenario: 解決済み全フォントが fontdb に登録される
- Given 複数の異なる family を `@font` で指定した TCML がある
- And すべての family が OS で解決可能である
- When `tchart png` でレンダリングする
- Then すべての family が PNG 出力のテキストで判別可能に描画されている

## @not-implemented
### Scenario: ジェネリックは family 解決結果があればそれを使う
- Given `@font sans-serif` を含む TCML がある
- When `tchart png` でレンダリングする
- Then PNG 内の `sans-serif` テキストは、その OS の `sans-serif` 解決結果のフォントで描画される

## @not-implemented
### Scenario: ジェネリック解決結果が無い場合はデフォルトを割り当てる
- Given TCML が `@font` を一切指定しない
- When `tchart png` でレンダリングする
- Then PNG 内のテキスト (CSS ジェネリックが SVG に出力されている場合も含む) はデフォルトフォントで描画される

---

## `@font` 途中切替の波及範囲

`font` はローカルパラメータ (`tcml-format.md` §ローカルパラメータ) であり、
途中で再指定するとその行以降に適用される。各テキスト要素 (信号名 / `@title` /
`@->` ラベル / 区間ラベル) の `<text>` 出力に正しく反映されることを保証する。

## @not-implemented @smoke
### Scenario: `@font` 途中切替が次行以降の信号名に適用される
- Given 1 つの TCML 内で `@font A` の後に信号 `S1` を 1 行、続いて `@font B` の後に信号 `S2` を 1 行記述する
- And `A` / `B` のどちらも OS で解決可能である
- When `tchart svg` でレンダリングする
- Then `S1` の信号名 `<text>` の `font-family` 属性は `A` である
- And `S2` の信号名 `<text>` の `font-family` 属性は `B` である

## @not-implemented
### Scenario: `@title` に直前の `@font` が適用される
- Given `@font A` の後に `@title X`、`@font B` の後に `@title Y` を記述した TCML がある
- When `tchart svg` でレンダリングする
- Then タイトル `X` の `<text>` の `font-family` 属性は `A` である
- And タイトル `Y` の `<text>` の `font-family` 属性は `B` である

### Scenario: `@->` のラベルに記述位置の `@font` が適用される
- Given `@font A` の後に `@-> (@a, @b) ラベル1` を記述する
- And 後続で `@font B` に切替えたあと信号行が続いても、上記 `@->` 行はそれより前にある
- When `tchart svg` でレンダリングする
- Then `ラベル1` の `<text>` の `font-family` 属性は `A` である (記述位置の `font`)

## @not-implemented
### Scenario: 区間ラベル (`<...>`) に直前の `@font` が適用される
- Given `@font A` の後に区間ラベル `<タグ>` を含む信号行を 1 行、続いて `@font B` の後に区間ラベル `<タグ2>` を含む信号行を 1 行記述する
- When `tchart svg` でレンダリングする
- Then `タグ` の `<text>` の `font-family` 属性は `A` である
- And `タグ2` の `<text>` の `font-family` 属性は `B` である

## @not-implemented
### Scenario: 途中切替された全 family が PNG fontdb 経由で描画される
- Given 信号名 / `@title` / `@->` ラベル / 区間ラベル のそれぞれに対して異なる `@font` を切替えながら指定した TCML がある
- And すべての family が OS で解決可能である
- When `tchart png` でレンダリングする
- Then 各テキスト要素は SVG の `font-family` 属性と一致する family 実体で描画される

---

## OS 別解決経路

## @not-implemented @linux
### Scenario: Linux で `fc-match` 経由で解決する
- Given Linux 環境で `fc-match` が PATH にある
- And `@font "Liberation Sans"` の TCML を渡す
- When `tchart svg` でレンダリングする
- Then `fc-match -f '%{file}' "Liberation Sans"` の結果のファイルが load される

## @not-implemented @linux
### Scenario: Linux で `fc-match` 不在時は固定候補にフォールバックする
- Given Linux 環境で `fc-match` が PATH に無い
- And `--font` / `TCHART_FONT` 未指定
- When `tchart svg` でレンダリングする
- Then `/usr/share/fonts/` 配下の固定候補のいずれかが load される

## @not-implemented @windows
### Scenario: Windows で `C:\Windows\Fonts\` 配下を走査する
- Given Windows 環境
- And `@font Arial` の TCML を渡す
- When `tchart svg` でレンダリングする
- Then `C:\Windows\Fonts\arial.ttf` 等が load される

## @not-implemented @macos
### Scenario: macOS で `/System/Library/Fonts` / `/Library/Fonts` を走査する
- Given macOS 環境
- And `@font Helvetica` の TCML を渡す
- When `tchart svg` でレンダリングする
- Then `/System/Library/Fonts/Helvetica.ttc` 等が load される

---

## エッジケース

## @not-implemented @edge-case
### Scenario: 不正な `--font` ファイルで終了コード 4
- Given `--font /not/exist.ttf` を指定する
- When `tchart svg chart.tc --font /not/exist.ttf` を実行する
- Then 終了コードが `4` である

## @not-implemented @edge-case
### Scenario: 空の `@font` 指定はパーサーで弾かれる
- Given `@font` の右辺が空である TCML
- When `tchart svg` でパースする
- Then パースエラーで終了コードが `2` である

## @not-implemented @edge-case
### Scenario: 同 family を 100 行参照しても load 1 回 (再掲、smoke 重複除外)
- Given 同じ `@font` を 100 信号行で繰り返す TCML
- When `tchart svg` でレンダリングする
- Then 該当 family の OS 解決問い合わせは 1 回のみ
- And 該当 family の `Font` インスタンスはプロセス内で 1 個だけ存在する

---

## 並列ワーカと共有フォントキャッシュ

仕様: `docs/spec/cli.md` §並列ワーカと共有フォントキャッシュ。

## @not-implemented @smoke
### Scenario: ピークメモリは入力数に比例しない
- Given `tchart batch svg` に 1000 個の TCML 入力を渡す (各入力は同程度のサイズ)
- And 同じく `tchart batch svg` に同じ TCML を 100 個渡す実行も用意する
- When 2 つの実行のピーク常駐メモリ (RSS) を計測する
- Then 1000 入力時の RSS は 100 入力時の RSS の 10 倍未満である
- And 増分はおおむね並列度 W × 1 ドキュメントの最大サイズで説明できる規模に収まる

## @not-implemented
### Scenario: ドキュメントの処理は他のドキュメントを待たない
- Given `tchart batch svg` に 2 入力 `slow.tc` `fast.tc` を渡す
- And `slow.tc` は意図的に巨大で完了に時間がかかる
- And `fast.tc` は小さくすぐ完了する
- When `tchart batch svg slow.tc fast.tc -o out/` を実行する
- Then `out/fast.svg` は `out/slow.svg` の完了を待たずに書き出される
- And `slow.tc` のパース完了が `fast.tc` の処理開始の前提になっていない

## @not-implemented
### Scenario: 異なるフォントは独立に並行ロードされる
- Given `a.tc` が `@font A`、`b.tc` が `@font B` を参照する (A と B は別フォントファイル)
- When `tchart batch svg a.tc b.tc -o out/` を実行する
- Then A のロードと B のロードは並行に発行される
- And A のロードの遅延が B を必要とするワーカの開始を遅らせない

## @not-implemented
### Scenario: 同 family の同時参照でも load は 1 回 (lazy cell)
- Given 多数の入力がいずれも `@font Same` を参照する
- And 複数ワーカが `Same` を同時に問い合わせる状況がある
- When `tchart batch svg ... -o out/` を実行する
- Then `Same` のロードはプロセス全体で 1 回のみ発生する
- And 後続ワーカは初回ロードの完了を待って同じフォント実体を取得する

## @not-implemented
### Scenario: CSV で右側候補は解決もロードもされない
- Given `@font "Liberation Sans, NoSuchFont"` を含む TCML がある (左の Liberation Sans が解決可能)
- When `tchart svg chart.tc` を実行する
- Then `NoSuchFont` への OS 解決問い合わせは 1 回も発生しない
- And `NoSuchFont` のロード試行は発生しない

## @not-implemented
### Scenario: 単一サブコマンドはワーカ 1 + 入力 1 に縮退する
- Given 単一 TCML 入力 `chart.tc` がある
- When `tchart svg chart.tc` を実行する
- Then 共有キャッシュにロードされるフォントは `chart.tc` から参照されるもののみ
- And 他文書のフォントは参照もロードもされない

## @not-implemented
### Scenario: `src` サブコマンドは共有フォントキャッシュを使わない
- Given TCML ソースが埋め込まれた SVG ファイル `chart.svg` がある
- When `tchart src chart.svg` を実行する
- Then フォント解決・ロードは一切発生しない
- And 共有フォントキャッシュは構築されない

---

## `batch` のフォントキャッシュ共有

## @not-implemented @smoke
### Scenario: 複数入力で同じ family を参照しても load は 1 回
- Given 3 つの TCML 入力 `a.tc` `b.tc` `c.tc` がある
- And いずれも `@font "Liberation Sans"` を参照する
- When `tchart batch svg a.tc b.tc c.tc -o out/` を実行する
- Then `Liberation Sans` の OS 解決問い合わせは 1 回のみ発生する
- And フォントファイルのロードは 1 回のみ発生する

## @not-implemented
### Scenario: 入力ごとに異なる family を参照する場合は和集合だけがロードされる
- Given `a.tc` が `@font A`、`b.tc` が `@font B`、`c.tc` が `@font A` を参照する
- And いずれも解決可能である
- When `tchart batch svg a.tc b.tc c.tc -o out/` を実行する
- Then ロードされるフォントは A と B の 2 本のみ (重複ロードなし)

## @not-implemented
### Scenario: 解決失敗の警告は family につき 1 回のみ (バッチ全体で集約)
- Given 複数入力が同じ `@font NoSuchFont` を参照する
- When `tchart batch svg ... -o out/` を実行する
- Then `NoSuchFont` の解決失敗警告は stderr に 1 回のみ出力される

---

## 観点A/B 補強: フォント解決 × エッジケース

### Scenario: `@font` に空文字列を指定
- Given TCML `@font ""`
- When `tchart svg` 実行
- Then 解決失敗扱いでデフォルトフォント、警告 1 行

### Scenario: `@font` に空白のみ
- Given TCML `@font "   "`
- Then パースエラーまたは解決失敗扱い (実装で固定)

### Scenario: `@font` の CSV で 1 個目が解決成功なら 2 個目以降は解決呼び出しなし
- Given `@font "Liberation Sans, NoSuchFont"` で前者解決可能
- Then OS 解決問い合わせは "Liberation Sans" 1 回のみ (`fc-match` などの観測点で確認)

### Scenario: `@font` の CSV で全て解決失敗
- Given `@font "NoFontA, NoFontB, NoFontC"` (全部失敗)
- Then デフォルトフォントにフォールバック、stderr 警告 1 行 (family 全体に対して 1 回)

### Scenario: 同名 family 警告は CLI プロセス全体で 1 回
- Given TCML に `@font NoSuchFont` を 5 回宣言
- Then stderr 警告は 1 行のみ (family ごと 1 回)

### Scenario: `batch` で同一 family を複数入力で参照
- Given 入力 3 件すべてに `@font CommonFont` (解決可能)
- Then ロードは 1 回 (共有キャッシュヒット)、各ワーカーが参照のみ取得

### Scenario: `batch` で 1 件目のフォントロード中に 2 件目が別フォントを要求
- Given 入力 A (FontX を要求)、入力 B (FontY を要求)、並列実行
- Then FontX のロードと FontY のロードが並行に進む (互いにブロックしない)

### Scenario: フォントファイルが TTF/OTF ではない (例: テキストファイル)
- Given `--font /path/to/text.txt` (TTF でない)
- Then 終了コード 4 (フォントエラー)

### Scenario: TCHART_FONT 環境変数が `--font` より低優先
- Given `TCHART_FONT=/path/A.ttf`、`tchart svg chart.tc --font /path/B.ttf`
- Then `/path/B.ttf` が使われる (`--font` 優先)

### Scenario: PNG ラスタライズで CSS ジェネリック `sans-serif` が解決済 family 名で fontdb に登録される
- Given TCML `@font sans-serif`、解決成功で実 family 名 "Liberation Sans" を取得
- Then PNG ラスタライズ時、fontdb には "Liberation Sans" が登録され、SVG 内 `font-family="sans-serif"` も対応する

---

## 観点H: `@font` / フォント解決の境界 (補強)

## @not-implemented
### Scenario: CSV 全候補解決失敗 → デフォルトフォントへ fallback の経路
- Given `@font NoSuchA, NoSuchB, NoSuchC` (全候補解決失敗)
- When `tchart svg in.tc --font /tmp/Default.ttf`
- Then 警告 "font family 'NoSuchA, NoSuchB, NoSuchC' not resolved" が 1 回
- And デフォルト (`/tmp/Default.ttf`) で描画
- And 警告は 1 行限り (CSV 各候補ごとに別行で出さない)

## @not-implemented
### Scenario: 引用付き family 名 (`"Noto Sans CJK JP"`) と CSV 混在
- Given `@font "Noto Sans CJK JP", Roboto, sans-serif`
- Then パーサは 3 候補を順序保持
- And 解決順は `"Noto Sans CJK JP"` → `Roboto` → `sans-serif`

## @not-implemented
### Scenario: 同一 family を別 `@font` 行で 2 回宣言 → 警告は 1 回限定
- Given `@font NoSuch\nA _\n@font NoSuch\nB _` (同一 family 2 回宣言、両方解決失敗)
- Then 警告 "font family 'NoSuch' not resolved" は 1 回のみ (2 回出さない)

## @not-implemented
### Scenario: `@font` の途中変更 (信号別) と PNG fontdb 登録の和集合
- Given `@font Roboto\nA _\n@font NotoSans\nB _\n@font Inter\nC _`
- Then PNG ラスタライズ時 fontdb には Roboto / NotoSans / Inter 3 件 (和集合) 登録
- And 各信号名 `<text>` は対応する family で描画

## @not-implemented
### Scenario: ジェネリック (`monospace`) と実 family の解決優先順位
- Given `@font monospace, "Courier New"`
- Then 解決順は左から (`monospace` → `"Courier New"`)
- And `monospace` が OS 解決可能ならそれを採用、`"Courier New"` は試行されない

## @not-implemented
### Scenario: `@font` 末尾セミコロン / 余分な空白の許容
- Given `@font   Roboto  ,  Inter   `
- Then 解決候補は `["Roboto", "Inter"]` (前後空白除去、間は 1 個に正規化または保持の仕様準拠)

## @not-implemented
### Scenario: `@font` に同一 family を CSV 内重複指定
- Given `@font Roboto, Roboto, Inter`
- Then 重複は uniq 化 (1 回のみ load 試行) または順序保持
- And 警告は対象 family につき 1 回

## @not-implemented
### Scenario: family 名にカンマ自体を含めたい場合の literal
- Given `@font "Sans, Bold"` (引用内カンマ)
- Then 1 候補として扱う (CSV 区切りで 2 個に分割しない)

## @not-implemented
### Scenario: `@font` 1 行で空 CSV 要素 (`Roboto,, Inter`) を含む
- Given `@font Roboto,, Inter` (中央が空)
- Then 空要素は無視 or パースエラー (仕様に従う)
- And エラーなら次行以降の処理は継続
