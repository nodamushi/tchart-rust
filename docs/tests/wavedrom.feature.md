# WaveDrom 変換

`tchart wavedrom` および `tchart-core::wavedrom` の WaveJSON 出力テスト仕様。
仕様: [`docs/spec/wavedrom.md`](../spec/wavedrom.md)、[`docs/spec/cli.md`](../spec/cli.md)。

WaveJSON 出力検証は `serde_json::Value` レベルで比較する (キー順依存しない)。文字列比較が必要な場合のみ整形済み JSON を assert する。

---

## 信号レベルの基本マッピング

## @not-implemented @smoke
### Scenario: Low / High / HiZ / Bus / DontCare の wave 文字列変換
- Given TCML 信号 `s : _ ~ - = ?` (各 1 unit) で先頭にダミー Bus アンカーを置き `?` を解決可能にしたファイル
- When `tchart-core::wavedrom::to_wavejson` を呼ぶ
- Then `signal[0].wave` は `"=01zx"` 系の (各 level に対応する) 5 文字となる
- And 各文字種のマッピングは仕様表 (`wavedrom.md` §wave 文字列マッピング) に従う

## @not-implemented
### Scenario: 連続 hold は先頭 1 文字 + `.` 反復
- Given TCML 信号 `s : ___ ~~ ==` (Low 3 unit、High 2 unit、Bus 2 unit)
- When 変換する
- Then `signal[0].wave` は `"0..1.=."` となる

## @not-implemented @smoke
### Scenario: Bus 区間内のテキスト断片は TCML の中央ラベル (空白連結) として 1 エントリに集約される
- Given TCML 信号 `data: =A=B=C` (parser merge 後 Bus 3 unit + 中央ラベル "A B C")
- When 変換する
- Then `signal[0].wave` は `"=.."` (Bus 1 セグメント 3 unit) となる
- And `signal[0].data` は `["A B C"]` (1 エントリ、空白連結された全ラベル) となる

## @not-implemented
### Scenario: Bus 区間 1 ラベルのみ
- Given TCML 信号 `data: ==A==` (Bus 4 unit、テキスト "A" 中央配置)
- When 変換する
- Then `signal[0].wave` は `"=..."` となる
- And `signal[0].data` は `["A"]` (1 エントリ) となる

## @not-implemented
### Scenario: BusCross で区切られた複数 Bus 区間はそれぞれ別エントリ
- Given TCML 信号 `data: =A=B=X=C=D` (Bus 2 + BusCross + Bus 3 で merge、各区間にテキスト)
- When 変換する
- Then `signal[0].wave` は `"=.=.."` 系の 2 個の Bus セグメントを含む形となる
- And `signal[0].data` は `["A B", "C D"]` (2 エントリ、各区間のラベル) となる

## @not-implemented
### Scenario: Bus 区間にテキストが全く無い場合は data フィールドを省略
- Given TCML 信号 `data: ====` (Bus 4 unit、テキストなし)
- When 変換する
- Then `signal[0].wave` は `"=..."` となる
- And `signal[0].data` フィールドは出力されない

## @not-implemented
### Scenario: 非 Bus 区間内のテキスト文字は落とす
- Given TCML 信号 `s : _A_` (Low 区間内にテキスト A)
- When 変換する
- Then `signal[0].wave` は `"0.0"` 系の Low only となる
- And `signal[0].data` フィールドは出力されない (省略)

## @not-implemented
### Scenario: DontCare 4 バリアント全部が `x` にマップ
- Given TCML 信号で `?` の直前 level が Low / High / HiZ / Bus の各ケース
- When 変換する
- Then いずれの場合も `?` 部分は wave 文字列で `x` 1 文字となる
- And `data` 配列には影響しない

---

## 透過要素・Gap

## @not-implemented
### Scenario: Gap (`:`) は WaveDrom `|` にマップ
- Given TCML 信号 `s : _:_` (Low → 1 unit Gap → Low)
- When 変換する
- Then `signal[0].wave` は `"0|0"` となる

## @not-implemented
### Scenario: Guide (`|`、0 width) は落とす
- Given TCML 信号 `s : _|_` (Low → Guide → Low、合計 2 unit)
- When 変換する
- Then `signal[0].wave` は `"0."` (= Low 2 unit) となる
- And Guide 由来の文字は出力されない

## @not-implemented
### Scenario: Highlight 区間 (`[` / `]`) は落とす
- Given TCML 信号 `s : _[~]_` (Low → ハイライト開始 → High → ハイライト終了 → Low)
- When 変換する
- Then `signal[0].wave` は `"010"` となる
- And `[` `]` 由来の文字は出力されない

---

## SignalRow 以外の Line

## @not-implemented @smoke
### Scenario: `@title` は `head.text` に集約
- Given TCML に `@title 同期回路` 1 行と信号 1 行
- When 変換する
- Then 出力 JSON の `head.text` は `"同期回路"` となる
- And `signal` 配列にタイトル由来の要素は含まれない

## @not-implemented
### Scenario: 2 個目以降の `@title` は欠落し、警告が出る
- Given TCML に `@title A` と `@title B` の 2 行 (最初が A、次が B)
- When 変換する
- Then `head.text` は `"A"` (最初のみ) となる
- And stderr に `warning: only the first @title is kept; 1 additional @title row(s) dropped` 形式の 1 行が出力される
- And 終了コードは 0 のまま

## @not-implemented
### Scenario: 3 個以上の `@title` でも保存されるのは最初のみ、警告には欠落数が入る
- Given TCML に `@title A`, `@title B`, `@title C` の 3 行
- When 変換する
- Then `head.text` は `"A"`
- And stderr の警告は `... 2 additional @title row(s) dropped` (欠落数 2)

## @not-implemented
### Scenario: `@skip(n)` は値によらず `{}` 1 個
- Given TCML に `@skip(2)` と `@skip(0.5)` がそれぞれ存在する 2 ファイル
- When 各々を変換する
- Then どちらも `signal` 配列にちょうど 1 個の空オブジェクト `{}` を含む

## @not-implemented
### Scenario: `@skip(0)` は parser 段階で落ちて変換側に届かない
- Given TCML に `@skip(0)` 行
- When 変換する
- Then `signal` 配列に空オブジェクトは追加されない

---

## period / 時間軸正規化

## @not-implemented @smoke
### Scenario: 全信号同一 step なら period 出力なし
- Given TCML で全信号が `@step 10` のもとに定義された 2 信号
- When 変換する
- Then 各信号オブジェクトに `period` フィールドは出力されない

## @not-implemented
### Scenario: 信号間で step が異なる場合は gcd で割って period 算出
- Given TCML 信号 A (step=10) と信号 B (step=14) が混在
- When 変換する
- Then gcd(10, 14) = 2 を基準に A.period = 5、B.period = 7 となる

## @not-implemented
### Scenario: step が gcd で割って 1 になる信号は period 省略
- Given TCML 信号 A (step=10) と信号 B (step=20)
- When 変換する
- Then gcd = 10、A は `period` 省略 (= 1)、B.period = 2 となる

## @not-implemented
### Scenario: 非整数 step は round して警告
- Given TCML 信号で `@step 10.6` が指定された信号
- When 変換する
- Then 該当信号の `step` は `11` として扱われる
- And stderr に `warning: signal "<name>" step rounded: 10.6 -> 11` 形式の 1 行が出力される
- And 終了コードは 0 のまま

## @not-implemented
### Scenario: 信号 0 件のときは period 計算なし
- Given TCML が `@title only` のみで信号行を持たない
- When 変換する
- Then `signal` 配列は空 (または `head.text` のみ)、`period` は出力されない
- And 警告も出ない

---

## clock 自動展開

## @not-implemented @smoke
### Scenario: `@clock(pos)` デフォルトパルス幅は `p` を反復
- Given TCML に `@clock(pos)` と空 wave の信号、`chart_units = 4`
- When 変換する
- Then 該当信号の `wave` は `"p..."` (= `p` + `.` × 3) となる

## @not-implemented
### Scenario: `@clock(neg)` デフォルトパルス幅は `n` を反復
- Given TCML に `@clock(neg)` と空 wave の信号、`chart_units = 3`
- When 変換する
- Then `wave` は `"n.."` となる

## @not-implemented
### Scenario: `@clock(pos, _=2, ~=3)` は `0`/`1` 列に自力展開
- Given TCML に `@clock(pos, _=2, ~=3)`、`chart_units = 5`
- When 変換する
- Then `wave` は parser 展開結果 (Low 2 + High 3) を §wave 文字列マッピングで変換した `"0.1.."` となる

## @not-implemented
### Scenario: `@clock(both)` は `0`/`1` 列に展開
- Given TCML に `@clock(both)`
- When 変換する
- Then `wave` は `p`/`n` を使わず `0`/`1` 列で出力される

## @not-implemented
### Scenario: clock 三角形マーカー (`mark_*`) は出力に現れない
- Given TCML に `@clock(pos, mark_position=0.3, mark_color=red)`
- When 変換する
- Then 出力 JSON にマーカー由来のフィールドは存在しない (warning も出ない)

## @not-implemented
### Scenario: per-row `@step` 下の auto 展開 wave 長は他信号のピクセル幅基準
- Given TCML に `@step 20\nClock _~_~_~_~_~_~\n@step 10\n@clock\nclock` (1 行目 12 units × step=20、2 行目 auto)
- When 変換する
- Then 1 行目 `wave` は `"010101010101"` (12 文字)、2 行目 auto `wave` は `"010101010101010101010101"` (24 文字 = `round(240 / 10)`)
- And `period` は gcd(20, 10) = 10 を基準に 1 行目 `period = 2`、2 行目 `period` 省略 (= 1)、結果として両行の物理時間幅 (= units × period) が揃う

## @not-implemented
### Scenario: per-row `@step` + 非対称 pulse の auto 拡張 wave
- Given TCML に `@step 20\nSig _~_~_~_~\n@step 10\n@clock(_=2, ~=3)\nck` (1 行目 8 units × step=20 = 160px)
- When 変換する
- Then 1 行目 `wave` は `"01010101"`、auto 行 ck の `wave` は target=16 units を `Low(2)→High(3)→Low(2)→High(3)→Low(2)→High(3)→Low(1)` で埋めた `"0.1..0.1..0.1..0"` (合計 16 文字)
- And `period` は gcd(20, 10) = 10 を基準に 1 行目 `period=2`、auto 行 `period` 省略

## @not-implemented
### Scenario: 全信号 auto の WaveDrom 出力は空 wave
- Given TCML が `@clock\nck1\n@clock\nck2` のみ
- When 変換する
- Then `signal[0].wave == ""`、`signal[1].wave == ""`、`period` 出力なし

---

## 信号名

## @not-implemented
### Scenario: 複数行信号名はスペース連結
- Given TCML 信号 `"a\nb": _~_`
- When 変換する
- Then `signal[0].name` は `"a b"` となる (改行はスペース 1 個に置換)

## @not-implemented
### Scenario: 信号名内のクォート文字は JSON エスケープ
- Given TCML 信号 `"a\"b": _`
- When 変換する
- Then 出力 JSON では `"name": "a\"b"` のように JSON エスケープされる

---

## アンカーと矢印

## @not-implemented @smoke
### Scenario: `@->` から参照されるアンカーは node 文字を割り当てる
- Given TCML に `clk : ~_@{a}_~` `data: ==@{b}==` `@-> (@{a}, @{b}) start`
- When 変換する
- Then `signal[0].node` は wave と同じ長さで、アンカー位置に `a`、それ以外は `.`
- And `signal[1].node` は同様で位置に `b`
- And top-level `edge` 配列は `["a->b start"]` を含む

## @not-implemented
### Scenario: `@->` から参照されないアンカーは node を割り当てない
- Given TCML に未参照アンカー `@{x}` を 1 つ
- When 変換する
- Then 該当信号オブジェクトに `node` フィールドは出力されない
- And `edge` 配列にも含まれない

## @not-implemented
### Scenario: 矢印頭オプションが style に反映
- Given `@-> (@{a}, @{b}, head=both)` (solid デフォルト + 両端頭)
- When 変換する
- Then `edge` 文字列は `"a<->b"` で始まる

## @not-implemented
### Scenario: 線種 dashed は curve 系で近似
- Given `@-> (@{a}, @{b}, dashed)`
- When 変換する
- Then `edge` 文字列は `"a-~>b"` で始まる

## @not-implemented
### Scenario: 矢印の色・太さは落とす
- Given `@-> (@{a}, @{b}, red, 3px) hello`
- When 変換する
- Then `edge` 文字列は `"a->b hello"` (色・太さ情報なし) となる

## @not-implemented
### Scenario: アンカー数 52 超過で警告 + 超過分の edge を落とす
- Given TCML にアンカー 53 個と各 2 つを参照する `@->`
- When 変換する
- Then stderr に `warning: more than 52 anchors; ...` の 1 行
- And `edge` 配列は最初の 52 個までで止まる

## @not-implemented
### Scenario: Title / Skip 行が混在していてもアンカー node が正しく出力される
- Given TCML に `@title` 行が先頭にあり、その後に `@skip` 行と signal 行が混在し、signal 行のアンカーが `@->` で参照される
- When 変換する
- Then title 直後の signal のアンカー node が正しい位置の node 文字で出力される
- And skip 行の後の signal のアンカー node も正しい位置の node 文字で出力される

---

## 落とす要素

## @not-implemented
### Scenario: 背景色・スタイル系は出力に現れない
- Given TCML に `@bg red`、`@bgcolor0 blue`、`@signal(overline)`、`@font Arial`、`@fontsize 16` 等が散在
- When 変換する
- Then 出力 JSON のどこにもこれらの値は現れない (`config` フィールドは出力されない)
- And 警告も出ない

## @not-implemented
### Scenario: `%` 文字書き込み行は出力されない
- Given TCML に `% 50 50 hello`
- When 変換する
- Then 出力 JSON にこの座標・テキストは現れない

---

## CLI: `tchart wavedrom` サブコマンド

## @not-implemented @smoke
### Scenario: デフォルト出力 (入力隣に `<STEM>.json`)
- Given 有効な TCML ファイル `chart.tc` がある
- When `tchart wavedrom chart.tc` を実行する
- Then `chart.json` ファイルが生成される
- And ファイルは strict JSON (コメント・末尾カンマなし) として `serde_json::from_str` でパースできる
- And 終了コードが `0` である

## @not-implemented
### Scenario: `-o` で出力ファイル指定
- Given 有効な TCML ファイル `chart.tc` がある
- When `tchart wavedrom chart.tc -o out.json` を実行する
- Then `out.json` ファイルが生成される
- And 終了コードが `0` である

## @not-implemented @negative
### Scenario: 複数入力はエラー
- Given 2 つの有効な TCML ファイル `a.tc` `b.tc` がある
- When `tchart wavedrom a.tc b.tc` を実行する
- Then 使用方法のエラーメッセージが stderr に出力される
- And 終了コードが `1` である

## @not-implemented @negative
### Scenario: 入力ファイル不在で終了コード 1
- Given 存在しないパス `missing.tc`
- When `tchart wavedrom missing.tc` を実行する
- Then 終了コードが `1` である

## @not-implemented @negative
### Scenario: TCML パースエラーで終了コード 2
- Given 文法エラーのある TCML ファイル `broken.tc`
- When `tchart wavedrom broken.tc` を実行する
- Then 終了コードが `2` である

## @not-implemented @negative
### Scenario: 出力先ディレクトリ書き込み不能で終了コード 3
- Given 有効な TCML ファイル、書き込み不能なディレクトリ
- When `tchart wavedrom chart.tc -o /readonly/out.json` を実行する
- Then 終了コードが `3` である

## @not-implemented
### Scenario: フォント関連オプションは受け付けない
- Given 有効な TCML ファイル
- When `tchart wavedrom chart.tc --font /path/font.ttf` を実行する
- Then 使用方法エラーで終了コードが `1` である

---

## 統合: 公式サンプル変換

## @not-implemented @regression
### Scenario: `docs/images/sample.tc` を変換しても WaveJSON 互換 JSON が生成される
- Given `docs/images/` 配下の代表サンプル TCML
- When `tchart wavedrom <sample>.tc -o /tmp/out.json` を実行する
- Then 終了コードが `0` である
- And `/tmp/out.json` が strict JSON としてパース可能
- And `signal` 配列の長さがサンプル中の SignalRow + SkipRow 数と一致する

---

## 観点A 補強: WaveDrom 単独仕様の境界

### Scenario: 信号 0 件かつ Title なしで空 JSON
- Given TCML が空 (lines 全くなし)
- When 変換する
- Then 出力 JSON は `{"signal":[]}` (signal フィールドは必須なので空配列で出る)

### Scenario: `@title` のみ、信号なしで `head.text` のみ出力
- Given TCML に `@title onlytitle` のみ (信号 0 件)
- Then 出力は `{"signal":[],"head":{"text":"onlytitle"}}` (signal は空配列、head.text は設定)

### Scenario: 信号名がスペースで終わる場合のスペース連結
- Given TCML 信号 `"a\n " : _~` (改行後にスペース 1 個)
- Then `signal[0].name` は `"a  "` (改行 → スペース 1 個に置換、原文末尾スペースは保持)

### Scenario: アンカー 1 個のみで edge 配列なしの場合
- Given TCML に未参照アンカー `@{a}` のみ (`@->` ゼロ)
- Then 出力 JSON に `edge` フィールドは出力されない (空配列も出さない)
- And 該当信号に `node` フィールドも出力されない

### Scenario: アンカー文字割り当てが a → z → A → Z の順
- Given アンカー 27 個を `@->` で参照 (1〜26 番 = a〜z、27 番目 = A)
- Then 27 番目アンカーの node 文字が `A`

### Scenario: アンカー 52 個 ちょうどは警告なし
- Given アンカー 52 個を `@->` で参照
- Then `edge` 配列に 52 件すべて含まれ、stderr に警告は出ない

### Scenario: アンカー 53 個目のみが落とされ、edge 数は 52 で打ち切り
- Given アンカー 53 個 (edge は端点ペアごとに 1 本、計 26 + 端数)
- Then `edge` 配列の長さは 52 まで (53 個目を端点に持つ edge のみ落とす)
- And stderr に警告 1 行

### Scenario: skip 行は signal 配列で順序保持
- Given TCML が `Sig1` `@skip(1)` `Sig2`
- Then signal 配列は `[Sig1 obj, {}, Sig2 obj]` 順 (空オブジェクト middle に挿入)

### Scenario: HiZ (`-`) wave マッピング
- Given TCML 信号 `s : ----` (HiZ 4 unit)
- Then `wave` は `"z..."`

### Scenario: 全 4 種 DontCare (Low/High/HiZ/Bus 文脈) いずれも `x` 1 文字
- Given 4 信号 `s1: _?_`, `s2: ~?~`, `s3: -?-`, `s4: =?=`
- Then 各信号の `?` 部分はすべて wave で `x` 1 文字 (line position の違いは表現できない)

### Scenario: `@title` の最初のみ保持、JSON に order 1 個のみ
- Given TCML に `@title A` `@title B` `@title C`
- Then `head.text == "A"` のみ
- And stderr に `2 additional` 警告

### Scenario: アンカー位置の node 文字が wave 文字列と同じ長さ
- Given TCML `s : ____@{a}_~_~`
- Then `signal[0].node` の長さが `signal[0].wave` の長さと一致

### Scenario: BusCross は wave に出力しないが last_level をリセットする
- Given TCML `data : =A=X=B=` (Bus + X + Bus)
- Then `wave` は `"=.=.."` (X 部分は wave 出力なし、ただし新セグメント開始)
- And `data` は `["A", "B"]` (2 セグメント)

### Scenario: 連続 `?` で merge された DontCareAlongLow,4 は wave で `"x..."`
- Given TCML `s : __??__` (DontCareAlongLow,4 にマージ)
- Then wave は `"x..."` (4 文字、先頭 `x` + `.` × 3)

### Scenario: clock `_=2, ~=2` のとき `p`/`n` を使わず `0`/`1` 列展開
- Given TCML `@clock(pos, _=2, ~=2)` 空クロック (chart_units=8)
- Then wave は `"0.1.0.1."` (p 不可、`0`/`1` 列で出力)

### Scenario: clock `_=1, ~=1` で `pos` のとき `p` 出力
- Given TCML `@clock(pos)` (= `_=1, ~=1` デフォルト) 空クロック (chart_units=4)
- Then wave は `"p..."`

### Scenario: clock `none` edge は `0`/`1` 列で出す (p/n は edge 必須)
- Given TCML `@clock(none)` 空クロック (chart_units=4)
- Then wave は `"0.1."` (`p`/`n` 不使用、`none` は両 edge も対象外)

### Scenario: clock `both` edge は `0`/`1` 列で出す
- Given TCML `@clock(both)` 空クロック (chart_units=4)
- Then wave は `"0.1."` (`p`/`n` 不使用)

### Scenario: 信号 1 個のみで step が gcd で 1 なら period 省略
- Given TCML 信号 1 個、step=10 (他なし)
- Then `period` フィールドは出力されない (gcd=10、period=10/10=1)

### Scenario: step 0 信号 (parser を通れば 0 にはならないが念のため)
- Given parser を通った後、たまたま step が 0 になる仮想ケース
- Then 全信号で period 省略 (§period §6 で「全 step 0 のとき省略」規定)

---

## 観点B 補強: 組合せ × WaveDrom (重要: 途中 step / clock auto / アンカー 等)

### Scenario: 途中 `@step` × `@clock(auto, _=N)` × WaveDrom (再現したかった元バグ)
- Given TCML:
  ```
  @step 10
  @clock(pos) clk
  @step 20
  data ====
  ```
- When 変換する
- Then `clk` の wave は `"p..."` (auto 展開、period は他信号と協調)
- And `data` の wave は `"=..."`
- And gcd(10, 20) = 10 → clk.period 省略 (=1)、data.period = 2

### Scenario: 途中 `@step` × `@clock(auto, _=2, ~=2)` × WaveDrom
- Given TCML:
  ```
  @step 10
  @clock(pos, _=2, ~=2) clk
  @step 20
  data ========
  ```
- Then `clk` は `0`/`1` 列展開、step=10 ベース
- And `data` は step=20 ベース、gcd=10、data.period=2

### Scenario: per-row `@step` 変更 × アンカー × WaveDrom edge
- Given TCML:
  ```
  @step 10
  Sig1 _~@{a}_~
  @step 20
  Sig2 ====@{b}====
  @-> (@{a}, @{b})
  ```
- Then Sig1.node 中に `a`、Sig2.node 中に `b`、edge 配列に `"a->b"`
- And period 計算: gcd(10, 20)=10、Sig1.period 省略、Sig2.period=2

### Scenario: per-row `@step` × `@->` ラベル付き
- Given 上記に `@-> (@{a}, @{b}) my-label`
- Then edge 配列要素 `"a->b my-label"`

### Scenario: clock auto × アンカー (本体内) × edge
- Given TCML `@clock(pos) clk _~@{a}__` (本体に `@{a}`、続きは自動展開)、もう 1 信号 `data ==@{b}===`、`@-> (@{a}, @{b})`
- Then clk 信号は `p` ではなく `0`/`1` 列で出力 (本体に明示波形があるため auto 展開モードでも edge 単位の `p`/`n` 短縮対象外、要仕様確認)
- And clk.node にアンカー位置の `a`、data.node に `b`、edge は `"a->b"`

### Scenario: BusCross 連続 (`=X=X=X=`) × WaveDrom
- Given TCML `s : =X=X=X=`
- Then wave は `"=.=.=."` (3 セグメント)
- And data フィールド省略 (テキストなし)

### Scenario: Gap 連続 (`__:_:__`) × WaveDrom
- Given TCML `s : __:_:__`
- Then wave は `"0.|0|0."` (Gap が `|` にマップ、Low の hold は `.` で続く)

### Scenario: Highlight 区間内に Bus + DontCare + Anchor
- Given TCML `s : _[==?==@{a}__]~~`
- Then wave は `[` `]` を落とし、`?` `@` を適切に処理した文字列
- And node 配列にアンカー位置の `a` を含む

### Scenario: 複数行信号名 × WaveDrom name フィールド
- Given TCML `"foo\nbar" : _~`
- Then `signal[0].name` は `"foo bar"` (改行スペース連結)

### Scenario: タイトル × `@bg` × WaveDrom (background は落とす)
- Given TCML `@bg red` `@title "T"` `Sig _~`
- Then 出力 JSON に `bg` 関連フィールドはなく、`head.text == "T"`、`signal` に Sig 1 件

### Scenario: 信号 0 件 × アンカー 0 個 × period 計算
- Given TCML が `@title "x"` のみ
- Then `signal: []`, `head.text: "x"`, `period`/`edge` は出力されない

### Scenario: アンカー 52 超過 × edge 順序保持 × 警告メッセージ書式
- Given アンカー 53 個 + 各 edge 1 本 (53 本 edge)
- Then edge 配列は最初の 52 本まで保持、53 本目は drop
- And stderr 警告 `warning: more than 52 anchors; edges referencing extra anchors are dropped` 1 行のみ

---

## 観点F 補強: WaveDrom 変換の負側 / 境界

### Scenario: 信号 0 個での WaveJSON 出力は `{"signal": []}` のみ
- Given TCML が空ファイル
- Then WaveJSON 出力は `{"signal":[]}` (head/foot/edge は出ない)

### Scenario: `@title` のみ + 信号 0 個
- Given TCML `@title "T"` のみ
- Then 出力は `{"signal":[],"head":{"text":"T"}}`

### Scenario: `@clock` なしの普通信号 `_~_~` は `0`/`1` 連で wave 化 (p/n を使わない)
- Given `clk _~_~`
- Then `wave` は `"0101"` (または同等表現)、`p` `n` を使わない

### Scenario: `wave` 末尾の hold (`.`) は省略されない
- Given `A _____` (Low 5 連)
- Then `wave == "0...."` (4 個の `.` がトリムされない)

### Scenario: `wave` 末尾の DontCare hold (`x` + `.` 列) も省略されない
- Given `A ____????`
- Then `wave == "0...x..."` (DontCareAlongLow 4 単位)

### Scenario: `@->` から参照されないアンカーには node 文字を割り当てない
- Given `A _@{a}@{b}~`、`@-> (@{a}, ...)` ではなく `@-> (foo, bar)` のみ (`a`/`b` は未参照)
- Then node 配列に `a`/`b` 用文字は出ない (ドット `.` のみ)

### Scenario: `@->` から参照されるアンカーのみ node 文字を取る (混在ケース)
- Given `A _@{x}@{y}@{z}~`、`@-> (@{x}, @{z})` のみ
- Then `x` と `z` だけが node 文字 (a/b など) を割り当てられ、`y` は `.`

### Scenario: アンカー 0 個 + 矢印 0 個では edge/node を出力しない
- Given `A _~_~` のみ
- Then 出力 JSON に `edge` フィールドなし、各信号に `node` フィールドなし

### Scenario: `@clock(auto)` 全信号 auto で wave が空になる + 警告
- Given `@clock(auto)\nclk1\n@clock(auto)\nclk2`
- Then 全信号の wave は空文字列、警告メッセージ `"all clock signals use auto and there is no reference signal to expand"` 等

### Scenario: 信号名内のクォート文字 `\"` の WaveJSON エスケープ
- Given 信号名 `"a\"b"` (literal `a"b`)
- Then 出力 JSON で `"name":"a\"b"` (JSON 規約に従ったエスケープ)

### Scenario: `head.text` 内の `"` も JSON エスケープ
- Given `@title "He said \"hi\""`
- Then `head.text == "He said \"hi\""`、JSON.parse 可能

### Scenario: アンカー 52 個ちょうどは警告ゼロ・edge 全保持
- Given アンカー 52 個 + edge 51 本
- Then 警告 0 件、edge 全 51 本保持

### Scenario: 1 信号のみで step gcd が 1 のとき period 省略
- Given `@step 7\nA _~_~`
- Then 出力 JSON に `period` なし (gcd で割って 1 になる)

---

## 観点L: WaveDrom 化の正側網羅 (補強)

## @not-implemented
### Scenario: 矢印 `head=start` のとき WaveDrom edge style に start 形式
- Given `@-> (@{a}, @{b}, head=start)`
- Then edge 配列要素の style は `<-` 系 (始点側矢印あり)

## @not-implemented
### Scenario: 矢印 `head=both` のとき WaveDrom edge style は `<->` 系
- Given `@-> (@{a}, @{b}, head=both)`
- Then edge style 文字列は `<->` (両端矢印)

## @not-implemented
### Scenario: 矢印 `head=none` のとき WaveDrom edge style はライン系
- Given `@-> (@{a}, @{b}, head=none)`
- Then edge style は `-` (頭なし)

## @not-implemented
### Scenario: 矢印 label に `\n` を含むケース
- Given `@-> (@{a}, @{b}, label="line1\nline2")`
- Then edge label 文字列は `"line1\nline2"` (改行は JSON エスケープ `\n` で保持)
- And JSON.parse 可能

## @not-implemented
### Scenario: 矢印 label に `\t` (タブ) を含むケース
- Given `@-> (@{a}, @{b}, label="a\tb")`
- Then edge label に `\t` が JSON エスケープで保持

## @not-implemented
### Scenario: node 文字割り当てが a..z を使い切って A..Z に移る境界 (27 番目)
- Given アンカー 27 個 + edge 1 本以上 (全アンカー参照)
- Then 26 番目までは `a..z`、27 番目は `A` (大文字開始)
- And node 文字列に重複なし

## @not-implemented
### Scenario: node 文字割り当て 52 番目ぴったりが `Z`
- Given アンカー 52 個全参照
- Then 52 番目の文字は `Z` (a..z で 26 + A..Z で 26 = 52)

## @not-implemented
### Scenario: node 文字 53 番目以降は割り当てなし (打ち切り)
- Given アンカー 53 個全参照
- Then 53 番目以降は `.` のまま、警告メッセージで超過数を報告

## @not-implemented
### Scenario: clock 信号と通常信号の混在で `period` gcd 計算
- Given `@step 4\n@clock(pos) clk\n@step 2\nA _~_~`
- Then period は `gcd(4, 2) = 2` から逆算した値
- And 各信号の wave 文字列長は同期

## @not-implemented
### Scenario: clock 信号 step 違いの混在で gcd が 1 になる
- Given `@step 3\n@clock(pos) clk1\n@step 5\n@clock(pos) clk2`
- Then gcd(3, 5) = 1 のため period は省略 (または 1)

## @not-implemented
### Scenario: `data` 配列の長さと wave bus 区間数の整合
- Given `data: =A=B=C` (Bus 3 区間、各区間にラベル)
- Then `wave` の `=` 個数 == `data` 配列長

## @not-implemented
### Scenario: `data` 配列の長さがラベル付き Bus 区間と一致 (空ラベル区間は飛ばさない)
- Given `data: =A==B` (Bus 1 + Bus 1、間に hold)
- Then `wave` は `"=.=."`、`data` は `["A", "B"]`

## @not-implemented
### Scenario: head.text が空文字列のとき head フィールド省略
- Given `@title ""`
- Then 出力 JSON に `head` フィールドなし (空 head は出さない)

## @not-implemented
### Scenario: head.text に `\n` を含む `@title`
- Given `@title "line1\nline2"`
- Then `head.text` の JSON 値は `"line1\nline2"` (エスケープ済み)

## @not-implemented
### Scenario: signal 配列の順序が TCML 文書順と一致 (ソートしない)
- Given `Z _\nA _\nM _` (信号名 Z, A, M の順)
- Then 出力 JSON `signal` 配列の順序も Z, A, M (アルファベット順にしない)

---

## 観点R 補強: WaveDrom 対応物のない要素

## @not-implemented
### Scenario: `@-> (a, b, style=dotted)` を WaveDrom edge style 近似
- Given dotted 矢印
- Then WaveDrom edge style は `~>` (curve) または `->` に dashed 近似
- Note: WaveDrom に dotted 専用 style がないため、近似ルールを spec で定める必要あり (spec gap 候補)

## @not-implemented
### Scenario: `@-> (a, b, head=none)` の WaveDrom edge 出力
- Given `head=none`
- Then WaveDrom edge style `-` (head なし) で出力
- And label もそのまま含む

## @not-implemented
### Scenario: 全信号の `data` 配列が空のとき `data` フィールド省略
- Given Bus 区間 0 のチャート (`A _~_~` のみ)
- Then 各信号の JSON に `data` フィールドが含まれない (空配列を出さない)

## @not-implemented
### Scenario: `data: []` (Bus 区間ありだが全空ラベル) の出力
- Given Bus 区間 3 個全てラベル省略 (`X==X==X==`)
- Then `data` フィールドの扱いは仕様準拠 (空文字列 3 個 vs フィールド省略)
- Note: 仕様未定義なら spec gap

## @not-implemented
### Scenario: `head.text` フィールドは空でない `@title` のみ出力
- Given `@title "T"`
- Then `head: { text: "T" }` 出力
- And `@title` なしのとき `head` フィールド省略

## @not-implemented
### Scenario: tchart 独自 `@bg` / `@bgcolor*` は WaveDrom 出力で破棄
- Given `@bg #ffeecc\nA _~`
- Then WaveJSON には背景色情報なし (config 拡張も出さない)
- Note: 仕様で出力するならフォーマット定義が必要

## @not-implemented
### Scenario: tchart `@page-margin` は WaveDrom 出力で破棄
- Given `@page-margin 20\nA _~`
- Then WaveJSON にページマージン情報なし

## @not-implemented
### Scenario: `@->` ラベル位置 (中点 / 始点 / 終点) は WaveDrom で破棄
- Given `@-> (a, b, label="X", label-pos=mid)`
- Then WaveDrom edge は label 文字のみ保持、位置情報は破棄
- Note: WaveDrom 仕様に label-pos がないため

## @not-implemented
### Scenario: tchart アンカー名と WaveDrom node 文字の対応関係をログ出力 (debug)
- Given アンカー `@{first}` `@{second}`
- Then debug ログに `first → a, second → b` のマッピングを出力 (仕様で定義されていれば)
- Note: 仕様未定義なら skip / spec gap

## @not-implemented
### Scenario: clock マーカー (▽ / △) は WaveDrom 出力で破棄 (clock 自体は wave に展開)
- Given `@clock(pos)\nclk _~_~`
- Then WaveJSON に EdgeMark 情報なし、wave のみ `"p"` 等で表現
- Note: clock の WaveDrom 表現は仕様確認

---

## 観点Bus-Data 補強: `LevelRun(Bus, n)` 行 65 と「Bus 区間と `data` 配列の対応」全体

仕様: [`docs/spec/wavedrom.md`](../spec/wavedrom.md) §wave 文字列マッピング 行 65 `LevelRun(Bus, n)`、§Bus 区間と `data` 配列の対応。

「1 Bus 区間 = `data` の 1 エントリ = 1 中央ラベル文字列 (parser merge 後の半角スペース連結)」というセマンティクスを多角的に検証する。

### 違反系 (Negative)

## @not-implemented @negative
### Scenario: parser merge 後 `=A=B=C` が 3 区間扱いになると違反 (1 区間に集約されるべき)
- Given TCML 信号 `data: =A=B=C` (parser merge で Bus 3 unit + 中央ラベル "A B C" に正規化)
- When 変換する
- Then `signal[0].wave` は `"=.."` (Bus 1 セグメント) でなければならない
- And `signal[0].data` は `["A B C"]` (1 エントリのみ) でなければならない
- And `signal[0].data` が `["A", "B", "C"]` のように 3 エントリ化されているのは違反

## @not-implemented @negative
### Scenario: 単一ラベル区間が `["", "A"]` 等に分割されるのは違反
- Given TCML 信号 `data: ==A==` (Bus 1 区間 + テキスト "A")
- When 変換する
- Then `signal[0].data` は `["A"]` (長さ 1) でなければならない
- And `["", "A"]` `["A", ""]` 等の長さ 2 以上の配列は違反

## @not-implemented @negative
### Scenario: 全 Bus 区間が無ラベルなのに `data: []` を出力するのは違反 (フィールド省略が正しい)
- Given TCML 信号 `data: ====` (Bus 4 unit、テキストなし)
- When 変換する
- Then 出力 JSON の `signal[0]` に `data` キー自体が存在しない (省略) でなければならない
- And `"data": []` のような空配列出力は違反

## @not-implemented @negative
### Scenario: 一部 Bus 区間のみラベル時に空文字埋めを省略するのは違反
- Given TCML 信号 `data: ==X==A==` (Bus 2 区間、先頭区間 ラベルなし、後ろの区間 "A") を仮定 (parser merge により 2 区間として分割される構造)
- When 変換する
- Then `signal[0].data` 配列長は wave 中の Bus セグメント数と一致しなければならない
- And ラベル無し区間に対応する位置は空文字列 `""` で埋めなければならない (`["", "A"]`)
- And 詰めて `["A"]` (長さ 1) にするのは違反

## @not-implemented @negative
### Scenario: ラベル区切りが半角スペース以外 (`,` `_` `/` 等) は違反
- Given TCML 信号 `data: =A=B=C` (parser merge 後 1 区間 + テキスト "A","B","C")
- When 変換する
- Then `data[0]` は `"A B C"` (半角スペース 1 個区切り) でなければならない
- And `"A,B,C"` `"A_B_C"` `"ABC"` 等は違反

## @not-implemented @negative
### Scenario: ラベル内既存スペースを潰すのは違反 (TCML テキスト文字列としての原文を維持)
- Given parser 出力でラベル文字断片が `"A"`, `" "`, `"B"` (空白を含むラベル) のような場合
- When 変換する
- Then `data` エントリは parser が確定した文字列をそのまま入れ、内部のスペースを 1 個に正規化するなどの追加変換は行わない
- And 既存スペースをトリムするのは違反

## @not-implemented @negative
### Scenario: `Transition(BusCross)` が wave 文字に出力されるのは違反
- Given TCML 信号 `data: =A=X=B=` (Bus + BusCross + Bus、parser 上の transition)
- When 変換する
- Then `signal[0].wave` 文字列は `Transition(BusCross)` 由来の文字 (`x` 等) を含まず、`"=.=.."` 形となる
- And wave に余計な遷移文字が混入するのは違反

## @not-implemented @negative
### Scenario: `data` 配列長が wave 中の `=` セグメント数と不一致は違反
- Given TCML 信号 `data: =A=B=X=C=D` (parser merge 後 Bus 2 区間)
- When 変換する
- Then `signal[0].wave` 中の独立 `=` 開始セグメント数 (2) と `signal[0].data.len()` (2) は一致しなければならない

### エッジケース

## @not-implemented
### Scenario: Bus 1 unit + ラベル 1 個 (最小ケース)
- Given TCML 信号 `data: =A` (Bus 1 unit、テキスト "A")
- When 変換する
- Then `signal[0].wave` は `"="` (1 文字)
- And `signal[0].data` は `["A"]`

## @not-implemented
### Scenario: Bus 1 unit + ラベル 0 個 (最小ケース、data 省略)
- Given TCML 信号 `data: =` (Bus 1 unit、テキストなし)
- When 変換する
- Then `signal[0].wave` は `"="`
- And `signal[0]` に `data` フィールドは出力されない

## @not-implemented
### Scenario: Bus 区間の中央ラベルが UTF-8 マルチバイト文字
- Given TCML 信号 `data: =あ=い=う` (parser merge 後 1 区間、ラベル断片 "あ","い","う")
- When 変換する
- Then `signal[0].wave` は `"=.."` (1 セグメント 3 unit)
- And `signal[0].data` は `["あ い う"]` (半角スペース連結)
- And JSON は UTF-8 として有効

## @not-implemented
### Scenario: 連続 `==` (ラベルなし) は 1 区間扱い、data 省略
- Given TCML 信号 `data: ==` (Bus 2 unit、テキストなし)
- When 変換する
- Then `signal[0].wave` は `"=."` (1 セグメント 2 unit)
- And `data` フィールド出力なし

## @not-implemented
### Scenario: 最先頭 Bus 区間のみラベル、後続ラベルなし
- Given TCML 信号 `data: ==A==X==` (parser merge により Bus 区間 2 個、先頭のみ "A")
- When 変換する
- Then `signal[0].data` は `["A", ""]` (先頭にラベル、末尾は空文字埋め)
- And 配列長 == セグメント数 == 2

## @not-implemented
### Scenario: 最末尾 Bus 区間のみラベル、先頭ラベルなし
- Given TCML 信号 `data: ===X==A=` (Bus 区間 2 個、後ろ側のみ "A")
- When 変換する
- Then `signal[0].data` は `["", "A"]` (先頭空文字埋め、末尾にラベル)

## @not-implemented
### Scenario: Bus 区間 5 個 + ラベル付き / 無し 混在
- Given TCML 信号 `data: =A=X==X=B=X==X=C` (parser merge により Bus 区間 5 個、ラベル順 "A","","B","","C")
- When 変換する
- Then `signal[0].data` の長さは 5
- And `signal[0].data` は `["A", "", "B", "", "C"]` (空文字埋め位置が正確)

## @not-implemented
### Scenario: Bus 区間で hold が長い + ラベル 1 個
- Given TCML 信号 `data: =A======` (Bus 7 unit、テキスト "A" 中央配置)
- When 変換する
- Then `signal[0].wave` は `"=......"` (1 セグメント 7 unit)
- And `signal[0].data` は `["A"]` (1 エントリ)

## @not-implemented
### Scenario: ラベルが数字のみ
- Given TCML 信号 `data: =1=2=3` (parser merge 後 1 区間、ラベル "1","2","3")
- When 変換する
- Then `signal[0].data` は `["1 2 3"]`

### 機能満足系

## @not-implemented @smoke
### Scenario: 行 65 仕様: `LevelRun(Bus, 1)` 単発は `=` 1 文字 + data 集約
- Given TCML `data: =X` (1 unit Bus 区間、X は parser 上は `Transition(BusCross)` + `LevelRun(Bus, 1)` に分解、テキストなし)
- When 変換する
- Then `signal[0].wave` は `"="` (1 文字、BusCross は wave に出ない)
- And `data` フィールド省略

## @not-implemented @smoke
### Scenario: 行 65 仕様: `LevelRun(Bus, n>1)` は `=` + `.` × (n-1)
- Given TCML 信号 `data: ====` (Bus 4 unit、テキストなし)
- When 変換する
- Then `signal[0].wave` は `"=..."` (1 文字 `=` + `.` × 3)

## @not-implemented
### Scenario: 仕様例 `==A==` (4 unit, 1 ラベル "A") → wave `=...` data `["A"]`
- Given TCML 信号 `data: ==A==`
- When 変換する
- Then `signal[0].wave == "=..."`
- And `signal[0].data == ["A"]`

## @not-implemented
### Scenario: 仕様例 `=A=B=C` (parser merge 後 3 unit, ラベル "A B C") → wave `=..` data `["A B C"]`
- Given TCML 信号 `data: =A=B=C`
- When 変換する
- Then `signal[0].wave == "=.."`
- And `signal[0].data == ["A B C"]`

## @not-implemented
### Scenario: 仕様例 `=A=B=X=C=D` (Bus 2 + BusCross + Bus 3) → wave `=.=..` data `["A B", "C D"]`
- Given TCML 信号 `data: =A=B=X=C=D`
- When 変換する
- Then `signal[0].wave == "=.=.."` (BusCross で last_level リセット、新セグメント開始)
- And `signal[0].data == ["A B", "C D"]`

## @not-implemented
### Scenario: Bus → Low → Bus の遷移で data は Bus 区間ぶんのみ集計
- Given TCML 信号 `s : =A=__=B=` (Bus 区間 + Low 区間 + Bus 区間)
- When 変換する
- Then `signal[0].wave` は `"=.0.=."` 系 (Low 区間は `0`/`.`、Bus は `=`/`.` 系)
- And `signal[0].data` は `["A", "B"]` (Bus 区間が 2 個、それぞれにラベル)

## @not-implemented
### Scenario: Bus 区間内 BusOpen / BusClose を跨ぐラベル
- Given TCML 信号 `data: ==A==` (BusOpen 〜 BusClose の Bus 区間 1 個 + ラベル "A")
- When 変換する
- Then `signal[0].data == ["A"]`
- And transition 自体は wave に出力されない

### 組合せ系

## @not-implemented
### Scenario: Bus 区間 × DontCare 区間: DontCare 中の `?` は data に入らない
- Given TCML 信号 `s : =A=??=B=` (Bus + DontCare + Bus、Bus 区間にのみラベル)
- When 変換する
- Then `signal[0].data` は `["A", "B"]` (DontCare は data に寄与しない)
- And `signal[0].wave` には `x` (DontCare) + `=` (Bus) が混在

## @not-implemented
### Scenario: Bus 区間 + Highlight 区間 `[` `]` + ラベル
- Given TCML 信号 `data: [=A=]=B=` (ハイライト内に Bus 区間 + 外に Bus 区間)
- When 変換する
- Then ハイライト記号は wave に出ず、`signal[0].data` は `["A", "B"]` (両 Bus 区間のラベル順)

## @not-implemented
### Scenario: Bus 区間内アンカー `@{a}` + ラベル: data には影響なし、node に位置反映
- Given TCML 信号 `data: =@{a}A=` (Bus 区間内に Anchor とテキスト "A")
- When 変換する
- Then `signal[0].data == ["A"]` (Anchor は data に入らない)
- And `signal[0].node` は wave と同長で Anchor 位置に `a`

## @not-implemented
### Scenario: Bus 区間 + Gap `:` + Bus 区間
- Given TCML 信号 `data: =A=:=B=` (Bus → Gap → Bus、ラベル "A" と "B")
- When 変換する
- Then `signal[0].wave` には `|` (Gap) が含まれ、Bus セグメントは Gap 前後で分かれる
- And `signal[0].data == ["A", "B"]` (Gap は data セグメントを区切る)

## @not-implemented
### Scenario: 複数信号で Bus ラベルあり / なし混在 (信号ごとに独立)
- Given TCML に 2 信号
  ```
  s1 : =A=B=
  s2 : ====
  ```
- When 変換する
- Then `signal[0].data == ["A B"]` (s1 はラベルあり)
- And `signal[1]` に `data` フィールドなし (s2 は全 Bus 無ラベル)

## @not-implemented
### Scenario: Bus 区間 5 個 + ラベル位置を入れ替えた違うケース ("X","","","Y","")
- Given TCML 信号 `data: =X=W=W=W=Y=W=` のように merge 後 Bus 5 区間 + ラベル位置 1 と 4 にあるケース (擬似的に表現、実 TCML は parser merge を経る)
- When 変換する
- Then `signal[0].data == ["X", "", "", "Y", ""]` (長さ 5、ラベル位置が正確)

## @not-implemented
### Scenario: `@step` 変更 × Bus ラベル: data はそのまま、period は別計算
- Given TCML
  ```
  @step 10
  s1 : =A=B=
  @step 20
  s2 : =C=
  ```
- When 変換する
- Then `signal[0].data == ["A B"]`、`signal[1].data == ["C"]`
- And data 配列の中身は `@step` 値の影響を受けない (data 内容と period 計算は独立)

## @not-implemented
### Scenario: 区間中央ラベルが連続区間境界で混ざらないこと
- Given TCML 信号 `s : =A==B=` (parser merge により Bus 2 区間 (前後)、ラベル "A" と "B" がそれぞれ別区間)
- When 変換する
- Then `signal[0].data` は `["A", "B"]` (前区間のラベル "A" と次区間 "B" が混ざって "A B" にならない)
- Note: 区切りは `Transition` (BusOpen/BusClose/BusCross) の有無で決まり、parser merge ルールに従う

## @not-implemented
### Scenario: 同 1 区間内ラベル断片が 4 個以上 (`=A=B=C=D`)
- Given TCML 信号 `data: =A=B=C=D` (parser merge 後 Bus 1 区間 4 unit、ラベル "A","B","C","D")
- When 変換する
- Then `signal[0].wave == "=..."` (1 セグメント 4 unit)
- And `signal[0].data == ["A B C D"]` (4 ラベルが半角スペース 1 個区切りで連結)

---

## 観点TopLevel: トップレベル `period` 不在 (フィールド表 / §period スコープ)

仕様: [`docs/spec/wavedrom.md`](../spec/wavedrom.md) §トップレベル構造 のフィールド表 (`signal` / `head.text` / `edge` のみ) と §period (各信号の `period` 計算は §1〜§6 のみ。トップレベルへの追加出力ルールは存在しない)。

WaveJSON 出力のトップレベルオブジェクトに `period` キーが現れないことを多角的に検証する。各信号 (`signal[i]`) の `period` フィールドは §period §5 のデフォルト 1 省略ルールに従う一方で、ルートには `period` を出力する規定が一切無いことに注意する。

### 違反系 (Negative)

## @not-implemented @negative
### Scenario: 全信号同一 step でもトップレベル `period` フィールドは出力されない
- Given TCML で全信号が `@step 10` のもとに定義された 2 信号 (`signal` 配列要素 2)
- When 変換する
- Then 出力 JSON のルートオブジェクトに `period` キーが存在しない (`obj.contains_key("period") == false`)
- And ルートの `period` を `10` / `1` / `null` 等いずれの値で出力するのも仕様違反

## @not-implemented @negative
### Scenario: 信号間 step が異なる場合でもトップレベル `period` は出力されない
- Given TCML 信号 A (step=10) と信号 B (step=14) が混在
- When 変換する
- Then 信号オブジェクト側に `signal[0].period = 5` / `signal[1].period = 7` (§period §4) が現れる
- And ルートオブジェクトには `period` キーが現れない
- And ルートに gcd = 2 を `period: 2` として出力するのは仕様違反 (§period のスコープは各信号のみ)

## @not-implemented @negative
### Scenario: gcd 基準値をトップレベル `period` として書き出すのは違反
- Given TCML 信号 A (step=10) と信号 B (step=20) (gcd = 10)
- When 変換する
- Then 出力 JSON のルートに `period` キーが現れない
- And ルートに `"period": 10` (gcd) や `"period": 1` (signal.period 既定) を出力するのは仕様違反

## @not-implemented @negative
### Scenario: 信号 0 件でもトップレベル `period` は出力されない
- Given TCML が `@title only` のみで信号行を持たない
- When 変換する
- Then 出力 JSON のルートに `period` キーが現れない (§period §6 「全信号 step が 0 / 信号 0 個のとき period 計算を省略」がトップレベル出力を許容しない)
- And `"period": 0` `"period": null` `"period": 1` 等の出力は仕様違反

## @not-implemented @negative
### Scenario: 全信号 step 整数化後 0 のケースでもトップレベル `period` は出さない
- Given parser を通った結果 (仮想ケース) で全信号の整数化 step が 0
- When 変換する
- Then §period §6 「全 step 0 のとき period 計算省略」は **各信号** の `period` を省略する規定であって、ルートに `period` を出力する許可ではない
- And ルートに `period` キーが現れたら違反

## @not-implemented @negative
### Scenario: clock 自動展開を含む場合でもトップレベル `period` は出さない
- Given TCML に `@clock(pos) clk` (空 wave) + `data ====` の 2 信号 (step 同一)
- When 変換する
- Then ルートに `period` キーが現れない
- And clock 信号が `"p..."` 形式で出力されることはトップレベル `period` を伴わない

## @not-implemented @negative
### Scenario: `@title` (`head.text` 出力) を伴う場合でもトップレベル `period` は出さない
- Given TCML `@title T\nA _~\nB _~` (head.text あり、信号 2 件、step 同一)
- When 変換する
- Then 出力ルートは `signal` と `head` のみを持ち、`period` キーは無い
- And `head` 直下にも `period` を出さない (フィールド表に `head.period` は存在しない)

## @not-implemented @negative
### Scenario: `@->` (`edge` 出力) を伴う場合でもトップレベル `period` は出さない
- Given TCML `clk : ~_@{a}_~` `data : ==@{b}==` `@-> (@{a}, @{b})`
- When 変換する
- Then 出力ルートは `signal` と `edge` を持ち得るが、`period` キーは現れない
- And `edge` 配列の存在はルート `period` 追加を許可しない

## @not-implemented @negative
### Scenario: トップレベル `period` を 1 として出力するのは違反 (WaveDrom デフォルトと一致しても省略)
- Given TCML 信号 1 件 step=1 (gcd で割って 1)
- When 変換する
- Then 信号側 `signal[0].period` は §period §5 により省略
- And ルート `period` キーも存在しない (`"period": 1` 出力は違反)

## @not-implemented @negative
### Scenario: フィールド表外の任意キー (今回は `period`) を勝手にトップレベルに追加するのは違反
- Given 任意の有効 TCML
- When 変換する
- Then ルートオブジェクトのキー集合は §トップレベル構造 のフィールド表に列挙された `signal` / `head` / `edge` の部分集合に限られる
- And `period` を含むこれら以外のキーが現れたら仕様違反

### エッジケース

## @not-implemented
### Scenario: 信号 1 件 step=1 で gcd=1 のとき、信号側 `period` 省略 + ルート `period` も無し
- Given TCML 信号 1 件 step=1
- When 変換する
- Then `signal[0]` に `period` フィールド無し (§period §5 デフォルト省略)
- And ルートにも `period` フィールド無し (§フィールド表に存在しない)

## @not-implemented
### Scenario: 信号 1 件 step=7 で gcd=7 → signal.period=1 省略、ルート `period` も無し
- Given TCML `@step 7\nA _~_~`
- When 変換する
- Then `signal[0]` に `period` フィールド無し (`step / gcd = 7 / 7 = 1`)
- And ルートにも `period` キー無し (gcd 値 7 をルートに残さない)

## @not-implemented
### Scenario: 信号 2 件で片方のみ `period` を持つケースでもルートに集約しない
- Given TCML 信号 A (step=10) / B (step=20)
- When 変換する
- Then `signal[0]` (= A) は `period` 省略 (gcd=10、period=1)
- And `signal[1]` (= B) は `period: 2`
- And ルートには `period` キーが現れない (片方が `period` を持つことはルート集約の根拠にならない)

## @not-implemented
### Scenario: signal 配列空 + head.text のみ
- Given TCML `@title T` のみ
- When 変換する
- Then 出力は `{"signal": [], "head": {"text": "T"}}` (この 2 キーのみ)
- And ルートに `period` キーは無い

## @not-implemented
### Scenario: 完全空 TCML
- Given TCML が空 (lines 全くなし)
- When 変換する
- Then 出力は `{"signal": []}` (`signal` のみ)
- And ルートに `period` キーは無い (`head` / `edge` も無い)

## @not-implemented
### Scenario: `@skip` のみで SignalRow 0 個
- Given TCML `@skip(1)\n@skip(2)` のみ
- When 変換する
- Then `signal` 配列には空オブジェクト `{}` が 2 個 (`@skip` 由来) のみ
- And §period §6「signal 配列に信号オブジェクトが 0 個」は SignalRow が 0 件のケースで適用 (skip 由来の `{}` は信号オブジェクトでない扱い) で各信号 `period` は計算されず、ルートにも `period` は無い

## @not-implemented
### Scenario: 非整数 step が round されてもルート `period` は出さない
- Given TCML 信号 `@step 10.6` の信号 1 件
- When 変換する
- Then 該当信号の step は 11 として扱われ、警告が 1 行出る
- And `signal[0].period` は §period §5 により省略 (gcd=11、period=1)
- And ルートに `period` キーは現れない (round 元値 10.6 / 整数化値 11 のいずれもルートに出さない)

### 機能満足系 (Positive)

## @not-implemented @smoke
### Scenario: ルートオブジェクトのキーは `signal` / `head` / `edge` の 3 種のみ (フィールド表準拠)
- Given 任意の有効 TCML (信号 1 件以上 + `@title` + `@->`)
- When 変換する
- Then ルートオブジェクトのキー集合 ⊆ {`signal`, `head`, `edge`}
- And `period` を含むそれ以外のキーは出現しない

## @not-implemented @smoke
### Scenario: 各信号 `period` のセマンティクスはルートに波及しない
- Given TCML 信号 A (step=10) / B (step=14) (gcd=2、A.period=5、B.period=7)
- When 変換する
- Then `signal[0].period == 5`、`signal[1].period == 7`
- And ルート `period` キーは無い (signal 側 `period` の存在はルート出力を意味しない)

## @not-implemented
### Scenario: §period の各ステップ §1〜§6 は signal 配列要素にのみ作用する
- Given §period の §1〜§6 が網羅されるよう、step 異種混在 / step 1 / step 0 ケース (仮想) を含む複数 TCML
- When 各 TCML を変換する
- Then 各ケースで signal 要素の `period` 出力有無は §period §5 / §6 に従う
- And どのケースでもルートに `period` キーは現れない (§period がトップレベル出力に作用しない)

### 組合せ系

## @not-implemented
### Scenario: `head` (`@title`) × signal.period 出力 × ルート `period` 不在
- Given TCML `@title T\n@step 10\nA _~\n@step 20\nB _~`
- When 変換する
- Then `head.text == "T"`、`signal[1].period == 2` (B のみ period)
- And ルートに `period` キー無し
- And `head` 内にも `period` キー無し

## @not-implemented
### Scenario: `edge` (`@->`) × signal.period 出力 × ルート `period` 不在
- Given TCML `@step 10\nclk : ~_@{a}_~\n@step 20\ndata: ==@{b}==\n@-> (@{a}, @{b})`
- When 変換する
- Then `signal[1].period == 2`、`edge[0]` は `"a->b"` 系
- And ルートに `period` キー無し

## @not-implemented
### Scenario: `head` × `edge` × signal.period × ルート `period` 不在 (全要素併用)
- Given TCML `@title sample\n@step 10\nclk : ~_@{a}_~\n@step 20\ndata: ==@{b}==\n@-> (@{a}, @{b})`
- When 変換する
- Then ルートキー集合は `{signal, head, edge}` のみ
- And `period` キーはルート・`head`・`edge`・`signal` 配列直下のいずれにも (signal 要素の中以外には) 現れない

## @not-implemented
### Scenario: clock 自動展開 (`p`/`n`) × ルート `period` 不在
- Given TCML `@clock(pos) clk` (chart_units=4) と `data ====` (step 同一) の 2 信号
- When 変換する
- Then `signal[0].wave == "p..."` (clock 自動展開)
- And `signal[0].period` / `signal[1].period` は共に省略 (gcd で割って 1)
- And ルートに `period` キー無し

## @not-implemented
### Scenario: clock auto + 通常信号 + step 違いの混在でも、`period` は信号側のみ
- Given TCML `@step 4\n@clock(pos) clk\n@step 2\nA _~_~`
- When 変換する
- Then `signal[i].period` は §period §4 に従って算出 (clk と A の gcd ベース)
- And ルートに `period` キーが現れない

## @not-implemented
### Scenario: `@skip` 混在 × step 違い × ルート `period` 不在
- Given TCML `@step 10\nA _~\n@skip(1)\n@step 20\nB _~`
- When 変換する
- Then `signal` 配列は `[A obj, {}, B obj]` の順
- And `signal[0].period` 省略 (gcd=10、period=1)、`signal[2].period == 2`
- And `signal[1]` (`@skip` 由来の `{}`) には `period` キー無し (空オブジェクトのまま)
- And ルートに `period` キー無し

## @not-implemented
### Scenario: WaveDrom 互換性確認: ルート `period` が存在しなくても WaveDrom が描画可能
- Given 任意の有効 TCML から生成した WaveJSON 出力
- When 出力 JSON を WaveDrom 仕様 (WaveJSON schema) で検証
- Then ルートに `period` キーが無いことは WaveJSON schema 違反にはならない (WaveDrom 仕様にトップレベル `period` は無く、tchart 出力もこれに整合)

## @not-implemented
### Scenario: 2 周目: 異なる経路 (`@step` × clock × アンカー × `@title` × `@skip`) でもルート `period` 不在
- Given TCML
  ```
  @title combo
  @step 10
  @clock(pos) clk
  @skip(1)
  @step 20
  data : ==@{a}==
  @-> (@{a}, @{a})
  ```
  (※ `@-> (@{a}, @{a})` は同一アンカー 2 端点という仮設定。spec が許容しない場合は 2 アンカーで置き換える)
- When 変換する
- Then ルートキー集合 ⊆ {`signal`, `head`, `edge`}
- And ルートに `period` キーが現れない
- And 信号側 `period` 値は §period §1〜§6 に従う

## @not-implemented
### Scenario: 2 周目: 全信号 auto clock で wave が空のケースでもルート `period` 不在
- Given TCML `@clock(pos) clk1\n@clock(pos) clk2` (auto 同士で展開不能)
- When 変換する
- Then `signal[0].wave == ""`、`signal[1].wave == ""`
- And §period §6 の「signal が 0 個または全 step が 0」相当判定で各信号 `period` 省略
- And ルートにも `period` キーが現れない (§period §6 の省略はトップレベル `period` の出力を許可しない)

## @not-implemented
### Scenario: 2 周目: ルートのキー順序ではなくキー集合で検証 (`period` の不在を直接 assert)
- Given 任意の WaveJSON 出力 (`serde_json::Value`)
- When `value.as_object().unwrap().keys()` を集める
- Then `keys.contains("period") == false`
- And この検証は他テストと併用してフィールド表 (§トップレベル構造) の網羅証拠とする
