# 統合テスト (End-to-End)

TCML テキスト入力から SVG 出力までの一連のフローを検証する統合テスト仕様。

---

## 既存サンプルの再現

## @not-implemented @smoke
### Scenario: sample.tc が描画できる
- Given `docs/images/sample.tc`
- When parser → clock 展開 → アンカー解決 → layout → SVG レンダラを通す
- Then エラーにならず、SVG 文字列が生成される
- And SVG に各信号の polyline が含まれる

## @not-implemented
### Scenario: all_transitions.tc の全遷移パターン
- Given `docs/images/all_transitions.tc`
- Then すべての TransitionKind (`SingleEdge` / `BusOpen` / `BusClose` / `BusCross`) が描画され、独立 `<line>` 要素は出現しない

## @not-implemented
### Scenario: gap.tc は前後の波形が切断される
- Given `docs/images/gap.tc`
- Then Gap 前後の polyline が独立要素として分離している

## @not-implemented
### Scenario: highlight_dontcare.tc
- Given `docs/images/highlight_dontcare.tc`
- Then `dontcares` レイヤに polygon (y_h〜y_l 範囲)、`waveforms` レイヤに DontCare 内部水平線
- And `highlights` レイヤに矩形が出力される

## @not-implemented
### Scenario: multiline.tc
- Given `docs/images/multiline.tc`
- Then 複数行信号名が `<tspan>` で改行表示される
- And capwidth が最長行に合わせて計算されている

## @not-implemented
### Scenario: vertical_line.tc
- Given `docs/images/vertical_line.tc`
- Then guides レイヤに縦線 `<line>` が含まれ、上下に page-margin/2 はみ出している

## @not-implemented
### Scenario: async_clock.tc
- Given `docs/images/async_clock.tc`
- Then 途中変更された `step` が後続信号に反映されている

## @not-implemented
### Scenario: labels.tc
- Given `docs/images/labels.tc`
- Then 各 LevelRun のラベルが区間中央に `<text>` として配置されている

---

## 新機能 (B-1 / B-2 / B-3) の E2E

## @not-implemented
### Scenario: `@->` 矢印を含む TCML
- Given アンカー `@{a}` `@{b}` 定義 + `@-> (@{a}, @{b}: red dashed) ラベル`
- When 全パイプラインを通す
- Then SVG `arrows` レイヤに矢印・ラベルが描画される

## @not-implemented
### Scenario: `@clock` 自動展開
- Given `@clock(pos)` のみのクロック信号 + 他信号
- When 全パイプラインを通す
- Then クロック信号が他信号と同じ長さで展開される
- And エッジ矢印が `arrows` レイヤに含まれる

## @not-implemented
### Scenario: `@clock` auto 展開行と明示波形行の polyline 末端 x が一致
- Given `@clock(none)` の後に auto 行 (本体空) と明示波形行 `~_~_~_~_` を並べた TCML
- When 全パイプラインを通す
- Then 両 clock 行の `<polyline>` 末端 x 座標が一致 (auto 行は `chart_units = 8` まで `~_~_~_~_` 相当に展開される)

## @not-implemented
### Scenario: `@step` を行ごとに変えても auto 展開行は他信号の右端におよそ揃う
- Given `@step 20` → `Clock _~_~_~_~_~_~` (12 units = 240px) → `@step 10` → `@clock` → 空 auto 行 を並べた TCML
- When 全パイプラインを通す
- Then 1 行目 polyline 末端 x ≈ 240px (12 × step=20) で、auto 行 polyline 末端 x との差が `step/2 = 5px` 以内に収まる (auto 行は `round(240 / 10) = 24` 単位に展開される)

## @not-implemented
### Scenario: 複数 auto 行 + explicit 1 行 で全 auto 行が同じ explicit 信号を参照
- Given `@clock` を 2 回連続で書いて auto 空行を 2 本作り、その後に `Sig _~_~_~_~_~_~` (12 units、step=10) を書いた TCML
- When 全パイプラインを通す
- Then 2 本の auto 行と explicit 信号行の polyline 末端 x がいずれも一致する (auto 行同士の母集団は explicit のみ、auto 同士は互いに参照しない)

## @not-implemented
### Scenario: auto 行より後ろにある explicit 信号も target 算出に含まれる
- Given `@clock` の auto 行 → `Sig _~_~_~_~_~_~_~_~` (16 units、step=10、160px) という順序の TCML
- When 全パイプラインを通す
- Then auto 行の polyline 末端 x が後続 explicit 行の末端 x と一致する (auto 行は 16 units に展開される)

## @not-implemented
### Scenario: 部分指定 clock 行は既存波形を保ち末尾だけ auto 拡張
- Given `Sig _~_~_~_~_~_~_~_~` (16 units、step=10) → `@clock` → `ck ~~__` (4 units 既存、step=10) という TCML
- When 全パイプラインを通す
- Then `ck` 行は既存 `~~__` (4 units) はそのまま、末尾に 12 units 追加されて合計 16 units、polyline 末端 x が `Sig` 行と一致

## @not-implemented
### Scenario: 全信号が auto の場合は空波形になる
- Given `@clock\nck1\n@clock\nck2` のように auto 行のみの TCML
- When 全パイプラインを通す
- Then `ck1` / `ck2` は units = 0 (波形要素 0)、polyline は描画されないか空

## @not-implemented
### Scenario: 非対称 pulse `_=2, ~=3` の auto 拡張は target で打ち切る
- Given `@clock(_=2, ~=3)` の auto 行 + `Sig _~_~_~_~_~_~` (12 units、step=10) の TCML
- When 全パイプラインを通す
- Then auto 行 ck は target=12 units を `Low(2) → High(3) → Low(2) → High(3) → Low(2)` (= 2 + 3 + 2 + 3 + 2 = 12 units) で埋めて末端 x が Sig と一致

## @not-implemented
### Scenario: 非対称 pulse の auto 拡張は最終 pulse 途中でも打ち切る (mid-High)
- Given `@clock(_=2, ~=3)` の auto 行 + `Sig _~_~_~_~_~_~_` (13 units、step=10) の TCML
- When 全パイプラインを通す
- Then auto 行 ck は target=13 units を `Low(2) → High(3) → Low(2) → High(3) → Low(2) → High(1)` (合計 13 units、最終 High が 1 unit で打ち切り) で埋め、末端 x が Sig と一致

## @not-implemented
### Scenario: `start=high` の auto 拡張は High から始める
- Given `@clock(_=2, ~=3, start=high)` の auto 行 + `Sig _~_~_~_~_~_~` (12 units、step=10)
- When 全パイプラインを通す
- Then auto 行 ck の波形は `High(3) → Low(2) → High(3) → Low(2) → High(2)` (合計 12 units、最終 High が 2 unit で打ち切り)

## @not-implemented
### Scenario: 部分指定 clock の最後の状態から pulse を継続する
- Given `Sig _~_~_~_~_~_~_~_~` (16 units、step=10) → `@clock(_=2, ~=3)` → `ck ~~__` (4 units 既存、最後は Low) の TCML
- When 全パイプラインを通す
- Then `ck` の auto 拡張部分 (12 units = 16 − 4) は **最後の状態 Low の次の pulse 段** = High から始める。具体的には `High(3) → Low(2) → High(3) → Low(2) → High(2)` (合計 12 units、最終 High が 2 unit で打ち切り)
- And 末端 x が Sig と一致
- And この解釈は既存スキャナリオ「部分指定からの自動繰り返し」(`@clock(pos)` の後に `ck ~~__`、続きが `~_~_`) と整合する (最後が Low なので次は High から)

## @not-implemented
### Scenario: per-row step + 非対称 pulse の auto 拡張
- Given `@step 20` → `Sig _~_~_~_~` (8 units、160px) → `@step 10` → `@clock(_=2, ~=3)` → auto 行
- When 全パイプラインを通す
- Then auto 行 target = `round(160 / 10) = 16` units、波形は `Low(2)→High(3)→Low(2)→High(3)→Low(2)→High(3)→Low(1)` (合計 16 units、最終 Low が 1 unit で打ち切り)、末端 x ≈ 160px (誤差 ≤ step/2 = 5px)

## @not-implemented
### Scenario: per-row step + auto clock + `pos` EdgeMark で立ち上がりに三角形
- Given `@step 20` → `Sig _~_~_~_~` (8 units、160px) → `@step 10` → `@clock(pos)` → auto 行
- When 全パイプラインを通す
- Then auto 行は target = 16 units の対称 pulse 波形 (`_~_~..._~`、Low(1)/High(1) × 8)、`pos` 指定により立ち上がりエッジ 8 本に `EdgeMark` triangle が生成される
- And triangle 中心 x は各立ち上がり位置 (auto 行の step=10 を反映)、Sig 行の triangle とは独立
- And SVG `class="edge-marks"` レイヤに 8 個の `<polygon>` (triangle) が現れる

## @not-implemented
### Scenario: per-row step + 部分指定 clock + EdgeMark
- Given `@step 20` → `Sig _~_~_~_~_~_~_~_~` (16 units、320px) → `@step 10` → `@clock(neg, _=2, ~=3)` → `ck ~~__` (4 units 既存)
- When 全パイプラインを通す
- Then ck の合計 units = round(320/10) = 32、既存 `~~__` (4) + 拡張 28 units = `High(3) → Low(2) → High(3) → Low(2) → ... → High(3) → Low(2)` (= 5 cycle + 余り) で target を埋める
- And `neg` 指定で各立ち下がり (High→Low) に EdgeMark、既存部分の `~_` 1 本 + 拡張部分の `~_` 5 本 = 計 6 本
- And ck 末端 x ≈ Sig 末端 x (誤差 ≤ step/2 = 5px)

## @not-implemented
### Scenario: `@slant` も per-row で行ごとに反映される
- Given `@step 20` `@slant 5` → `Sig1 _~_~_~_~` → `@step 10` `@slant 2` → `Sig2 _~_~_~_~` の TCML
- When 全パイプラインを通す
- Then Sig1 の各立ち上がり/立ち下がりの斜め部分の x 進行幅は 5px、Sig2 では 2px となり、行ごとに `@slant` のスナップショットが効いている (`@step` と同じローカルパラメータ規則)

## @not-implemented
### Scenario: per-row step + auto clock + WaveDrom + EdgeMark で edge 文字列が 1 行目のみに出る
- Given `@step 20\nSig _~_~_~_~\n@step 10\n@clock(pos)\nck` の TCML を WaveDrom に変換
- When 変換する
- Then `signal[0]` (Sig) と `signal[1]` (ck) は wave 文字列が出力され、period でピクセル整合 (gcd=10、Sig.period=2、ck.period 省略)、EdgeMark は WaveDrom の triangle とは別系統なので JSON `edge` 配列には現れない (既存ルール「マーカーは `arrows` には混入しない」と同じ)

## @not-implemented
### Scenario: `@step` 途中変更で行ごとに進行幅が異なる
- Given `@step 20` → `Clock _~_~_~_~_~_~` → `@step 10` → `Clock _~_~_~_~_~_~` という TCML
- When 全パイプラインを通す
- Then 1 行目の `<polyline>` 末端 x − 始端 x が 12 × 20 = 240px 相当、2 行目は 12 × 10 = 120px 相当 (どちらも slant 込みで step に比例)
- And 2 行の polyline 座標は一致しない

## @not-implemented
### Scenario: `@signal(overline)` で信号名上線
- Given `@signal(overline)` 後の信号
- Then SVG の信号名 `<text>` に `text-decoration="overline"` 属性

## @not-implemented
### Scenario: `@skip` で空白行が挿入される
- Given 信号 A + `@skip(2)` + 信号 B
- Then 信号 A と B の間に高さ 2lh の空白が空く
- And SkipRow は bgcolor 偶奇カウントから除外される

## @not-implemented
### Scenario: `@title` でタイトル行が挿入される
- Given `@title 同期回路` + 信号複数
- Then `titles` レイヤにタイトル `<text>` が含まれる

---

## 抽出 (Extract) 往復

## @not-implemented @smoke
### Scenario: SVG → TCML 抽出
- Given レンダリング済み SVG
- When `extract` を呼ぶ
- Then 元の TCML テキストと完全一致した文字列が返る

## @not-implemented
### Scenario: PNG iTXt チャンク → TCML 抽出
- Given レンダリング済み PNG (iTXt 埋め込み)
- When `extract` を呼ぶ
- Then 元の TCML テキストが返る

## @not-implemented
### Scenario: 往復が冪等
- Given 元 TCML T0
- When SVG 生成 → 抽出 → 再 SVG 生成 → 再抽出
- Then 抽出結果が常に T0 と一致する

---

## バグ回帰

## @not-implemented @regression
### Scenario: Gap 後対角線が出ない
- Given `____:~~~~`
- Then 出力に Low 末尾と High 先頭を結ぶ斜線が **存在しない**

## @not-implemented @regression
### Scenario: 直接遷移後に x がずれない
- Given `____----` (slant=2)
- Then HiZ polyline の開始 x が Low の遷移終端 x と一致 (差が 0)

## @not-implemented @regression
### Scenario: Bus↔Single 共有辺で水平ブリッジが描かれる
- Given `____====` (slant=2)
- Then Bus 上端 polyline の終点 x と High polyline の始点 x の間に水平ブリッジ線が描画されている (slant 幅のギャップなし)

## @not-implemented @regression
### Scenario: 同字数波形は遷移本数によらず幅が揃う
- Given `_~_~` と `__==` (ともに 4 文字、step=10、slant=2)
- When 両方を E2E パイプラインに通す
- Then どちらの信号も `signal_box.size.width == 40` で一致する
- And 旧仕様 (`w_hold` + `w_transient`) では `_~_~` = 46、`__==` = 42 とずれていた点が解消されている

## @not-implemented @regression
### Scenario: 信号上下に gap が対称配分される
- Given 1 つの SignalRow
- Then `bbox.origin.y` 直後と `signal_box.origin.y` の差が `h_space/2` (上に gap/2 が配分されている)

## @not-implemented @regression
### Scenario: 最終行も他行と同じ高さ計算式で扱う
- Given 3 つの SignalRow
- Then 最終行の `bbox.size.height` が他行と同じ計算式 (waveform_height + h_space)

---

## エラー伝播

## @not-implemented @negative
### Scenario: パースエラーが行番号付きで返る
- Given 不正な TCML (例: 先頭 `?`)
- When 全パイプラインを通す
- Then `ParseError::DontCareWithoutAnchor` が **行番号・列番号付き**で返る

## @not-implemented @negative
### Scenario: パースエラー時はレイアウト・レンダリングが実行されない
- Given パースエラーが発生する TCML
- Then `ChartDocument` 構築前に処理が停止し、SVG 文字列は生成されない

---

## 観点B 補強: 組合せ E2E (実バグの温床)

### Scenario: 途中 `@step` 変更で後続信号の幅が変わる (回帰防止)
- Given TCML:
  ```
  @step 10
  Sig1 ____
  @step 20
  Sig2 ____
  ```
- When parser → layout → SVG レンダリングを通す
- Then SVG 中の `Sig1` 波形 polyline 幅は 40px、`Sig2` は 80px (途中変更が確実に反映)
- And `<g class="row-backgrounds">` 内の rect 幅は 2 行とも `chart_inner_width` (= 80 + capwidth + namepad) で揃う

### Scenario: 途中 `@step` × `@clock(pos)` 自動展開 → SVG
- Given TCML:
  ```
  @step 10
  @clock(pos) clk
  @step 20
  data ====
  ```
- When E2E パイプラインを通す
- Then `clk` 信号の波形は step=10 の幅で展開 (chart_units=4)、`data` は step=20 で 80px
- And EdgeMark 三角形が clk の各立ち上がり位置に配置

### Scenario: 途中 `@step` × `@clock(pos)` 自動展開 → WaveDrom
- Given 同上 TCML
- When `tchart wavedrom` で変換
- Then `clk.wave == "p..."`、`data.wave == "=..."`
- And gcd(10, 20)=10 で `data.period = 2`、`clk.period` 省略

### Scenario: per-row `@step` × DontCare × アンカー × `@->` × WaveDrom (フル組合せ)
- Given TCML:
  ```
  @step 10
  Sig1 _?@{a}_~
  @step 20
  Sig2 ===@{b}===
  @-> (@{a}, @{b}, red, dashed) trans
  ```
- When E2E + WaveDrom 変換
- Then SVG: 両信号の幅が個別 step で計算、`@->` は赤 dashed 矢印 + ラベル
- And WaveDrom: edge `"a-~>b trans"` (dashed → `-~>`、色 red は落とす)、period 計算 gcd(10,20)=10

### Scenario: `@signal(overline)` × per-row `@step` × `@bg`
- Given TCML:
  ```
  @step 10
  @bg #ff0
  @signal(overline) nReset _~__
  @step 20
  Data ====
  ```
- Then nReset 行: 背景 #ff0、上線あり、step=10
- And Data 行: 背景デフォルト、step=20

### Scenario: `@clock(pos, _=2, ~=2)` × アンカー × WaveDrom
- Given TCML:
  ```
  @step 10
  @clock(pos, _=2, ~=2) clk
  data ==@{x}==
  @-> (@{x}, @{x})
  ```
- When E2E + WaveDrom
- Then clk wave は `0`/`1` 列展開
- And data に node、edge `"x->x"`

### Scenario: `@bgcolor0/1` × `@bg` × `@skip` × `@title` 行混在の偶奇カウント
- Given TCML:
  ```
  @bgcolor0 #eee
  @bgcolor1 #ccc
  Sig1 _~
  @skip(1)
  @title "Mid"
  Sig2 _~
  @bg #f0f
  Sig3 _~
  ```
- Then Sig1 = #eee (idx 0)、Sig2 = #ccc (idx 1、Skip/Title 飛ばす)、Sig3 = #f0f (`@bg` 優先、idx 2 だが上書き)

### Scenario: `@dontcare_color` 途中変更 × 同色再出現で ID 共有
- Given TCML:
  ```
  @dontcare_color #c00
  Sig1 _?_
  @dontcare_color #06c
  Sig2 _?_
  @dontcare_color #c00
  Sig3 _?_
  ```
- Then `<defs>` 内の pattern は 2 個 (`dontcare-hatch-1` = #c00、`dontcare-hatch-2` = #06c)
- And Sig1 と Sig3 の `<rect fill>` は同じ ID (`url(#dontcare-hatch-1)`)

### Scenario: アンカー番号 + 名前付きの `@->` 混在
- Given TCML:
  ```
  Sig1 _~@{start}_~
  Sig2 ===@1===
  @-> (@{start}, @1) flow
  ```
- Then SVG arrows レイヤに矢印 1 本 (両端の座標は信号別に解決)
- And WaveDrom edge `"a->b flow"` (順序: start=a、@1=b)

### Scenario: 複数行信号名 × overline × アンカー × `@->` × `@bg`
- Given TCML:
  ```
  @bg #ff0
  @signal(overline)
  "n\nReset"  ___@{r}___
  Data        ===@{d}===
  @-> (@{r}, @{d})
  ```
- Then 信号 nReset: bg=#ff0, 2 行信号名, overline, アンカー解決
- And SVG arrows に矢印 1 本

### Scenario: 同名アンカー番号競合 (`@1` `@{1}` 混在) × `@->`
- Given TCML:
  ```
  Sig1 _~@1_~@{1}_
  @-> (@1, @{1})
  ```
- Then `@1` と `@{1}` は別アンカーとして登録 (名前空間別)
- And 矢印は 2 つの異なる x 座標を結ぶ

### Scenario: ハイライト × DontCare × アンカー × エッジマーカー
- Given TCML:
  ```
  @clock(pos) clk
  data __[?@{a}?]__
  ```
- Then clk に EdgeMark、data にハイライト矩形 + DontCare polygon (y_h〜y_l) + アンカー登録

### Scenario: `@scale` × per-row `@step` × SVG width 属性
- Given TCML `@scale 2.0` `@step 10` Sig1 (4 文字), `@step 20` Sig2 (4 文字)
- Then SVG `width` 属性 = `(max(40, 80) + capwidth + namepad + 2*page-margin) * 2.0`

### Scenario: `@font` 解決失敗 × デフォルトフォントフォールバック (E2E、警告)
- Given TCML `@font NoSuchFont` を含む
- When `tchart svg` を実行
- Then 出力 SVG は生成され、stderr に警告 1 行
- And SVG 内の `<text font-family>` はデフォルト fallback 値

### Scenario: 矢印ラベル `<>` を含む × 白縁取り × XML エスケープ
- Given TCML `@-> (@{a}, @{b}) <signal-set>` (label に `<` `>`)
- Then 出力 SVG のラベル `<text>` 内に `&lt;signal-set&gt;` (エスケープ済み)
- And paint-order/stroke 属性も付与される

---

## 観点B 補強: バグ回帰 (実例)

### Scenario: 途中 `@step` 変更が無視される (回帰)
- Given TCML `@step 10` Sig1, `@step 20` Sig2 (両者 4 文字波形)
- When E2E パイプラインを通す
- Then `Sig2.signal_box.size.width == 80` (`Sig1` の step=10 が 「sticky」に残らない)

### Scenario: per-row `@step` × アンカー位置ズレ (回帰)
- Given TCML `@step 10` `Sig1 ___@1__`, `@step 20` `Sig2 ___@2__`
- When E2E
- Then `@1.x = capwidth + namepad + 30`, `@2.x = capwidth + namepad + 60`
- And SVG arrow line がこの x 値で描画される

### Scenario: `@clock(auto)` 展開後の信号幅が他信号の `@step` 変更を反映しない (回帰)
- Given `@step 10` `@clock(pos) clk` 直後に `@step 20` `data ====`
- Then clk は step=10 で展開 (= 4 unit × 10 = 40px)、data は step=20 (= 80px)
- And chart_units = 4 (data 4 unit が他信号最大長)

---

## 観点C 補強: アンカー・矢印の高度な組合せ (E2E)

### Scenario: 同信号内に連続アンカー × `@->` 全組合せ参照
- Given `A _@{a}@{b}@{c}~` + `@-> (@{a}, @{b})` + `@-> (@{b}, @{c})` + `@-> (@{a}, @{c})`
- Then SVG に 3 本の矢印 `<line>` (3 本とも別座標)
- And レイアウトでアンカー 3 個の x 座標が同一

### Scenario: `@->` 自己ループ (`@{a} → @{a}`) の SVG 描画
- Given `A _@{a}~` + `@-> (@{a}, @{a})`
- Then SVG arrow が 1 本描画される (始終端同点)
- And パイプライン全体が panic しない

### Scenario: 矢印 100 本のスケール E2E
- Given アンカー 100 個 + `@->` 100 本
- Then SVG パース成功、`<g class="arrows">` 配下に `<line>` 100 本

### Scenario: clock EdgeMark 0 件 (`@clock(none)` のみ) でも arrows レイヤに polygon が紛れない
- Given `@clock(none)\nclk _~_~`
- Then `<g class="waveforms">` 内 polygon が 0 個

### Scenario: clock EdgeMark 50 件 + 矢印 1 本 で z-order
- Given `@clock(pos)\nclk _~_~_~...` (50 立ち上がり) + `@-> (@{a}, @{b})`
- Then arrows レイヤが waveforms レイヤより前面 (clock マーカーより上)

---

## 観点B 補強: 空状態とゼロ値の境界 (E2E)

### Scenario: 0 信号 (空ファイル) で SVG 出力
- Given 空 TCML
- Then SVG が出力される (root + page-margin のみのフレーム)、エラーなし
- And `<svg width=...>` 属性は page-margin × 2 程度

### Scenario: `@title` のみの 0 信号 SVG
- Given `@title "T"` のみ
- Then SVG に Title `<text>` 1 個、信号レイヤなし

### Scenario: `@scale 1000` (極大) でも SVG が破綻しない
- Given `@scale 1000\nA _~`
- Then SVG `width`/`height` が overflow なく出力、レンダリング完了

### Scenario: `@fontsize 0.5` (極小) でテキスト幅が 0 にならない
- Given `@fontsize 0.5\nA _~`
- Then 信号名 `<text>` の x 座標が capwidth より大きい (テキストはレイアウトされる)

### Scenario: 1 文字波形 `A _` のみ
- Given `A _`
- Then SVG に LevelRun(Low,1) 1 本のポリライン (水平線 1 本)
- And chart_inner_width == 1 step

---

## 観点D 補強: SVG 構造的不変条件 (E2E)

### Scenario: SVG round-trip (TCML → SVG → TCML) で TCML 完全一致
- Given 任意の有効 TCML 入力
- When `tchart svg` で SVG 化 → `tchart src` で TCML 復元
- Then 復元 TCML が元と byte 単位で一致

### Scenario: PNG round-trip で UTF-8 信号名の iTXt 復元が壊れない
- Given 信号名にマルチバイト (`日本語`) を含む TCML
- When `tchart png` → `tchart src`
- Then 復元 TCML 中の信号名が正しい UTF-8 で復元される

### Scenario: SVG ソース埋め込みに `]]>` が含まれても復元成功
- Given TCML 中に `# ]]>` を含むコメント
- When SVG round-trip
- Then 復元 TCML が一致 (CDATA を使わないことの validation)
