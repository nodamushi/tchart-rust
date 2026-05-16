# WaveDrom 変換仕様

TCML を [WaveDrom (WaveJSON)](https://wavedrom.com/tutorial.html) 形式の JSON に変換する仕様。完全互換は目指さず、**WaveDrom が描画可能な TCML 要素のみを近似マッピング** する。WaveDrom に対応物のない要素は黙って落とす (一部例外は §警告)。

> 本仕様は TCML 本体の `@step` / `@slant` 命名 (level char 1 個ぶんの x 進行幅 = `step`、直前に遷移があるとき遷移は後続 level 先頭 `slant` 部分を占める、`step <= slant` は `ParseError::InvalidStepSlant`) を前提とする。詳細は [`tcml-format.md`](tcml-format.md) §補助記号。

## 概要

- 入力: `ChartDocument` (parser 出力)。TCML テキストから直接ではなく、parser を通った後の中間表現を変換する。
- 出力: WaveJSON 仕様の strict JSON テキスト (UTF-8、コメント・末尾カンマなし)。
- 拡張子: `.json`。
- 変換ロジック本体: `tchart-core::wavedrom` 配下に置く (`tchart-core::wavedrom::to_wavejson(&ChartDocument) -> String` 相当)。CLI (`tchart wavedrom`) は薄いラッパー。
- JSON シリアライゼーションには `serde_json` を用いる (本タスクで workspace 依存に新規追加)。

## トップレベル構造

WaveJSON 出力は次のフィールドを持つ。値が空のフィールドは出力しない (省略)。

| WaveJSON フィールド | TCML 由来 |
|---|---|
| `signal` (必須) | 信号配列 (§信号配列) |
| `head.text` | `@title` がある場合のみ。**最初の 1 個のみ** が保存され、2 個目以降は欠落する (WaveDrom `head.text` は単一文字列のため。情報損失あり、警告は出さない) |
| `edge` | `@->` が 1 つ以上ある場合のみ (§矢印) |

`config` (`hscale` 等)・`foot`・`head.tick` / `head.tock` / `head.every` は出力しない (TCML 側に対応概念なし)。

## 信号配列

`ChartDocument.lines` を出現順に走査し、`Line` 種別ごとに以下のように `signal` 配列要素を生成する。

| TCML `Line` | WaveJSON `signal` 配列要素 |
|---|---|
| `SignalRow` | 信号オブジェクト (§信号オブジェクト) |
| `SkipRow` (`@skip`) | 空オブジェクト `{}` 1 個 (`n` の値によらず常に 1 個) |
| `TitleRow` (`@title`) | (出力しない。タイトルは `head.text` に集約) |

`@skip(0)` は parser 段階で `SkipRow` を生成しないため WaveDrom 変換側で見ることはない。

## 信号オブジェクト

各 `SignalRow` を以下のフィールドを持つ JSON オブジェクトに変換する。空のフィールドは出力しない。

| WaveJSON フィールド | TCML 由来 |
|---|---|
| `name` | 信号名。複数行 (改行 `\n` 含む) は半角スペース 1 個で連結 (例: `"a\nb"` → `"a b"`) |
| `wave` | wave 文字列 (§wave 文字列マッピング) |
| `data` | Bus level 区間内のテキスト文字 1 個ごとに 1 要素。Bus 区間内にテキスト文字がない場合は省略 |
| `node` | 信号にアンカー (`@{name}` / `@N`) が含まれる場合のみ。長さは `wave` と一致、アンカー位置に node 文字、それ以外は `.` (§矢印) |
| `period` | 全信号 `step` の gcd を基準に算出。`period == 1` の場合は省略 (WaveDrom デフォルト) |

`phase` フィールドは TCML に対応概念なし、常に省略。

`SignalRow` のスタイル系フィールド (`signal_color`, `signal_width`, `bg`, `overline` 等) は §落とす要素 に従い無視する。

## wave 文字列マッピング

`SignalRow.waveform` の各 `WaveformElement` を WaveJSON 文字に置換し連結する。

| TCML `WaveformElement` | WaveJSON 文字 | 補足 |
|---|---|---|
| `LevelRun(Low, n)` | `0` + `.` × (n-1) | 多 unit hold は先頭 1 個 + `.` 反復 |
| `LevelRun(High, n)` | `1` + `.` × (n-1) | |
| `LevelRun(HiZ, n)` | `z` + `.` × (n-1) | |
| `LevelRun(Bus, n)` | `=` + `.` × (n-1) (1 セグメント保持) | 同区間内のテキストラベル全体を半角スペース連結した 1 文字列を `data` の対応エントリに置く (区間中央配置という TCML セマンティクスを保存) |
| `LevelRun(DontCareAlong*, n)` | `x` + `.` × (n-1) | 4 バリアント (Low/High/HiZ/Bus) すべて `x`。線位置は WaveDrom 側で再現不能 |
| `Transition(SingleEdge / BusOpen / BusClose / BusCross)` | (出力しない) | WaveDrom は隣接 wave 文字から自動的に決定する |
| `Gap` (`:`) | `\|` | WaveDrom `\|` は「直前 level を 1 unit 延長しつつ視覚的 break」のセマンティクス |
| `Guide` (`\|`、0 width) | (落とす) | WaveDrom に 0 幅 guide はない |
| `HighlightStart` (`[`) / `HighlightEnd` (`]`) | (落とす) | |
| `Anchor(@{...})` / `Anchor(@N)` | (wave には出力しない) | `node` 文字列に位置情報を保持 (§矢印) |
| `Text(c)` (Bus 区間内) | (wave には出力しない) | `data` 配列に蓄積 |
| `Text(c)` (非 Bus 区間内) | (落とす) | WaveDrom は `0`/`1`/`z`/`x` にラベルを付けられない |

### Bus 区間と `data` 配列の対応

TCML はバス区間内の複数のテキスト文字断片 (例: `=A=B=C` の `A`/`B`/`C`) を、parser の merge パスで連続同一レベル区間にまとめた上で、半角スペース連結した 1 個のラベル ("A B C") を区間中央に配置する。各 `=` ごとに別ラベルではなく **1 区間 = 1 中央ラベル** がセマンティクス。

WaveDrom 側でもこのセマンティクスに従う。各 Bus セグメント (`=` で始まり `.` で extend する 1 区間) は WaveDrom の `data` 配列の 1 エントリと対応し、その値は **TCML 側でその区間に集約されたテキスト全体** (parser が空白連結済みのもの) をそのまま入れる。

- TCML `==A==` (Bus 4 unit, テキスト "A") → wave `=...` data `["A"]`
- TCML `=A=B=C` (parser merge 後 Bus 3 unit, テキスト "A B C") → wave `=..` data `["A B C"]`
- TCML `=A=B=X=C=D` (Bus 2 + BusCross + Bus 3、後半は X body と merge) → wave `=.=..` data `["A B", "C D"]` (`Transition(BusCross)` は wave に出力しないが last_level をリセットするので 2 セグメント目は新しい `=` から開始)
- 全 Bus 区間にテキストが 1 つも無い場合: `data` フィールド自体を出力しない (省略)
- 一部の Bus 区間のみテキストありの場合: `data` 配列の対応位置を空文字列 `""` で埋める (例: 2 セグメントで先頭のみラベルなら `data=["", "X"]`)

### `X` (Bus cross marker) の扱い

`X` は parser 段階で `Transition(BusCross)` + `LevelRun(Bus, 1)` に分解されている (`docs/spec/types.md` §4.1)。本変換では `Transition` を出力しないため、`X` 由来の Bus body は次の Bus LevelRun と同様 `=` 1 文字 (場合によっては `data` 要素 1 個) として現れる。

## period / 時間軸正規化

TCML はローカルパラメータ `step` を信号ごとに変えられるため、各信号の 1 unit 視覚幅が異なりうる。WaveDrom 側は `signal.period` (自然数) で同等を表現する。

1. 各 `SignalRow` の有効 `step` (`Px`) を取得する (`step` の値は parser 段階で確定済みの想定)。
2. 各 `step` を `round` で 1 px 単位の整数に変換する。`round` で値が変化した信号があれば §警告 に従い stderr に 1 行出力する。
3. 整数化後の全 `step` の最大公約数 `g = gcd(step1, step2, ...)` を求める (`gcd` は 2 引数版を fold で適用、初期値 0)。
4. 各信号の `period = step / g`。
5. `period == 1` の信号は `period` フィールドを省略 (WaveDrom デフォルト = 1)。
6. `signal` 配列に信号オブジェクトが 0 個、または整数化後の全 `step` が 0 の場合は `period` 計算を省略 (全信号で `period` フィールド省略)。

`step <= slant` の境界処理は parser 側で `@step` 変更時に slant を自動クランプする (`docs/spec/tcml-format.md` §`@step` / §`@slant` 参照)。WaveDrom 出力に `slant` 概念がない (period 計算は `step` のみを使う) ため、本変換側ではこの調整に追加対応する必要はない。

## clock 自動展開

`@clock(...)` 装飾付き `SignalRow` は次のルールで wave 文字列を生成する。

1. 編集子が **デフォルト** (`_=1, ~=1`、つまり編集子が省略されているか共に 1) かつ `edge` ∈ {`pos`, `neg`} の場合のみ:
   - `edge=pos` → wave = `p` + `.` × (chart_units - 1)
   - `edge=neg` → wave = `n` + `.` × (chart_units - 1)
2. それ以外 (`_=N` / `~=N` 指定あり、または `edge` ∈ {`both`, `none`}): TCML parser の clock 展開後の wave (LevelRun 列) を §wave 文字列マッピング に従って `0`/`1` 列として出力する。
3. clock 三角形マーカー (`mark_position`, `mark_height`, `mark_width`, `mark_color`) は WaveDrom に対応物なし、落とす。

`p` / `n` を使う場合の `period` 計算は §period と同じ (clock 信号も他信号と同じ `step` を持つ)。

## アンカーと矢印

矢印 (`@->`) はアンカー (`@{name}` / `@N`) を端点とする。WaveDrom は `signal[].node` プロパティと top-level `edge` 配列で同等を表現する。

### node 文字割り当て

1. `ChartDocument.annotations.anchors` (`AnchorRegistry`) から、`@->` の端点として実際に参照されているアンカーを抽出する。`@->` から参照されないアンカーは node を割り当てない。
2. アンカーを TCML 出現順 (信号行順 + 信号内出現順) で列挙し、WaveDrom node 文字を割り当てる。文字順は `a`, `b`, ..., `z`, `A`, `B`, ..., `Z` の 52 個。
3. アンカー数が 52 を超えたら §警告 に従い stderr に 1 行出力し、超過分のアンカーを参照する `@->` を `edge` 配列から落とす (該当 node 文字割り当ても行わない)。
4. 各 `SignalRow` について、その信号にアンカーが 1 個以上含まれる場合のみ `node` フィールドを出力する。文字列長は wave 文字列と一致、アンカー位置には対応 node 文字、それ以外の位置には `.` を置く。

### edge 配列

`ChartDocument.annotations.arrows` (= `@->` 由来のみ、clock 由来 EdgeMark は混入しない契約。詳細は `types.md` §6.6) を出現順に走査する。各 `Arrow` を WaveJSON `edge` 文字列にエンコードし `edge` 配列に追加する。

書式: `"<from_node><style><to_node>[ <label>]"`

style は次の表から決定する。

| TCML 線種 | TCML 矢印頭 | WaveJSON style | 備考 |
|---|---|---|---|
| `solid` | `end` | `->` | デフォルト |
| `solid` | `both` | `<->` | |
| `solid` | `none` | `-` | |
| `dashed` | `end` | `-~>` | WaveDrom には dashed 直線がないため curve `~` 系で近似 |
| `dashed` | `both` | `<-~>` | |
| `dashed` | `none` | `-~` | |
| `dotted` | (任意) | `dashed` と同じ | WaveDrom は dotted/dashed を区別しない、近似 |

ラベル (`@->` 第 4 トークン以降のテキスト) があれば WaveJSON edge 文字列の末尾に半角スペースを挟んで連結する。

色 (`red` 等)・太さ (`2px` 等) は WaveDrom edge には設定経路がないため落とす (§落とす要素)。

### 例

TCML 入力:
```tcml
clk : ~_~_@{a}_~_
data: ==X==@{b}==
@-> (@{a}, @{b}) start
```

WaveJSON 出力 (整形後):
```json
{
  "signal": [
    { "name": "clk",  "wave": "1010101", "node": ".....a." },
    { "name": "data", "wave": "=.=.=.=.", "data": ["", "", "", ""], "node": "....b..." }
  ],
  "edge": ["a->b start"]
}
```

(本例は対応関係を示す概念図、実際の `wave` 構築結果と完全一致するとは限らない。)

## 落とす要素

WaveDrom に対応物なし。出力時に warning は出さない (TCML 文書として有効、WaveDrom 出力にはこれらの情報が含まれないことを利用者が許容する想定)。

- `@->` の色・太さ
- `@bg` / `@bgcolor0` / `@bgcolor1`
- `@signal(overline)` (信号名上線)
- `@titlealign`
- `@dontcare_color` / `@highlight_style`
- `@font` / `@fontsize` / `@lineheight` 等のフォント・サイズ系
- `@scale` / `@page-margin` / `@capwidth` / `@namepad` 等のレイアウト系
- `@step` / `@slant` のローカル変更 (period 計算に使うが、WaveDrom 側に直接表現する場所はない)
- `@clock` の `mark_*` オプション (clock 三角形マーカー)
- `[` / `]` ハイライト区間
- `\|` (Guide, 0 width)
- `%` 文字書き込み行
- 非 Bus 区間内のテキスト文字
- `@->` から参照されないアンカー
- 2 個目以降の `@title` (WaveDrom `head.text` は単一文字列のため、最初の 1 個のみ保存され、それ以降は欠落する。欠落時は §警告 に従い stderr に 1 行出力する)

## 警告

stderr に 1 行ずつ出す。終了コードには影響しない (常に 0)。

| 警告条件 | メッセージ書式 |
|---|---|
| `step` の `round` で値が変化 | `warning: signal "<name>" step rounded: <orig> -> <rounded>` |
| アンカー数が 52 超過 | `warning: more than 52 anchors; edges referencing extra anchors are dropped` |
| 2 個目以降の `@title` が欠落 | `warning: only the first @title is kept; <N> additional @title row(s) dropped` |

## エラー

WaveDrom 変換段階では parser エラー以外を発しない設計とする (parser を通った `ChartDocument` は変換可能とみなす)。出力 JSON の書き出し失敗 (CLI 層) は CLI 終了コード 3 (`docs/spec/cli.md` §終了コード) に従う。

## 参考

WaveDrom セマンティクス調査の出典:
- WaveJSON tutorial: <https://wavedrom.com/tutorial.html>
- WaveJSON schema: <https://github.com/wavedrom/schema/blob/master/WaveJSON.md>
- WaveJSON wiki: <https://github.com/wavedrom/wavedrom/wiki/wavejson>
