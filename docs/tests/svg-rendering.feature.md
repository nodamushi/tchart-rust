# SVG レンダリング

`ChartDocument` (layout-resolved) から SVG 文字列を生成する機能のテスト仕様。

仕様: [`docs/spec/svg-rendering.md`](../spec/svg-rendering.md)、[`docs/spec/types.md`](../spec/types.md) §6.5 「遷移描画の網羅契約」。

---

## 構造

## @not-implemented @smoke
### Scenario: ルート要素は `<svg>` で名前空間付き
- Given 任意の ChartDocument
- When SVG レンダリングする
- Then 出力は `<svg xmlns="http://www.w3.org/2000/svg" xmlns:tchart="http://tchart-rust/1.0" ...>` で始まる

## @not-implemented
### Scenario: レイヤ順 (z-order)
- Given 任意の ChartDocument
- Then `<g>` レイヤが順に: `row-backgrounds`, `highlights`, `dontcares`, `signal-labels`, `waveforms`, `guides`, `titles`, `arrows`, `overlays`

## @not-implemented
### Scenario: TCML ソースが metadata に埋め込まれる
- Given 元 TCML "Clock _~_~"
- Then `<metadata><tchart:source>` 要素に XML エスケープされた TCML が含まれる

---

## 行背景 (`@bgcolor0` / `@bgcolor1`)

##
### Scenario: 偶奇行が交互に塗られる
- Given 3 つの SignalRow、`@bgcolor0 #eee`、`@bgcolor1 #ccc`
- Then `row-backgrounds` レイヤに 3 つの `<rect>` (色: #eee, #ccc, #eee)

##
### Scenario: bbox 全体 (上下 gap/2 を含む) を塗る
- Given SignalRow A の `bbox.size = (300, 24)` (waveform_height=20, gap=4)
- Then 行背景 `<rect>` の高さは 24 (gap/2 上下を含む)

##
### Scenario: SkipRow / TitleRow は偶奇カウント外
- Given SignalRow, SkipRow, SignalRow, TitleRow, SignalRow
- Then 行背景は 1 行目=#eee, 3 行目=#ccc, 5 行目=#eee の 3 つ

##
### Scenario: `@bgcolor` が `none` のとき出力なし
- Given `@bgcolor0 none` のみ指定
- Then `row-backgrounds` レイヤに偶数行の `<rect>` は出力されない

## @smoke
### Scenario: 信号長が異なる行でも row-background はチャート全幅で揃う
- Given SignalRow A (波形長 100px) と SignalRow B (波形長 40px)、`@bgcolor0 #eee` `@bgcolor1 #ccc`、capwidth=20
- When SVG を出力する
- Then `row-backgrounds` レイヤに 2 つの `<rect>` が出る
- And 両方の `<rect>` の `width` は同一値 (= `Line.bbox.size.width` = 120px)
- And 短い信号行 B の背景も A と同じ幅で塗られる

---

## ローカル背景 (`@bg`)

## @not-implemented
### Scenario: `@bg` は Line.bbox 全体を塗る
- Given `@bg #f0f` の SignalRow、`bbox = (10, 10, 240, 24)`
- Then `row-backgrounds` レイヤに `<rect x=10 y=10 width=240 height=24 fill="#f0f"/>` が出る

## @not-implemented
### Scenario: `@bg` 指定行は bgcolor0/1 を重ねない
- Given 偶数行で `@bgcolor0 #eee` 指定 + `@bg #f0f`
- Then その行の背景は `#f0f` のみ (`#eee` は出力されない)

## @not-implemented
### Scenario: `@bg` は Title 行にも適用される
- Given `@bg #ff0` の直後に `@title "X"`
- Then その Title 行の `Line.bbox` 全体が `#ff0` で塗られる

## @not-implemented
### Scenario: `@bg` は Skip 行にも適用される
- Given `@bg #ff0` の直後に `@skip(1)`
- Then その Skip 行の `Line.bbox` 全体が `#ff0` で塗られる

## @not-implemented
### Scenario: `@bg none` は保留中の値を破棄
- Given `@bg #f0f` の直後に `@bg none`、その後信号行 A
- Then 信号行 A は `bgcolor0/1` のフォールバックを受ける (`#f0f` は出ない)

---

## LevelRun 描画

## @not-implemented
### Scenario: Low は y_low 単線
- Given LevelRun(Low, 2)
- Then `<polyline>` に y=y_low の 2 点が含まれる

## @not-implemented
### Scenario: High は y_high 単線
- Given LevelRun(High, 2)
- Then `<polyline>` に y=y_high の 2 点が含まれる

## @not-implemented
### Scenario: HiZ は破線で独立 polyline
- Given LevelRun(HiZ, 2)
- Then 独立 `<polyline stroke-dasharray="...">` が y=y_mid で出力される

## @not-implemented
### Scenario: Bus は上下 2 本の polyline
- Given LevelRun(Bus, 2)
- Then `<polyline>` 2 つ (top rail at y_high, bottom rail at y_low)

---

## DontCare 描画 (`?`)

## @not-implemented @smoke
### Scenario: DontCareAlongLow (両端 signal start/end) は矩形 polygon + y_low に内部水平線
- Given LevelRun(DontCareAlongLow, 2)、信号行頭〜行末で隣接遷移なし
- Then `dontcares` レイヤに `<polygon>` 4 頂点 (y_h〜y_l 範囲の矩形): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And `waveforms` レイヤに y=y_low の polyline 区間が含まれる

## @not-implemented
### Scenario: DontCareAlongHigh (両端 signal start/end) は矩形 polygon + y_high 内部水平線
- Given LevelRun(DontCareAlongHigh, 2)、信号行頭〜行末で隣接遷移なし
- Then `<polygon>` 4 頂点 (y_h〜y_l 範囲の矩形) + y=y_high 水平線

## @not-implemented
### Scenario: DontCareAlongHiZ (両端 signal start/end) は矩形 polygon + y_mid 破線
- Given LevelRun(DontCareAlongHiZ, 2)、信号行頭〜行末で隣接遷移なし
- Then `<polygon>` 4 頂点 (y_h〜y_l 範囲の矩形) + y=y_mid の破線 polyline

### Scenario: DontCareAlongBus (両側 Bus continue) は矩形塗り
- Given `==?==` (前後とも Bus continue)、slant=2
- Then `dontcares` レイヤに `<polygon>` (左上(x_l, y_high), 右上(x_r, y_high), 右下(x_r, y_low), 左下(x_l, y_low))
- And `waveforms` レイヤに y=y_high と y=y_low の polyline 2 本

### Scenario: `_=?=_` (Low 両端) は四角形 `/=\` (Low 側で y_low に縮退)
- Given `_=?=_` (Low → BusOpen → ? → BusClose → Low)、slant=2
- Then `<polygon>` 4 頂点: 左下 (BusOpen_start, y_low) → 左上 (BusOpen_end, y_high) `/` 斜辺、上辺水平 (BusOpen_end, y_high) → (BusClose_start, y_high)、右上 → 右下 (BusClose_end, y_low) `\` 斜辺、下辺水平
- And BusOpen Low→Bus は top rail のみ斜辺 (bottom は y_low 共有)、polygon 左辺は top rail に追従

### Scenario: `~=?=~` (High 両端) は四角形 `\=/`
- Given `~=?=~`、slant=2
- Then `<polygon>` 4 頂点: 上辺水平、左下 (BusOpen_end, y_low) `\` 斜辺、右上 (BusClose_start, y_high) `/`

### Scenario: `--==?==--` (HiZ 両端) は六角形 (両端 y_mid に縮退)
- Given `--==?==--`、slant=2
- Then `<polygon>` 6 頂点: 左頂点 (BusOpen_start, y_mid)、左上 (BusOpen_end, y_high)、右上 (BusClose_start, y_high)、右頂点 (BusClose_end, y_mid)、右下 (BusClose_start, y_low)、左下 (BusOpen_end, y_low)
- And HiZ→Bus / Bus→HiZ は両レール斜辺 (上下発散 / 収束)、polygon 各端は y_mid 縮退の 1 点

### Scenario: `--==?==` (HiZ → Bus + ? + 信号末) は五角形 (HiZ 側で y_mid に縮退、右垂直)
- Given `--==?==`、slant=2
- Then `<polygon>` 5 頂点: 左頂点 (BusOpen_start, y_mid)、左上 (BusOpen_end, y_high)、右上 (signal_end, y_high)、右下 (signal_end, y_low)、左下 (BusOpen_end, y_low)
- And HiZ→Bus は両レール斜辺、polygon 左辺は y_mid 1 点 + 上下発散

### Scenario: `==?==--` (信号始端 + ? + Bus → HiZ) は対称な五角形
- Given `==?==--`、slant=2
- Then `<polygon>` 5 頂点: 左上 (signal_start, y_high)、右上 (BusClose_start, y_high)、右頂点 (BusClose_end, y_mid)、右下 (BusClose_start, y_low)、左下 (signal_start, y_low)

### Scenario: `__==?==` (Low → Bus + ? + 信号末) は四角形 (左 `/`、右垂直)
- Given `__==?==`、slant=2
- Then `<polygon>` 4 頂点: 左下 (BusOpen_start, y_low)、左上 (BusOpen_end, y_high)、右上 (signal_end, y_high)、右下 (signal_end, y_low)

### Scenario: `==?==__` (信号始端 + ? + Bus → Low) は四角形 (左垂直、右 `\`)
- Given `==?==__`、slant=2
- Then `<polygon>` 4 頂点: 左上 (signal_start, y_high)、右上 (BusClose_start, y_high)、右下 (BusClose_end, y_low)、左下 (signal_start, y_low)

### Scenario: `=X?X=` の DontCareAlongBus polygon は X cross 中点を頂点にする六角形
- Given `=X?X=` (Bus → BusCross → ? → BusCross → Bus)、slant=2、step=10
- Then `dontcares` レイヤの `<polygon>` は六角形 `>▲■▲<` 形状: 左頂点 = X1 cross 中点 (`x_X1 + slant/2`, y_mid)、上辺 (14, y_high) → (24, y_high) の body 上端、右頂点 = X2 cross 中点 (26, y_mid)、下辺 (24, y_low) → (14, y_low)

### Scenario: `=X?=` の polygon は左 X 半分 + body + 右垂直で五角形
- Given `=X?=`、slant=2、step=10
- Then `<polygon>` の左頂点 = X cross 中点 (`x_X + slant/2`, y_mid)、右辺は signal 末端で垂直 (前辺 = signal 末端 x、y_high〜y_low)、上下 = body 上下レール

### Scenario: DontCareAlongBus の前後組合せ網羅
- Given 前要素 5 種 (Bus continue / Low / High / HiZ / X cross) × 後要素 5 種の組合せの `?` 領域
- Then 各組合せで左右辺が境界に正しく追従し、塗りが `signal_box` 全高にはみ出さない

## #1: `?` (DontCare) の塗り polygon と隣接 slant の完全網羅 (GitHub issue #1)

DC-X (X ∈ {Low, High, HiZ, Bus}) の塗り polygon は **左右各境界の遷移種別** が決める。

座標規約:
- `x_a` = DC 区間の左端 x、`x_b` = DC 区間の右端 x (= `x_a + W * step`、`W` = DC 区間のセル数)
- `s` = `@slant`、`step` = `@step`、`y_h` / `y_mid` / `y_l` は signal_box の上 / 中 / 下端

共通ルール:
- DC-X 内部水平線: DC-Low は y_l、DC-High は y_h、DC-HiZ は y_mid (破線)、DC-Bus は y_h と y_l の 2 本。
- 信号 polyline の遷移 slant は `?` の有無に関わらず `@slant` どおりに描画される (垂直 / slant=0 に縮退してはならない)。
- polygon の左右辺は **隣接遷移の斜辺に追従** する。境界が垂直 (signal start/end / 同レベル拡張) のときは矩形の垂直辺。
- 半 slant (HiZ 隣接、y_mid を経由する遷移) では polygon に **y_mid 中継頂点** が 1 個加わり、頂点数が 5 になる。両側半 slant なら 6 頂点。
- y_mid 中継頂点の x 位置は **slant が y_mid と交わる x**。Pos-half (y_l→y_mid) では slant_end_x、Neg-half going down to y_l (y_mid→y_l) では slant_start_x、Neg-half going down from y_h to y_mid (y_h→y_mid) では slant_end_x、Pos-half going up from y_mid to y_h (y_mid→y_h) では slant_start_x。
- BusOpen-from-HiZ / BusClose-to-HiZ は両 rail が半 slant で y_mid から発散/収束 → polygon に 1 個の y_mid wedge 頂点 (5 頂点)。
- BusOpen / BusClose で single 側が Low/High の場合、polygon は **斜辺 rail のみ** (Low なら top rail、High なら bottom rail) に追従、もう一方の rail (水平) は polygon の上下辺に吸収される (4 頂点扱い)。

### DC-Low (内部水平線 y_l) × 16 組

## @not-implemented
### Scenario: `_?` DC-Low (start | end) は矩形 (#1)
- Given `_?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 信号 polyline: 内部水平線 (x_a, y_l) → (x_b, y_l)

## @not-implemented
### Scenario: `_?~` DC-Low (start | Pos) は右上に台形拡張 (#1)
- Given `_?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a, y_l)
- And 信号 polyline: 内部水平線 (x_a, y_l) → (x_b, y_l)、Pos slant (x_b, y_l) → (x_b+s, y_h)、High hold (x_b+s, y_h) → (x_b+step, y_h)

## @not-implemented
### Scenario: `_?-` DC-Low (start | Pos-half to HiZ) は右に y_mid 中継の五頂点 (#1)
- Given `_?-`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a, y_l)
- And 信号 polyline: 内部水平線 (x_a, y_l) → (x_b, y_l)、Pos-half slant (x_b, y_l) → (x_b+s, y_mid)、HiZ 破線 (x_b+s, y_mid) → (x_b+step, y_mid)

## @not-implemented
### Scenario: `_?=` DC-Low (start | BusOpen-from-Low) は右上に台形拡張 (#1)
- Given `_?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a, y_l)
- And BusOpen-from-Low: top rail (x_b, y_l) → (x_b+s, y_h) 斜辺、bottom rail (x_b, y_l) → (x_b+s, y_l) 水平
- And polygon 右辺は BusOpen top rail に追従

## @not-implemented
### Scenario: `~_?` DC-Low (Neg | end) は左上から斜辺で降りる台形 (#1)
- Given `~_?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a+s, y_l)
- And 信号 polyline: High hold (... → x_a, y_h)、Neg slant (x_a, y_h) → (x_a+s, y_l)、内部水平線 (x_a+s, y_l) → (x_b, y_l)

## @not-implemented
### Scenario: `~_?~` DC-Low (Neg | Pos) は左右斜辺の平行四辺形 (#1)
- Given `~_?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a+s, y_l)
- And 左 Neg slant (x_a, y_h) → (x_a+s, y_l)、右 Pos slant (x_b, y_l) → (x_b+s, y_h)

## @not-implemented
### Scenario: `~_?-` DC-Low (Neg | Pos-half) は左斜辺 + 右 y_mid タブの五頂点 (#1)
- Given `~_?-`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a+s, y_l)
- And 左 Neg slant、右 Pos-half (x_b, y_l) → (x_b+s, y_mid)

## @not-implemented
### Scenario: `~_?=` DC-Low (Neg | BusOpen-from-Low) は左右斜辺の平行四辺形 (#1)
- Given `~_?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a+s, y_l)

## @not-implemented
### Scenario: `-_?` DC-Low (Neg-half | end) は左 y_mid 起点の五頂点 (#1)
- Given `-_?`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a+s, y_l), (x_a, y_mid)
- And Neg-half slant (HiZ→Low): (x_a, y_mid) → (x_a+s, y_l)、y_mid 中継頂点は (x_a, y_mid) (= slant_start_x)

## @not-implemented
### Scenario: `-_?~` DC-Low (Neg-half | Pos) は左 y_mid + 右斜辺の五頂点 (#1)
- Given `-_?~`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a+s, y_l), (x_a, y_mid)

## @not-implemented
### Scenario: `-_?-` DC-Low (Neg-half | Pos-half) は両側 y_mid タブの六頂点 (#1)
- Given `-_?-`、@slant 10 @step 25
- Then dontcares polygon 6 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a+s, y_l), (x_a, y_mid)

## @not-implemented
### Scenario: `-_?=` DC-Low (Neg-half | BusOpen-from-Low) は左 y_mid + 右斜辺の五頂点 (#1)
- Given `-_?=`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a+s, y_l), (x_a, y_mid)

## @not-implemented
### Scenario: `=_?` DC-Low (BusClose-to-Low | end) は左上斜辺の台形 (#1)
- Given `=_?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a+s, y_l)
- And BusClose-to-Low: top rail (x_a, y_h) → (x_a+s, y_l) 斜辺、bottom rail (x_a, y_l) → (x_a+s, y_l) 水平
- And polygon 左辺は BusClose top rail に追従

## @not-implemented
### Scenario: `=_?~` DC-Low (BusClose | Pos) は左右斜辺の平行四辺形 (#1)
- Given `=_?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a+s, y_l)

## @not-implemented
### Scenario: `=_?-` DC-Low (BusClose | Pos-half) は左斜辺 + 右 y_mid タブの五頂点 (#1)
- Given `=_?-`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a+s, y_l)

## @not-implemented
### Scenario: `=_?=` DC-Low (BusClose | BusOpen) は左右両 top rail 斜辺の平行四辺形 (#1)
- Given `=_?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a+s, y_l)

### DC-High (内部水平線 y_h、DC-Low と上下対称) × 16 組

## @not-implemented
### Scenario: `~?` DC-High (start | end) は矩形 (#1)
- Given `~?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 信号 polyline: 内部水平線 (x_a, y_h) → (x_b, y_h)

## @not-implemented
### Scenario: `~?_` DC-High (start | Neg) は右下に台形拡張 (#1)
- Given `~?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l)
- And Neg slant (x_b, y_h) → (x_b+s, y_l)

## @not-implemented
### Scenario: `~?-` DC-High (start | Neg-half to HiZ) は右 y_mid 中継の五頂点 (#1)
- Given `~?-`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a, y_l)
- And Neg-half slant (High→HiZ): (x_b, y_h) → (x_b+s, y_mid)、y_mid 中継頂点 (x_b+s, y_mid)

## @not-implemented
### Scenario: `~?=` DC-High (start | BusOpen-from-High) は右下に台形拡張 (#1)
- Given `~?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l)
- And BusOpen-from-High: top rail (x_b, y_h) → (x_b+s, y_h) 水平、bottom rail (x_b, y_h) → (x_b+s, y_l) 斜辺

## @not-implemented
### Scenario: `_~?` DC-High (Pos | end) は左下斜辺の台形 (#1)
- Given `_~?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And Pos slant (x_a, y_l) → (x_a+s, y_h)

## @not-implemented
### Scenario: `_~?_` DC-High (Pos | Neg) は左右斜辺の平行四辺形 (#1)
- Given `_~?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `_~?-` DC-High (Pos | Neg-half) は左斜辺 + 右 y_mid 中継の五頂点 (#1)
- Given `_~?-`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `_~?=` DC-High (Pos | BusOpen-from-High) は左右斜辺の平行四辺形 (#1)
- Given `_~?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `-~?` DC-High (Pos-half | end) は左 y_mid 中継の五頂点 (#1)
- Given `-~?`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l), (x_a, y_mid)
- And Pos-half slant (HiZ→High): (x_a, y_mid) → (x_a+s, y_h)、y_mid 中継頂点 (x_a, y_mid) (= slant_start_x)

## @not-implemented
### Scenario: `-~?_` DC-High (Pos-half | Neg) は左 y_mid + 右斜辺の五頂点 (#1)
- Given `-~?_`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l), (x_a, y_mid)

## @not-implemented
### Scenario: `-~?-` DC-High (Pos-half | Neg-half) は両側 y_mid タブの六頂点 (#1)
- Given `-~?-`、@slant 10 @step 25
- Then dontcares polygon 6 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a, y_l), (x_a, y_mid)

## @not-implemented
### Scenario: `-~?=` DC-High (Pos-half | BusOpen-from-High) は左 y_mid + 右斜辺の五頂点 (#1)
- Given `-~?=`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l), (x_a, y_mid)

## @not-implemented
### Scenario: `=~?` DC-High (BusClose-to-High | end) は左下斜辺の台形 (#1)
- Given `=~?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And BusClose-to-High: top rail (x_a, y_h) → (x_a+s, y_h) 水平、bottom rail (x_a, y_l) → (x_a+s, y_h) 斜辺
- And polygon 左辺は BusClose bottom rail に追従

## @not-implemented
### Scenario: `=~?_` DC-High (BusClose | Neg) は左右斜辺の平行四辺形 (#1)
- Given `=~?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `=~?-` DC-High (BusClose | Neg-half) は左斜辺 + 右 y_mid 中継の五頂点 (#1)
- Given `=~?-`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `=~?=` DC-High (BusClose | BusOpen) は左右両 bottom rail 斜辺の平行四辺形 (#1)
- Given `=~?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l)

### DC-HiZ (内部破線 y_mid) × 17 組 (`-?-` 含む)

DC-HiZ は polygon が **常に矩形** (隣接遷移の斜辺に追従しない、`==` 1-cell と同じ範囲)。隣接遷移の slant は波形 polyline 側で別途維持される。

注: `-?-` は現状 tchart で無視されているが、`-?-` は `==` と同じく 1-cell DC-HiZ 矩形として扱う (新規対応)。

## @not-implemented
### Scenario: `-?` DC-HiZ (start | end) は矩形 (#1)
- Given `-?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 信号 polyline: 内部破線 (x_a, y_mid) → (x_b, y_mid)

## @not-implemented
### Scenario: `-?-` DC-HiZ (同レベル拡張、`==` 1-cell と同範囲) は矩形 (#1)
- Given `-?-`、@slant 10 @step 25 (DC-HiZ,1 として扱う、`?` 0-width で同レベル `-` 同士を統合)
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 信号 polyline: 内部破線 (x_a, y_mid) → (x_b, y_mid)
- And 範囲は同 step 数の `==` と同じ (新規対応: 現状 tchart は `-?-` を無視している)

## @not-implemented
### Scenario: `-?_` DC-HiZ (start | Neg-half to Low) は矩形 (#1)
- Given `-?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 信号 polyline: 内部破線 (x_a, y_mid) → (x_b, y_mid)、Neg-half slant (x_b, y_mid) → (x_b+s, y_l) (slant=10 維持)

## @not-implemented
### Scenario: `-?~` DC-HiZ (start | Pos-half to High) は矩形 (#1)
- Given `-?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And Pos-half slant (x_b, y_mid) → (x_b+s, y_h) (slant=10 維持)

## @not-implemented
### Scenario: `-?=` DC-HiZ (start | BusOpen-from-HiZ) は矩形 (#1)
- Given `-?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And BusOpen-from-HiZ: 上 rail (x_b, y_mid) → (x_b+s, y_h)、下 rail (x_b, y_mid) → (x_b+s, y_l) (両半 slant 発散、slant=10 維持)

## @not-implemented
### Scenario: `_-?` DC-HiZ (Pos-half from Low | end) は矩形 (#1)
- Given `_-?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And Pos-half slant (Low→HiZ): (x_a, y_l) → (x_a+s, y_mid) (slant=10 維持)
- And 内部破線 (x_a+s, y_mid) → (x_b, y_mid) (slant 終了後の hold 部分)

## @not-implemented
### Scenario: `_-?_` DC-HiZ (Pos-half from Low | Neg-half to Low) は矩形 (#1)
- Given `_-?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 左 Pos-half (x_a, y_l) → (x_a+s, y_mid)、右 Neg-half (x_b, y_mid) → (x_b+s, y_l) (両方 slant=10)

## @not-implemented
### Scenario: `_-?~` DC-HiZ (Pos-half from Low | Pos-half to High) は矩形 (#1)
- Given `_-?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 左 Pos-half (x_a, y_l) → (x_a+s, y_mid)、右 Pos-half (x_b, y_mid) → (x_b+s, y_h)

## @not-implemented
### Scenario: `_-?=` DC-HiZ (Pos-half from Low | BusOpen-from-HiZ) は矩形 (#1)
- Given `_-?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 左 Pos-half、右 BusOpen-from-HiZ 両 rail (すべて slant=10)

## @not-implemented
### Scenario: `~-?` DC-HiZ (Neg-half from High | end) は矩形 (#1)
- Given `~-?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And Neg-half slant (High→HiZ): (x_a, y_h) → (x_a+s, y_mid) (slant=10 維持)

## @not-implemented
### Scenario: `~-?_` DC-HiZ (Neg-half from High | Neg-half to Low) は矩形 (#1)
- Given `~-?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 左 Neg-half (x_a, y_h) → (x_a+s, y_mid)、右 Neg-half (x_b, y_mid) → (x_b+s, y_l)

## @not-implemented
### Scenario: `~-?~` DC-HiZ (Neg-half from High | Pos-half to High) は矩形 (#1)
- Given `~-?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 左 Neg-half、右 Pos-half (両方 slant=10)

## @not-implemented
### Scenario: `~-?=` DC-HiZ (Neg-half from High | BusOpen-from-HiZ) は矩形 (#1)
- Given `~-?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `=-?` DC-HiZ (BusClose-to-HiZ | end) は矩形 (#1)
- Given `=-?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And BusClose-to-HiZ: 上 rail (x_a, y_h) → (x_a+s, y_mid)、下 rail (x_a, y_l) → (x_a+s, y_mid) (両半 slant 収束、slant=10 維持)

## @not-implemented
### Scenario: `=-?_` DC-HiZ (BusClose | Neg-half to Low) は矩形 (#1)
- Given `=-?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `=-?~` DC-HiZ (BusClose | Pos-half to High) は矩形 (#1)
- Given `=-?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)

## @not-implemented
### Scenario: `=-?=` DC-HiZ (BusClose | BusOpen) は矩形 (#1)
- Given `=-?=`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 左 BusClose-to-HiZ 両 rail、右 BusOpen-from-HiZ 両 rail (すべて slant=10)

### DC-Bus 既存 scenario の補完 (`=?` / `?=` 片側 single)

## @not-implemented
### Scenario: `=?` DC-Bus,1 (start | end) は矩形 (#1)
- Given `=?` (DC-Bus,1 単独)、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And 内部水平線 y_h と y_l の 2 本

## @not-implemented
### Scenario: `_=?` (Low | BusOpen-from-Low | DC-Bus,1 | end) は左下 `/` の四頂点 (#1)
- Given `_=?`、@slant 10 @step 25 (DC-Bus は `=?` 部分の 1 セル、x_a = BusOpen 開始 x、x_b = signal 末端)
- Then dontcares polygon 4 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l)
- And BusOpen-from-Low の top rail (x_a, y_l) → (x_a+s, y_h) で polygon 左辺

## @not-implemented
### Scenario: `~=?` (High | BusOpen-from-High | DC-Bus,1 | end) は左下 `\` の四頂点 (#1)
- Given `~=?`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a+s, y_l)
- And BusOpen-from-High の bottom rail (x_a, y_h) → (x_a+s, y_l) で polygon 左辺

## @not-implemented
### Scenario: `-=?` (HiZ | BusOpen-from-HiZ | DC-Bus,1 | end) は左 wedge の五頂点 (#1)
- Given `-=?`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a+s, y_h), (x_b, y_h), (x_b, y_l), (x_a+s, y_l), (x_a, y_mid)
- And BusOpen-from-HiZ の両 rail (x_a, y_mid) → (x_a+s, y_h) / (x_a, y_mid) → (x_a+s, y_l)、y_mid wedge 頂点 (x_a, y_mid)

## @not-implemented
### Scenario: `=?_` (start | DC-Bus,1 | BusClose-to-Low | Low) は右下 `\` の四頂点 (#1)
- Given `=?_`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b+s, y_l), (x_a, y_l)
- And BusClose-to-Low の top rail (x_b, y_h) → (x_b+s, y_l) で polygon 右辺

## @not-implemented
### Scenario: `=?~` (start | DC-Bus,1 | BusClose-to-High | High) は右下 `/` の四頂点 (#1)
- Given `=?~`、@slant 10 @step 25
- Then dontcares polygon 4 頂点 (CW): (x_a, y_h), (x_b+s, y_h), (x_b, y_l), (x_a, y_l)
- And BusClose-to-High の bottom rail (x_b, y_l) → (x_b+s, y_h) で polygon 右辺

## @not-implemented
### Scenario: `=?-` (start | DC-Bus,1 | BusClose-to-HiZ | HiZ) は右 wedge の五頂点 (#1)
- Given `=?-`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b+s, y_mid), (x_b, y_l), (x_a, y_l)
- And BusClose-to-HiZ の両 rail (x_b, y_h) → (x_b+s, y_mid) / (x_b, y_l) → (x_b+s, y_mid)、y_mid wedge 頂点 (x_b+s, y_mid)

## @not-implemented
### Scenario: `=X?` (Bus | BusCross | DC-Bus,1 | end) は左 BusCross 中点の五頂点 (#1)
- Given `=X?`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b, y_l), (x_a, y_l), (x_a + s/2, y_mid)
- And BusCross の中点 (x_a + s/2, y_mid) が左辺の wedge

## @not-implemented
### Scenario: `?X=` (start | DC-Bus,1 | BusCross | Bus) は右 BusCross 中点の五頂点 (#1)
- Given `?X=`、@slant 10 @step 25
- Then dontcares polygon 5 頂点 (CW): (x_a, y_h), (x_b, y_h), (x_b + s/2, y_mid), (x_b, y_l), (x_a, y_l)

### `#1` 補足: 隣接遷移 slant の保持 (回帰確認)

## @not-implemented
### Scenario: `?` を含む信号行で SingleEdge slant は維持される (#1)
- Given `_?_~_` または `~_?_~` のような `?` を含む信号行、`@slant 10`
- Then Pos / Neg 遷移は **slant 幅 10** で描画される (垂直 / slant=0 に縮退してはならない)

## @not-implemented
### Scenario: `?` を含む信号行で BusOpen / BusClose slant は維持される (#1)
- Given `_?__===_?_` のような `?` と Bus を含む信号行、`@slant 10`
- Then BusOpen / BusClose は **slant 幅 10** で描画される (垂直に縮退してはならない)

## @not-implemented
### Scenario: `~~?~~===~?~` (Bug Bus2) の BusOpen / BusClose 下 rail は両方 slant=10 で描画される (#1)
- Given `~~?~~===~?~` (DC-High,4 + Bus,3 + DC-High,2)、`@slant 10`
- Then BusOpen 下 rail は (x_b1, y_h) → (x_b1+10, y_l) の斜辺として独立 polyline で描画 (下 rail が宙に浮く現象なし)
- And BusClose 下 rail は (x_b2, y_l) → (x_b2+10, y_h) の斜辺で対称に描画
- And 上 rail は DC-High 内部線 + BusOpen 上 rail 水平 + Bus 上 rail + BusClose 上 rail 水平 + DC-High 内部線 が全幅 y_h で連結

## @not-implemented
### Scenario: DontCare の前後で polyline が連結される
- Given `_?_` (Low,1 + DontCareAlongLow,1 + Low,1)
- Then 全 3 区間が **1 本の polyline** に蓄積される (`<line>` 独立要素は使われない)

---

## #2: HiZ 経由の遷移で実線 polyline が貫通しない (GitHub issue #2)

`-` (HiZ) は破線スタイルで実線と統合できないため、Low/High と HiZ の境界で実線 polyline は分断される。SingleEdge の slant 線 (`~-` / `_-` / `-~` / `-_`) は **HiZ 蓄積器** (破線) に書き出され、HiZ 区間自身の hold と同じ 1 本の HiZ polyline に取り込まれる。実線 polyline が HiZ 区間 (y_mid 経由) を「貫通」する余計な直線は **絶対に** 出してはいけない。

座標規約:
- `x_b1` = HiZ 区間の左端 x (`~~` → `--` 境界)、`x_b2` = HiZ 区間の右端 x (`--` → `__` 境界)
- `s` = `@slant`、`step` = `@step`
- `y_h` / `y_mid` / `y_l` は signal_box の上端 / 中央 / 下端

## @not-implemented @smoke
### Scenario: `~~----___` (High → HiZ → Low) は実線 2 本 + 破線 1 本 (#2)
- Given `@slant 10` `@step 25` の信号行 `~~----___`
- Then 実線 polyline 1 本目: `~~` (`... → x_b1, y_h)` で終端、HiZ 区間を貫通する点を含まない
- And 破線 polyline (HiZ): `(x_b1, y_h) → (x_b1+s, y_mid) → (x_b2, y_mid) → (x_b2+s, y_l)` を 1 本の `<polyline stroke-dasharray="...">` で出力
- And 実線 polyline 2 本目: `(x_b2+s, y_l) → ...` で始まる `___`
- And `(x_b1, y_h)` から `(x_b2+s, y_l)` へ直結する **実線**斜辺は出力されない (バグ症状の禁止)

## @not-implemented
### Scenario: `__----~~~` (Low → HiZ → High、`~~----___` の対称) は実線 2 本 + 破線 1 本 (#2)
- Given `@slant 10` `@step 25` の信号行 `__----~~~`
- Then 実線 polyline 1 本目 `__` は `(... → x_b1, y_l)` で終端
- And 破線 polyline: `(x_b1, y_l) → (x_b1+s, y_mid) → (x_b2, y_mid) → (x_b2+s, y_h)`
- And 実線 polyline 2 本目 `~~~` は `(x_b2+s, y_h) → ...` で開始
- And 実線で `(x_b1, y_l)` から `(x_b2+s, y_h)` を直結する斜辺は出力されない

## @not-implemented
### Scenario: `~~--__--~~` (HiZ で 2 回挟む) は実線 3 本 + 破線 2 本 (#2)
- Given `@slant 10` `@step 25` の信号行 `~~--__--~~`
- Then 出力 polyline は計 5 本: 実線 `~~` (y_h) → 破線 (`~- + -- + -_`、y_h→y_l) → 実線 `__` (y_l) → 破線 (`_- + -- + -~`、y_l→y_h) → 実線 `~~` (y_h)
- And 各実線 polyline は **対応する level の y** のみを持つ水平線 (`~~` は (..., y_h) 点列、`__` は (..., y_l) 点列)
- And 破線 polyline は HiZ 蓄積器が境界で flush されるため 2 本に分離 (実線 `__` を挟む)

## @not-implemented
### Scenario: `~~--~~` (HiZ 経由 High→High 復帰) は実線 2 本 + 破線 1 本 (#2)
- Given `@slant 10` `@step 25` の信号行 `~~--~~`
- Then 実線 polyline `~~` 1 本目は (... → x_b1, y_h) で終端
- And 破線 polyline: `(x_b1, y_h) → (x_b1+s, y_mid) → (x_b2, y_mid) → (x_b2+s, y_h)` (V 字形、y_h→y_mid→y_h)
- And 実線 polyline `~~` 2 本目は `(x_b2+s, y_h) → ...` で開始
- And 実線で `(x_b1, y_h)` から `(x_b2+s, y_h)` を直結する水平線は出力されない (両 `~~` は別 polyline)

## @not-implemented
### Scenario: `__--__` (HiZ 経由 Low→Low 復帰) は実線 2 本 + 破線 1 本 (#2)
- Given `@slant 10` `@step 25` の信号行 `__--__`
- Then 実線 polyline `__` 2 本 + 破線 polyline 1 本 (U 字形、y_l→y_mid→y_l) が独立出力

## @not-implemented
### Scenario: `~~-___` (HiZ,1 で Single → Single の 1-cell HiZ 経由) も polyline 分断 (#2)
- Given `@slant 10` `@step 25` の信号行 `~~-___`
- Then HiZ 1-cell ぶんでも実線 polyline は分断される (`~~` と `___` は別 polyline)
- And HiZ 破線 polyline は `(x_b1, y_h) → (x_b1+s, y_mid) → (x_b2, y_mid) → (x_b2+s, y_l)` を 1 本で出力 (x_b2 - x_b1 = 1 × step、hold 部分は `step - s`)

## @not-implemented
### Scenario: DontCare polygon のデフォルト塗りはハッチパターン参照
- Given `@dontcare_color` を指定しないチャート (`?` を含む)
- Then DontCare の `<polygon>` の `fill` 属性は `url(#dontcare-hatch-1)` となる
- And `<polygon>` には `fill` 以外の属性は出力されない (アウトラインなし)
- And SVG ルート直下の `<defs>` 内に `<pattern id="dontcare-hatch-1" patternUnits="userSpaceOnUse" patternTransform="rotate(45)" ...>` が 1 つだけ出力される
- And `<pattern>` は内部に 1 本の `<line stroke="<DEFAULT_DONTCARE_HATCH_STROKE_COLOR>" ...>` (右上がり斜線になる方向) を持つ

## @not-implemented
### Scenario: @dontcare_color でハッチ線色を上書きできる
- Given `@dontcare_color #c00` を 1 度だけ指定したチャート (`?` を含む)
- Then `<defs>` には `<pattern id="dontcare-hatch-1">` が 1 つだけ出力され、内部の `<line stroke="#c00"/>` が出力される
- And DontCare の `<polygon>` の `fill` 属性は `url(#dontcare-hatch-1)` となる
- And `<polygon>` には `fill` 以外の属性は付与されない

## @not-implemented
### Scenario: @dontcare_color を途中で書き換えると行ごとに色が切り替わる
- Given チャートに行 A (`?` 含む)、行 B (`?` 含む)、行 C (`?` 含む) があり、行 A の前で `@dontcare_color #c00`、行 C の前で `@dontcare_color #06c` が指定されている
- Then `<defs>` には `<pattern id="dontcare-hatch-1">` (`<line stroke="#c00"/>`) と `<pattern id="dontcare-hatch-2">` (`<line stroke="#06c"/>`) の 2 つが、初出順 (`#c00` → `#06c`) で出力される
- And 行 A と行 B の `<polygon>` の `fill` は `url(#dontcare-hatch-1)`、行 C の `<polygon>` の `fill` は `url(#dontcare-hatch-2)`

## @not-implemented
### Scenario: 同じ色を再指定しても `<defs>` の `<pattern>` は重複しない
- Given チャートに行 A (`#c00`)、行 B (`#06c`)、行 C (`#c00` に戻す) がある
- Then `<defs>` には `<pattern id="dontcare-hatch-1">` (`#c00`) と `<pattern id="dontcare-hatch-2">` (`#06c`) の 2 つだけが出力される (`#c00` は再採番せず ID 1 を再利用)
- And 行 A と行 C の `fill` は `url(#dontcare-hatch-1)`、行 B の `fill` は `url(#dontcare-hatch-2)`

## @not-implemented
### Scenario: チャートに `?` が無ければ `<defs>` を出力しない
- Given `?` を一切含まないチャート (DontCare レンダリングが発生しない)
- Then SVG ルート直下に `<defs>` 要素が出力されない (空 `<defs></defs>` も出さない)

---

## Transition 描画

## @not-implemented @smoke
### Scenario: SingleEdge は 1 本の線
- Given Transition(SingleEdge, Low→High), x=10, slant=2
- Then `<polyline>` に `(10, y_low) → (12, y_high)` の点列が含まれ、独立 `<line>` は出力されない

## @not-implemented
### Scenario: BusOpen は上下 2 本 (共有辺含む)
- Given Transition(BusOpen, Low→Bus), x=10, slant=2
- Then 上端: `(10, y_low) → (12, y_high)` (斜線)
- And 下端: `(10, y_low) → (12, y_low)` (共有辺の水平ブリッジ)
- And どちらも前後の polyline と連結

## @not-implemented
### Scenario: BusClose は上下 2 本
- Given Transition(BusClose, Bus→High), x=10, slant=2
- Then 上端: `(10, y_high) → (12, y_high)` (共有辺水平)
- And 下端: `(10, y_low) → (12, y_high)` (斜線)

## @not-implemented
### Scenario: BusCross は cross 部のみ (幅 slant)、X body は後続 LevelRun(Bus)
- Given `=X=` で X cross 開始 x=10、step=10、slant=2
- Then x_c = x + slant = 12 とする
- And 線 A (cross): `(10, y_high) → (12, y_low)` で 前 Bus 上端 → 後 Bus 下端
- And 線 B (cross): `(10, y_low) → (12, y_high)` で 前 Bus 下端 → 後 Bus 上端
- And `Transition(BusCross).width == slant` (cross 部のみ)
- And X body と後続 `=` は merge された後続 `LevelRun(Bus, 2)` として描画される (前要素が遷移なので幅 = `2×step - slant = 18`、`(12, y_high) → (30, y_high)` 等)
- And 蓄積器は cross により top/bottom が **入れ替わる**

## @not-implemented
### Scenario: BusCross で X 前に bus 信号が無い場合は cross を描画しない
- Given `XXXX` (先頭から連続 X) または信号行頭に X が来る場合
- Then 最初の X は `Transition(BusCross)` を生成せず、body の `LevelRun(Bus, 1)` のみ
- And 2 番目以降の X は通常通り `Transition(BusCross)` (cross 描画) + `LevelRun(Bus, 1)` (新値 body)

## @not-implemented
### Scenario: TransitionKind の match は網羅的
- Given レンダラ実装に `_ =>` パターンを追加した状態
- Then コンパイル失敗 (網羅性チェック、機械検証)

##
### Scenario: Low→HiZ SingleEdge で slant=2 のとき前後 polyline が接続される
- Given Low(1) + SingleEdge(Low→HiZ) + HiZ(1), slant=2
- Then Low を含む polyline の末点 x と HiZ 破線 polyline の始点 x が同一
- And 視覚的ギャップがない (slant 幅の切断が生じない)

##
### Scenario: HiZ→Low SingleEdge で slant=2 のとき前後 polyline が接続される
- Given HiZ(1) + SingleEdge(HiZ→Low) + Low(1), slant=2
- Then HiZ 破線 polyline の末点 x と Low を含む polyline の始点 x が同一
- And 視覚的ギャップがない

##
### Scenario: High→HiZ SingleEdge で slant=2 のとき前後 polyline が接続される
- Given High(1) + SingleEdge(High→HiZ) + HiZ(1), slant=2
- Then High を含む polyline の末点 x と HiZ 破線 polyline の始点 x が同一

##
### Scenario: HiZ→High SingleEdge で slant=2 のとき前後 polyline が接続される
- Given HiZ(1) + SingleEdge(HiZ→High) + High(1), slant=2
- Then HiZ 破線 polyline の末点 x と High を含む polyline の始点 x が同一

---

## Text (レベル文字列中のテキスト文字)

`tcml-format.md` §「レベル文字列中のテキスト文字」と `svg-rendering.md` §「`Text` — レベル文字列中のテキスト文字」に基づく。

## @not-implemented @smoke
### Scenario: 区間中央に `<text>` が出力される
- Given `SigA __abc__` (4 単位 Low 区間にテキスト `abc`), step=10
- Then `<g class="waveforms">` 内に `<text text-anchor="middle">abc</text>` が 1 つ出力される
- And `x` 属性は区間の中点 (区間 origin.x + 4*step/2)
- And `y` 属性は SigA 信号 box の垂直中央

## @not-implemented
### Scenario: 同区間内の複数断片は空白結合された 1 つの `<text>`
- Given `SigA __a__b_` (5 単位 Low に `a` と `b`)
- Then `<text>a b</text>` が 1 つだけ出力される (`<text>a</text>` と `<text>b</text>` の 2 つではない)

## @not-implemented
### Scenario: `font-family` は属性として `<text>` に直接出力 (CSS インジェクション回避)
- Given `@font "Comic Neue"`、`SigA __abc__`
- Then `<text font-family="Comic Neue" ...>abc</text>` が出力される (CSS `<style>` ではなく属性)

## @not-implemented
### Scenario: テキスト幅 > 区間幅でも clip しない (はみ出し許容)
- Given `SigA _abc_` (2 単位 Low に幅広テキスト `abc`、step=10)
- Then `<text>` は `<clipPath>` で囲まれず、SVG attribute も `clip-path` を持たない
- And テキストが区間外にはみ出して描画される

## @not-implemented
### Scenario: 文字列は XML エスケープされる
- Given `SigA __<a&b>__`
- Then 出力 `<text>` の中身が `&lt;a&amp;b&gt;` となる

## @not-implemented
### Scenario: クォート内の level 記号は描画されるが波形にならない
- Given `SigA =="X"==`
- Then `waveforms` レイヤに `<text>X</text>` が 1 つ
- And 波形は 4 単位連続 Bus (上端・下端 polyline) として出力され、BusCross の X 字線は出ない

## @not-implemented
### Scenario: テキストは波形 polyline に影響しない
- Given `SigA __abc__~~~~` (4 単位 Low + 4 単位 High、Low にテキスト `abc`)
- Then Low polyline は `(x_low_start, y_low) → (x_low_start + 4*step, y_low)` の通常 4 単位幅
- And `<text>` 要素は別個に出力され、polyline 経路に頂点として追加されない

---

## Gap

## @not-implemented @smoke
### Scenario: Gap で polyline が flush される
- Given `____:~~~~` (Low,4 + Gap + High,4), step=10
- Then 出力に **2 つの独立 polyline** が含まれる
- And Low の末端 (x=40) と High の先頭 (x=50) が **1 本の対角線で繋がっていない**

## @not-implemented
### Scenario: Gap 後の最初の polyline は新規開始
- Given `__:~~~`
- Then Low polyline の終点と High polyline の始点が独立要素として分離されている

---

## Guide (`|`)

### Scenario: Guide は Title 境界まで縦断 (Title 無し)
- Given Guide が信号 x=50 にある、Title を含まない 3 行のチャート
- Then `guides` レイヤの `<line>` が y1=チャート最上行 origin.y - page_margin/2、y2=チャート最下行 origin.y + bbox.size.height + page_margin/2 で出力 (上下にはみ出し)

### Scenario: Guide は上下最近 Title で止まる (Title 貫通禁止)
- Given Title 行 / 信号 A / 信号 B (Guide 起点) / 信号 C / Title 行 / 信号 D の順、信号 B の x=50 に `|`
- Then `<line>` の y1 は上方 Title 行 bbox の下端
- And y2 は下方 Title 行 bbox の上端
- And Title 行を貫通しない

### Scenario: Guide は guide_color/guide_width で描画
- Given `@guide_color blue`, `@guide_width 1`
- Then `<line stroke="blue" stroke-width="1">` で出力

---

## Highlight (`[ ]`)

## @not-implemented
### Scenario: ハイライト矩形は上下最近 Title まで縦断 (Title 無し → チャート全行はみ出し)
- Given Title を含まない 3 信号、信号 1 に `__[~~~]__`
- Then `<rect x="20" width="30">` の y はチャート最上行 origin.y - page_margin/2
- And bottom (y + height) はチャート最下行 origin.y + bbox.size.height + page_margin/2

## @not-implemented
### Scenario: ハイライト矩形は Title で止まる (Title 貫通禁止)
- Given Title / 信号 A / 信号 B (`[..]` 起点) / 信号 C / Title / 信号 D の順
- Then `<rect>` の y は上方 Title 行 bbox の下端
- And bottom (y + height) は下方 Title 行 bbox の上端
- And Title 行を貫通しない
- And 同じ Title 区間内なら `[..]` をどの信号行に書いても y/height は同じ

## @not-implemented
### Scenario: highlight_style が反映される
- Given `@highlight_style fill="#0f0" stroke="green"`
- Then ハイライト `<rect>` に `fill="#0f0" stroke="green"` 属性

---

## 信号名

## @not-implemented
### Scenario: 単一行信号名
- Given SignalRow.name = "Clock"
- Then `signal-labels` に `<text>Clock</text>`

## @not-implemented
### Scenario: 複数行信号名は tspan で改行
- Given SignalRow.name = "Data\nBus"
- Then `<text>` 内に `<tspan x dy>` × 2 で改行表示

## @not-implemented @smoke
### Scenario: `@signal(overline)` は独立した `<line>` を出す
- Given SignalRow.decorations.name_overline = true、信号名 "nReset"、`overline_gap=2`、`overline_thickness=1`
- Then `signal-labels` レイヤに `<text>` が出る (中身は `nReset`)
- And **`<text>` には `text-decoration` 属性が付かない**
- And `signal-labels` レイヤに独立した `<line>` 要素が 1 つ出る
- And `<line>` の `y1 == y2 == cap_top - overline_gap` (cap_top = baseline - cap_height)
- And `<line>` の `stroke-width == "1"`、`stroke == LabelStyle.color`

##
### Scenario: 単一行信号名の上線 x 範囲はテキスト実幅
- Given 単一行信号名 "nReset" の実幅が 50px、text-anchor=end、x_anchor=100
- Then `<line>` の `x2 == 100`、`x1 == 50`
- And label_box の幅 (例えば 80px) は使用しない

##
### Scenario: 複数行信号名でも上線は最上行に 1 本のみ・幅は最長行
- Given 信号名 "nChip\nEnable" で name_overline=true (上行 "nChip" の実幅 < 下行 "Enable" の実幅)
- Then `signal-labels` レイヤに `<line>` が **1 本のみ** 出る (最上行 "nChip" の上に位置)
- And 2 行目 "Enable" の上には `<line>` が出ない
- And `<line>` の x 幅は **全行のうち最長行 "Enable" のテキスト実幅と一致** する (最上行 "nChip" の幅ではない)

## @not-implemented
### Scenario: `@overline_gap` / `@overline_thickness` で上線位置・太さを制御
- Given `@overline_gap 5` `@overline_thickness 2` の後に `@signal(overline)` 信号
- Then `<line stroke-width="2"/>`、cap_top - 5 の高さに引かれる

## @not-implemented
### Scenario: name_overline=false なら `<line>` は出ない
- Given SignalRow.decorations.name_overline = false
- Then `signal-labels` レイヤに上線用 `<line>` は出力されない

---

## タイトル

## @not-implemented @smoke
### Scenario: TitleRow はデフォルトでチャート幅の中央に配置
- Given `@title "Synchronous Circuit"` のみ (`@titlealign` 未指定)
- And `Line.bbox.size.width == 200`、`Line.bbox.origin.x == 10` (chart_inner_width)
- Then `titles` レイヤの `<text>` は `x="110"` (= 10 + 200/2)
- And `text-anchor="middle"`
- And ` ChartStyle.title.align == HorizontalAlign::Center` (DEFAULT_TITLE_ALIGN)

## @not-implemented
### Scenario: `@titlealign left` で左寄せ
- Given `@titlealign left` の後に `@title "X"`、page_margin=10
- And `Line.bbox.origin.x == 10`
- Then `<text>` は `x="20"` (= bbox.origin.x + page_margin)、`text-anchor="start"`

## @not-implemented
### Scenario: `@titlealign right` で右寄せ
- Given `@titlealign right` の後に `@title "X"`、page_margin=10
- And `Line.bbox.origin.x == 10`、`Line.bbox.size.width == 200`
- Then `<text>` は `x="200"` (= bbox.origin.x + bbox.size.width - page_margin)、`text-anchor="end"`

## @not-implemented
### Scenario: `@titlealign` 途中変更は後続 `@title` から有効
- Given `@title "A"` → `@titlealign right` → `@title "B"`
- Then "A" の `<text>` は `text-anchor="middle"`、"B" の `<text>` は `text-anchor="end"`

## @not-implemented
### Scenario: 複数行タイトルは tspan で改行
- Given TitleRow("A\nB")
- Then `<text>` 内に `<tspan>` × 2 (各行が dy で配置)

---

## 矢印 (`@->`)

## @not-implemented @smoke
### Scenario: 最小矢印
- Given `Arrow { from: (10, 20), to: (50, 20), style: default }`
- Then `arrows` レイヤに `<line x1=10 y1=20 x2=50 y2=20>` または `<polyline>`
- And 終点に矢印頭 (`<path>` または `<polygon>`)

## @not-implemented
### Scenario: 線種 dashed
- Given `Arrow.style.line = Dashed`
- Then `<line stroke-dasharray="...">` が出力

## @not-implemented
### Scenario: 矢印頭 BothEnds
- Given `Arrow.style.head = BothEnds`
- Then 始点・終点両方に矢印頭が出力

## @not-implemented
### Scenario: 矢印ラベル
- Given `Arrow.label = Some("変化")`
- Then 線分中点に `<text>変化</text>`

### Scenario: 矢印ラベルは白縁取り付き
- Given `Arrow.label = Some("hello")`
- Then 矢印ラベルの `<text>` 要素に `paint-order="stroke fill"` 属性が付く
- And `stroke="#ffffff"` 属性が付く (デフォルト白、ハードコード)
- And `stroke-width="2"` 属性が付く (デフォルト 2px、ハードコード)
- And `stroke-linejoin="round"` 属性が付く
- And ラベル本文の `fill` 色は元の値 (縁取りで上書きされない)

##
### Scenario: 遷移後アンカーに向かう矢印が正しい座標に描画される
- Given `@step 10` `@slant 2`
  ```
  SigA  ___@{a}~~~~@{b}___
  SigB  __________@{c}___
  @-> (@{a}, @{b}) ab
  @-> (@{a}, @{c}) ac
  ```
- When SVG をレンダリングする
- Then `@-> (@{a}, @{b}) ab` の終点 x = 70 (Low×3=30 + Transition(slant=2) + High×4 で High 区間幅は `4×step - slant = 38`、累積 30+2+38=70)
- And `@-> (@{a}, @{c}) ac` の終点 x = 100 (Low×10、step×10 = 100。前後に遷移なし)
- And アンカー x 座標は前要素の `width()` 累積位置で確定する

---

## クロックエッジマーカー (`edge-marks`)

## @not-implemented @smoke
### Scenario: `@clock(pos)` で各立ち上がりエッジに `<polygon>` 三角形
- Given `@clock(pos)` で `_~_~` 展開、`slant=2`、`mark_height=5`、`mark_width=4`、`mark_position=0.5`、`signal_color=black`
- Then `<g class="waveforms">` 内に該当信号の polyline と並んで `<polygon points="...">` が 2 つ出る
- And 各 `<polygon>` は 3 頂点 (apex / base_left / base_right) を持つ
- And `fill="black"` (signal_color 継承)、`stroke="none"`
- And `<g class="arrows">` には clock 由来の要素が **存在しない**

## @not-implemented
### Scenario: `@clock(neg)` で各立ち下がりエッジに `<polygon>`
- Given `@clock(neg)` で `~_~_` 展開
- Then 立ち下がり遷移線 2 箇所に `<polygon>` (apex は右下方向)

## @not-implemented
### Scenario: `@clock(both)` は両エッジに `<polygon>`
- Given `@clock(both)` で `_~_~` 展開
- Then 立ち上がり 2 + 立ち下がり 1 = 3 個の `<polygon>` が出る (`_~_~` の場合)

## @not-implemented
### Scenario: `@clock(none)` は `<polygon>` なし
- Given `@clock(none)` の空クロック
- Then 該当信号の `<polygon>` は 0 個

## @not-implemented
### Scenario: 三角形の頂点座標は仕様式に一致
- Given `@clock(pos)` で 1 エッジ、`y_low=20`、`y_high=0`、`x=0`、`slant=2`、`mark_height=5`、`mark_width=4`、`mark_position=0.5`
- And `line_length = sqrt(2*2 + 20*20) ≈ 20.0998`
- And `apex_distance = (line_length - 5) * 0.5 + 5 ≈ 12.5499`
- Then `<polygon>` の `apex` は `line_start + line_direction * apex_distance` の座標と一致

## @not-implemented
### Scenario: `mark_color=red` 個別指定が `<polygon fill>` に反映
- Given `@clock(pos, mark_color=red)`
- Then `<polygon fill="red"/>`

## @not-implemented
### Scenario: 三角形は信号と同じ `<g class="waveforms">` グループに置かれる
- Given 任意の `@clock(pos)` 信号
- Then 該当信号の polyline と `<polygon>` が同一 `<g class="waveforms">` 親要素の直接の子である

## @not-implemented
### Scenario: `clockmark_width` デフォルト解決時に step 連動縮小が適用される (step×2/3 < 6)
- Given `@step 6` の後に `@clock(pos)` 空クロック (`@clockmark_width` 未指定、`@clockmark_height` 未指定)
- Then 各 `<polygon>` の底辺幅が `min(6, 6 × 2/3) = 4` で描画される (base_left と base_right が perpendicular 方向に幅 4 の底辺を構成する)
- And 三角形の高さ (apex_distance - base_center_distance) はデフォルト値 7.5 のまま (height は縮小されない)

## @not-implemented
### Scenario: `clockmark_width` デフォルト解決時に step が十分大きいと縮小されない (step×2/3 ≥ 6)
- Given `@step 15` の後に `@clock(pos)` 空クロック (`@clockmark_width` 未指定)
- Then 各 `<polygon>` の底辺幅が `min(6, 15 × 2/3) = min(6, 10) = 6` (= デフォルト値そのまま)

## @not-implemented
### Scenario: `clockmark_width` デフォルト解決時の step×2/3 = 6 境界
- Given `@step 9` の後に `@clock(pos)` 空クロック (`@clockmark_width` 未指定)
- Then 各 `<polygon>` の底辺幅が `min(6, 9 × 2/3) = min(6, 6) = 6`

## @not-implemented
### Scenario: グローバル `@clockmark_width` 明示指定時は step 連動縮小されない
- Given `@step 3` `@clockmark_width 8` の後に `@clock(pos)` 空クロック
- Then 各 `<polygon>` の底辺幅が `8` (step×2/3 = 2 と比べて大きくても、ユーザの明示指定がそのまま採用される)

## @not-implemented
### Scenario: グローバル `@clockmark_width` がデフォルト値と同値でも明示指定として扱われる
- Given `@step 3` `@clockmark_width 6` の後に `@clock(pos)` 空クロック
- Then 各 `<polygon>` の底辺幅が `6` (デフォルト値 6 と数値は同じだが、明示指定があったため step×2/3=2 に縮小されない)

## @not-implemented
### Scenario: ローカル `@clock(..., mark_width=<px>)` 指定時は step 連動縮小されない
- Given `@step 3` の後に `@clock(pos, mark_width=12)` 空クロック (`@clockmark_width` グローバルは未指定)
- Then 各 `<polygon>` の底辺幅が `12` (step×2/3 = 2 を大きく超えてもユーザ指定が優先)

## @not-implemented
### Scenario: ローカル `mark_width` 指定はグローバル未指定 + 縮小発生条件でも縮小しない
- Given `@step 6` の後に `@clock(pos, mark_width=8)` 空クロック (`@clockmark_width` グローバルは未指定)
- Then 各 `<polygon>` の底辺幅が `8` (グローバルが未指定でも、ローカル明示があれば min(...) の縮小は適用されない)

## @not-implemented
### Scenario: `clockmark_height` は step が小さくても縮小されない
- Given `@step 3` の後に `@clock(pos)` 空クロック (`@clockmark_height` 未指定)
- Then 各 `<polygon>` の高さ (apex から base_center までの線方向距離) がデフォルト値 `7.5` のまま
- And 同 polygon の底辺幅は `min(6, 3 × 2/3) = 2` に縮小される (width だけ縮み、height は縮まない)

## @not-implemented
### Scenario: グローバル `@clockmark_height` 明示指定はそのまま採用 (height に縮小ルールなし)
- Given `@step 3` `@clockmark_height 20` の後に `@clock(pos)` 空クロック (`@clockmark_width` 未指定)
- Then 各 `<polygon>` の高さが `20` (height は step 連動縮小の対象外)
- And 同 polygon の底辺幅は `min(6, 3 × 2/3) = 2` (width はデフォルト解決のため縮小される)

## @not-implemented
### Scenario: デフォルト `clockmark_width=6` / `clockmark_height=7.5` で十分大きい step での値
- Given `@clockmark_width` も `@clockmark_height` も未指定、`@step 20` の後に `@clock(pos)` 空クロック
- Then 各 `<polygon>` の底辺幅 = `6`、高さ = `7.5` (デフォルト値そのまま、縮小条件にも該当しない)

## @not-implemented
### Scenario: per-row `@step` 途中変更時、clock 行ごとに縮小値を再計算
- Given 1 信号目: `@step 12` の後に `@clock(pos)` 空クロック、2 信号目: `@step 3` の後に別の `@clock(pos)` 空クロック (いずれも `@clockmark_width` は未指定)
- Then 1 信号目の polygon は底辺幅 `min(6, 12 × 2/3) = 6`、2 信号目の polygon は底辺幅 `min(6, 3 × 2/3) = 2` で描画される (clock 行生成時点の `step` が縮小判定に使われる)

## @not-implemented
### Scenario: グローバル `@clockmark_width` 明示後、`@step` を縮小発動値に変えても縮小しない
- Given `@clockmark_width 8` の後に `@step 3` `@clock(pos)` 空クロック
- Then 各 `<polygon>` の底辺幅が `8` (グローバル明示が有効である間は step を小さくしても min(...) は適用されない)

## @not-implemented
### Scenario: ローカル指定はグローバル指定より優先 (どちらも縮小なし)
- Given `@step 3` `@clockmark_width 6` の後に `@clock(pos, mark_width=10)` 空クロック
- Then 各 `<polygon>` の底辺幅が `10` (ローカル `mark_width=10` がグローバル `@clockmark_width 6` を上書き、いずれの場合も縮小なし)

---

## オーバーレイ (`%`)

## @not-implemented
### Scenario: TextOverlay が `<text>` で出力
- Given `% 100 50 注釈`
- Then `overlays` レイヤに `<text x="100" y="50">注釈</text>`

---

## エスケープ・型安全

## @not-implemented @smoke
### Scenario: 信号名の `<` `>` `&` がエスケープされる
- Given 信号名 "A<B&C>"
- Then `<text>A&lt;B&amp;C&gt;</text>` で出力

## @not-implemented
### Scenario: TCML ソースもエスケープされる
- Given TCML に `<` を含む
- Then `<tchart:source>` 内で `&lt;` に変換されている

## @not-implemented
### Scenario: ユーザー値を lit() に渡すと型エラー
- Given レンダラ実装で `SignalName` を `SvgBuf::lit()` に渡そうとする
- Then コンパイル失敗 (型システムレベルでのエスケープ漏れ防止)

## @not-implemented
### Scenario: highlight_style はホワイトリスト経由でのみ出力
- Given `@highlight_style onload="alert(1)"` のような不正属性
- Then 出力されない (ホワイトリストにない属性は無視)

---

## CSS / フォント

## @not-implemented
### Scenario: `<style>` に共通スタイルが含まれる
- Given 任意の ChartDocument
- Then `<style>` 内に `.waveforms polyline { fill: none }` 等の固定 CSS

## @not-implemented
### Scenario: ユーザー font-family は CSS でなく属性として出力
- Given `@font "Helvetica, sans-serif"`
- Then `<text font-family="Helvetica, sans-serif">` (CSS 注入回避)

---

## 観点A 補強: SVG 構造・defs・スタイル境界

### Scenario: `<style>` は `<metadata>` の直後 / `<defs>` の直前に出力
- Given `?` を含む TCML
- Then 出力 SVG で `<metadata>...</metadata><style>...</style><defs>...</defs>` の順序が保持される

### Scenario: `<defs>` が `?` ゼロ件で省略される
- Given `?` を含まない TCML
- Then 出力 SVG に `<defs>` 要素自体が存在しない

### Scenario: `<defs>` 内 `<pattern>` ID は `dontcare-hatch-1` から連番
- Given チャート内に異なる `@dontcare_color` が 2 色出現
- Then `<defs>` 内に `dontcare-hatch-1` と `dontcare-hatch-2` の 2 パターンが連番で出力される
- And ID `dontcare-hatch` (番号なし) は出現しない

### Scenario: 同色 `@dontcare_color` 多数行で ID 共有
- Given `@dontcare_color #c00` で 3 行 `?` を使う
- Then `<defs>` 内 pattern は 1 個のみ (`dontcare-hatch-1`)、3 個別々には出力されない

### Scenario: pattern の `patternTransform="rotate(45)"` 固定
- Given 任意の `?` 含む TCML
- Then 全 pattern で `patternTransform="rotate(45)"` が出力される

### Scenario: pattern の `patternUnits="userSpaceOnUse"` 固定
- Given 任意の `?` 含む TCML
- Then 全 pattern で `patternUnits="userSpaceOnUse"` が出力される

### Scenario: TCML ソースの XML エスケープ — `<` `>` `&`
- Given TCML 中に `@title <foo&bar>`
- Then `<tchart:source>` 内に `&lt;foo&amp;bar&gt;` としてエスケープされて埋め込まれる

### Scenario: TCML ソース内に `]]>` を含んでも CDATA 形式は使わない
- Given TCML 中に `]]>` を含む文字列 (`@title "x ]]> y"`)
- Then 出力 SVG は `<tchart:source>` 内にエスケープして格納し、CDATA セクションは使わない

### Scenario: `<svg>` の `width`/`height` 属性は page_margin 含む
- Given chart_inner_width=100、最終行末端 y=80、page_margin=10
- Then `<svg width="120" height="100">` で出力

### Scenario: row-backgrounds レイヤが空の場合は `<g class="row-backgrounds">` 自体を省略しない (空グループでも出す) または省略する (実装決定)
- Given `@bgcolor0 none` `@bgcolor1 none` (背景なし)
- Then `<g class="row-backgrounds">` の挙動を仕様で固定 (空でも出す/省略を明示)

### Scenario: `arrows` レイヤに clock 由来 EdgeMark が混入しない
- Given `@clock(pos)` 1 信号 + `@->` 矢印 1 本
- Then `<g class="arrows">` 内には `@->` 由来の 1 本のみ
- And clock の三角形マーカーは `<g class="waveforms">` 内に出力される

### Scenario: アンカーは描画されない (0 幅マーカー)
- Given 信号行内に `@{a}` `@1` を含む
- Then 出力 SVG の波形 polyline に該当アンカー由来の頂点や `<text>` は含まれない

### Scenario: Title align=center で text-anchor=middle
- Given `@titlealign center` `@title T`
- Then Title `<text>` の `text-anchor="middle"`、x=`bbox.x + bbox.w/2`

### Scenario: Title align=left で text-anchor=start, x=bbox.x+page_margin
- Given `@titlealign left` `@title T`、page_margin=10
- Then Title `<text>` の `text-anchor="start"`、x=`bbox.x + 10`

### Scenario: Title align=right で text-anchor=end, x=bbox.x+bbox.w-page_margin
- Given `@titlealign right` `@title T`、page_margin=10
- Then `text-anchor="end"`、x=`bbox.x + bbox.w - 10`

### Scenario: Bus セグメント中央のテキストは `<text x=region.center>` で出力
- Given `Sig ==A==` (Bus 4 unit、テキスト "A")
- Then `<text x=region.center.x text-anchor="middle">A</text>` が waveforms レイヤに出力される

### Scenario: テキスト幅 > 区間幅 でも clip しない
- Given `Sig __VeryLongText__` (狭い区間に長い文字列)
- Then `<text>` 要素に `clip-path` 属性が付与されず、自然にはみ出して描画される

### Scenario: HiZ 区間は dasharray で破線、独立 polyline
- Given `Sig ____----____` (Low + HiZ + Low)
- Then HiZ 区間は `<polyline stroke-dasharray="...">` で独立し、Low の polyline と統合されない

### Scenario: 矢印ラベルが白縁取り (paint-order/stroke 属性) を持つ
- Given `@-> (@{a}, @{b}) label1`
- Then ラベル `<text>` に `paint-order="stroke fill"` `stroke="#ffffff"` `stroke-width="2"` `stroke-linejoin="round"` が付与される

### Scenario: 矢印ラベルなしのとき stroke 属性も出ない
- Given `@-> (@{a}, @{b})` (ラベルなし)
- Then ラベル `<text>` 自体が出力されない (paint-order も付かない)

### Scenario: 矢印頭 `head=none` のとき `<polygon>` が出力されない
- Given `@-> (@{a}, @{b}, head=none)`
- Then arrows レイヤに矢印頭 `<polygon>` (path) が含まれない、線のみ

### Scenario: 矢印頭 `head=both` で両端に三角形
- Given `@-> (@{a}, @{b}, head=both)`
- Then arrows レイヤに矢印頭の `<polygon>` (path) が始点・終点の 2 個出力される

### Scenario: 矢印 dashed の `stroke-dasharray`
- Given `@-> (@{a}, @{b}, dashed)`
- Then 矢印 `<line>` または `<polyline>` に `stroke-dasharray` 属性が付く

### Scenario: 矢印 dotted は dashed と異なる dasharray
- Given `@-> (@{a}, @{b}, dotted)`
- Then `stroke-dasharray` 値が dashed のそれと異なる

### Scenario: ガイド (`|`) の y 範囲に Title 行を貫通しない
- Given Title 行 + 信号 A (`|` 含) + Title 行 + 信号 B
- Then 信号 A の `|` は上下の Title 行 bbox の境界で打ち切られる

### Scenario: ガイド (`|`) の上端: 起点行より上に Title 無しならチャート上端から page_margin/2 はみ出し
- Given Title 行なし、信号 A (`|`) のみ
- Then `<line y1=...>` がチャート最上行 bbox.origin.y - page_margin/2 まで伸びる

### Scenario: ハイライト (`[...]`) の y 範囲もガイドと同じ規則
- Given Title 行なし、信号 A (`[___]`) のみ
- Then `<rect y=...>` の高さが起点行の上下に page_margin/2 はみ出す範囲

### Scenario: ハイライト矩形は `highlight_style` 属性をホワイトリスト経由で持つ
- Given `@highlight_style fill="#8f8" stroke="green"`、信号 A `[__]`
- Then `<rect>` に `fill="#8f8"` `stroke="green"` が出力される

### Scenario: clock EdgeMark の polygon 頂点座標
- Given `@clock(pos)` 空クロック、step=10、slant=2、`@clockmark_height 5`、`@clockmark_width 4`、`@clockmark_position 0.5`
- Then 各 `<polygon>` の 3 頂点が仕様式で導出された座標と一致する (apex/base_left/base_right)

### Scenario: clock EdgeMark の `fill` 属性
- Given `@clock(pos, mark_color=red)` 空クロック
- Then `<polygon fill="red" stroke="none">` で出力

### Scenario: clock EdgeMark で line_length < height のクランプ
- Given `@step 10` `@slant 2`、`@clockmark_height 100` (極端に大)、`@clock(pos)` 空クロック
- Then height が line_length にクランプされ、頂点位置が逆転しない

### Scenario: 信号名 overline は `<text text-decoration="overline">` を使わず独立 `<line>`
- Given `@signal(overline) nReset _~`
- Then 出力 SVG の `<text>` に `text-decoration` 属性は付かない
- And 同じ `signal-labels` レイヤ内に独立 `<line>` 要素が出力される

### Scenario: overline `<line>` の長さが最長行 text 実幅
- Given `@signal(overline) "short\nverylongline" _~`
- Then `<line>` の x1〜x2 幅が "verylongline" の text 実幅と一致 (短い行幅 "short" ではない)

### Scenario: overline 色が LabelStyle.color を継承
- Given `@signal_color blue` `@signal(overline) nReset _~`
- Then overline の `<line stroke="blue">`

### Scenario: bus polygon (DontCareAlongBus) の前要素 BusContinue で左辺垂直
- Given `Sig =?=` (Bus continue 両側)
- Then DontCare polygon が矩形 (4 頂点で 上辺=y_high, 下辺=y_low, 左右辺=垂直)

### Scenario: bus polygon の前 Low (`_=?=_`) で左辺斜辺
- Given `Sig _=?=_` (BusOpen + ? + BusClose)
- Then polygon は六角形 `/=\` 形 (左辺/右辺は斜辺)

### Scenario: bus polygon の前 BusCross (`=X?X=`) で左辺右辺ともに cross 中点
- Given `Sig =X?X=`
- Then polygon は六角形 `>▲■▲<`、左右辺の頂点が X の cross 中点 `(x + slant/2, y_mid)`

### Scenario: dontcare polygon にアウトライン (stroke) なし
- Given `Sig _?_`
- Then 出力 `<polygon>` の `stroke` 属性は付かない or `stroke="none"` (アウトライン無し)

### Scenario: SVG の `<style>` に CSS インジェクション可能な値を出さない
- Given `@font "monospace; }body{...}"` (悪意ある family 名)
- Then `<style>` 内には font-family 値は出力されず、`<text font-family="...">` 属性として escape して出力

### Scenario: PolyAccum は Gap 通過後にフラッシュされ独立 polyline になる
- Given `Sig ____:____`
- Then 出力に Low 区間 polyline が 2 本独立して出る (1 本ではない)

### Scenario: PolyAccum は HighlightStart/End では flush しない
- Given `Sig __[__]__`
- Then Low 区間 polyline は連続 1 本 (Highlight は波形蓄積に影響しない)

### Scenario: PolyAccum はアンカー通過で flush しない
- Given `Sig __@{a}__`
- Then Low 区間 polyline は連続 1 本

---

## 観点B 補強: SVG レンダリングの組合せ

### Scenario: per-row `@step` 変更 × signal_box 内位置の正しさ
- Given Sig1 step=10、Sig2 step=20、ともに 4 文字波形
- Then Sig2 の `<polyline points>` の x 座標は step=20 ベースで Sig1 の 2 倍

### Scenario: `@clock(auto)` × `@bg` × bgcolor0 (`@bg` 優先)
- Given `@bgcolor0 #eee` `@bg #f0f` `@clock(pos)` 空クロック
- Then クロック信号行の背景は `#f0f` (bgcolor0 優先しない)

### Scenario: per-row `@step` 変更 × `@->` 矢印描画
- Given Sig1 step=10 で `@1`、Sig2 step=20 で `@2`、`@-> (@1, @2)`
- Then 矢印の始点/終点が各信号の step ベースで個別に解決され `<line>` 描画される

### Scenario: DontCare × アンカー × ハイライト 同居
- Given `Sig __[?]@{a}__`
- Then DontCare polygon、ハイライト矩形、アンカー (描画なし) がそれぞれ独立レイヤに出力される

### Scenario: BusCross × ハイライト × DontCare
- Given `Sig =[X?X]=`
- Then BusCross の cross 描画、DontCare polygon、ハイライト矩形が共存

### Scenario: `@signal(overline)` × 複数行信号名 × `@bg` 適用行
- Given `@bg #ff0` `@signal(overline) "ne\nrst" _~`
- Then 信号行 background `#ff0`、複数行信号名、最上行のみに overline `<line>`、上線幅は最長行幅

### Scenario: Title align=right × `@bg` 適用 × bgcolor1 を上書き
- Given `@bgcolor1 blue` `@bg red` `@titlealign right` `@title "T"`
- Then Title 行は `<rect fill="red">` (bgcolor1 出力なし)、`<text text-anchor="end">`

### Scenario: 多レイヤの z-order 検証 (DontCare × ハイライト × 波形 × アンカー)
- Given DontCare、ハイライト、信号波形、アンカー、矢印を含むチャート
- Then 出力 SVG で `<g>` 順が `dontcares` → `highlights` ?  仕様順 (`row-backgrounds` → `highlights` → `dontcares` → `signal-labels` → `waveforms` → `guides` → `titles` → `arrows` → `overlays`) を満たす

## ユーザ承認反映シナリオ (2026-05-10)

### Scenario: `@->` 矢印が clock エッジマーカーより前面
- Given `@clock(pos)` 信号と、その立ち上がりエッジ位置を始端とする `@->` を含むチャート
- Then 出力 SVG で `<g class="arrows">` が `<g class="waveforms">` (clock マーカー含む) より後 (= z-order 上、矢印が前面)
- And アンカー位置で矢印と clock マーカーが重なるとき、矢印が常に上に描かれる

### Scenario: `clockmark_color` 未指定は clock 行生成時の signal_color を焼き込む
- Given チャート: `@signal_color red\n@clock(pos)\nclkA _~_~\n@signal_color blue\nclkB _~_~`
- Then clkA の clock マーカー fill = red (生成時点の signal_color)
- And clkB の clock マーカー fill = blue (現在 signal_color)
- And clkA を再着色しない (静的に確定)

---

## 観点D 補強: SVG 構造的不変条件

### Scenario: ルート `<svg>` 直下の子順序が常に固定 (metadata → style → defs → g.row-backgrounds → g.highlights → ...)
- Given 任意の有効 TCML
- Then 出力 SVG の ルート直下の子要素順は仕様で固定された順 (途中省略可だが順序逆転なし)

### Scenario: `<defs>` が `?` 0 個のチャートで出力されない
- Given `A _~_~` (DontCare なし、ハッチ pattern も不要)
- Then 出力 SVG に `<defs>` 要素なし

### Scenario: `<defs>` が `?` 1 個 (最小 DontCare) で出力される
- Given `A _?_`
- Then 出力 SVG に `<defs><pattern id="dontcare-hatch-1">...</pattern></defs>`

### Scenario: 信号名内 `<` `>` `&` のエスケープ一貫性 (1 行・複数行両方)
- Given 信号名 `"<foo>&<bar>"` と `"<a>\n<b>"` を別々に持つチャート
- Then すべての `<text>` 要素中で `&lt;` `&gt;` `&amp;` に置換される (literal `<` を出さない)

### Scenario: `@title` 内 `<` `>` `&` のエスケープ
- Given `@title "<T>&<U>"`
- Then 出力 `<text>` 中身は `&lt;T&gt;&amp;&lt;U&gt;`

### Scenario: `@->` ラベル内 `<` `>` `&` のエスケープ
- Given `@-> (@{a}, @{b}) <abc>&<xyz>`
- Then ラベル `<text>` 中身は `&lt;abc&gt;&amp;&lt;xyz&gt;`

### Scenario: TCML ソース埋め込みでも `<` `>` `&` がエスケープされる (CDATA は使わない)
- Given TCML ソース内に `<` を含むコメント
- Then `<tchart:source>` 子テキストは `&lt;` で表現

### Scenario: TCML ソースに `]]>` を含んでも CDATA 形式を採用しない
- Given TCML ソースに `]]>` を含む文字列
- Then 出力 SVG は `&lt;![CDATA[` 系を使わず通常エスケープのみ

### Scenario: z-order: row-backgrounds が常に最背面
- Given 任意の塗りなしチャート (要素ゼロ) でも `<g class="row-backgrounds">` が他レイヤより前に置かれる順序を保つ
- Then 出力 SVG で `<g class="row-backgrounds">` は他 `<g>` のうち最初

### Scenario: z-order: arrows が overlays より背面 (overlays が最前面)
- Given `%` 行 (overlay) と `@->` 矢印を同時に含むチャート
- Then `<g class="arrows">` の後に `<g class="overlays">` が置かれる

### Scenario: 空グループ省略 (`<g class="dontcares">` を要素 0 で省く)
- Given `?` ゼロのチャート
- Then 出力 SVG に `<g class="dontcares">` が存在しない (空グループを書かない)

### Scenario: 空グループ省略 (`<g class="arrows">` を矢印ゼロで省く)
- Given `@->` を 1 個も持たないチャート
- Then 出力 SVG に `<g class="arrows">` が存在しない

### Scenario: 単一 `<defs>` のみで `<pattern>` ID が `dontcare-hatch-1` から開始
- Given チャート初登場の `?` (色 `none` 以外)
- Then `<pattern id="dontcare-hatch-1">` が最初の登録

### Scenario: 同色 DontCare の重複登録なし
- Given 4 行 × 各行 `?`、全行同じ `@dontcare_color red`
- Then `<pattern>` は 1 個のみ、ID は 1 個

---

## 観点G: テキスト文字 (筑波大方式) のエッジケース

## @not-implemented
### Scenario: 同一 Bus 区間内に 5 個以上のテキスト断片
- Given `data: =A=B=C=D=E=F=G===` (パーサ merge 後 Bus 1 区間 + テキスト 7 断片)
- Then SVG `<text>` 要素のうち、Bus 中央ラベルは 1 個のみ (空白連結された 7 断片)
- And ラベル本文は `"A B C D E F G"` (順序保持、空白 1 個区切り)

## @not-implemented
### Scenario: テキストに全角文字 (CJK) を含む Bus ラベル
- Given `data: =日=本=語==`
- Then `<text>` 中央ラベルは `"日 本 語"` (UTF-8 完全保持)
- And テキスト x 座標は Bus 区間中心に配置される

## @not-implemented
### Scenario: テキストに絵文字 (BMP 外、サロゲートペア) を含む
- Given `data: ="🎉"==` (絵文字 1 文字)
- Then `<text>` の本文は `🎉` (UTF-8 4 バイト保持)
- And SVG 出力 byte 列を `from_utf8` できる

## @not-implemented
### Scenario: テキストに結合文字 (合字 / 濁点) を含む
- Given `data: ="が"==` (`か` + `゛` の合字、または NFC `が`)
- Then `<text>` の本文は元 byte 列を保持し、SVG 上で 1 グリフ相当を表現

## @not-implemented
### Scenario: BusCross 直後のテキストが直前 / 直後どちらの Bus に属するか
- Given `data: =a=Xab==` (Bus + テキスト + BusCross + テキスト + Bus)
- Then `a` は左 Bus 区間、`ab` は右 Bus 区間に所属する `<text>` として配置される
- And 左右で 2 個の `<text>` が出力される

## @not-implemented
### Scenario: BusCross の直接挟み (`XaXb=`) のテキスト所属
- Given `data: ===XaXb=`
- Then `a` は (左の) BusCross 直後の Bus 区間ラベル、`b` は (次の) BusCross 直後の Bus 区間ラベルに所属
- And BusCross 自身にはテキストが付かない

## @not-implemented
### Scenario: クォート literal 中の `\"` × level 記号混入
- Given `data: ="A\"_~B"==` (literal 内に `\"` と `_`/`~` 文字を含む)
- Then `<text>` の本文は `A"_~B` (アンエスケープ後)
- And `_`/`~` は level 記号として解釈されず文字列として保持される

## @not-implemented
### Scenario: クォート literal 中の `\\` バックスラッシュ
- Given `data: ="path\\to\\dir"==`
- Then `<text>` の本文は `path\to\dir` (バックスラッシュ 1 個ずつ)

## @not-implemented
### Scenario: 区間幅 < テキスト幅でも `<text>` は出力される (はみ出しを許容)
- Given `data: =VeryLongLabelName==` (Bus 1 unit、ラベル長 > step 幅)
- Then `<text>` 要素が 1 個出力される (描画は SVG 描画系に委ねる)
- And `<text>` の x 座標は Bus 区間中心 (はみ出し配置を許容)

## @not-implemented
### Scenario: 連続 BusCross + テキスト (`=XaXbXc=`) の所属順序
- Given `data: =Xa=XbXc=`
- Then 4 個の `<text>` 要素 (a, b, c に加え区間ラベルがあれば)、SVG 上の x 座標で左から所属を判定可能

## @not-implemented
### Scenario: テキストに XML 危険文字 (`<`, `>`, `&`) を含む Bus ラベル
- Given `data: ="<&>"==`
- Then `<text>` 本文は `&lt;&amp;&gt;` (XML エスケープ済み)
- And SVG として well-formed

## @not-implemented
### Scenario: テキスト中の `]]>` シーケンス
- Given `data: ="a]]>b"==`
- Then `<text>` 本文は `a]]&gt;b` (CDATA を使わず通常エスケープのみ)

## @not-implemented
### Scenario: 空文字列ラベル `""` を含む Bus 区間
- Given `data: =""==` (空 literal)
- Then 空 `<text>` 要素は出力しない (空文字列はラベルなし扱い)

## @not-implemented
### Scenario: 同一 Bus 区間に空白のみのラベル `" "` を含む
- Given `data: =" "==`
- Then `<text>` の本文は `" "` (空白文字を保持) として 1 個出力

---

## 観点I 補強: `@bgcolor0/1` × 多行構成のエッジ

## @not-implemented
### Scenario: Skip 直後の SignalRow の偶奇インデックス
- Given `A _\n@skip(1)\nB _` (SignalRow / SkipRow / SignalRow)
- Then 偶奇カウントは Skip 行を除外せず行うため、`B` は奇数行扱い
- And `B` 行の row-background 色は `@bgcolor1` (Skip カウント込みなら奇)

## @not-implemented
### Scenario: Title が連続したあとの偶奇カウント (Title はカウント外)
- Given `@title T1\n@title T2\nA _\nB _` (Title 2 行 + 信号 2 行)
- Then `A` 行は index 0 (`@bgcolor0`)、`B` 行は index 1 (`@bgcolor1`)
- And Title 行は row-background を持たない

## @not-implemented
### Scenario: `@bgcolor0` のみ指定 (奇数行は無背景)
- Given `@bgcolor0 #eee\nA _\nB _\nC _`
- Then 偶数行 (`A`, `C`) のみ `<rect fill="#eee">`、奇数行 (`B`) は rect 出力なし

## @not-implemented
### Scenario: `@bgcolor1` のみ指定 (偶数行は無背景)
- Given `@bgcolor1 #eee\nA _\nB _\nC _`
- Then 奇数行 (`B`) のみ rect 出力、偶数行 (`A`, `C`) は rect なし

## @not-implemented
### Scenario: 全色 `none` で `<rect>` 出力ゼロ
- Given `@bgcolor0 none\n@bgcolor1 none\nA _\nB _\nC _`
- Then 出力 SVG に `<g class="row-backgrounds">` の子要素は 0 個 (空グループ自体も省略)

## @not-implemented
### Scenario: `@bg` (1 行限り) と `@bgcolor*` 同時指定の優先関係
- Given `@bgcolor0 #eee\n@bgcolor1 #ddd\n@bg yellow\nA _\nB _`
- Then `A` 行は `@bg yellow` (per-row 優先)、`B` 行は `@bgcolor1 #ddd`
- And 同行で重ね描きはしない (`@bg` が勝つ)

---

## 観点J: `@->` 矢印の高度ケース

## @not-implemented
### Scenario: 矢印が複数行にまたがる (始終端が別信号行)
- Given `A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b})`
- Then 矢印 `<path>` または `<line>` の y 座標が 2 行ぶん異なる
- And 始点 / 終点が各信号行の中心 y に着地する

## @not-implemented
### Scenario: 同一アンカー対の矢印を 2 本書く (順序保持・重ね描き)
- Given `A _@{a}~\nB _~@{b}\n@-> (@{a}, @{b})\n@-> (@{a}, @{b}, color=red)`
- Then `<g class="arrows">` 内に矢印が 2 本、TCML 順 (1 本目=デフォルト色、2 本目=赤)
- And 2 本目が後に描画される (z 順は SVG 文書順)

## @not-implemented
### Scenario: 矢印 `head=both` × `width=0px` × `style=dashed`
- Given `@-> (@{a}, @{b}, head=both, width=0px, style=dashed)`
- Then `<line>` または `<path>` の `stroke-width="0"` が出力 (描画されないが構造は残る)
- And 両端 `<polygon>` 矢印頭が 2 個出力 (head=both)
- And `stroke-dasharray` 属性が dashed に対応する値で出力

## @not-implemented
### Scenario: 矢印ラベルの位置計算 (中点配置)
- Given `@-> (@{a}, @{b}, label="L")`、`@{a}` と `@{b}` の x 座標差が既知
- Then ラベル `<text>` の x 座標は `(x_a + x_b) / 2`
- And y 座標は矢印中点 + ベースラインオフセット (font メトリクス依存)

## @not-implemented
### Scenario: 矢印ラベルが空白を含む長文
- Given `@-> (@{a}, @{b}, label="long label here")`
- Then `<text>` 本文は `"long label here"` (空白保持)
- And 縁取り stroke 属性が付く

## @not-implemented
### Scenario: 矢印の z-order が clock マーカー直上
- Given `@clock(pos)` 信号 + `@->` を同一行に持つチャート
- Then 出力 SVG で `<g class="clock-marks">` (または相当) が `<g class="arrows">` より前 (背面)
- And `arrows` グループ全体が clock マーカーより前面

## @not-implemented
### Scenario: 矢印 100 本 × 同一信号行
- Given アンカー 200 個 + 矢印 100 本 (全て同一信号行内)
- Then `<g class="arrows">` 子要素数が 100 (ラベルなしなら) または 200 (ラベル付きで `<path>` + `<text>`)
- And SVG パース成功

## @not-implemented
### Scenario: 矢印頭 `head=start` のとき終端側 `<polygon>` が出ない
- Given `@-> (@{a}, @{b}, head=start)`
- Then 始点側 `<polygon>` 1 個のみ、終点側矢印頭は出力しない

## @not-implemented
### Scenario: 矢印 `style=solid` 明示で `stroke-dasharray` 属性が出ない
- Given `@-> (@{a}, @{b}, style=solid)`
- Then `<line>` / `<path>` 属性に `stroke-dasharray` を含まない (solid デフォルト)

---

## 観点N 補強: SVG 出力での Unicode

## @not-implemented
### Scenario: RTL 信号名 (アラビア語) の `<text>` 出力
- Given `العربية _~_~`
- Then `<text>` 本文が UTF-8 で正しく出力
- And `direction="rtl"` または `xml:lang` 属性が出力されるかは仕様準拠
- Note: spec 未定義なら spec gap

## @not-implemented
### Scenario: ヘブライ語信号名で BiDi 制御文字を出力しない
- Given `שלום _~`
- Then `<text>` 本文に U+202E などの制御文字が混入しない (純粋に原文のみ)

## @not-implemented
### Scenario: 結合文字 (NFD) のテキスト出力で順序保持
- Given Bus ラベル `é` (e + U+0301)
- Then SVG 出力中の `<text>` 本文 byte 列が NFD のまま (NFC 変換しない)

## @not-implemented
### Scenario: サロゲートペア絵文字 1 個の `<text>` 描画
- Given Bus ラベル `🎉`
- Then `<text>` 本文に UTF-8 4 byte がそのまま出力
- And フォント解決失敗時は警告を出すが SVG 自体はエラーなく生成

## @not-implemented
### Scenario: 全角空白を含む信号名のレイアウト
- Given `A　B _~`
- Then `<text>` 本文は `A　B`、capwidth は全角分含めて算出

## @not-implemented
### Scenario: ゼロ幅スペースを含むラベルで visible 幅 0
- Given Bus ラベル `A​B` (中央 ZWSP)
- Then `<text>` 本文には ZWSP も含まれる
- And visible 幅は `AB` とほぼ同じ (ZWSP 0 width)

---

## 観点O 補強: SVG ID / class 一意性

## @not-implemented
### Scenario: 同色 DontCare が複数信号で連続出現しても `<pattern>` ID は 1 個のみ
- Given 3 信号 全てに `@dontcare_color #ff0000` の DontCare
- Then `<defs>` 内 `<pattern id="dontcare-hatch-...">` は 1 個 (色キーで de-dup)

## @not-implemented
### Scenario: 異色 DontCare が 5 種類で `<pattern>` ID は 5 個 (連番 1..5)
- Given 5 信号 異なる `@dontcare_color`
- Then `<defs>` 内 `<pattern id="dontcare-hatch-1">` ... `<pattern id="dontcare-hatch-5">`
- And ID 重複なし、欠番なし

## @not-implemented
### Scenario: polyline に id 属性は付けない (class のみ)
- Given 任意のチャート
- Then `<polyline>` 要素に `id="..."` 属性は出力しない
- And class 名は仕様で定義されたもののみ

## @not-implemented
### Scenario: 同じ TCML を 2 回レンダーして SVG byte 一致 (deterministic)
- Given 同一 TCML 入力 X を `tchart svg` 2 回実行
- Then 出力 SVG が byte 単位で完全一致
- And HashMap 順序差異 / 浮動小数 NaN bit 差異 なし

## @not-implemented
### Scenario: 同名 anchor を別信号で再宣言
- Given `A _@{a}~\nB _@{a}~` (`@{a}` を 2 信号に重複定義)
- Then ParseError (anchor 名は document scope で unique) または最初を採用
- Note: 仕様確認 (spec gap 候補)

## @not-implemented
### Scenario: 矢印グループ内 `<g class="arrows">` は最大 1 個
- Given 矢印 100 本
- Then `<g class="arrows">` は 1 個、その下に 100 個の `<path>` / `<line>`
- And `<g class="arrows">` を 2 個に分割しない

## @not-implemented
### Scenario: 全 `<defs>` 子要素の id 属性が unique
- Given 任意の複雑チャート (DontCare 複数色 + clock + 矢印)
- Then `<defs>` 直下要素の id 集合に重複なし

## @not-implemented
### Scenario: SVG class 名は仕様で定義された語のみ
- Given 任意のチャート
- Then class 属性値が ホワイトリスト (`signal-name`, `clock-marks`, `arrows`, `level-low`, ...) に含まれる
- Note: spec で class 一覧が未定義なら spec gap

## @not-implemented
### Scenario: 同 TCML の差異な改行 (LF vs CRLF) でも SVG byte 一致
- Given 同一 TCML を LF 版 / CRLF 版で 2 つ
- Then `tchart svg` 出力が byte 単位で一致 (改行差は parser で吸収)

---

## 観点S 補強: SVG round-trip (rendering)

## @not-implemented
### Scenario: SVG round-trip 2 回目以降の出力が初回と一致
- Given TCML X → SVG_1 → 抽出 TCML → SVG_2 → 抽出 TCML → SVG_3
- Then SVG_2 == SVG_3 (byte 一致)
- And TCML 抽出も 2 回目以降は一致

## @not-implemented
### Scenario: 末尾改行ありなしの round-trip 一致 (SVG 内 source)
- Given TCML 末尾 LF あり / なしの 2 ファイル
- Then SVG 内 `<tchart:source>` は元 byte をそのまま保持 (改行差を残す)
- And round-trip で改行差は消えない

---

## `@ruler` 背景縦線 (`<g class="rulers">`)

`docs/spec/svg-rendering.md` §「`rulers` (`@ruler` 由来の背景縦線)」および §「描画順 (z-order)」の検証。

寄付モデルのパース側 (各行のサイドカー `Vec<RulerContribution>`) は `tcml-parser.feature.md` §「`@ruler` 寄付モデル」を参照。本セクションは **SVG 出力形式・マージ動作・z-order** のみを規定する。

### 空レイヤ省略 (デフォルト / 寄付ゼロ)

## @not-implemented @smoke
### Scenario: `@ruler` を一切指定しないチャートでは `<g class="rulers">` がデフォルト on で出力される
- Given `@ruler` 系ディレクティブを一切含まない TCML (例: `@step 10` の後に `A _~_~_~_~`、units = 8)
- When SVG レンダリングする
- Then 出力 SVG に `<g class="rulers">` 要素が出力される (デフォルト `ruler=on` のため信号行が寄付する)
- And その中に `units + 1` 本の `<line>` (x = `0, 10, ..., 80`) が含まれる

## @not-implemented
### Scenario: `@ruler off` のみで寄付ゼロのときも `<g class="rulers">` が省略される
- Given `@ruler off` のみ指定し、すべての信号行が off 状態で commit されている TCML
- When SVG レンダリングする
- Then `<g class="rulers">` 要素は出力されない

## @not-implemented
### Scenario: 全行が `@ruler off` 状態で寄付ゼロでも他レイヤは通常通り
- Given `@ruler on`、その直後すぐに `@ruler off`、信号行 A、B (= A も B も off 状態で commit)
- When SVG レンダリングする
- Then `<g class="rulers">` は省略される
- And `<g class="row-backgrounds">` `<g class="waveforms">` 等の他レイヤは通常通り出力される

### 基本出力形式

## @not-implemented @smoke
### Scenario: `@ruler on` 状態の信号行は `<g class="rulers">` に `<line>` を出力する
- Given `@step 10`、`@ruler on`、信号行 `A _~_~_~` (units = 6)、`chart_inner_height = 24`
- When SVG レンダリングする
- Then `<g class="rulers">` 要素が出力される
- And その中に 7 本の `<line>` が含まれる (x = `0, 10, 20, 30, 40, 50, 60`)
- And 各 `<line>` は `x1` と `x2` が同値、`y1="0"`、`y2="24"` を持つ
- And 各 `<line>` は `stroke="#a0a0a0"`、`stroke-width="0.5"`、`stroke-dasharray="3 5"` を持つ

## @not-implemented
### Scenario: 線の y 範囲は `chart_inner_height` 全体 (page_margin 外側は侵さない)
- Given 複数行を含むチャート (`Σ Line.bbox.size.height = 80`)、`@ruler on` で寄付あり
- When SVG レンダリングする
- Then `<g class="rulers">` 内の各 `<line>` は `y1="0"`、`y2="80"` を持つ
- And `page_margin` の外側余白には侵入しない (y1, y2 は page_margin 内側起点)

## @not-implemented
### Scenario: ruler `<line>` は x 昇順で並ぶ
- Given `@step 10`、`@ruler on`、複数信号行 (寄付 x は `{0, 10, 20, 30}` を含む)
- When SVG レンダリングする
- Then `<g class="rulers">` 内の `<line>` は x の昇順 (`x="0"` → `x="10"` → `x="20"` → `x="30"`) で出力される

## @not-implemented
### Scenario: `stroke-width` と `stroke-dasharray` は固定値
- Given `@ruler on` + 任意の信号行 (寄付あり)
- When SVG レンダリングする
- Then 出力 `<line>` の `stroke-width` は常に `0.5`
- And `stroke-dasharray` は常に `3 5`
- Note: ユーザ可変パラメータではない (タスク決定値)

### last-wins マージ (同 x 重複)

## @not-implemented @smoke
### Scenario: 同じ x に複数行が寄付した場合、線は 1 本のみ
- Given `@step 10`、`@ruler on`、`@ruler_color #aaa`、信号行 A (units=4)、信号行 B (units=4)
- When SVG レンダリングする
- Then x=0, 10, 20, 30, 40 ぞれぞれに `<line>` が **1 本ずつ** (合計 5 本) 出力される
- And A と B が同じ x に寄付しても重複 `<line>` は出力されない

## @not-implemented
### Scenario: 同 x への寄付は後の行 (= 下の行) の色が勝つ (last-wins)
- Given `@step 10`、`@ruler on`、`@ruler_color #aaa`、信号行 A (units=4)、`@ruler_color #bbb`、信号行 B (units=4)
- When SVG レンダリングする
- Then x=0, 10, 20, 30, 40 の各 `<line>` の `stroke` は `#bbb` (= 後の行 B のスナップショット色)
- And A の寄付色 `#aaa` の `<line>` は出力されない

## @not-implemented
### Scenario: 異なる x への寄付は別々の色で両方残る
- Given `@step 10`、`@ruler on`、`@ruler_color #aaa`、信号行 A (units=2, x ∈ {0, 10, 20})、`@ruler_color #bbb`、`@step 25`、信号行 B (units=2, x ∈ {0, 25, 50})
- When SVG レンダリングする
- Then `<g class="rulers">` に 5 本の `<line>` が出力される (x = `0, 10, 20, 25, 50`)
- And x=0 の `<line>` は `stroke="#bbb"` (B の color が後で上書き)
- And x=10, 20 の `<line>` は `stroke="#aaa"` (A のみが寄付した x)
- And x=25, 50 の `<line>` は `stroke="#bbb"` (B のみが寄付した x)

## @not-implemented
### Scenario: 3 行以上が同 x に寄付しても、最後の行の色が勝つ
- Given `@step 10`、`@ruler on`、`@ruler_color #aaa` 信号行 A、`@ruler_color #bbb` 信号行 B、`@ruler_color #ccc` 信号行 C (全行同じ範囲)
- When SVG レンダリングする
- Then 各 x の `<line>` の `stroke` は `#ccc` (最後の行 C の color)
- And `#aaa` `#bbb` の `<line>` は 1 本も出力されない

### `@ruler off` でグローバル全消えにならない (核心)

## @not-implemented @smoke
### Scenario: 途中で `@ruler off` しても、それより前の行の寄付は残る
- Given `@step 10`、`@ruler on`、信号行 A (units=4)、`@ruler off`、信号行 B (units=4)
- When SVG レンダリングする
- Then `<g class="rulers">` に 5 本の `<line>` が出力される (A の寄付 x = `{0, 10, 20, 30, 40}`)
- And B 由来の `<line>` は 1 本も含まれない
- And `@ruler off` で A の過去寄付が「グローバル全消え」されないことが確認できる

## @not-implemented @edge-case
### Scenario: `@ruler on` → `@ruler off` → `@ruler on` のトグルで各行寄付が独立に残る
- Given `@ruler on`、信号行 A、`@ruler off`、信号行 B、`@ruler on`、信号行 C (各行 units=2、`@step 10`)
- When SVG レンダリングする
- Then `<g class="rulers">` に A と C の寄付 (同 x が重複した分は 1 本) が出力される
- And B 由来の `<line>` は含まれない
- And 同 x には C の color が勝つ (last-wins)

### `@step` 途中変更とのスナップショット

## @not-implemented
### Scenario: `@step` を途中で変えても古い行の寄付位置は固定
- Given `@step 10`、`@ruler on`、信号行 A (units=4、寄付 x = `{0, 10, 20, 30, 40}`)、`@step 25`、信号行 B (units=4、寄付 x = `{0, 25, 50, 75, 100}`)
- When SVG レンダリングする
- Then `<g class="rulers">` に 9 本の `<line>` が出力される (x = `0, 10, 20, 25, 30, 40, 50, 75, 100`)
- And `@step 25` 変更で A の寄付位置 (`10, 20, 30, 40`) が `25, 50, 75, 100` に書き換わったりはしない

### `@skip` 行の寄付

## @not-implemented
### Scenario: `@skip` 行も寄付源として `<g class="rulers">` に反映される
- Given `@step 10`、`@ruler on`、`@skip(3)` (units=3、寄付 x = `{0, 10, 20, 30}`)、信号行 A (units=3、同じ x)
- When SVG レンダリングする
- Then `<g class="rulers">` に 4 本の `<line>` (x = `0, 10, 20, 30`、A の色で上書き、last-wins)

## @not-implemented
### Scenario: `@skip` 単体でも寄付が `<g class="rulers">` に出る
- Given `@step 10`、`@ruler on`、`@skip(2)` のみ (信号行なし)
- When SVG レンダリングする
- Then `<g class="rulers">` に 3 本の `<line>` (x = `0, 10, 20`) が出力される

### z-order (描画順)

## @not-implemented @smoke
### Scenario: `<g class="rulers">` は `row-backgrounds` の直後、`highlights` の直前
- Given `@bgcolor0 #eee`、`@ruler on`、`[A]` ハイライト、信号行を含むチャート
- When SVG レンダリングする
- Then SVG 内のレイヤ順は `<g class="row-backgrounds">` → `<g class="rulers">` → `<g class="highlights">` の順に出現する
- And `<g class="rulers">` は `<g class="waveforms">` `<g class="edge-marks">` `<g class="guides">` `<g class="arrows">` のすべてより前 (= z-order 上は背面) に出力される

## @not-implemented
### Scenario: 空レイヤ省略時も相対順序は不変
- Given `@bgcolor0` 未指定 (row-backgrounds 省略) + `@ruler on` で寄付あり
- When SVG レンダリングする
- Then `<g class="row-backgrounds">` は出力されない (空レイヤ省略)
- And `<g class="rulers">` は出力される
- And `<g class="rulers">` と `<g class="highlights">` (出力される場合) の相対順序は `rulers` → `highlights` のまま保たれる

### 組み合わせシナリオ

## @not-implemented
### Scenario: `@ruler` × `@bg` (行ローカル背景) × 信号行
- Given `@ruler on`、`@bg #ff0` で 1 行目を行ローカル背景、信号行 A
- When SVG レンダリングする
- Then `<g class="row-backgrounds">` 内に `@bg #ff0` の `<rect>` が出力される
- And その**前面** (`<g class="rulers">`) に A の寄付 ruler `<line>` が描画される (z-order: row-backgrounds → rulers)
- And ruler 線は行背景 `<rect>` の上を貫通して描画される (1 本の `<line>` が全行を縦断)

## @not-implemented
### Scenario: `@ruler` × `[ ]` ハイライト
- Given `@ruler on`、`[A]` 区間を持つ信号行
- When SVG レンダリングする
- Then `<g class="rulers">` の `<line>` は `<g class="highlights">` の `<rect>` よりも背面 (z-order 上は先に出力)
- And ハイライト矩形 `<rect>` は ruler 線を覆い隠す位置関係になる

## @not-implemented @edge-case
### Scenario: `@ruler on` の状態で TCML round-trip しても SVG 出力が一致
- Given `@ruler on` を含む TCML X → SVG_1 → 抽出 TCML → SVG_2
- When 両 SVG を比較する
- Then SVG_1 == SVG_2 (byte 一致)
- And `<g class="rulers">` 内の `<line>` 数・順序・属性も一致

## @not-implemented @edge-case
### Scenario: 1 本の寄付しかなくても `<g class="rulers">` は出力される
- Given `@step 10`、`@ruler on`、`@skip(0)` のみ (units=0 → 寄付は x=0 の 1 本のみ)
- When SVG レンダリングする
- Then `<g class="rulers">` 要素が出力される
- And `<line>` は 1 本のみ (x=0)
- Note: 「空レイヤ省略」の対象は「寄付ゼロ」のみ。1 本でもあれば省略しない。
