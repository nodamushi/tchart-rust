# TCML フォーマット仕様

TCML (Timing Chart Markup Language) は、タイミングチャートを記述するためのテキストベースの言語。

参考実装:
- [オリジナル tchart (東北学院大)](https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/)
- [tchart-coffee (筑波大)](https://github.com/osamutake/tchart-coffee)

データ型は実装ソース (`tchart-core/src/`) が正本。実装構造とアルゴリズム概要は [`types.md`](types.md) 参照。本ドキュメントは TCML 表記の文法を扱う。

> **重要**: 本ファイルを更新したら、ユーザー向けのスタンドアロン HTML ヘルプ (`help/output/tcml-format.html`) を必ず再生成すること。生成は `python3 help/build.py` で行う (入力データは `help/source.py`、SVG は実行時に `tchart` CLI でレンダリングして inline 埋め込みされる)。詳細は `help/README.md` を参照。仕様変更時は `source.py` 側の文章とサンプルを追従させ、再ビルドする。

## ファイル拡張子

`.tc`

## 行の種類

| 種類 | 先頭文字 | 説明 |
|------|----------|------|
| コメント行 | `//` | 無視される |
| パラメータ行 | `@` | パラメータ設定 / 行ディレクティブ (`@title` / `@skip` / `@clock` / `@->`) / 信号属性 (`@signal`) |
| 文字書き込み行 | `%` | 指定座標に文字列を配置 (overlay) |
| タイミング記述行 | その他 | 信号名とレベル文字列 |

`//` (連続するスラッシュ 2 文字) は行頭・行中いずれでもコメント開始記号として作用する。クォート (`"..."`) 外で `//` が出現した時点で以降は行末まで無視される (タイミング記述行・パラメータ行・文字書き込み行・空行いずれの行種別でも同じ)。単独の `/` 1 文字は通常文字 (テキスト文字 / 識別子の一部として扱う)。`//` 自体を literal として書きたい場合はクォート (`"//"`) で囲む。`#` は特殊な扱いをせず通常文字 (テキスト文字 / 識別子の一部) として扱う (例: 色値 `#ff0000` をクォート外で書ける)。

## タイミング記述行

```
<信号名> <空白> <レベル文字列>
```

信号名とレベル文字列は空白で区切る。区切りとなる「空白」は半角空白 (U+0020) またはタブ (U+0009) に限定する。他の Unicode 空白文字 (NBSP U+00A0 / 全角空白 U+3000 / ZWSP U+200B 等) は信号名の一部として保持し、区切り文字として扱わない。

### 信号名

- 有効な UTF-8 文字列。
- 制御文字を含めてはならない (`\n` のみ例外)。
- 空文字不可。

### ファイル先頭の BOM

ファイル先頭の UTF-8 BOM (U+FEFF) は許容し、字句解析の最初に黙ってスキップする。BOM はパース結果の `TcmlSource` (元ソースの保持) には影響しない。BOM が先頭以外の位置で現れた場合の挙動は spec 範囲外 (パーサが通常のテキスト文字として扱うか reject するかは未規定)。

### 空 wave の許容

信号名のみで level 文字列が空の行 (例: `SigA `、または末尾に空白のみ) は許容され、波形要素を 1 つも持たない `SignalRow` を生成する。`?` がないので `DontCareWithoutAnchor` 等のエラーにはならない。空 wave 行は `chart_units` の最大値計算に **寄与しない** (= 0 unit 相当)。

#### 複数行信号名

信号名を `"` で囲むと内部に改行を含められる。

```tcml
"Data
Bus" ========X========
```

- 開始の `"` は行頭になければならない。
- 閉じの `"` の後に空白を挟んでレベル文字列が続く。
- `"` を使わない従来の単一行構文も有効。

##### エスケープシーケンス（`"..."` 内のみ）

| シーケンス | 意味 |
|-----------|------|
| `\"` | リテラルの `"` |
| `\n` | 改行 |
| `\\` | リテラルの `\` |

### レベル記号

| 記号 | 意味 | 形状 |
|------|------|------|
| `_` | ローレベル (Low) | 単線・下 |
| `~` | ハイレベル (High) | 単線・上 |
| `-` | ハイインピーダンス (Hi-Z) | 単線・中央 |
| `=` | バス (Bus) | 二重線 (上下レール) |

### 補助記号

| 記号 | 意味 | x 進行 |
|------|------|--------|
| `:` | Gap (1 単位の空白、信号連続性を断絶) | `step` |
| `X` | Bus 値変化 (BusCross) | `step` (cross 部 `slant` + body 部 `step - slant`、または信号行頭等で cross 省略時は body のみ `step`) |
| `?` | Don't care マーカー | 0 |
| `\|` | 縦線 (ガイド線) | 0 |
| `[` | ハイライト開始 | 0 |
| `]` | ハイライト終了 | 0 |
| `@{name}`, `@N` | アンカー (矢印用) | 0 |

#### 寸法定数

| 名前 | デフォルト | 意味 |
|------|----------|------|
| `step` | 25px | level char 1 個ぶんの x 進行幅 (1 単位時間)。直前に遷移があるとき、その遷移は本 `step` 幅の先頭 `slant` 部分を占める |
| `slant` | 5px | 遷移幅 (0 でエッジ垂直)。SingleEdge / BusOpen / BusClose / BusCross すべてに適用。`step <= slant` はパースエラー (`ParseError::InvalidStepSlant`) |
| `h_space` | 10px | 信号行間スペース |

##### 幅モデル

level char 1 個は **常に** `step` 幅を消費する (行頭か直前が遷移かに関わらない)。直前に遷移 (SingleEdge / BusOpen / BusClose / BusCross) があるとき、その遷移は **後続 level の先頭 `slant` 部分** として描画され、level の純粋な hold 部分は `step - slant` になる。直前に遷移がない level (信号行頭、Gap 直後、同レベル continue 等) は全幅 `step` が hold。これにより、同じ文字数の波形は遷移本数によらず同じ全幅になる。

#### `X` の構造

X = **cross 遷移** + **body** (Bus 1 単位、新値) の 2 部品で構成。cross は body の **前** に付き、X 全体で level char 1 個ぶん = `step` 幅を占める。

- 前に bus 信号があるとき: cross 部 `slant` + body 部 `step - slant`、合計 `step`。
- 前に bus 信号がない (信号行頭等): cross を省略、body のみで `step`。

cross は前 bus の上下レールから body の上下レールに 2 本の線が交差して接続。中点 `(x + slant/2, y_mid)` で 2 本が触れる。`slant = 0` のとき cross は縦線に縮退。

#### `?` の構造

`?` は幅 0 のマーカー。周辺の連続レベル区間を不定値領域として塗る (詳細は §「Don't care (`?`) の解決ルール」)。

### Don't care (`?`) の解決ルール

`?` は幅 0 のマーカー。`?` を含む同値 Bus 区間 (`?` を挟んで連続する Bus level run + X body) 全体を 1 つの不定値領域として塗る (`<g class="dontcares">` polygon)。

#### 領域決定

1. `?` の直前のレベル文字 (`_`/`~`/`-`/`=`) または X (BusCross) を文脈とする。
2. 前後に同種レベル記号 / 別の `?` が続く間、領域に取り込む。
3. 別レベル / `:` (Gap) / 行端で打ち切り。
4. X (BusCross) は **領域境界として扱い**、polygon の左右辺が cross の半分を取り込む (`>▲▲<` 形)。X 前と X 後で別の bus 値なので別領域。
5. 透過要素 (`@{...}`、`@N`、`|`、`[`、`]`) は領域計算上スキップ。

#### polygon 左右辺の形状

| 境界 | polygon の辺 |
|------|-------------|
| Bus continue (隣接 `=`) | 垂直 |
| BusOpen (`_=` `~=` `-=`) | 斜辺、Low/High/HiZ 側で y_mid に縮退 (1 点) |
| BusClose (`=_` `=~` `=-`) | 対称 |
| X (BusCross) | cross 中点 (`x_cross + slant/2`, y_mid) を polygon 頂点とし、cross の半分を polygon 内に取り込む |
| 信号行頭 / 行末 / Gap / 別レベル | 垂直 |

#### 例 (step=10, slant=2)

| 入力 | polygon 概形 | polygon 幅 |
|------|------------|----------|
| `==?==` | 矩形 | 4 × step = 40 |
| `_=?=_` | 六角形 `/=\` | BusOpen 斜辺 〜 BusClose 斜辺 |
| `=X?X=` | 六角形 `>▲■▲<` | X1 cross 中点 〜 X2 cross 中点 |
| `=X?=` | 五角形 (左 X 半分、右垂直) | X cross 中点 〜 信号末端 |
| `=?X=` | 五角形 (左垂直、右 X 半分) | 信号始端 〜 X cross 中点 |
| `____????` | 矩形 (Low) | 4 × step |
| `==?==X==` | 矩形 (X で打ち切り) | 4 × step |

#### 内部線位置

| 直前文脈 | 線 |
|---------|-----|
| `_` Low | y_low |
| `~` High | y_high |
| `-` HiZ | y_mid (破線) |
| `=` Bus / X 直後 | bus 包絡 (上下 2 本) |

#### エラー

`?` で行頭 (前にレベル文字も X も無い): `ParseError::DontCareWithoutAnchor`

#### `X?` および non-bus 隣接 X パターン

| パターン | 解釈 |
|---------|------|
| `=X=` | Bus(1) + X(cross + body, 新値) + Bus(1) |
| `=X?` | Bus(1) + X(cross + body) + ? (X body が dontcare) |
| `=X?=` | ? 領域 = X body + 後 `=` (= 2 × step ぶん、左 polygon 辺 = X cross 中点 から) |
| `=?X=` | ? 領域 = 前 `=` (= 1 × step、右 polygon 辺 = X cross 中点 まで) |
| `?X=` | エラー (先頭 `?`) |
| `~X_` | High(1) + BusOpen(`~→Bus`、`\` 形) + Bus(1, X body) + BusClose(`Bus→_`、`\` 形) + Low(1) → 視覚 `~\=\_` |
| `_X~` | Low(1) + BusOpen(`_→Bus`、`/` 形) + Bus(1, X body) + BusClose(`Bus→~`、`/` 形) + High(1) → 視覚 `_/=/~` |
| `_____X=====` | Low(5) + BusOpen + Bus(6) (X body と後続 `=====` が merge) |
| `=====X_____` | Bus(5) + BusCross(cross + body) + BusClose + Low(5) (X は通常の cross 遷移、後続が non-bus なので BusClose) |
| `XX=` | Bus(1, 1 つ目の X body、信号行頭で cross 省略) + BusCross + Bus(2, 2 つ目の X body と `=` が merge) |

### 連続レベルのマージ

同一レベルの連続 (`__`, `~~`, `??` etc.) は parser が 1 つの区間にマージする。

### レベル文字列中のテキスト文字 (筑波大 tchart-coffee 方式)

レベル記号 (`_~=-?`)・遷移記号 (`X`)・特殊記号 (`@[]|:`)・空白・クォート (`"`) 以外の任意文字 (例: `a`, `<`, `>`, `あ`, `#`, `/`) はレベル文字列中で **テキスト文字** として扱われ、所属区間の中央に `<text>` で描画される。`<` `>` も信号名・`@title` / `@->` ラベル等のテキスト中と同様にただの文字として SVG に出力される (XML エスケープ)。ただし `//` (連続スラッシュ 2 文字) は §「行の種類」記載のとおりクォート外でコメント開始扱いとなり、レベル文字列はそこで切断される。`//` を literal text として配置したいときは `"..."` 引用を用いる (例: `__"// note"__`)。

#### ルール

1. **所属区間**: テキスト文字は、その出現位置を含む **連続する同一レベル区間** に所属する。テキスト文字はレベル区間の連続性を切らない (例: `__a__` は merged 4 単位 Low 区間。`a` を中央配置)。
2. **空白結合**: 同一区間内に複数のテキスト断片があれば空白で結合し 1 つの文字列として中央配置する (例: `__a__b_` → 5 単位 Low 区間の中央に `"a b"`)。元の筑波大 tchart-coffee 仕様では `a` と `b` が同じ位置に重なって表示されるが、空白結合により 1 文字列として読めるよう改めた。
3. **行頭はレベル必須**: 信号行のレベル文字列はレベル記号で始まらなければならない。テキスト文字で始まる行 (例: `a__~~`) は `ParseError::MissingInitialLevel`。
4. **遷移直後のテキストは遷移先区間に所属**: BusCross (`X`) や `_~` 等の単一エッジ遷移の直後に来るテキストは遷移先 level の区間に属する。`Xa==` は `X=a=` と等価で、`a` は X 後ろの 2 単位 Bus 区間の中央。
5. **行末トレーリングテキスト**: レベル文字列末尾のテキスト (後続 level char 無し) は直前の連続区間に所属する。例: `~~~~~~~~かきくけこ` は 8 単位 High 区間の中央に `"かきくけこ"`。
6. **クォート literal**: `"..."` で囲った内容は level 記号 (`_~=-?`)・遷移記号 (`X`)・特殊記号 (`@[]|:`)・空白を含めすべて literal text として扱う。クォートが閉じていない場合は `ParseError::UnclosedQuote`。具体例:
    - `=="X"==`: 4 単位 Bus 連続区間の中央に文字 `X` (BusCross にならない)。
    - `__"hello world"__`: 4 単位 Low 区間の中央に空白を含む 1 文字列 `hello world`。
    - `__"_~="__`: 4 単位 Low 区間の中央に literal 文字列 `_~=` (level 記号として解釈されない)。
    - `__"[@|]"__`: 4 単位 Low 区間の中央に literal 文字列 `[@|]` (Highlight / Anchor / Guide として解釈されない)。
    - `__a"b c"d__`: bare text `a`、quoted text `b c`、bare text `d` の 3 断片を空白結合し、4 単位 Low 区間の中央に `a b c d`。
    - `=="a"=="b"==`: 6 単位 Bus 連続区間 (テキストは連続性を切らない) の中央に `a b`。
7. **はみ出し**: テキスト文字幅は波形 x 進行に加算されない。区間幅は level char (および遷移) のみで決まる。テキスト幅 > 区間幅 の場合、テキストは区間の中央を基準に **左右にはみ出して描画** する (区間を広げない、clip しない)。

#### 例

```
"<request>"  _~_~~~__
sigA         __ack__~~done~~
@-> (@{a}, @{b}) <signal-set>
```

`sigA` 行は前 4 単位 Low 区間の中央に `ack`、後 4 単位 High 区間の中央に `done` を配置 (テキスト文字は連続性を切らないため、`__ack__` は merged 4 単位 Low 区間)。信号名 `<request>` は `<>` 込みで描画。矢印ラベル `<signal-set>` も `<>` 込みで描画。

### アンカー埋め込み

```
foo  _~@{edge}_~
bar  ___~~~@1__
```

- `@{name}` または `@N` (1 以上の整数) を波形文字列中の任意位置に埋め込める。
- 0 幅マーカー (x 進行に影響しない)。
- アンカーの座標は parser → layout 解決後に確定 (x = 累積位置、y = 直前 LevelRun の線位置)。
- 名前付きと番号付きは別名前空間。`@{1}` と `@1` は別物。
- 同一 ID の重複定義は `ParseError::DuplicateAnchor`。
- `AnchorName` の文字種: `[A-Za-z0-9_][A-Za-z0-9_-]*`。先頭文字に数字を許す (例: `@{1}` は純数字でも valid な named anchor)。`@{...}` の中括弧の有無で named / numbered を区別するので、`@{1}` と `@1` は別の名前空間に属する別アンカーになる。
- `?` の解決ルックアップではアンカーは透過しない (アンカー単独では `?` の直前レベルにならない)。

## パラメータ行

書式: `@<パラメータ名> <値>`

### パラメータ名の表記揺れ

以下を **すべて同一パラメータとして受け付ける**:

- 大文字小文字の違い (`fontsize` / `FontSize` / `FONTSIZE`)
- ハイフン区切りとアンダースコア区切り (`fontsize` / `font-size` / `font_size`)

例: `@fontsize 14` / `@font-size 14` / `@FONT_SIZE 14` はすべて同じ。

### グローバルパラメータ（途中変更不可）

| パラメータ | デフォルト | 説明 |
|------------|------------|------|
| `fontsize` | `14` | フォントサイズ (px)。レイアウトの基準。**正値必須** (0 / 負 / 非有限 はパースエラー `ParseError::InvalidLength`) |
| `lineheight` | `1.2` | 波形高さの係数。波形高さ = `fontsize × lineheight`。**正値必須** (0 / 負 / 非有限 はパースエラー `ParseError::InvalidLength`) |
| `capwidth` | `0` | 信号名欄の幅 (px)。0 のとき全信号名から自動計算 |
| `namepad` | `8` | 信号名右端と波形左端の間の余白 (px) |
| `scale` | `1.0` | SVG ルート要素の表示倍率。出力 SVG の `<svg width="W" height="H">` 属性の数値 (`W` / `H`) に `scale` を掛けるだけで、内部座標系 (`Line.bbox` / `signal_box` / `<polyline>` の点列 / アンカー座標 / `viewBox` など) はすべて `1.0` 倍のまま (= scale 非依存)。これは SVG 標準のビューポート拡大手法に相当する (`width`/`height` 属性のみ変えてレンダラ側で拡大表示)。**正値必須** (0 / 負 / 非有限 はパースエラー `ParseError::InvalidLength`) |
| `page-margin` | `10` | チャート全体の四方の固定余白 (px)。**`Line.bbox` の積み上げには関与しない** (外側にのみ加算) |
| `bgcolor0` | `none` | 偶数行 (信号インデックス 0, 2, 4, …) の `Line.bbox` 全体の背景色 |
| `bgcolor1` | `none` | 奇数行 (信号インデックス 1, 3, 5, …) の `Line.bbox` 全体の背景色 |

### ローカルパラメータ（途中変更可能）

途中で再指定すると、**その行以降の信号に適用**される。

| パラメータ | デフォルト | 説明 |
|------------|------------|------|
| `step` | `25` | level char 1 個ぶんの x 進行幅 (px)。直前に遷移ありの場合、その遷移は本 `step` 幅の先頭 `slant` 部分として描画される。**正値かつ有限値必須** (0 / 負 / 非有限 はパースエラー `ParseError::InvalidLength`)。`@step` で新しい値を設定したとき、まだ `@slant` がユーザーにより明示されていなければ slant を `step / 2` に自動クランプする (小さな `@step` 値で既定 slant=5 が干渉して `ParseError::InvalidStepSlant` を起こさないように)。明示済みの `@slant` がある状態で `step <= slant` となった場合はそのまま `ParseError::InvalidStepSlant` |
| `slant` | `5` | 遷移幅 (px)。0 = 垂直エッジ。SingleEdge / BusOpen / BusClose / BusCross すべてに適用。**非負必須** (負値はパースエラー `ParseError::InvalidLength`)。`@slant` が一度でも記述されると、それ以降の `@step` は slant を自動クランプしない (`@step` 側で明示優先) |
| `h_space` | `10` | 信号行の上下余白合計 (px)。**`Line.bbox` の上下に `gap/2` ずつ対称配分**。§補助記号「寸法定数」の `h_space` と同一値。**非負必須** (負値はパースエラー `ParseError::InvalidLength`) |
| `font` | `sans-serif` | フォントファミリー名。空白を含む場合 `"` で囲む。カンマ区切り複数指定可 (フォールバック順)。実体ロード／フォールバック解決の詳細は `cli.md` §フォント解決および `web.md` 参照 |
| `signal_color` | `black` | 信号線の色 |
| `signal_width` | `1` | 信号線の幅 (px) |
| `guide_color` | `red` | 縦線 (ガイド) の色 |
| `guide_width` | `0.6` | 縦線の幅 (px) |
| `bg` | `none` | 次の 1 行 (種別問わず) の `Line.bbox` 全体の背景色 (ローカル上書き) |
| `highlight_style` | `fill="#ff8" stroke="none"` | ハイライト区間のスタイル (SVG 属性を空白区切り) |
| `dontcare_color` | `#bbb` | DontCare 矩形のハッチ線色。`@dontcare_color #c00` のように単一の色値を指定すると、それ以降の行の `?` ハッチ線色がその値に切り替わる (途中で再宣言可)。値書式は `@bgcolor0` 等と同じ |
| `titlealign` | `center` | `@title` 行の横揃え。`center` / `left` / `right`。途中変更すると、それ以降の `@title` から適用 |
| `clockmark_position` | `0.5` | クロック三角形マーカーの頂点位置 (線方向比、`0.0..=1.0`) |
| `clockmark_height` | `7.5` | クロック三角形マーカーの高さ (px) |
| `clockmark_width` | `6` | クロック三角形マーカーの底辺の幅 (px)。デフォルト値で解決されたときのみ「step 連動縮小」が適用される (§「`clockmark_width` の step 連動縮小」を参照) |
| `clockmark_color` | `signal_color` を継承 | クロック三角形マーカーの塗り色。未指定なら **clock 信号行を生成した時点での `signal_color` を焼き込む** (静的)。後で `signal_color` を書き換えても、過去に出力済みの clock マーカーの色は変わらない (パラメータは常に「定義した位置より後ろ」にしか影響しないという TCML 一般則) |
| `overline_gap` | `2` | 信号名上線とテキスト cap-top の間の隙間 (px) |
| `overline_thickness` | `1` | 信号名上線の太さ (px) |
| `ruler` | `on` | 信号変化点比較用の薄い背景縦線 (ガイド線) の有効/無効。値は `on` / `off`。各信号行・skip 行が「自分の有効 step 境界」にあたる x 位置で ruler 線を **寄付** する仕組み。詳細は §「`@ruler` の詳細」参照 |
| `ruler_color` | `#a0a0a0` | ruler 線の色。寄付モデルに従い、各行が commit される時点で有効な色を **スナップショット** して以降に焼き込む (signal_color と同様の途中変更可能ローカル設定) |

### `@bg` の詳細

- **ローカル設定**。直後に出現する **1 行 (種別問わず: Signal / Skip / Title)** の
  `Line.bbox` 全体に背景色を適用する。
- 1 回限り。1 行を消費した時点で自動的に `none` にリセット。
- 行を消費するまで保持される。間に他のディレクティブ (`@bgcolor0/1` を含む)
  が来ても消えず、次に出現する Line に適用される。
- 明示リセット: `@bg none` (保留中の値を破棄)。
- `@bgcolor0/1` (グローバル設定) より **常に優先** される。`@bg` 指定行では
  `bgcolor0/1` を重ね描きしない。

### `@bgcolor0` / `@bgcolor1` の詳細

- **グローバル設定**。チャート全体の偶数行・奇数行を交互に塗り分ける。
- `Line.bbox` 全体 (`h_space/2` の上下余白を含む) を塗る。
- `@bg` (ローカル設定) が指定された行はそちらが優先され、`bgcolor0/1` は描かない。
- 偶奇インデックスは **`SignalRow` のみカウント** (`Skip` 空白行と `Title` 行は除外)。
- 片方のみ指定可 (もう片方は `none` 扱い)。

### `@highlight_style` の詳細

```tcml
@highlight_style fill="#8f8" stroke="green" stroke-width="1"
```

SVG 属性を `key="value"` 形式で空白区切り指定。

#### 命名規約: `_color` と `_style`

ローカル/グローバルパラメータの色・スタイル系命名は以下の暗黙ルールに従う。

| 接尾辞 | 値の形式 | 例 |
|--------|---------|----|
| `_color` | 単一色値 (引用なし、`Color::parse` 可) | `signal_color`, `dontcare_color`, `bgcolor0` |
| `_style` | SVG 属性 (`key="value"` を空白区切り、ホワイトリスト) | `highlight_style` |

色だけを変えたい場合は `_color`、塗り・線を含む複合スタイルにしたい場合は `_style` 系を使う想定で命名されている。

### `@dontcare_color` の詳細

```tcml
@dontcare_color #c00
```

単一の色値を取る (引用なし、`@bgcolor0` 等と同じ書式)。ハッチ線の色をその値に切り替え、それ以降の行に適用される (チャート途中で再宣言可)。DontCare 矩形は常に `<defs>` 内のハッチパターン参照のみで、アウトライン (枠線) は持たない。

`<defs>` には、チャートで実際に使われた色ごとに `<pattern>` が ID `dontcare-hatch-1`, `dontcare-hatch-2`, … として出力される (同色は ID 共有)。チャート内に `LevelRun(DontCareAlong*)` が 1 つ以上存在すれば `<defs>` は常に出力される (`?` はパース後 `LevelRun(DontCareAlong*)` に変換されるため、有効な `?` が 1 つでもあれば該当する)。

### `@ruler` の詳細

`@ruler on` / `@ruler off` で **信号変化点比較用の薄い背景縦線** (ruler) を制御する。デフォルトは `on` (= 信号変化点比較が一目で読めるようにする初心者向け既定値)。明示的に消したい場合は `@ruler off` を宣言する。

#### 寄付モデル

「グローバル全消えバグ」(途中で `@ruler off` にすると先頭まで遡って全消えする) を避けるため、ruler は **行単位の寄付モデル** で扱う:

- `@ruler on` の状態で「信号行」または「`@skip`」を commit するとき、その行は **自分の表示範囲の step 境界** に ruler 線を寄付する。
  - 「自分の表示範囲」= その行が `step` 幅 × `units` 個ぶん横方向を占める領域。
  - 寄付する x 集合 = `{ i × step | 0 ≤ i ≤ units }` (両端含む。units+1 本)。
  - 寄付する color = その行を commit する **時点で有効な** `ruler_color` の値 (スナップショット)。signal_color と同じく、後で `@ruler_color` を変えても過去に commit された行の色は変わらない。
  - `step` / `units` も commit 時の値で固定。後で `@step` を変えても過去行の寄付位置は動かない。
- `@ruler off` の状態で commit された行は**何も寄付しない**。
- 「`@title`」「コメント行」「ディレクティブ単独行」は寄付しない (信号/skip のみが寄付源)。
- 寄付情報は行のサイドカー `Vec<RulerContribution { x: Px, color: Color }>` として保持される (実装側の表現)。

#### レンダー時のマージ (last-wins per x)

ruler 線は **背景レイヤ** (波形より背面、行背景より前面) に描画される。レンダー時に **全行を順走査** して x 位置ごとに以下を決定する:

1. 各行の寄付 `(x, color)` を順次 map に積む。
2. 同じ x に複数の行が寄付した場合、**後の行 (= 下にある行) の color で上書き** する (last-wins)。
3. 最終的に各 x 位置に **full-height (チャート全体を貫通)** の縦線を 1 本、決定された color で描画する。

これにより:

- 同じ x に複数行が寄付しても線は 1 本だけ (重複統合)。
- `@ruler_color` を途中で変えると、それ以降の行が寄付する分のみ新しい color になる。古い行の color とは別の x 位置なら両方とも描かれ、同じ x 位置なら **下の行 (後の行) の色が勝つ**。
- `@ruler off` した行は寄付ゼロ。これより前の行の寄付は残るので、`@ruler off` で**全消えにはならない** (グローバル全消えバグ回避の核心)。

#### 線種・太さ (固定)

線種は仕様上 **固定の薄い点線**:

- `stroke-dasharray="3 5"`
- `stroke-width="0.5"`

`stroke` は寄付時にスナップショットされた `ruler_color` を使う。これらの SVG 属性は今のところユーザ可変ではない (将来拡張する場合は別タスクで切る)。詳細な SVG 出力は `svg-rendering.md` §「ruler 背景縦線」参照。

#### 例

```tcml
@step 15
@ruler on
@ruler_color #a0a0a0

CLK   _~_~_~_~_~_~
Data  =D0====XD1====XD2====

@ruler off
DontCount  __~_~~  // この行は ruler 寄付ゼロ
```

上記では CLK と Data の 2 行が `[0, 15, 30, 45, ..., 180]` の x に `#a0a0a0` を寄付し、ruler 線が 13 本背景に描かれる (DontCount 行は寄付しない)。

## 属性 `key=value` の共通ルール

TCML 中で `=` を使う属性記述は **すべて** 以下に従う。対象ディレクティブの例:

- `@clock(...)` の各オプション (例: `period=10ns`)
- `@-> (..., ..., head=end)` の `head=`
- `@highlight_style key="value"`

ルール:

- `=` の前後の空白は **任意**。`key=value` / `key =value` / `key= value`
  / `key = value` は等価。
- キー名・値はそれぞれ両端空白を除いた上で評価される。
- 値に空白を含めたい場合は `"..."` で囲む (既存規則)。
- **キー名は大文字小文字を区別せず、`-` と `_` を等価扱い** とする (例: `mark-position` / `mark_position` / `MARK-POSITION` / `MARK_POSITION` はすべて同義。`color=red` と `COLOR=red` も同義)。これは `@->` の `head=`、`@highlight_style` の `fill=` `stroke=` 等を含むすべての `key=value` 記述に適用される。
- **例外**: `@clock(...)` の `_=N` / `~=N` は **キーではなくレベル記号** (Low / High) なので等価ルールの対象外。`_` を `-` に書き換えてはならない (`@clock(-=2)` は無効)。

## 行ディレクティブ

特定の行効果を生む `@` ディレクティブ。

### `@skip` — 空白行

```tcml
@skip(2)        # 2 lh の縦空白
@skip 2         # 同上 (空白区切り、括弧省略形)
@skip(2.5)      # 2.5 lh
@skip(20px)     # 20 px
@skip 20px      # 同上
```

- 引数の括弧 (`@skip(...)`) と空白区切り (`@skip ...`) は **どちらも受け付ける** (他の `@step 10` などと同じ慣習)。
- 単位なし数値 → `lh` (line-height 単位)。
- 数値 + `px` → ピクセル。単位サフィックス `px` は ASCII case-insensitive (`px` / `Px` / `pX` / `PX` を等価扱い)。
- **負値・パース不能**: `ParseError::InvalidSkipAmount`。
- **0 は許可するが無視** (`SkipRow` を生成しない)。
- `Skip` 行は `bgcolor0/1` の偶奇インデックスに含めない。

### `@title` — タイトル行

```tcml
@title 同期回路の動作

@title "クロック同期
データ転送回路"
```

- 引数文字列をタイトル行として描画。
- 引数は **必須**。`@title` 単独 (引数欠落) はパースエラー `ParseError::Message`。空タイトル行が必要な場合は明示的に `@title ""` と書く。
- 複数行は `"..."` 引用 (`SignalName` と同じエスケープ規則)。
- 1 ファイル中に複数回出現可能。
- `Title` 行は `bgcolor0/1` の偶奇インデックスに含めない。
- 横位置は `@titlealign` (グローバル) で決定する。デフォルトは `center` (`defaults.rs::DEFAULT_TITLE_ALIGN`)。

### `@titlealign` — タイトルの横揃え

```tcml
@titlealign center    # デフォルト
@titlealign left
@titlealign right
```

- グローバル設定。`@title` が描画される際の横揃えを指定する。
- 値: `center` / `left` / `right` (大文字小文字を区別しない)。それ以外はパースエラー。
- 設定後に出現するすべての `@title` 行に適用される。**設定変更が複数回出現した場合、それぞれ後続の `@title` から有効**。
- 描画ピクセル位置への変換は `svg-rendering.md` §「タイトル (titles)」のマッピング表参照。

### `@clock` — クロック自動展開

```tcml
@clock
clock                            # `@clock` 単独 = `@clock(none)`、マーカーなし auto 展開
```

```tcml
@clock(pos, _=2, ~=3)
clock                            # 本体省略 → 自動展開
```

```tcml
@clock(neg)
ck    ~~__                       # 部分指定 → 続きを最後の状態から自動繰り返し
```

書式: `@clock` / `@clock()` / `@clock([<edge>] [, _=<n>] [, ~=<n>] [, start=<low|high>] [, mark_position=<f32>] [, mark_height=<px>] [, mark_width=<px>] [, mark_color=<color>])`

`@clock` 単独および `@clock()` は `@clock(none)` と等価 (マーカーなしで auto 展開のみ)。

| 属性 | 値 | 説明 |
|------|----|------|
| `edge` | `pos` / `neg` / `both` / `none` | 三角形マーカーの対象 (省略時 `none`) |
| `_=<n>` | 正整数 | Low の単位時間数 (省略時 1) |
| `~=<n>` | 正整数 | High の単位時間数 (省略時 1) |
| `start=<low\|high>` | キーワード | 開始相 (省略時 `low`) |
| `mark_position=<f32>` | `0.0..=1.0` | 三角形の頂点位置 (線方向比)。省略時 `clockmark_position` を継承 |
| `mark_height=<px>` | 正値 | 三角形の高さ。省略時 `clockmark_height` を継承 |
| `mark_width=<px>` | 正値 | 三角形の底辺の幅。省略時 `clockmark_width` を継承 |
| `mark_color=<color>` | 色 | 塗り色。省略時 `clockmark_color` (デフォルトは現行 `signal_color` を継承) |

- 属性順は不問。
- 属性キー名は大文字小文字を区別せず、`-` と `_` を等価扱いとする (例: `mark-position` と `mark_position` と `MARK_POSITION` はすべて同義)。これは ユーザがどちらの記法でも書けるようにする ためであり、実装側で短縮・削除してはならない。
- 直後の信号行を **クロック信号**として扱う。本体波形が空または部分的なら、最後の状態から `pulse` を繰り返し展開して右端をチャート全体に揃える。
  - 展開後の総 units 数は `target = round(max(他信号 i の row_i.units × row_i.step) / 当該行の step)` で算出 (per-row step を考慮)。これにより同 step なら他信号と units 数が一致し、`@step` を行ごとに変えていても右端がおよそ一致する。
  - 「他信号」は **auto 拡張対象でない explicit な信号行**(普通の信号行と、部分指定 clock 行の explicit 部分)のみ。auto 行同士は互いの target 算出に含めない (循環回避)。
  - 全信号が auto 拡張対象の場合は `target = 0` (auto 行は空波形)。
  - 部分指定 clock の auto 拡張: 既存 explicit 部分は不変。auto 部分は `target − 既存 units` だけ末尾に追加。`target ≤ 既存 units` の場合は何もしない (打ち切りもなし)。
  - `pulse = (Low(_=n), High(~=m))` 単位で展開し、最終 pulse の途中で units 数に到達した場合はそのまま打ち切る (Low / High の途中でも切る)。
  - 複数の auto 行は **互いに独立**に同じ explicit 集合から target を計算する。step が異なれば units 数も異なる。
  - 対象行が auto 行より前にあっても後にあっても等価。chart 全体で max を取る。
- `edge` に応じて立ち上がり/立ち下がりの遷移線に **三角形マーカー** (`EdgeMark`) を自動挿入する。
  - `pos`: 立ち上がりエッジ (`_~`) に三角形
  - `neg`: 立ち下がりエッジ (`~_`) に三角形
  - `both`: 両方
  - `none`: マーカーなし、波形展開のみ
- 三角形の幾何 (頂点位置・サイズ・向き) は `svg-rendering.md` §「クロックエッジマーカー (`edge-marks`)」参照。
- `@->` 矢印とは **完全に別系統**。clock 由来は `@->` の Arrow には混入しない。

### `clockmark_width` の step 連動縮小

三角形の底辺幅 `width` の確定は次の優先順位:

1. `@clock(..., mark_width=<px>)` のローカル指定があればその値 (縮小なし)。
2. `@clockmark_width <px>` のグローバル指定があればその値 (縮小なし)。
3. 上記いずれも未指定なら **`min(clockmark_width デフォルト = 6, step × 2/3)`** を採用 (= step 連動縮小)。

要点:

- **縮小はデフォルト値の解決にのみ適用される**。ユーザが `mark_width` または `@clockmark_width` のどちらか一方でも明示すれば、その値はそのまま採用される (縮小しない)。これは「ユーザの明示指定を尊重する」ためのルールであり、`mark_color` / `clockmark_color` の継承規則と同じ哲学。
- 縮小判定に使う `step` は **clock 信号行の生成時点で有効な `@step` 値**。`@clockmark_width` を未指定のまま `@step` を途中で変えると、それ以降の clock 行は新しい step に応じて縮小値を再計算する。
- `height` には縮小ルールを設けない。`height` は遷移線方向 (slant 方向) の伸びであり、step の密度とは独立だから。

### `@signal` — 信号属性

```tcml
@signal(overline)
nReset    ~~__~~~~
```

直後の信号行に属性を適用。1 回限り (適用後リセット)。

| 属性 | 効果 |
|------|------|
| `overline` | 信号名に上線 (負論理表記) を引く。複数行信号名では **最上行のみ** に 1 本。位置・太さは `@overline_gap` / `@overline_thickness` で制御。SVG 出力は `text-decoration` 属性ではなく独立した `<line>` 要素 |

将来拡張: `bold`, `italic` などをここに追加。

#### `@overline` (alias)

`@signal(overline)` の短縮形として `@overline` を受け付ける (`@overline` のみで `@signal(overline)` と同一動作。引数なし)。新規記述では `@overline` を推奨。`@signal(...)` 構文は将来 `bold` / `italic` を持たせる際の本命のため温存する。

## 矢印宣言 `@->`

```tcml
@-> (@{edge}, @1)                                     # 最小: 既定スタイル
@-> (@{edge}, @1, red, 2px, dashed) 変化               # フル: 属性 + ラベル
```

書式: `@-> (<始端>, <終端> [, <属性>, <属性>, ...]) [<テキスト>]`

- 括弧内は **すべてカンマ区切り** で、両端の空白は無視される。
- 先頭 2 トークンが `<始端>` `<終端>` (`@{name}` または `@N`)。3 番目以降が属性。
- `<属性>...` は順序不問。同カテゴリ重複は `ParseError::DuplicateArrowAttribute`。

| カテゴリ | 例 (positional 形式) | 等価な `key=value` 形式 | 判別 |
|----------|----|----|------|
| 色 (Color) | `red`, `#f0f`, `#ff8800` | `color=red`, `color=#f0f` | `Color::parse` 成功 / `color=` プレフィクス |
| 太さ (Width) | `2`, `2px`, `1.5px` | `width=2px` | 数値 (単位 `px` 省略可) / `width=` プレフィクス |
| 線種 | `solid`, `dashed`, `dotted` | `style=dashed` | キーワード / `style=` プレフィクス |
| 矢印頭 | `head=end`, `head=both`, `head=none` | (key= 形式のみ) | `head=` プレフィクス |
| ラベル | (後置 `[<テキスト>]` 形式) | `label="..."` | `label=` プレフィクス |

両形式は同一トークン中で混在可。`color=red` も `red` も等価。`key=value` 形式は将来 CSS 色名と線種キーワードが衝突した際の曖昧性回避用 (例: `color=solid` で `solid` 色を指定したい場合 — 現状の CSS 色名表に `solid` は無いが将来追加されたとき)。

ラベルは後置 `[<テキスト>]` 形式と `label="..."` 形式の **どちらか一方** を使う。両方を同時に指定した場合は `ParseError::DuplicateArrowAttribute`。`label="..."` 内の JSON エスケープ (`\n`, `\t` 等) は文字どおりの 2 文字シーケンスとして保持され、出力経路 (SVG / WaveDrom) でそれぞれの規約に従って扱われる (`UserText::parse` を経由する)。

キー名 (`color`, `width`, `style`, `head`, `label`) は ASCII case-insensitive で、`-` と `_` は等価 (`COLOR=red` / `color=red` / `Color=red` はすべて同義)。これは「属性 `key=value` の共通ルール」セクションのとおり。値の側 (`red`, `dashed`, `end` 等) のキーワードは小文字で記述する。

デフォルト: 色 = `signal_color`、太さ = `1px`、線種 = `solid`、矢印頭 = `end`、ラベル = なし。

- 前方参照可。`@->` 行は TCML 中の任意位置に書ける。前方参照されるのは
  **端点アンカーのみ**。ラベル等のスタイル (`font`, `signal_color` 等の
  ローカルパラメータ) は `@->` 行を記述した位置のローカル設定が適用される
  (ローカルパラメータ一般則のとおり)。
- 未定義アンカー参照: `ParseError::UndefinedAnchor`。
- 重なり回避は行わない (作者責務)。

## 文字書き込み行 `%`

```
% <x座標> <y座標> <文字列>
```

指定座標 (px) にテキストを overlay として配置。座標はチャート左上原点。

## 記述例

### 基本的なタイミングチャート

```tcml
@fontsize 14
@step 5
@slant 0

@title 同期回路

Clock   _~_~_~_~_~_~
Data    ========X========X========
Enable  ____~~~~____
Output  _______~~~~~____
```

### 複数行信号名と空白行

```tcml
@step 10

"System
Clock"  _~_~_~_~

@skip(0.5)

Data    ========X========
```

### 非同期クロック

```tcml
@step 10
ClkA    _~_~_~_~_~_~_~_~

@step 14
ClkB    _~_~_~_~_~_~
```

### 不定値とハイライト

```tcml
@step 10

Clock   _~_~_~_~_~_~
Data    _?=========[========]====
Status  __[~~~~]______
```

### `@clock` 自動展開とエッジ矢印

```tcml
@clock(pos, _=2, ~=3)
CLK

@signal(overline)
nReset    __~~~~~~~~~~~~~~~~~~~~
Data      ==========X========
```

### 矢印 (`@->`)

```tcml
Clock   _~@{rise}_~_~
Data    ___~~~~@1____

@-> (@{rise}, @1, red, dashed) setup
```

