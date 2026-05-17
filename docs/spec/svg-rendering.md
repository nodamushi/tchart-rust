# SVG レンダリング仕様

[rust.md](../coding/rust.md) を必ず一読し、本プロジェクトにおける Rust のコーディングルールを把握すること。

> 描画対象データ型は実装ソース (`tchart-core/src/`) が正本。実装構造とアルゴリズム概要は [`types.md`](types.md)。本ドキュメントは `ChartDocument` (layout-resolved) → SVG 文字列の変換規則を扱う。

## セキュリティ: 型による信頼境界

SVG レンダリングはユーザー入力を SVG / XML テキストへ流すため、**型による信頼境界**を守る。

- ユーザー由来の文字列は `SignalName` / `UserText` / `FontFamily` / `Color` 等の **NewType** でレンダラーに到達する。生 `String` / `&str` は受け取らない。
- 出力バッファ `SvgBuf` は次の API のみを公開:

  | API | 用途 |
  |-----|------|
  | `lit(&'static str)` | 内部固定値 (タグ名・属性名・固定属性値) |
  | `escaped<T: UserValue>(&T)` | ユーザー入力を XML エスケープ |
  | `user_attr<T: UserValue>(name: &'static str, value: &T)` | `name="escaped"` 形式 |
  | `static_attr(name: &'static str, value: &'static str)` | 内部固定属性 |
  | `safe_presentation_attrs(&SvgAttrList)` | `highlight_style` 等をホワイトリスト経由で書き出し |

- `UserValue` は `SignalName` / `UserText` / `FontFamily` / `Color` などが実装する密閉トレイト。`String` は実装しない。
- `UserValue` は `Display` を実装しない。`format!` / `write!` での生埋め込みは型エラー。
- `&UserValue → &str` は `as_unsafe_str()` のみ。命名で危険性を明示する。

これにより新規ユーザー値の SVG 出力箇所は型システムレベルでエスケープ漏れを防ぐ。詳細は [`architecture.md`](architecture.md)「型による信頼境界」参照。

## 出力 SVG の構造

```xml
<svg xmlns="http://www.w3.org/2000/svg"
     xmlns:tchart="http://tchart-rust/1.0"
     width="..." height="..." viewBox="0 0 ... ...">
  <metadata>
    <tchart:source>...XML エスケープされた TCML ソース...</tchart:source>
  </metadata>
  <style>...</style>
  <defs>...</defs>                       <!-- パターン定義。? 不定値ハッチ用 (省略可) -->
  <g class="row-backgrounds">...</g>     <!-- @bgcolor0/1 と @bg: Line.bbox 全体 -->
  <g class="rulers">...</g>              <!-- @ruler 由来の薄い縦線 (背景ガイド) -->
  <g class="highlights">...</g>          <!-- [...] 区間 -->
  <g class="dontcares">...</g>           <!-- ? の塗り (デフォルトはハッチ) -->
  <g class="signal-labels">...</g>       <!-- 信号名 / @signal(overline) -->
  <g class="waveforms">...</g>           <!-- 波形 polyline -->
  <g class="edge-marks">...</g>          <!-- @clock(...) 由来の三角形マーカー (`<polygon>`) -->
  <g class="guides">...</g>              <!-- | 縦線 -->
  <g class="titles">...</g>              <!-- @title -->
  <g class="arrows">...</g>              <!-- @-> 由来のみ (`Annotations.arrows` には clock マーカーは混入しない、types.md §6.6) -->
  <g class="overlays">...</g>            <!-- % 行 -->
</svg>
```

レイヤ順は z-order を兼ねる (上から先に描画 = 後の要素ほど前面)。

### ルート `width` / `height` / `viewBox` と `@scale`

`<svg>` 開始タグの `width` / `height` / `viewBox` 属性は以下で決定する。

- 内部チャート寸法 (`chart_outer_width` / `chart_outer_height`) は `Line.bbox` を積み上げた結果に `page_margin` を加算した値で、`@scale` の影響を受けない。
- `<svg width="W" height="H">` の数値 `W` / `H` は `chart_outer_width × @scale` / `chart_outer_height × @scale`。`@scale 1.0` (デフォルト) のときは内部寸法と一致する。
- **`viewBox="0 0 chart_outer_width chart_outer_height"` を常に併記する** (内部座標系の寸法、つまり scale 適用前の値)。`@scale` の値や 1.0 か否かに関わらず常に出力する。これにより SVG ビューポート拡大が確実に働き、`width × height` への等倍表示が保証される。
- 内部座標 (`<polyline>` の点列、`<rect>`・`<text>` の位置、アンカー、`viewBox` の数値) は **`@scale` 非適用** で、1 倍の論理座標のまま出力する。
- 結果として、`@scale 2.0` で書き出すと SVG 表示サイズが 2 倍に拡大され (内部座標系は同一)、ブラウザや SVG ビューワは内部座標を `width × height` に等比でフィットして描画する。`viewBox` を省略すると拡大表示が成立せずキャンバスだけ大きくなる挙動になるため必ず出力する。

### 空レイヤーの省略

中身を 1 つも持たない `<g class="...">` レイヤーは **出力しない** (空タグ
`<g class="..."></g>` も出さない)。`<defs>` の空時省略 (§「`<defs>` (パターン定義)」
の「出力省略条件」) と同じ扱いで、全 10 レイヤーに一律適用する。

- 例: `@->` が 1 つも宣言されないチャートでは `<g class="arrows">` は出力されない。
- 例: `?` を含まないチャートでは `<g class="dontcares">` も `<defs>` も出力されない。
- 例: `%` 注釈が無いチャートでは `<g class="overlays">` を出力しない。

レイヤーが出力される場合の順序は上の擬似 XML の通り (z-order を兼ねる)。レイヤー
省略によって順序関係が変わることはない (先頭/末尾どちらが消えても残りの相対順序は
不変)。

## 入力データの埋め込み

`<metadata>` 内の `<tchart:source>` カスタム要素に元 TCML を XML エスケープして格納。

- 名前空間 URI: `http://tchart-rust/1.0`
- エスケープ対象: `<` → `&lt;`、`>` → `&gt;`、`&` → `&amp;`
- CDATA は使用しない (`]]>` 問題回避)。

PNG 出力時は `iTXt` チャンク (`tchart-source` キー) に同じ TCML を格納する。

## `<defs>` (パターン定義)

`<svg>` 直下、`<style>` の直後に `<defs>` 要素を出力し、以下の SVG パターンを定義する。

### `dontcare-hatch-N` — 不定値 (`?`) のハッチ (色ごと採番)

`<defs>` には、チャート内で実際に使われる **ハッチ線色ごとに 1 つ**の `<pattern>` を出力する。同色は ID を共有する。ID は **`dontcare-hatch-1` から始まる連番**で、初出順に採番する (1 オリジン)。

```xml
<defs>
  <pattern id="dontcare-hatch-1" patternUnits="userSpaceOnUse"
           width="<DEFAULT_DONTCARE_HATCH_TILE_PX>"
           height="<DEFAULT_DONTCARE_HATCH_TILE_PX>"
           patternTransform="rotate(45)">
    <line x1="0" y1="0"
          x2="0" y2="<DEFAULT_DONTCARE_HATCH_TILE_PX>"
          stroke="<color_1>"
          stroke-width="<DEFAULT_DONTCARE_HATCH_STROKE_WIDTH_PX>"/>
  </pattern>
  <pattern id="dontcare-hatch-2" ...>
    <line ... stroke="<color_2>" .../>
  </pattern>
  ...
</defs>
```

- ID は連番 `dontcare-hatch-1`, `dontcare-hatch-2`, ... `dontcare-hatch` (番号なし) は使わない。
- `patternUnits="userSpaceOnUse"` 固定 (利用側 bbox 依存にしない、ズームしても縞間隔が一定)。
- `patternTransform="rotate(45)"` 固定 — `////` 方向 (右上がり) のハッチ。
- 縞間隔 (タイル一辺) / 縞太さは `defaults.rs` の定数 (`DEFAULT_DONTCARE_HATCH_TILE_PX`, `DEFAULT_DONTCARE_HATCH_STROKE_WIDTH_PX`) でハードコードする。
- 縞太さはタイル一辺より十分小さくし (細線)、隙間を大きく取って下地の波形が透けて見えるようにする。
- 縞色 `<line stroke="...">` は **行単位**で決まる。各行の `LevelRun(DontCareAlong*)` は、その行構築時点で有効な `@dontcare_color` の値 (未指定なら `DEFAULT_DONTCARE_HATCH_STROKE_COLOR`) を使う。`@dontcare_color` をチャート途中で書き換えれば、それ以降の行のハッチ色が切り替わる。生成時にパターンの `<line stroke>` に直接埋め込み、`currentColor` のような継承は使わない (resvg / 一部レンダラがパターン参照経由の `currentColor` を解決しないため)。

### 出力省略条件

チャート内に `LevelRun(DontCareAlong*)` が **1 つも無い場合** は `<defs>` 自体を出力しない (空の `<defs></defs>` も出さない)。

## 行ジオメトリと背景

すべての行 (`Line`) は `Line.bbox` が縦に隙間なく積み上がる ([`types.md`](types.md) §3.1「行ジオメトリ規約 (対称ギャップ)」)。

### `row-backgrounds` (`@bgcolor0` / `@bgcolor1` / `@bg`)

行ごとに 1 つの背景矩形 (`Line.bbox` 全体) を出力する。`@bg` (ローカル) と
`@bgcolor0/1` (グローバル) の優先関係は以下:

1. **`Line.background == Some(color)` (`@bg` 由来)**: `color` で塗る。`bgcolor0/1`
   は重ね描きしない。`color` が `none` の場合は何も出力しない。
2. **`Line.background == None`**: 偶奇ストライプ規則を適用する。
   - 偶奇インデックスは **`SignalRow` のみカウント** (`Skip` / `Title` は除外)。
   - 偶数信号行 (idx 0, 2, ...): `@bgcolor0` で塗る。
   - 奇数信号行 (idx 1, 3, ...): `@bgcolor1` で塗る。
   - `Skip` / `Title` 行は塗らない (`@bg` 指定が無い限り背景無し)。
   - 色が `none` の場合は `<rect>` を出力しない。

`Line.bbox` 全体 (`h_space/2` の上下余白を含む) を塗ることで隣接行の塗り分けが
隙間なく連続する。

**矩形の幅**: `<rect width="...">` は `Line.bbox.size.width` をそのまま使う。
`Line.bbox.size.width` は **チャート全体で全行同一値** (= `chart_inner_width`) である
ことが `types.md` §6.2 で保証されているため、この描画ルールにより
**短い信号行・Skip 行・Title 行も含め row-background はすべて同じ幅で揃う**。
レンダラ側で chart 幅を別途計算したり `max(bbox.size.width)` を取り直したりしてはならない。

### `rulers` (`@ruler` 由来の背景縦線)

`@ruler` の詳細仕様 (寄付モデル・last-wins マージ) は `tcml-format.md` §「`@ruler` の詳細」を参照。本節は **SVG 出力形式のみ**を規定する。

各行は寄付情報 `Vec<RulerContribution { x: Px, color: Color }>` をサイドカーとして持つ。レンダー時:

1. 全行を **上から順** に走査し、`(x, color)` をマップに積む (キー: 量子化済 x、値: color)。
2. 同じ x への寄付は **後の値で上書き** (last-wins per x)。
3. 最終マップを **x 昇順** で走査し、各 x 位置に 1 本の `<line>` を `<g class="rulers">` に出力する。
4. 線の y 範囲は **チャート内部高さ全体** (= `Σ Line.bbox.size.height`)。`page_margin` の外側余白には侵入しない。

SVG 出力 (1 本ぶん):

```xml
<line x1="<x>" y1="0" x2="<x>" y2="<chart_inner_height>"
      stroke="<color>" stroke-width="0.5" stroke-dasharray="3 5"/>
```

- `stroke-width` と `stroke-dasharray` の値は **固定** (タスク決定値、将来パラメータ化する場合は別の `@ruler_style` 等を新設する)。
- `stroke` 属性に色を直接書き、CSS 経由ではない (他の線種と同じ流儀)。
- 1 本も寄付がなければ `<g class="rulers">` 自体を省略する (§「空レイヤーの省略」)。

**z-order**: `row-backgrounds` の直後、`highlights` の直前 (= 行背景より前面、ハイライトより背面)。波形・矢印・clock マーカーよりも完全に背面で、ガイド線 (`|`) よりも背面。

**x 量子化**: 寄付された x 値は浮動小数なので、同一性判定でハッシュキーにする際は `Px` のビット表現ではなく整数 mil-pixel (`(x * 1000).round() as i64` 等) でキー化することを推奨 (実装裁量。仕様としては「`x` 同一の寄付は 1 本に統合される」ことだけが保証される)。

## 信号名 (`signal-labels`)

`SignalRow.label_box` に従って配置。

- フォント: `LabelStyle.font`
- 色: `LabelStyle.color`
- アライン: `LabelStyle.align` (Start / Middle / End)
- 複数行信号名: `<text>` + `<tspan x dy>` で各行を出力。行高は `LabelStyle.font.line_height`。

### 信号名上線 (`@signal(overline)`)

`SignalDecorations.name_overline == true` の場合、`signal-labels` レイヤに **`<text>` とは独立した `<line>` 要素** を出力して上線を引く。**`text-decoration="overline"` 属性は使わない** (フォント・レンダラ依存で文字に被るため)。

- **対象**: 信号名の **最上行のテキストの上にのみ** 1 本だけ引く。複数行信号名でも 2 行目以降には引かない (負論理の慣例として、名前全体に 1 本の帯)。
- **位置 (y)**: 最上行テキストの **cap-top から `overline_gap` だけ上**。
  - cap_top = `text_baseline - cap_height`
  - cap_height = `LabelStyle.font.size * cap_height_ratio` (`cap_height_ratio` はフォントメトリクス由来。実装側でフォントから取得できなければ `defaults.rs::DEFAULT_CAP_HEIGHT_RATIO = 0.7` を使用)
  - `<line y1="cap_top - overline_gap" y2="cap_top - overline_gap"/>`
- **位置 (x 範囲)**: 信号名の **全行のうち最長行のテキスト実幅と一致** (label_box の幅ではない)。
  - 行ごとの実幅をフォントメトリクスから測り、その最大値を採用する。
  - 線は最上行の上に 1 本だけ引かれるが、その水平方向の長さは「下の行が長ければそちらの長さに合わせる」(負論理の慣例として名前全体を覆う 1 本の帯)。
  - 開始/終了 x はテキストの `text-anchor` (Start/Middle/End) に応じて決まる。
- **太さ**: `overline_thickness` (`defaults.rs::DEFAULT_OVERLINE_THICKNESS_PX = 1.0`)
- **色**: `LabelStyle.color` を継承 (専用色パラメータは無し)
- **SVG 出力例**:
  ```xml
  <line x1="<text_left>" y1="<cap_top - gap>"
        x2="<text_right>" y2="<cap_top - gap>"
        stroke="<label_color>" stroke-width="<overline_thickness>"/>
  ```
- **同レイヤ順**: その信号の `<text>` の **直前または直後**に出力 (順序は描画結果に影響しない、実装の都合で OK)。

## 波形 (`waveforms`)

`SignalRow.waveform.elements` を順に処理し、`<polyline>` を構築する。

### Polyline 蓄積器 (`PolyAccum`)

連続する単線・遷移の点列を 1 本の `<polyline>` に集約するため、レンダラは内部に蓄積器を持つ。

- `accum.push(point)`: 点を追加。
- `accum.flush()`: 現在の蓄積を `<polyline>` として出力し、蓄積をクリア。
- Bus は **上下 2 本の蓄積器** (top rail / bottom rail) を並行して持つ。
- DontCare 矩形 (`?`) や Gap (`:`) など「断絶」が来たら必ず `flush()` する。

### 要素ごとの描画契約

#### `LevelRun(Single)` — `Low` / `High` / `HiZ`

- 1 本の水平線。x: 開始位置 → 開始位置 + `LevelRun.width()`。`width` は `types.md` §6.4 の規則 (前要素が遷移なし → `units × step`、前要素が遷移 → `units × step - slant`)。y: レベル位置 (Low → y_low, High → y_high, HiZ → y_mid)。
- HiZ は `stroke-dasharray` で破線スタイル (詳細属性は `defaults.rs`)。
- HiZ は **独立 polyline** として出力 (連続する Low / High polyline と統合しない)。スタイルが違うため。

#### `LevelRun(Bus)` — `=`

- 上下 2 本の水平線。y: y_high (top rail) と y_low (bottom rail)。
- 内部にラベル (`<text>`) を中央に配置 (`LevelRun.label`)。
- ラベル文字列が空 (`""`) の場合は `<text>` 要素自体を出力しない (見えない空要素で DOM を汚さない)。空白のみのラベル (`" "` など) は通常通り出力する。
- 上下それぞれの蓄積器に push。

#### `LevelRun(DontCareAlong*)` — `?`

`<g class="dontcares">` レイヤに **塗り polygon** を出力 + `<g class="waveforms">` レイヤに **内部の水平線** (および隣接遷移の slant) を出力。

| Variant | 塗り y 範囲 | 内部水平線 | 左右辺が斜辺追従するか |
|---------|-------------|-----------|------------------------|
| `DontCareAlongLow`  | `y_high`〜`y_low` | y_low に 1 本 | する (下記 `DontCareAlongLow / High / Bus の塗り形状`) |
| `DontCareAlongHigh` | `y_high`〜`y_low` | y_high に 1 本 | する (同上) |
| `DontCareAlongHiZ`  | `y_high`〜`y_low` | y_mid に 1 本 (HiZ スタイル: 破線) | **しない** (常に矩形、下記 `DontCareAlongHiZ の塗り形状`) |
| `DontCareAlongBus`  | `y_high`〜`y_low` | y_high と y_low に 2 本 | する (同上) |

- DC polygon の上辺 y は **常に y_high**、下辺 y は **常に y_low**。`signal_box` 全高にはみ出してはいけない (旧仕様 「`signal_box` 全高」は撤廃)。
- `<polygon>` の `fill` は **常に** その行のハッチ色に対応する `url(#dontcare-hatch-N)` (`<defs>` のいずれかのパターンへの参照)。それ以外の属性は出力しない (アウトラインなし)。
- `@dontcare_color` の値は `<defs>` の対応する `<pattern>` 内 `<line stroke>` に焼き込まれる。同色は `<defs>` 内で 1 つのパターンに統合される (ID 共有)。
- `@dontcare_color` をチャート途中で書き直すと、それ以降の行のハッチ色が切り替わる (`<defs>` には新色のパターンが追加され、ID が割り当てられる)。
- チャートに `?` が 1 つでも存在すれば `<defs>` は常に出力する。
- 内部水平線は **直前の polyline と連結**する (Single の場合は単一蓄積器に push、Bus の場合は上下蓄積器に push)。
- DC 区間の左右垂直辺には固有の遷移 `slant` 幅は挿入されない。ただし **DC の隣接遷移 (SingleEdge / BusOpen / BusClose / BusCross / Pos-half / Neg-half) は `?` の有無に関わらず `@slant` どおりに描画される** (`?` がレベル列に存在しても 0 (垂直) に縮退してはならない)。

##### `DontCareAlongLow` / `DontCareAlongHigh` / `DontCareAlongBus` の塗り形状

塗り範囲は **前後の波形要素が `?` の左右に張る境界線で囲まれた範囲**。出力は `<polygon>` (4〜6 点)。上辺 y_high、下辺 y_low。左右辺は **隣接遷移の斜辺 / 中継頂点** に追従する。

- DC の左端 x を `x_a`、右端 x を `x_b`、`@slant` を `s` とする。`y_h` / `y_mid` / `y_l` は signal_box の上端 / 中央 / 下端。
- 隣接遷移は次のように分類:
  - **Single full slant** — Low/High 間の遷移 (`_~` Pos、`~_` Neg)。Single ↔ Bus 開閉も含む (`_=`, `~=`, `=_`, `=~`)。
  - **Half slant** — HiZ を経由する Low/High ↔ HiZ 遷移 (`_-`, `-_`, `~-`, `-~`、および BusOpen-from-HiZ `-=`, BusClose-to-HiZ `=-`)。HiZ 側で **`y_mid` を中継頂点**として polygon に追加する (1 個)。
  - **BusCross** — `=X` / `X=`。X の cross 中点 `(x_cross + s/2, y_mid)` が polygon の wedge 頂点 (`>▲▲<` 形)。
  - **垂直境界** — 信号行頭 / 行末 / Gap / 同レベル continue (`=?=` の `=`、`_?_` の `_` 等)。

- 各 DC variant の **左右境界 4 通り (start / Pos 系 / half / BusClose 系)** × **4 通り (end / Neg 系 / half / BusOpen 系)** = 16 組合せをすべて網羅すること。サボって「同じ斜辺」みたいな手抜き禁止。具体ケース表は `docs/tests/svg-rendering.feature.md` §「`#1`」参照。

- 内部水平線 (`y_low` / `y_high` / Bus 上下) は **DC 区間の x 範囲 (`x_a` → `x_b`)** で出力。polygon は左右辺が斜辺に伸びても、内部水平線は `x_a`〜`x_b` 区間に固定。
- DC の隣接遷移自体 (Pos/Neg/Pos-half/Neg-half/BusOpen/BusClose/BusCross) は **DC 区間外** に独立した `Transition` 要素として配置される。DC polygon は遷移の slant 範囲だけを「塗り潰しの範囲」として取り込むだけで、遷移線の描画責務は持たない。

##### `DontCareAlongHiZ` の塗り形状

DC-HiZ は **常に矩形** (4 点、`(x_a, y_h)`, `(x_b, y_h)`, `(x_b, y_l)`, `(x_a, y_l)`)。隣接遷移の斜辺には追従しない。

- 隣接が `=` (BusOpen-from-HiZ / BusClose-to-HiZ) や Pos-half / Neg-half でも、polygon は矩形のまま。遷移の `slant` は **波形 polyline 側で別途維持される** (slant=10 等)。
- `-?-` (HiZ - `?` - HiZ) は **DC-HiZ,1** として扱う (1-cell の DC-HiZ 矩形)。範囲は同 step 数の `==` と同じ。`?` の 0 幅マーカーが両側の `-` を 1 つの DC-HiZ 区間に統合する。

#### `Transition` — `_~` / `~=` / `=_` / `X` 等

`TransitionKind` ごとに描画契約を網羅する。
**いかなる場合も独立 `<line>` 要素は使わない**。蓄積器に push して隣接 polyline と連結する。
共有辺 (slant 後に同 y で続く線) も明示的に水平ブリッジを描く。

##### `SingleEdge` — Single ↔ Single

例: `_~`, `~_`, `~-`, `-_`, `_-`, `-~`

- 線本数: **1**
- 蓄積器: 1 つ (Single 用)
- 描画: `(x, y_from)` → `(x + slant, y_to)` を蓄積器に push。
- `from` の y は `from.into_shape()` から: Low → y_low, High → y_high, HiZ → y_mid。`to` も同様。

##### `BusOpen` — Single → Double

例: `_=`, `~=`, `-=`

- 線本数: **2** (上端 + 下端)
- 蓄積器: 2 つ (Bus top rail, Bus bottom rail)
- 描画:

  | 線 | 始点 | 終点 |
  |----|------|------|
  | 上端 (top rail) | `(x, y_from)` | `(x + slant, y_high)` |
  | 下端 (bottom rail) | `(x, y_from)` | `(x + slant, y_low)` |

- `y_from` = Low/High/HiZ のいずれか。
- 共有辺の場合 (例: `_=` で `y_from == y_low`)、下端は `(x, y_low) → (x + slant, y_low)` という水平線になる。これも明示的に描画する。

##### `BusClose` — Double → Single

例: `=_`, `=~`, `=-`

- 線本数: **2** (上端から、下端から)
- 描画:

  | 線 | 始点 | 終点 |
  |----|------|------|
  | 上端から | `(x, y_high)` | `(x + slant, y_to)` |
  | 下端から | `(x, y_low)` | `(x + slant, y_to)` |

##### `BusCross` — Double ↔ Double (`X`)

`Transition(BusCross)` は **cross 部のみ** を描画する (幅 `slant`)。X の body (Bus 1 単位、新値) は後続 `LevelRun(Bus / DontCareAlongBus, 1)` として別要素で描画され、その width は cross 直後では `step - slant` (X 全体で 1 step を消費)。X が信号行頭等で cross 省略の場合は `Transition(BusCross)` 自体が生成されず、body の `LevelRun` のみが幅 `step` で残る。

cross 開始 x を `x`、終点 x を `x_c = x + slant` とする。

- cross 区間 `[x, x_c]` で 2 本の線が交差:

  | 線 | 始点 | 終点 | 接続先 |
  |----|------|------|--------|
  | 線 A | `(x, y_high)` | `(x_c, y_low)` | 前 Bus 上端 → 後 Bus 下端 |
  | 線 B | `(x, y_low)` | `(x_c, y_high)` | 前 Bus 下端 → 後 Bus 上端 |

- 中点 `(x + slant/2, y_mid)` で 2 本が交わる。
- `slant = 0` のとき cross は縦線 1 本に縮退 (上下レールの値変化点)。
- cross 後の body 描画は後続 `LevelRun(Bus / DontCareAlongBus, 1)` の責務 (通常の `LevelRun(Bus)` と同じく上下レール水平を描く)。
- `from` / `to` が `DontCareAlongBus` の場合も同じ描画 (Bus 上下レールと DontCare 内部水平線の接続点が一致するため)。

#### `Gap` — `:`

- 全蓄積器を **`flush()`** する。
- 描画は何もしない (空白)。
- x は 1 step 進む (蓄積器は flush 後の新規 polyline 開始位置として扱う)。
- Gap 通過後の最初の LevelRun から新しい polyline が始まる。

#### `Guide` — `|`

- `<g class="guides">` レイヤに `<line>` を出力。
- y 範囲: `|` を書いた信号行 (起点行) から、上方向で最も近い `Title` 行の
  bbox 下端まで、下方向で最も近い `Title` 行の bbox 上端までを縦断する。
  起点行の上方/下方に `Title` が無い側はチャート上端 / 下端から
  `page_margin/2` 分はみ出す。`Title` 行は貫通しない。
  - 上端: 起点行より上で最も近い `Title` の bbox 下端。
    無ければ `(チャート最上行の bbox.origin.y) - page_margin/2`。
  - 下端: 起点行より下で最も近い `Title` の bbox 上端。
    無ければ `(チャート最下行の bbox.origin.y + bbox.size.height) + page_margin/2`。
- スタイル: `GuideStyle.color` / `GuideStyle.width`。
- 波形蓄積器には push しない (波形と独立)。

#### `HighlightStart` / `HighlightEnd` — `[` `]`

- `<g class="highlights">` レイヤに `<rect>` を出力。
- x 範囲: 開始位置 → 終了位置。
- y 範囲: `[` を書いた信号行 (起点行) を起点に、`Guide` と同一の
  Title 境界バウンド規則で算出する。`Title` は貫通しない。Title が
  無い側はチャート上端 / 下端から `page_margin/2` はみ出す。
- スタイル: `highlight_style` (`SvgAttrList`)。
- 波形蓄積器には影響しない。

#### `Anchor`

- 描画は **何もしない** (0 幅マーカー)。
- `AnchorRegistry` で座標を確定するためだけに使われる。

#### `Text` — レベル文字列中のテキスト文字

- `<g class="waveforms">` 内に `<text>` 要素を出力。波形蓄積器 (`PolyAccum`) には push しない (波形線と独立)。
- `tcml-format.md` §「レベル文字列中のテキスト文字」で規定された所属区間 (連続する同一レベル区間) ごとに 1 つの `<text>` を出力する。同区間内の複数テキスト断片は parser 段階で空白結合済みの 1 文字列として扱う。
- 描画位置:
  - `x`: 所属区間の中点 (`region.origin.x + region.width / 2`)。`text-anchor="middle"`。
  - `y`: 区間の信号行に対応する垂直中央 (信号 box の中央 y)。`dominant-baseline="middle"` または `alignment-baseline="central"` 相当の指定。
- スタイル: `font-family` は信号名・タイトル等と同様の規則で `<text>` の **属性** として直接出力 (CSS インジェクション回避、`font-family` のような **ユーザー値**を含む CSS 値は `<style>` ではなく `<text>` 属性に出すという既存規則に従う)。`font-size` も同様。色は `signal_color` に従う。
- はみ出し: テキスト幅 > 区間幅 でも clip しない。区間中央を基準に左右へ自然にはみ出す。文字列内容は XML エスケープして出力。
- 0 幅要素 (波形 x 進行に加算しない、`types.md` §6.4)。

## 矢印 (`arrows`)

`Annotations.arrows` をすべて描画。

### 線

- 始点 / 終点: `Arrow.from.at`、`Arrow.to.at` (層解決後の `ResolvedAnchor.at`)。
- 線種: `LineDashStyle`
  - `Solid`: そのまま `<line>` または `<polyline>`
  - `Dashed`: `stroke-dasharray="<dash>"` (定数: `defaults.rs`)
  - `Dotted`: `stroke-dasharray="<dot>"`
- 太さ: `ArrowStyle.width`
- 色: `ArrowStyle.color`

### 矢印頭

- `ArrowHead::EndOnly`: 終点のみに三角形 (path で実装、向きは `from → to` ベクトル)。
- `ArrowHead::BothEnds`: 両端に三角形。
- `ArrowHead::None`: 矢印頭なし、線のみ。
- 矢印頭のサイズ・形状は `defaults.rs` の定数 (`DEFAULT_ARROW_HEAD_SIZE_PX` 等)。

### ラベル

- `Arrow.label` が `Some` の場合、線分の中点に `<text>` を配置。
- 配置: 線の上 (進行方向に対して左側、または線が水平なら上側)。詳細は `defaults.rs` の `ARROW_LABEL_OFFSET_PX`。
- **白縁取り**: ラベル `<text>` が矢印線や波形線と被って読みにくくなるのを防ぐため、`paint-order="stroke fill"`、`stroke="<DEFAULT_ARROW_LABEL_OUTLINE_COLOR>"`、`stroke-width="<DEFAULT_ARROW_LABEL_OUTLINE_WIDTH_PX>"` 属性を必ず付与する。色 / 幅は `defaults.rs` の定数 (`DEFAULT_ARROW_LABEL_OUTLINE_COLOR = "#ffffff"`, `DEFAULT_ARROW_LABEL_OUTLINE_WIDTH_PX = 2.0`) でハードコードし、ユーザーカスタマイズ属性は提供しない。`paint-order` により stroke が先に描かれ fill が上書きするため、文字本体は元の色のまま縁取りだけが下のオブジェクトを隠す。`stroke-linejoin="round"` を併記して角の尖りを抑える。

## クロックエッジマーカー (`edge-marks`)

`SignalRow.edge_marks: Vec<EdgeMark>` に格納された **三角形マーカー** を SVG `<polygon>` として描画する。`@->` 矢印 (Arrow) とは **完全に別系統**で、`Annotations.arrows` には含まれない。

### レイヤ配置

専用レイヤ `<g class="edge-marks">` に出力する。z-order は `waveforms` の直後、`guides` の直前 (波形より前面、`@->` 矢印より背面)。`SignalRow` ごとに、その信号の `edge_marks` を順次 `<polygon>` として出力する。レイヤ全体で空 (どの信号も `edge_marks` を持たない) の場合は `<g class="edge-marks">` 自体を省略する (空レイヤー省略の一般ルール)。

### 1 つの三角形の幾何

`EdgeMark { line_start, line_end, style: ClockMarkStyle { position, height, width, color } }` から **3 頂点 (`apex`, `base_left`, `base_right`)** を以下で求める。

`style.width` / `style.height` はパース時に確定済の値 (= ユーザの明示指定があればその値、なければデフォルト解決後の値) であり、レンダラ側で再決定や再縮小は行わない。デフォルト解決時の step 連動縮小ルールは `tcml-format.md` §「`clockmark_width` の step 連動縮小」参照。

```
delta_x        = line_end.x - line_start.x
delta_y        = line_end.y - line_start.y
line_length    = sqrt(delta_x * delta_x + delta_y * delta_y)

line_direction.x      = delta_x / line_length          // 線方向の単位ベクトル X
line_direction.y      = delta_y / line_length          //   〃            Y

perpendicular_unit.x  = -line_direction.y              // 線に直交する単位ベクトル X
perpendicular_unit.y  =  line_direction.x              //   〃                 Y
                                                       // (時計回り 90° 回転)

apex_distance         = (line_length - height) * position + height
base_center_distance  = (line_length - height) * position

apex.x                = line_start.x + line_direction.x * apex_distance
apex.y                = line_start.y + line_direction.y * apex_distance

base_center.x         = line_start.x + line_direction.x * base_center_distance
base_center.y         = line_start.y + line_direction.y * base_center_distance

base_left.x           = base_center.x + perpendicular_unit.x * (width / 2)
base_left.y           = base_center.y + perpendicular_unit.y * (width / 2)

base_right.x          = base_center.x - perpendicular_unit.x * (width / 2)
base_right.y          = base_center.y - perpendicular_unit.y * (width / 2)
```

`apex_distance - base_center_distance == height` なので、頂点は底辺の中心から線方向に `height` 進んだ位置にある (= 三角形の「高さ」が線方向に一致)。

`position` の意味:
- `position = 0.0`: `base_center` が `line_start` に重なる (頂点は線根元から `height` 進んだ位置)。
- `position = 1.0`: `apex` が `line_end` に重なる (底辺中心は `line_end` から `height` 戻った位置)。
- `position = 0.5` (デフォルト): 線の中央付近 (`apex` と `base_center` がそれぞれ `line_length/2 ± height/2`)。

### 立ち上がり/立ち下がりでの `line_start` / `line_end`

clock 展開パスで以下のように設定される (再掲、`types.md` §4.3 step 5):

| `ClockEdge` | 対象遷移 | `line_start` | `line_end` |
|-------------|----------|--------------|-----------|
| `Pos`       | `Low → High` (`_~`) | `(x, y_low)` | `(x + slant, y_high)` |
| `Neg`       | `High → Low` (`~_`) | `(x, y_high)` | `(x + slant, y_low)` |

- 結果として `Pos` の三角形は **線方向 (右上向き)** に頂点を持ち、`Neg` の三角形は **線方向 (右下向き)** に頂点を持つ。
- `slant = 0` (垂直エッジ) の場合: `line_length = |y_high - y_low|`、`line_direction = (0, ±1)`、`perpendicular_unit = (∓1, 0)`。底辺は水平に広がる。式は破綻しない。
- `line_length < height` の場合 (極端に短いエッジ + 大きい三角形): `(line_length - height)` が負になり頂点位置が逆向きになるため、レンダラは `height = min(height, line_length)` でクランプしてから計算する (実装側の安全策、spec ではクランプ後の値で上記式を適用すると規定)。

### SVG 出力

```xml
<polygon points="apex.x,apex.y base_left.x,base_left.y base_right.x,base_right.y"
         fill="<color>" stroke="none"/>
```

- 頂点座標は `Px` を 3 桁程度の浮動小数点で出力 (既存 polyline と同じフォーマット)。
- `fill` には `ClockMarkStyle.color` の `Color::to_css_string()` を使う。
- `stroke="none"` 固定 (枠線なし)。

### `ClockEdge::None` または `edge_marks` が空の場合

`<polygon>` を一切出力しない。

## オーバーレイ (`%` 行)

`Annotations.overlays` を `<text>` として出力。

- 座標: `TextOverlay.at` (チャート左上原点、Px)。
- フォント: `ChartStyle.canvas.font` (デフォルト) または個別指定。

## タイトル (`titles`)

`Line` のうち `LineContent::Title` のもの。

- フォント: `TitleStyle.font`、色: `TitleStyle.color`、アライン: `TitleStyle.align` (デフォルト `HorizontalAlign::Center`、`defaults.rs::DEFAULT_TITLE_ALIGN`)。
- 複数行テキストは `<tspan>` で改行 (信号名と同じ規則、行高 = `TitleStyle.font.line_height`)。
- y 座標: `Line.bbox` の縦中央 (`bbox.origin.y + bbox.size.height / 2`、ベースライン補正は信号名と同じ規則)。複数行は `<tspan dy>` で上下均等分配。
- x 座標と `text-anchor` は `align` に応じて以下に決まる:

| `TitleStyle.align` | x 座標 | `text-anchor` |
|--------------------|--------|---------------|
| `HorizontalAlign::Left`   | `Line.bbox.origin.x + page_margin` | `start` |
| `HorizontalAlign::Center` | `Line.bbox.origin.x + Line.bbox.size.width / 2` | `middle` |
| `HorizontalAlign::Right`  | `Line.bbox.origin.x + Line.bbox.size.width - page_margin` | `end` |

**注**: `Line.bbox.size.width` は `types.md` §6.2 によりチャート全行で同一値 (= `chart_inner_width`) のため、`Center` の場合 `<text>` はチャート全幅の中央に揃う。`@bgcolor` や信号行の波形長に左右されない。

## エスケープと NewType の流れ

| データ | 型 | 出力経路 |
|--------|----|---------|
| 信号名 | `SignalName` | `<text>` 内に `escaped` |
| ラベル | `UserText` | `<text>` / `<tspan>` 内に `escaped` |
| 色 | `Color` | `Color::to_css_string()` を `lit` 経由 (`#rgb` / 名前付きはホワイトリスト確認済み) |
| `highlight_style` | `SvgAttrList` | `safe_presentation_attrs` でホワイトリスト経由 |
| `@dontcare_color` 値 | `Color` | `Color::to_css_string()` を `lit` 経由 (パターン `<line stroke=>` に流す) |
| TCML ソース | `TcmlSource` | `<tchart:source>` 内に `escaped` |

`Color::to_css_string()` は内部で **CSS 名前付き色のホワイトリスト** + `#rrggbb` / `#rrggbbaa` の固定フォーマット出力のみ行うため、`lit` 経由で安全に流せる (出力時点で固定文字列とみなせる)。CSS 名前付き色 (`red` / `blue` 等) でパースされた `Color` は、その入力名 (ホワイトリスト内の小文字正規化済み文字列) をそのまま再出力する。`#rrggbb` 等のヘックス入力は対応する `#rrggbb` (常に小文字 6 桁) を出力する。これによりユーザーが `mark_color=red` と書いた色は SVG にも `fill="red"` として現れる。

## CSS 共有スタイル (`<style>`)

```css
.waveforms polyline { fill: none; }
.guides line { ... }
.highlights rect { ... }
text { font-family: <FontFamily>; }
```

- `font-family` のような **ユーザー値**を含む CSS 値は `<style>` ではなく **`<text>` の属性**として直接出力する (CSS インジェクション回避)。

## 描画順 (z-order)

1. `<style>`, `<metadata>`
2. `row-backgrounds`
3. `rulers` (`@ruler` 由来の薄い背景縦線)
4. `highlights`
5. `dontcares`
6. `signal-labels`
7. `waveforms` (波形 polyline)
8. `edge-marks` (clock エッジマーカー `<polygon>`)
9. `guides`
10. `titles`
11. `arrows`
12. `overlays`

矢印は波形より前面 (描画注釈のため)。`@->` 矢印 (`arrows` レイヤ) と clock エッジマーカー (`edge-marks` レイヤ) が重なる場合、`@->` 矢印が常に前面 (上) に描かれる — z-order 表通りの結果。

## 不変条件

- `TransitionKind` の `match` は **網羅的**。`_ =>` パターン禁止 (新 kind 追加時にコンパイル失敗させる)。
- 連結すべき polyline 間で独立 `<line>` を出力してはならない。
- Gap で全蓄積器を flush する義務。
- ユーザー値の出力は `escaped` / `user_attr` / `safe_presentation_attrs` 経由のみ。`lit` に NewType 値を直接渡すのは型エラー。
