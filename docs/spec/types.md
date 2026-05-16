# 実装構造とアルゴリズム概要

> 本ドキュメントは tchart-core の **実装構造** と **主要アルゴリズム** の概要を扱う。
>
> 詳細な型定義は実装ソース (`tchart-core/src/`) が正本。マジックナンバー・デフォルトは [`tchart-core/src/defaults/`](../../tchart-core/src/defaults/mod.rs) に集約 (詳細は実装参照)。
>
> TCML 表記は [`tcml-format.md`](tcml-format.md)、SVG 描画契約は [`svg-rendering.md`](svg-rendering.md)、クレート構成・データフロー・各層の責務は [`architecture.md`](architecture.md) を参照。

## 1. 設計原則

1. **行 (Line) を中核に据える**。信号 (Signal) はあくまで行の一形態。
2. **単位 (lh / px) を型で分離**。レイアウト/レンダラ層では解決済みの `Px` のみを扱う。
3. **NewType でバリデーションを強制**。生 `String` を構造体フィールドに置かない。
4. **構造体のフィールドはサブ構造体にまとめる**。1 構造体あたり 5 フィールド程度を目安。
5. **マジックナンバー禁止**。デフォルト値は `defaults/` に集約。
6. **ドメインごとにファイル/ディレクトリを分割**。`types.rs` のような雑な集約ファイルは作らない。

## 2. モジュール構成 (`tchart-core/src/`)

| モジュール | 主な型 |
|------------|--------|
| `units` | `Length` (Lh / Px), `Px` |
| `geometry` | `Point`, `Size`, `Rect` |
| `color` | `Color`, `ColorError` |
| `text` | `SignalName`, `UserText`, `FontFamily`, `FontSpec` |
| `style` | `ChartStyle`, `CanvasStyle`, `BackgroundStyle`, `LayoutParams`, `SignalStyle`, `LabelStyle`, `TitleStyle`, `SvgAttrList`, `HorizontalAlign`, `DefaultRowStyles` |
| `line` | `Line`, `LineContent`, `SignalRow`, `SkipRow`, `TitleRow`, `SignalGeometry`, `SignalDecorations`, `Waveform`, `WaveformElement`, `LevelRun`, `SignalLevel`, `LevelShape`, `Transition`, `TransitionKind`, `EdgeMark` |
| `clock` | `ClockSpec`, `ClockEdge`, `ClockPulse`, `ClockPhase`, `ClockMarkStyle` |
| `anchor` | `AnchorId`, `AnchorName`, `AnchorRegistry`, `ResolvedAnchor`, `AnchorSource` |
| `arrow` | `Arrow`, `ArrowEnd`, `ArrowStyle`, `LineDashStyle`, `ArrowHead` |
| `document` | `ChartDocument`, `Annotations`, `TextOverlay`, `TcmlSource` |
| `defaults` | デフォルト定数・色・フォント名集約 |
| `errors` | `ParseError`, `ColorError`, `NameError`, `TextError`, ... |
| `parser` | TCML テキスト → `ChartDocument` (raw) |
| `layout` | 行積み上げ + 信号ジオメトリ確定 + アンカー座標確定 |
| `svg` | `ChartDocument` → SVG 文字列 |

### 2.1 NewType による信頼境界

ユーザー入力に由来するすべての文字列は用途別 NewType を経由する。検証は **NewType の `parse` / `try_from` 内**で行い、後段で「文字列のまま検証関数を通す」設計は禁止。

| 型 | 検証 |
|----|------|
| `SignalName` | UTF-8、`\n` のみ許可、その他制御文字禁止、空不可 |
| `UserText` | 制御文字 (改行 `\t` を除く) 禁止 |
| `FontFamily` | 制御文字禁止、カンマ区切り複数指定対応 |
| `Color` | `#rgb` / `#rrggbb` / `#rrggbbaa` / CSS 名前付き / `none` |
| `AnchorName` | `[A-Za-z0-9_][A-Za-z0-9_-]*` (純数字 `@{1}` も許容、ただし numbered anchor `@1` とは別名前空間) |

`ChartDocument` 出力以降で `String` をユーザー値として持つフィールドは存在しない。SVG 出力レイヤーは NewType だけを引数に取るエスケープ API を提供する。

## 3. 行モデル概要

`Line` は「縦に積み上がる単位」のみを表す。ジオメトリ内訳は `LineContent` の各バリアント固有:

- **`Skip`** (`@skip`): 空白行。`Line.bbox` のみ。`label_box` / `signal_box` を持たない。`@skip(0)` は無視 (`SkipRow` を生成しない)。
- **`Title`** (`@title`): タイトル行。`Line.bbox` のみ。テキストは `bbox` 全体を使い、`align` に従う。複数回出現可。
- **`Signal`**: 信号行。`SignalRow` 内に `SignalGeometry { label_box, signal_box }` を持つ。

### 3.1 行ジオメトリ規約 (対称ギャップ)

`signal_box.size.height = waveform_height = canvas.line_height` で **チャート全体で固定**。複数行ラベルが来ても信号体は引き伸ばさず、`Line.bbox.size.height` をラベル分だけ拡大して `signal_box` を **垂直中央配置** する。

```
Line.bbox の縦内訳:
  ┌──────────────────────────────────┐
  │ extra padding (上)               │   (bbox.h - signal_box.h) / 2
  │ ┌──────────────────────────────┐ │
  │ │ signal_box (waveform_height) │ │   高さ固定。中央配置。
  │ └──────────────────────────────┘ │
  │ extra padding (下)               │
  └──────────────────────────────────┘

label_total_height     = name.lines().count() * line_height
Line.bbox.size.height  = max(waveform_height, label_total_height) + h_space
signal_box.origin.y    = (bbox.size.height - waveform_height) / 2
label_box.origin.y     = (bbox.size.height - label_total_height) / 2
```

`h_space` は隣接行を区切る最小余白で、`signal_box` の上下に必ず `gap/2` ずつ対称配分する (詳細は §6.1)。

### 3.2 Waveform 要素

```
WaveformElement = Level(LevelRun) | Transition | Gap | Guide
                | HighlightStart | HighlightEnd | Anchor(AnchorId)
```

- **`LevelRun`**: `_` `~` `-` `=` `?` の連続区間。`SignalLevel` ∈ {`Low`, `High`, `HiZ`, `Bus`, `DontCareAlong{Low, High, HiZ, Bus}`}。`level` から `LevelShape` (Single/Double/FillSingle/FillDouble) が一意に決まる。
- **`Transition`**: レベル間遷移。`TransitionKind` ∈ {`SingleEdge` (1本線), `BusOpen` (1→2本), `BusClose` (2→1本), `BusCross` (Xクロス2本)}。
- **`Gap`** (`:`): 1 単位の空白 (= `step` 幅)。連続性を切るマーカー。
- **`Guide`** (`|`), **`HighlightStart`** (`[`), **`HighlightEnd`** (`]`): 透過要素 (幅 0)。
- **`Anchor`** (`@{name}` / `@N`): 矢印の始終点指定用 0 幅マーカー。

各要素は明示的な `width(): Px` を持ち、レイアウトの x 進行は `x_after = x_before + element.width()` の単一ルールで行う。

## 4. 主要アルゴリズム

### 4.1 DontCare (`?`) 解決 — parser 第 2 パス

`?` の `SignalLevel` (線位置 = 下/上/中央/bus 包絡) は **直前のアンカー** から決定する。アンカーは「解決済み非 `?` `LevelRun`」または「`X` 記号 (bus 遷移マーカー)」。

| 直前アンカー | `?` の解決 | 矩形内の線 |
|--------------|-----------|------------|
| `Low` (`_`)  | `DontCareAlongLow`  | 下端 |
| `High` (`~`) | `DontCareAlongHigh` | 上端 |
| `HiZ` (`-`)  | `DontCareAlongHiZ`  | 中央 |
| `Bus` (`=`)  | `DontCareAlongBus`  | bus 包絡 (上下 2 本) |
| `X` 記号     | `DontCareAlongBus`  | bus 包絡 |

**走査**: parser は 2 パスで処理する。

1. **第 1 パス**: 入力を順走査し各レベル文字を仮 `LevelRun` 化。`?` は `DontCarePending` で保持。
2. **第 2 パス**: 仮列を左→右に走査。`DontCarePending` は **後方走査** で直前アンカーを探し (透過要素 `:` `|` `[` `]` はスキップ) 上表に従って `DontCareAlong*` に書き換え。直前アンカー不在なら `ParseError::DontCareWithoutAnchor` (前方走査は行わない)。

**出力不変条件**: parser 出力の `Vec<WaveformElement>` に `DontCarePending` は残らない。layout / renderer 段階は `level` から線位置を一意に決定でき、後付け推論は行わない。

**境界 Transition**: `?` を含むあらゆる境界 (`_?` / `?~` / `=?` 等) では `Transition` 要素を **生成しない**。`LevelRun(DontCareAlong*)` がフラット垂直端で隣接 `LevelRun` に突き当たり、内部の水平線は `level` 自体に焼き込まれる。

**`X` の扱い**: `X` は常に **Bus body (1 単位)** を生成する。前後の level に応じて遷移を以下のように補完する:

| 前要素 | 生成する遷移 |
|-------|------------|
| Bus / DontCareAlongBus | `Transition(BusCross)` (cross 部のみ、幅 `slant`) |
| Low / High / HiZ | `Transition(BusOpen)` (Single → Bus) |
| なし (信号行頭) / Gap 直後 | なし (X body は単に Bus 1 単位として開く) |

| 後要素 | 生成する遷移 |
|-------|------------|
| Bus / DontCareAlongBus | なし (X body と後続 `LevelRun` を merge、または次の `Transition(BusCross)` で続く) |
| Low / High / HiZ | `Transition(BusClose)` (Bus → Single) |
| なし (信号末) / Gap | なし |

`X?` パターンでは X が後続 `?` に bus 文脈を伝える。X は常に valid (`DanglingBusTransition` エラーは廃止)。

### 4.2 `TransitionKind` 決定規則

parser は隣接 `LevelRun` の `(from.into_shape(), to.into_shape())` から `TransitionKind` を決定する:

| from \ to    | Single        | Double      | Fill*           |
|--------------|---------------|-------------|-----------------|
| **Single**   | `SingleEdge`  | `BusOpen`   | (Transition 無) |
| **Double**   | `BusClose`    | `BusCross`  | (Transition 無) |
| **Fill\***   | (無)          | (無)        | (同 level なら `LevelRun` マージ) |

- `(Single, Single)` で `from == to` (例: `~~`) は `LevelRun` マージで Transition 不要。
- `(Double, Double)` の値変化は `X` 明示が必須 (`Bus X Bus` → `BusCross` 1 つ)。
- 連続 `?` は直前レベルが同じなので同一 `DontCareAlong*` にマージ。

### 4.3 Clock 自動展開パス

通常パース後に実行する parser パス:

1. 全信号行を通常パースし、各信号の `total_units` を集計。
2. `chart_units = max(全信号の total_units, ClockRow を除く)` を確定。
3. 各 `clock` 装飾付き信号行 (`SignalDecorations.clock = Some(ClockSpec)`) について:
   - 本体波形が **空** なら `start` 相から `pulse` を `chart_units` 分繰り返し展開。
   - 本体波形が **非空** で `total_units < chart_units` なら、最後の状態 (Low/High) から `pulse` を繰り返し追記。
4. 展開で生成された `_~` / `~_` 遷移は `Transition(SingleEdge)` を通常通り挿入。
5. `edge` (`Pos` / `Neg` / `Both` / `None`) に応じた **三角形マーカー** (`EdgeMark`) を `SignalRow.edge_marks` に追加。
   - `Pos`: 各 `Low → High` 遷移について `EdgeMark { line_start: (x, y_low), line_end: (x + slant, y_high), style: spec.mark_style }`。
   - `Neg`: 各 `High → Low` 遷移について上下反転版。
   - `Both`: 上記両方。`None`: 生成しない。
   - `slant = 0` (垂直エッジ) でも同式が成立 (`line_start.x == line_end.x`)。
6. `edge` 由来の `Arrow` を `Annotations.arrows` に追加してはならない。`arrows` は `@->` 由来のみ。

**展開後不変条件**: `clock` 装飾付き SignalRow も通常 SignalRow と同じ構造に正規化される (波形は `waveform`、マーカーは `edge_marks`)。layout / renderer は `decorations.clock` を直接読まない。

### 4.4 アンカー解決パス

矢印 (`@->`) は TCML 中の任意位置に書け、参照アンカーが宣言行より後でもよい (前方参照可)。

1. parser は全行パース後にアンカー解決パスを走らせる。
2. 信号行内 `@{name}` / `@N` を走査し、各 `AnchorId` を `AnchorRegistry` に登録 (重複は `ParseError::DuplicateAnchor`)。`Named` (`@{...}`) と `Indexed` (`@N`) は同名前空間内で重複禁止だが互いに別空間 (`@{1}` と `@1` は別物)。
3. 矢印の `ArrowEnd::Anchor` を `AnchorRegistry` で解決。未定義参照は `ParseError::UndefinedAnchor`。
4. アンカー座標は **layout 段階** で確定する:
   - x = 直前までの要素の `width()` 累積位置。
   - y = 直前 `LevelRun` の線位置 (Single は 1 点、Double は中央線、Fill 系は焼き込まれた線位置)。

`@->` の重なり回避は **やらない**。複数矢印が同一座標を通る場合でも parser / layout は何もしない (重ならない配置は TCML 作者の責務)。

### 4.5 レイアウトアルゴリズム

行を縦に積み上げるだけ。「直前のレベルから遷移を再構築」「最終行は gap 半分」のような分岐は **禁止**。

1. `ChartStyle.canvas.line_height = font.size * lineheight` を確定。
2. **パス 1**: 各 `Line` の高さと「仮幅」を計算する:
   - `Skip` → 高さ = `amount.resolve(line_height)` / 仮幅 = `Px::ZERO`
   - `Title` → 高さ = `text 行数 * line_height + h_space` / 仮幅 = `capwidth + タイトル幅` (Signal と同じく `h_space` を加算し、上下に `h_space/2` ずつ対称配分する)
   - `Signal` → 高さ = `max(name 行数 * label_font.line_height, signal_height) + h_space` / 仮幅 = `capwidth + 要素列 width() の総和`
3. **パス 2**: `chart_inner_width = max(全 Line の仮幅)` を求め、**全 `Line.bbox.size.width` に一律代入** (Skip/Title/Signal 区別なし)。
4. 各 `SignalRow` の `label_box` / `signal_box` を `bbox` ローカルで計算。`signal_box.size.width` は要素列 `width()` 合計 (capwidth と短信号の余白を除いた純粋な波形領域)。
5. `Line[i].bbox.origin.y` を上から `Line[i-1].bbox.size.height` の累積で確定。
6. `WaveformElement::Anchor` の x/y を確定し `AnchorRegistry` に書き込み。
7. `Arrow.from`/`to` を `AnchorRegistry` で解決し座標化。

`row_pitch_between(prev_name, next_name)` のような関数は登場しない。`page_margin` はチャート全体の外側にだけ加算し、`Line.bbox` の積み上げには関与しない。

## 5. ドキュメント全体構造

```
ChartDocument
├── style: ChartStyle
├── lines: Vec<Line>
├── annotations: Annotations
│   ├── overlays: Vec<TextOverlay>      // % 行
│   ├── arrows:   Vec<Arrow>            // @-> のみ (clock 由来は混入禁止)
│   └── anchors:  AnchorRegistry        // 解決済み座標
└── source: TcmlSource                  // SVG 埋め込み用 TCML 原文
```

レイアウトエンジン出力 = `ChartDocument`。SVG レンダラはこの 1 構造体だけを入力に取り、外部から状態を引かない。

## 6. 設計レベル不変条項

設計レベルで再発不能にするための条項。

### 6.1 行ジオメトリの非対称・特殊扱い禁止

- `h_space` は `Line.bbox` の上下に対称に `gap/2` ずつ配分。片側集中・非対称配分は禁止。
- 先頭行・最終行・単一行を含め、全 `SignalRow` が同じジオメトリ規約に従う。「最終行は gap 無し」「先頭行は上余白を `page-margin` から取る」のような分岐コードを禁止。
- `@bgcolor0` / `@bgcolor1` / `@bg` (ローカル) は `Line.bbox` 全体 (上下 gap/2 含む) を塗る。背景塗り領域と行積み上げ領域を一致させ、隙間や重なりを発生させない。
- `Line[i+1].bbox.origin.y == Line[i].bbox.origin.y + Line[i].bbox.size.height` を debug_assert で検証。

### 6.2 `Line.bbox.size.width` の全行一様

- レイアウト出力時点で **すべての `Line.bbox.size.width` は同一値** (= `chart_inner_width`)。Skip / Title / Signal の区別なし。
- `row-backgrounds` / `@bg` / その他「`Line.bbox` 全体を塗る」要素は同じ幅で描画される (レンダラは `Line.bbox.size.width` をそのまま `<rect width=...>` に渡す)。レンダラ側で chart 幅を再計算したり `max(bbox.size.width)` を取り直したりする実装を禁止 (二重計算の温床)。
- SVG 全体の `width` 属性 = `Line.bbox.size.width + 2 * page_margin`。

### 6.3 断絶情報の保持

- 信号の連続性を切る要素 (`Gap` 等) は parser → layout → renderer の全段階で同一要素として伝播する。
- レイアウト要素列挙型は「描画情報」だけでなく「制御情報 (フラッシュ命令)」も保持できる構造とする。
- レンダラは断絶マーカー受信時に polyline 蓄積をフラッシュする義務を負う。

### 6.4 x 座標の単一ソース

- すべての `WaveformElement` は明示的な `width(): Px` を持つ。レイアウトの x 進行は `x_after = x_before + element.width()` の単一ルール。
- 要素種別ごとに後付け補正を入れる設計を禁止。chart 全幅の計算も要素列の累積 `width()` から導出する。
- `SignalRow.geometry.signal_box.size.width` は要素列 `width()` 合計と必ず一致しなければならない (debug_assert)。

| WaveformElement | width |
|-----------------|-------|
| `LevelRun` (`_` / `~` / `-` / `=`) (前要素が遷移なし) | `units * step` |
| `LevelRun` (前要素が `Transition::*`) | `units * step - slant` (最初の 1 単位の先頭 `slant` 部分は前 `Transition` が消費済みのため、その分だけ短縮) |
| `Transition::SingleEdge` / `BusOpen` / `BusClose` | `slant` |
| `Transition::BusCross` | `slant` (cross 部のみ)。X の body は後続 `LevelRun(Bus / DontCareAlongBus, 1)` に分離保持し、本ケースでは body の width も上記ルールに従い `step - slant` (X が信号行頭等で cross 省略の場合は `Transition(BusCross)` 自体を生成せず body の `LevelRun` のみ、その width は `step`) |
| `Gap` | `step` |
| `Guide` / `HighlightStart` / `HighlightEnd` / `Anchor` / `DontCareMarker` (`?`) | `Px::ZERO` |
| `Text` | `Px::ZERO` |

幅モデルの根拠: level char 1 個は **常に `step` 幅** を消費する。直前に遷移があるとき、その遷移は後続 level char の `step` 幅の先頭 `slant` 部分として描画され、level の純粋な hold 部分は `step - slant` になる。これにより、入力の文字数 (level char + Gap + X) が同じ波形は遷移本数によらず常に同じ全幅になる (`Transition` 単独では文字を消費せず、前後 level char の枠内に収まる)。

寸法定数 (`tcml-format.md` §補助記号 参照):
- `step`: level char 1 個ぶんの x 進行幅 (デフォルト 10px)
- `slant`: 全遷移の幅 (デフォルト 2px)。`step <= slant` はパースエラー (`ParseError::InvalidStepSlant`)
- `h_space`: 信号行間スペース (デフォルト 10px)
- 旧名 `w_hold` / `w_transient` は廃止 (alias 受理もしない)

`Text` バリアントは `tcml-format.md` §「レベル文字列中のテキスト文字」で規定されたテキスト文字列を保持する。内容は既存の `text::UserText` を再利用する (新規テキスト型は作らない)。所属区間 (連続する同一レベル区間) と中央配置位置の決定はレイアウト段階で行い、`Text` 自体は波形 x 進行に加算しない (はみ出し許容、テキスト幅 > 区間幅 でも区間を広げない)。

### 6.5 遷移描画の網羅契約

各 `TransitionKind` に対して、描画される **線の本数・始終点・接続先 polyline** を網羅的に定義する (詳細は [`svg-rendering.md`](svg-rendering.md))。

| TransitionKind | from-side y | to-side y | 線の本数 | 接続先 |
|----------------|-------------|-----------|---------|--------|
| `SingleEdge`   | (1 点)      | (1 点)    | 1       | 前後の Single polyline |
| `BusOpen`      | (1 点)      | (2 点)    | 2       | 前 Single + 後 Bus 上下 |
| `BusClose`     | (2 点)      | (1 点)    | 2       | 前 Bus 上下 + 後 Single |
| `BusCross`     | (2 点)      | (2 点)    | 2 (X 字) | 前 Bus 上下 + 後 Bus 上下 (cross) |

- 「共有辺の追加線が不要」のような暗黙の最適化を仕様書に書かない。共有辺がある場合でも明示的に水平線を描画する。
- 遷移線は隣接 polyline と必ず連結する。独立 `<line>` 要素として描画してはならない (接合部にシームが生じるため)。`stroke-linecap: round` 等の見栄え調整に依存する設計も禁止。

### 6.6 Clock マーカーと `@->` 矢印の分離

- `SignalRow.edge_marks` は clock 展開パスでのみ追加される。`@->` 由来の Arrow は混入しない。
- `Annotations.arrows` には `@->` 由来の Arrow しか含まれない。clock 由来は `edge_marks` に分離 (debug_assert で検証)。

### 6.7 信号名上線は独立 `<line>` 要素

- `SignalDecorations.name_overline: bool`。**信号名全体への上線**のみ対応。
- 描画は `<text text-decoration="overline">` ではなく独立した `<line>` 要素 (詳細は [`svg-rendering.md`](svg-rendering.md))。
- 上線位置はフォントメトリクスの `cap_top` から `DEFAULT_OVERLINE_GAP_PX` 上に置く (cap_height 取得不可時は `font.size * DEFAULT_CAP_HEIGHT_RATIO`)。

### 6.8 `@title` のデフォルト揃え

- `@title` のデフォルト揃えは `Center`。`DEFAULT_TITLE_ALIGN` は `defaults` で集約。

### 6.9 設計レベル不変条件 (debug_assert)

これら条項を機械的に検証するため、型レベルの assert を入れる:

- `Line[i+1].bbox.origin.y == Line[i].bbox.origin.y + Line[i].bbox.size.height` (全隣接行)
- 全 `Line` で `bbox.size.width` が同一値
- 全 `SignalRow` で `(bbox.size.height - signal_box.size.height) == h_space` (対称ギャップ)
- 全 `SignalRow` で `signal_box.size.width == sum(elements.width())`
- 各 `TransitionKind` のレンダラ実装は `match` 網羅性を Rust コンパイラに強制 (`_ =>` 禁止)
- `Annotations.arrows` に clock 由来 Arrow が混入しないこと

## 7. パーサ設計の要点

- 既存パーサ枠組み (行分割・コメント除去・`@param value` 切り出し) を流用。
- `parse_string_param` のような汎用ディスパッチではなく **パラメータごとの専用パーサ関数** をテーブル駆動で持つ (`PARAM_SPECS: &[ParamSpec]`)。
- 値はすべて対応する NewType の `parse` で検証してから格納 (生 `String` の素通し禁止)。
- 行番号 / 列番号付き `ParseError` を返す。

### 7.1 表記揺れ許容

`fontsize` / `font-size` / `font_size` のような表記揺れを同一パラメータとして受け付ける:

- `ParamSpec.names: &'static [&'static str]` で複数別名を持てる (先頭が正規名)。
- 大文字小文字は区別しない (パーサ入口で小文字化してマッチ)。
- 実行時は `OnceLock<HashMap<&'static str, &'static ParamSpec>>` で線形走査を回避。
- 別名衝突はテストで検出 (`tests/parser.rs`)。
