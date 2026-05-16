# レイアウトエンジン

`ChartDocument` (parser-level) と `FontMetrics` から、ジオメトリ確定済みの `ChartDocument` (layout-resolved) を生成する機能のテスト仕様。

仕様: [`docs/spec/types.md`](../spec/types.md) §3.1 「行ジオメトリ規約 (対称ギャップ)」、§4.5 「レイアウトアルゴリズム」、§6 「過去バグ防止条項」。

---

## 行積み上げ

## @not-implemented @smoke
### Scenario: Line.bbox が縦に隙間なく積み上がる
- Given 3 つの `SignalRow` を持つ `ChartDocument`
- When レイアウトを実行する
- Then 各隣接行で `Line[i+1].bbox.origin.y == Line[i].bbox.origin.y + Line[i].bbox.size.height` が成立する

## @not-implemented
### Scenario: 先頭行の origin.y は page-margin
- Given 1 つの SignalRow を持つ ChartDocument
- Then 先頭行の `bbox.origin.y == page-margin`

## @not-implemented
### Scenario: page-margin は積み上げに関与しない
- Given 2 つの SignalRow
- Then `Line[1].bbox.origin.y == Line[0].bbox.origin.y + Line[0].bbox.size.height` (page-margin が間に入らない)

---

## 対称ギャップ

## @not-implemented @smoke
### Scenario: signal_box は bbox 上下に gap/2 ずつ余白
- Given `h_space=4`, `waveform_height=20` の SignalRow
- Then `signal_box.origin.y == 2.0` (gap/2)
- And `signal_box.size.height == 20.0`
- And `bbox.size.height == 24.0` (waveform_height + gap)

## @not-implemented
### Scenario: 最終行も同じ規約
- Given 3 行の SignalRow
- Then 最終行も `bbox.size.height == waveform_height + h_space` で計算される (特殊扱いなし)

## @not-implemented
### Scenario: 単一行も同じ規約
- Given 1 行の SignalRow
- Then `bbox.size.height == waveform_height + h_space`

## @not-implemented
### Scenario: h_space の上下分配は均等
- Given `h_space=5` の SignalRow
- Then `signal_box.origin.y == 2.5` (gap/2)
- And `bbox.size.height - signal_box.size.height == 5.0` (gap)

---

## SkipRow

## @not-implemented
### Scenario: `@skip(2)` の bbox 高さは 2lh
- Given `@skip(2)` を含む ChartDocument、line_height=24px
- Then SkipRow の `bbox.size.height == 48.0`

## @not-implemented
### Scenario: `@skip(20px)` の bbox 高さは 20px
- Given `@skip(20px)`
- Then SkipRow の `bbox.size.height == 20.0`

## @not-implemented
### Scenario: SkipRow は bgcolor0/1 の偶奇カウントから除外
- Given SignalRow A, SkipRow, SignalRow B
- Then A は `bgcolor0`、B は `bgcolor1` で塗られる (Skip は飛ばす)

---

## TitleRow

## @not-implemented
### Scenario: TitleRow の bbox 高さはフォント行高
- Given `@title 同期回路` (1 行) でフォント size=14, lineheight=1.2
- Then TitleRow の `bbox.size.height == 14 * 1.2`

## @not-implemented
### Scenario: 複数行 TitleRow の bbox 高さ
- Given `@title "A\nB"` (2 行)
- Then TitleRow の `bbox.size.height == 14 * 1.2 * 2`

## @not-implemented
### Scenario: TitleRow も偶奇カウントから除外
- Given SignalRow A, TitleRow, SignalRow B
- Then A は `bgcolor0`、B は `bgcolor1` で塗られる

---

## 信号幅と要素 width 合計

## @not-implemented @smoke
### Scenario: signal_box.size.width は文字数 × step (遷移は step の中に内包)
- Given waveform `_~_~` (LevelRun×4 + Transition×3、step=10、slant=2)
- Then `signal_box.size.width == 4 × step == 40`
- And 内訳は LevelRun(Low,1)=10 + SingleEdge=2 + LevelRun(High,1)=8 + SingleEdge=2 + LevelRun(Low,1)=8 + SingleEdge=2 + LevelRun(High,1)=8 (前要素が遷移の LevelRun は `step - slant`)

## @not-implemented @smoke
### Scenario: 同字数なら遷移本数によらず幅が一致する
- Given waveform A `_~_~` と waveform B `__==`、ともに step=10、slant=2
- Then 両方の `signal_box.size.width == 4 × step == 40`
- And これは旧 `w_hold` + `w_transient` モデルで A=46、B=42 とずれていた致命バグの解決を意味する (詳細は `docs/bugs.md` 参照)

## @not-implemented
### Scenario: BusCross の cross 部は slant、X body は step - slant (合計で 1 step)
- Given `=X=` (LevelRun(Bus,1) + Transition(BusCross) + LevelRun(Bus,2)、step=10、slant=2)
- Then `signal_box.size.width == 10 + 2 + 18 == 30` (= 3 文字 × step、内訳 LevelRun(Bus,1)=10 + Transition(BusCross)=2 + LevelRun(Bus,2)=`2×step - slant = 18`)
- And `Transition(BusCross).width == slant` (cross 部のみ、body は後続 `LevelRun(Bus, 1)` に分離して LevelRun 幅ルールに従う)

## @not-implemented
### Scenario: Gap も width に寄与 (step 幅)
- Given `__:__` (Low,2 + Gap + Low,2、step=10)
- Then `signal_box.size.width == 20 + 10 + 20 == 50` (Gap は遷移を伴わないので step そのまま、後続 LevelRun も Gap の後ろは「遷移なし」扱いで step×2)

## @not-implemented
### Scenario: Anchor は 0 幅
- Given `_~@{a}_` (Low,1 + Edge + High,1 + Anchor + Low,1)
- Then Anchor の width は 0、合計幅に影響しない

---

## チャート全幅: Line.bbox.size.width の全行一様

## @not-implemented @smoke
### Scenario: 信号長が異なる行でも Line.bbox.size.width は全行一致
- Given SignalRow A (波形長 100px) と SignalRow B (波形長 40px)、capwidth=20
- When レイアウトを実行する
- Then 全 Line の `bbox.size.width` は同一値で、A の仮幅 (= 120px) と一致する
- And 短い行 B の `bbox.size.width` も 120px となる

## @not-implemented
### Scenario: Skip / Title 行も同じ幅で揃う
- Given SignalRow A (波形長 100px) + SkipRow + TitleRow (テキスト幅 30px) + SignalRow B (波形長 40px)、capwidth=20
- Then 全 Line の `bbox.size.width` は最大値 120px で一致する (Skip/Title/Signal 区別なし)

## @not-implemented
### Scenario: signal_box.size.width は依然として要素 width 合計のまま
- Given SignalRow A (波形長 100px) と SignalRow B (波形長 40px)
- Then `A.signal_box.size.width == 100`、`B.signal_box.size.width == 40` (信号自体は伸びない)
- And ただし `A.bbox.size.width == B.bbox.size.width == 120`

---

## アンカー解決

## @not-implemented @smoke
### Scenario: Anchor の x 座標は累積位置
- Given `SigA __@{a}__` (Low,2 + Anchor + Low,2、step=10、capwidth=20、namepad=8)
- When レイアウトを実行する
- Then `AnchorRegistry["a"].at.x == 20+8+20 == 48`

## @not-implemented
### Scenario: Anchor の y 座標は直前 LevelRun の線位置
- Given `SigA ~~@{a}__`
- Then `AnchorRegistry["a"].at.y == y_high` (直前が High)

## @not-implemented
### Scenario: Bus 文脈の Anchor は中央
- Given `SigA ==@{a}__`
- Then `AnchorRegistry["a"].at.y == (y_high + y_low) / 2`

## @not-implemented
### Scenario: Arrow の参照解決
- Given `@{a}` `@{b}` 定義 + `@-> (@{a}, @{b})`
- Then `Arrow.from.at` `Arrow.to.at` が確定 Px 値で埋まっている

## @not-implemented @negative
### Scenario: 未定義 Anchor 参照は parse 段階でエラー (layout には到達しない)
- Given `@-> (@{undef}, @{b})` のみ
- Then パーサーが `ParseError::UndefinedAnchor` を返し、レイアウトは実行されない

---

## 信号名・ラベルジオメトリ

## @not-implemented
### Scenario: capwidth=0 で自動計算
- Given 信号名 `Foo` (幅 30px) と `LongerName` (幅 80px)、`namepad=8`
- Then `capwidth == 80 + 8`

## @not-implemented
### Scenario: capwidth 明示指定
- Given `@capwidth 100`
- Then `capwidth == 100` (信号名の実幅にかかわらず)

## @not-implemented
### Scenario: 複数行信号名の高さは行数 × line_height
- Given `"A\nB" _~`、name フォント size=14 lineheight=1.2
- Then label_box.size.height >= 14*1.2*2 == 33.6

## @not-implemented
### Scenario: 信号名の縦中央寄せ
- Given 単一行信号名と waveform_height=24
- Then label_box の center.y が signal_box の center.y と一致する

---

## 全体寸法

## @not-implemented
### Scenario: chart 全幅は最大 signal_box 幅 + capwidth + page-margin*2
- Given 最大 signal_box.size.width = 100, capwidth=80, page-margin=10
- Then chart 幅 == 100 + 80 + 20 == 200

## @not-implemented
### Scenario: chart 全高は最終 Line.bbox 末端 + page-margin
- Given 全 Line の bbox 積み上げ末端 y = 150, page-margin=10
- Then chart 高さ == 150 + 10

---

## Clock 展開後のレイアウト

## @not-implemented
### Scenario: clock 展開後は通常 SignalRow と同じレイアウト
- Given `@clock(pos)` で展開された SignalRow
- Then `h_space` の対称配分など通常と同じ規約に従う

## @not-implemented
### Scenario: EdgeMark の line_start / line_end は SingleEdge と一致
- Given `@clock(pos)` で `_~_~` 展開、step=10、slant=2
- Then 各 `EdgeMark.line_start.x` が対応する `SingleEdge` (`Low → High`) の開始 x
- And `line_start.y == y_low`、`line_end = (line_start.x + slant, y_high)`
- And clock 由来の Arrow は `Annotations.arrows` に **入らない**

---

## エッジケース

## @not-implemented
### Scenario: 空 ChartDocument のレイアウト
- Given lines が空の ChartDocument
- Then エラーにならず、chart 幅 = page-margin*2、高さ = page-margin*2

## @not-implemented
### Scenario: SkipRow のみ
- Given `@skip(2)` のみ
- Then chart 高さ == page-margin*2 + 2*line_height

## @not-implemented @negative
### Scenario: signal_box.size.width と要素 width 合計の不一致は debug_assert で検出
- Given レイアウト計算をバグらせて size.w と sum(width) を不一致にした場合
- Then debug_assert に失敗

---

## 観点A 補強: 単独仕様の境界・上書き

### Scenario: page-margin が 0 でも積み上げ計算は変わらない
- Given page-margin=0、SignalRow 2 つ
- Then `Line[0].bbox.origin.y == 0`、`Line[1].bbox.origin.y == Line[0].bbox.size.height`

### Scenario: page-margin が大きい (例 50) でも内側 bbox 積み上げは同じ
- Given page-margin=50、SignalRow 2 つ
- Then `Line[0].bbox.origin.y == 50`、`Line[1].bbox.origin.y == 50 + Line[0].bbox.size.height`
- And chart 全幅 = max(bbox.size.width) + 100 (page-margin × 2)
- And chart 全高 = 末端 + 100

### Scenario: h_space が小数 (例 4.5) でも対称配分される
- Given h_space=4.5
- Then `signal_box.origin.y == 2.25` (4.5 / 2)
- And `bbox.size.height - signal_box.size.height == 4.5`

### Scenario: capwidth ぴったり一致 (信号名幅 == capwidth)
- Given 信号名 `Foo` 幅 30px、`@capwidth 30`、namepad=8
- Then label_box.size.width == 30、波形開始 x = 30 + 8 = 38

### Scenario: namepad=0 のとき信号名と波形が密着
- Given `@namepad 0`、capwidth=20
- Then 波形開始 x = 20 (namepad なし)

### Scenario: lineheight が 1.0 のとき信号高さと line_height が一致
- Given fontsize=14、lineheight=1.0
- Then `canvas.line_height == 14`、`waveform_height == 14`

### Scenario: `@scale` が 2.0 のとき出力サイズはレイアウト計算後にスケール
- Given `@scale 2.0`、レイアウト後 chart 幅 100
- Then SVG 出力 `width` 属性は 200 (スケール適用後)
- And 内部 bbox 値はスケール前の値を保持 (Px ベース)

---

## 観点B 補強: ローカル途中変更 × 各種仕様 (layout 観点)

### Scenario: 途中 `@step` 変更で信号別 signal_box.size.width が個別に決まる
- Given `@step 10` の Sig1 (波形 4 文字)、`@step 20` の Sig2 (波形 4 文字)
- Then `Sig1.signal_box.size.width == 40`、`Sig2.signal_box.size.width == 80`
- And `chart_inner_width = max(40, 80) + capwidth + namepad` で全行 bbox.width が揃う

### Scenario: 途中 `@h_space` 変更は当該信号以降に適用
- Given `@h_space 4` の Sig1、`@h_space 10` の Sig2
- Then `Sig1.bbox.size.height == waveform_height + 4`、`Sig2.bbox.size.height == waveform_height + 10`
- And 各信号の signal_box は当該行の h_space に従って対称配分

### Scenario: 途中 `@step` 変更 × アンカー位置が新 step を反映
- Given `@step 10` で Sig1 (`___@1`)、`@step 20` で Sig2 (`___@2`)
- Then `@1.x = capwidth + namepad + 30`、`@2.x = capwidth + namepad + 60`

### Scenario: `@clock(auto)` 信号が他信号より長い (chart_units が clock を含めて決まるか)
- Given `@clock(pos)` 部分指定 `_~_~_~_~_~_~` (12 unit)、他信号は最大 4 unit
- Then chart_units = 12 (clock 含む) or 4 (clock 除外、自動展開なし) — 仕様 §4.3 step 2 では「ClockRow を除く」だが、本体波形が長い場合の扱いは要明示

### Scenario: 信号別 step × signal_box.size.width × Line.bbox.size.width 全行一様
- Given `@step 10` で 4 文字波形、`@step 50` で 4 文字波形 (= 200 px)
- Then 全 Line の bbox.size.width が `max(40, 200) + capwidth + namepad` で一致
- And 短い信号 (40px) の background はこのチャート全幅で塗られる

### Scenario: per-row `@step` × アンカー × `@->` の中点
- Given Sig1 step=10 で `@1` at x=30、Sig2 step=20 で `@2` at x=60、矢印 `@-> (@1, @2)`
- Then ラベル中点が `(30+capwidth+namepad + 60+capwidth+namepad) / 2` に配置

### Scenario: 途中 `@step` × `@bg` 適用行
- Given `@bg #f0f` `@step 20` の信号 Sig
- Then Sig の `Line.background == Some(#f0f)` かつ step=20 で展開

### Scenario: per-row `@h_space` × `@signal(overline)` 上線位置
- Given `@h_space 8` `@signal(overline) nReset _~__`
- Then 上線は label_box の `cap_top - overline_gap` に置かれ、`label_box.origin.y` は `(bbox.h - label_h) / 2` (上下対称配分)

### Scenario: `@skip` 直後の `@bg` は次信号に適用 (skip ではなく後続行)
- Given `@bg #f0f` `@skip(2)` 信号 A
- Then Skip 行が `Line.background == Some(#f0f)` (`@bg` は次の 1 行 = Skip)
- And 信号 A の `Line.background == None`

### Scenario: 多信号で chart_inner_width が capwidth に依存しない (capwidth 自動計算優先)
- Given 信号名 `Sig1` (幅 30) と `LongerSignalName` (幅 100)、`@capwidth 0`
- Then `capwidth = 100 + namepad` で自動計算され、波形開始 x もそれに揃う

### Scenario: `@page-margin` 偶数値と奇数値の対称性
- Given page-margin=11 (奇数)、SignalRow 2
- Then `Line[0].bbox.origin.y == 11`、chart 全幅 = max(bbox.w) + 22

## ユーザ承認反映シナリオ (2026-05-10)

### Scenario: `h_space` デフォルト値が 10
- Given パラメータ未指定で SignalRow A、SignalRow B
- Then `Line[B].bbox.origin.y - Line[A].bbox.origin.y == signal_height + 10`
- And デフォルト `h_space` は 10 px (`@h_space` 未指定時)

### Scenario: `@h_space 10` 明示指定でデフォルトと一致
- Given `@h_space 10`
- Then 任意の SignalRow の bbox 高さが「未指定時 (デフォルト)」と一致

### Scenario: `@title` 行の高さに h_space が加算される
- Given `@title T` 1 行のみのチャート
- Then `Line[0].bbox.size.height == text_lines * line_height + h_space`
- And bbox 上下に `h_space/2` ずつ余白配分

### Scenario: 連続 `@title` 行の積み上げに h_space が反映
- Given `@title A\n@title B`
- Then `Line[1].bbox.origin.y - Line[0].bbox.origin.y == line_height + h_space` (行 0 の下半分 + 行 1 の上半分 = h_space)

---

## 観点B 補強: 空状態とゼロ値の境界 (layout)

### Scenario: 空 TCML のレイアウトで chart 全幅 = page-margin*2
- Given 空 TCML、`@page-margin 11`
- Then `chart.size.width == 22`、`chart.size.height == 22`

### Scenario: `@title` のみのチャートで chart 高さ = title bbox + page_margin*2
- Given `@title T` のみ、`@page-margin 11`
- Then `chart.size.height == TitleRow.bbox.h + 22`

### Scenario: `@scale 1000` でも capwidth・signal_box 幅は scale 前の論理値で計算される
- Given `@scale 1000\nA _~`
- Then `Line[0].bbox.size.width` は scale 抜きの値 (SVG 出力時のみ scale 乗算)

### Scenario: `@step 1` 最小境界での signal_box 幅
- Given `@step 1\nA _~_~`
- Then `signal_box.size.width == 4` (4 文字 × 1)

### Scenario: `@slant 0` で SingleEdge polyline が垂直 (水平区間 0)
- Given `@slant 0\nA _~`
- Then transition の polyline 中点が垂直線

### Scenario: 1 文字波形 `A _` の signal_box.size.width == step
- Given `@step 16\nA _`
- Then `signal_box.size.width == 16`

### Scenario: アンカー単独 (`A _@{a}_`) の Line.bbox 幅は LevelRun のみで決まる
- Given `A _@{a}_` (アンカー 0 幅)
- Then `signal_box.size.width == 2 * step`

---

## 観点M 補強: 数値精度・累積誤差 (layout)

## @not-implemented
### Scenario: 信号 50 行 × step 1000 個でも累積 Px が許容誤差内
- Given 50 信号、各 1000 step、`@step 16`
- Then 最終信号末尾 X = 16 * 1000 = 16000、`Line[*].bbox.x` も同値
- And f32 累積誤差は 1px 未満

## @not-implemented
### Scenario: `@scale 0.001` で signal_box 寸法は内部論理値、SVG 出力で乗算
- Given `@scale 0.001\nA _~_~`
- Then `signal_box.size.width` は論理 64 (4 step × 16)、SVG 出力時のみ 0.064 px に圧縮
- And 内部レイアウトは scale 抜きで計算 (丸め誤差発生せず)

## @not-implemented
### Scenario: `@scale 1000` × `@step 999` 極端値で overflow しない
- Given `@scale 1000\n@step 999\nA _~_~`
- Then layout 計算で f32 overflow / inf / NaN にならない
- And SVG width = 999 * 4 * 1000 = 3996000 px (整数で表現可能)

## @not-implemented
### Scenario: 小数 `@step 0.5` の丸め (layout 側受理時)
- Given `@step 0.5\nA _~_~`
- Then parser が小数を受理する場合、`signal_box.size.width == 2.0` (4 * 0.5)
- And 受理しない場合は ParseError、layout には到達しない (spec gap 候補)

## @not-implemented
### Scenario: `@slant 0.1` 小数受理時の transition 描画位置
- Given `@slant 0.1\nA _~`
- Then transition polyline 中間点 X が 0.1 px ずれる (slant 適用)
- And rounding が必要なら f32 を保持 (整数化しない)

## @not-implemented
### Scenario: `@step 1000` 大きい値で chart_inner_width 計算
- Given `@step 1000\nA _~_~`
- Then `chart_inner_width == 4000` (4 文字 × 1000)
- And page-margin / capwidth と合わせた最終 SVG width に overflow なし

## @not-implemented
### Scenario: `@slant 999` 巨大 slant で transition 横幅が step を超える挙動
- Given `@step 16\n@slant 999\nA _~`
- Then slant > step/2 のとき clamp する仕様なら clamp 値で描画
- And clamp しない仕様なら隣接 transition と重なるが layout は continue
- Note: 仕様確認 (spec gap 候補)

## @not-implemented
### Scenario: signal 100 行 × アンカー 100 個ずつで X 座標累積
- Given 100 信号 × 各 100 アンカー
- Then 各アンカー X 座標が文書順に単調増加、ジャンプなし
- And f32 比較で 1e-3 px 未満の誤差

## @not-implemented
### Scenario: `@scale 0.001` × `@fontsize 0.5` 最小組合せで bbox 0 にならない
- Given `@scale 0.001\n@fontsize 0.5\nA _`
- Then `Line[0].bbox.size.height > 0` (0 px height は SVG で消える)
- And テキスト bbox も 0 でない

## @not-implemented
### Scenario: gcd 計算で `@step` が分数を含むとき (f32 → 整数 gcd)
- Given `@step 0.5\n@clock(pos)\nclk _~_~`
- Then WaveDrom period 算出のため整数化が必要 → 仕様に従う (round / floor / error)
- Note: 仕様未定義なら spec gap として登録
