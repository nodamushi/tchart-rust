# TCML パーサー

TCML テキストを解析して `ChartDocument` を生成する機能のテスト仕様。
実装構造とアルゴリズム概要は [`docs/spec/types.md`](../spec/types.md)、構文は [`docs/spec/tcml-format.md`](../spec/tcml-format.md) を参照。

---

## 基本的な行のパース

## @not-implemented @smoke
### Scenario: コメント行が無視される
- Given `// これはコメントです` という行を含む TCML テキストがある
- When パーサーでパースする
- Then `ChartDocument.lines` および `Annotations` にコメント行は含まれない

## @not-implemented
### Scenario: タイミング記述行の行末 `//` コメントは無視される
- Given `A _~_~ // 末尾コメント` という行がある
- When パーサーでパースする
- Then 信号名 `A` と波形 `_~_~` のみがパースされ、`// 末尾コメント` 以降は破棄される
- And エラーにならない

## @not-implemented
### Scenario: パラメータ行の行末 `//` コメントは無視される
- Given `@step 10 // 単位` という行がある
- When パーサーでパースする
- Then `@step 10` と等価に処理され、`// 単位` 以降は破棄される
- And エラーにならない

## @not-implemented
### Scenario: 文字書き込み行の行末 `//` コメントは無視される
- Given `%5 hello // 注釈` という行がある
- When パーサーでパースする
- Then 文字書き込み行は `%5 hello` として処理され、`// 注釈` 以降は破棄される
- And エラーにならない

## @not-implemented
### Scenario: 行中の `//` 複数出現は最初の `//` 以降をすべて破棄
- Given `A _~_~ // foo // bar` という行がある
- When パーサーでパースする
- Then 最初の `//` 以降が行末まで破棄され、`// foo // bar` 全体がコメントとして無視される
- And 波形は `_~_~` のみ

## @not-implemented @smoke
### Scenario: 空行が無視される
- Given 空行を含む TCML テキストがある
- When パーサーでパースする
- Then エラーにならず正常にパースされる

## @not-implemented @smoke
### Scenario: 空ファイルのパース
- Given 内容が空の TCML テキストがある
- When パーサーでパースする
- Then エラーにならず、`lines` が空の `ChartDocument` が返る

## @not-implemented @negative
### Scenario: 信号名のみでレベル文字列が空
- Given `SigA ` というレベル文字列が空の行がある
- When パーサーでパースする
- Then `ParseError::InvalidLevelChar` または空 `Waveform` の `SignalRow` (仕様で要決定) となる

---

## SignalName

## @not-implemented
### Scenario: 単一行信号名
- Given `Clock _~_~` という行がある
- When パーサーでパースする
- Then `SignalRow.name` が `SignalName("Clock")` となる

## @not-implemented
### Scenario: 複数行信号名 (`"..."`)
- Given `"Data\nBus" ========` という行がある
- When パーサーでパースする
- Then `SignalRow.name` が `SignalName("Data\nBus")` となり、`name.lines()` が 2 行を返す

## @not-implemented
### Scenario: 引用内エスケープ `\"` `\n` `\\`
- Given `"a\"b\n\\c" _~` という行がある
- When パーサーでパースする
- Then `SignalRow.name` が `SignalName("a\"b\n\\c")` となる

## @not-implemented @negative
### Scenario: 制御文字を含む信号名はエラー
- Given 信号名に `\t` を含む TCML がある
- When パーサーでパースする
- Then `ParseError::InvalidSignalName` となる

## @not-implemented @negative
### Scenario: 閉じない `"..."` はエラー
- Given `"Data Bus _~_~` (閉じない `"`) という行がある
- When パーサーでパースする
- Then `ParseError::UnclosedQuote` となる

---

## レベル記号のパース

## @not-implemented
### Scenario: 単一レベル記号
- Given `SigA ____` という行がある
- When パーサーでパースする
- Then `Waveform.elements` が `[LevelRun(Low, 4)]` の 1 要素となる

## @not-implemented
### Scenario: ハイレベル `~`
- Given `SigA ~~~~` という行がある
- When `Waveform.elements` が `[LevelRun(High, 4)]` となる

## @not-implemented
### Scenario: HiZ `-`
- Given `SigA ----` という行がある
- Then `[LevelRun(HiZ, 4)]` となる

## @not-implemented
### Scenario: Bus `=`
- Given `SigA ====` という行がある
- Then `[LevelRun(Bus, 4)]` となる

## @not-implemented
### Scenario: 異種レベルの隣接で Transition が挿入される
- Given `SigA _~` という行がある
- When パーサーでパースする
- Then `Waveform.elements` が `[LevelRun(Low,1), Transition(SingleEdge, Low→High), LevelRun(High,1)]` の 3 要素となる

## @not-implemented @negative
### Scenario: クォート外の制御文字は InvalidLevelChar
- Given `SigA _\x01_~` (制御文字 `\x01` を含む) という行がある
- When パーサーでパースする
- Then `ParseError::InvalidLevelChar` となる (テキスト文字としても解釈不能)

---

## DontCare (`?`) 解決

`?` は幅 0 のマーカー。`?` を含む連続レベル区間全体を 1 つの不定値領域として塗る。
領域 width = 同種レベル文字 (`_`/`~`/`-`/`=`) と X の累積。`?` 自体はピクセルを持たない。
`X` / 別レベル / `:` / 行端で打ち切り。透過マーカー (`@{...}`、`@N`、`|`、`[`、`]`) は範囲計算上スキップ。

## @not-implemented @smoke
### Scenario: `_?_` は `_` 2 個の `DontCareAlongLow,2` にマージ (`?` は 0 幅)
- Given `SigA _?_` という行がある
- When パーサーでパースする
- Then `elements` が `[LevelRun(DontCareAlongLow, 2)]` となる
- And `?` の前後に `Transition` が **挿入されない**

## @not-implemented
### Scenario: `~?~` は `DontCareAlongHigh,2` にマージ (`?` は 0 幅)
- Given `SigA ~?~` という行がある
- Then `elements` が `[LevelRun(DontCareAlongHigh, 2)]` となる

## @not-implemented
### Scenario: `-?-` は `DontCareAlongHiZ,2` にマージ (`?` は 0 幅)
- Given `SigA -?-` という行がある
- Then `elements` が `[LevelRun(DontCareAlongHiZ, 2)]` となる

## @not-implemented
### Scenario: `=?=` は `DontCareAlongBus,2` にマージ (`?` は 0 幅)
- Given `SigA =?=` という行がある
- Then `elements` が `[LevelRun(DontCareAlongBus, 2)]` となる

## @not-implemented
### Scenario: 連続 `?` は同一 LevelRun にマージ (前後 `_` も取り込み、`?` は 0 幅)
- Given `SigA _???_` という行がある
- Then `elements` が `[LevelRun(DontCareAlongLow, 2)]` となる (前後の `_` 2 個のみ幅に寄与)

## @not-implemented
### Scenario: `==?==` は前後の `=` を取り込み 4 単位の DontCareAlongBus (`?` は 0 幅)
- Given `SigA ==?==` という行がある
- Then `elements` が `[LevelRun(DontCareAlongBus, 4)]` となる

## @not-implemented
### Scenario: `____????` は Low 4 単位の DontCareAlongLow (`?` は 0 幅)
- Given `SigA ____????` という行がある
- Then `elements` が `[LevelRun(DontCareAlongLow, 4)]` となる

## @not-implemented
### Scenario: 直前のみで決定 (`_?=` は別レベルで前方打ち切り、`?` は 0 幅)
- Given `SigA _?=` という行がある
- Then `elements` が `[LevelRun(DontCareAlongLow, 1), LevelRun(Bus, 1)]` となる
- And `?` 領域 = `_` 1 個のみ (`?` は 0 幅)
- And `=` は別レベルで打ち切り (`?↔=` 境界に Transition なし)

## @not-implemented
### Scenario: `_~?` は前方の `~` を取り込み、`_` は別レベルで打ち切り (`?` は 0 幅)
- Given `SigA _~?` という行がある
- Then `elements` が `[LevelRun(Low, 1), LevelRun(DontCareAlongHigh, 1)]` となる

## @not-implemented
### Scenario: `_=?=_` (Low → Bus + ? + Bus → Low、両端 BusOpen/BusClose)
- Given `SigA _=?=_` という行がある
- Then `elements` が `[LevelRun(Low,1), Transition(BusOpen, Low→Bus), LevelRun(DontCareAlongBus,2), Transition(BusClose, Bus→Low), LevelRun(Low,1)]` となる
- And ? 領域 = 中央 Bus 2 単位 (X 含まず、`?` は 0 幅、左右の `=` を取り込む)

## @not-implemented
### Scenario: `--==?==` (HiZ → BusOpen + ? を含む Bus、信号末で打ち切り)
- Given `SigA --==?==` という行がある
- Then `elements` が `[LevelRun(HiZ,2), Transition(BusOpen, HiZ→Bus), LevelRun(DontCareAlongBus,4)]` となる
- And ? 領域 = Bus 4 単位 (前後の `=` を取り込む、HiZ は別レベルで打ち切り)

## @not-implemented
### Scenario: `==?==--` (信号始端 Bus + ? + Bus → HiZ)
- Given `SigA ==?==--` という行がある
- Then `elements` が `[LevelRun(DontCareAlongBus,4), Transition(BusClose, Bus→HiZ), LevelRun(HiZ,2)]` となる

## @not-implemented @negative
### Scenario: 信号行先頭の `?` はエラー
- Given `SigA ?==` という行がある
- When パーサーでパースする
- Then `ParseError::DontCareWithoutAnchor` となる

## @not-implemented @negative
### Scenario: 透過要素のみ先行する `?` はエラー
- Given `SigA :?_~` という行がある
- Then `ParseError::DontCareWithoutAnchor` となる

## @not-implemented @negative
### Scenario: アンカーのみ先行する `?` はエラー
- Given `SigA @{a}?_~` という行がある
- Then `ParseError::DontCareWithoutAnchor` となる

## @not-implemented @negative
### Scenario: `?` のみの信号行はエラー
- Given `SigA ???` という行がある
- Then `ParseError::DontCareWithoutAnchor` となる

---

## `X` (BusCross) と `X?` パターン

## @not-implemented
### Scenario: `=X=` は BusCross Transition + X body (新値 Bus) を生成
- Given `SigA =X=` という行がある
- Then `elements` が `[LevelRun(Bus,1), Transition(BusCross, Bus→Bus), LevelRun(Bus,2)]` となる
- And X body (新値 Bus 1 単位) と後続 `=` (1 単位) は同値で merge され `LevelRun(Bus, 2)` になる

## @not-implemented
### Scenario: `=X?` は X 後の bus 区間を dontcare に (`?` は 0 幅、X body 1 単位を dontcare として塗る)
- Given `SigA =X?` という行がある
- Then `elements` が `[LevelRun(Bus,1), Transition(BusCross, Bus→DontCareAlongBus), LevelRun(DontCareAlongBus,1)]` となる
- And X 自体は省略されない (cross + body 構造を保持)

## @not-implemented
### Scenario: `=X?=` の ? 領域 = X body (1) + 後 `=` (1) = 2 単位 DontCareAlongBus
- Given `SigA =X?=` という行がある
- Then `elements` が `[LevelRun(Bus, 1), Transition(BusCross, Bus→DontCareAlongBus), LevelRun(DontCareAlongBus, 2)]` となる
- And X 後ろの bus 区間 (X body 1 単位 + 後 `=` 1 単位) が `?` 領域として merge され、polygon は X cross 中点 〜 信号末端まで伸びる

## @not-implemented
### Scenario: `=?X=` パターン (`?` は前の `=` を取り込み、X body は別 Bus 値、`?` は 0 幅)
- Given `SigA =?X=` という行がある
- Then `elements` が `[LevelRun(DontCareAlongBus, 1), Transition(BusCross, DontCareAlongBus→Bus), LevelRun(Bus, 2)]` となる
- And `?` 領域 = 前 `=` 1 単位、X body + 後 `=` は新値 Bus で merge され `LevelRun(Bus, 2)`

## @not-implemented
### Scenario: `=X?X=` パターン (X1 と X2 の間の dontcare、`?` は 0 幅)
- Given `SigA =X?X=` という行がある
- Then `elements` が `[LevelRun(Bus, 1), Transition(BusCross, Bus→DontCareAlongBus), LevelRun(DontCareAlongBus, 1), Transition(BusCross, DontCareAlongBus→Bus), LevelRun(Bus, 2)]` となる
- And `?` 領域 = X1 body 1 単位、X2 で打ち切り、polygon は X1 cross 中点 〜 X2 cross 中点 (六角形 `>▲■▲<`)

## @not-implemented
### Scenario: `XXXX` (信号行頭の連続 X) は valid、最初の X は cross 省略
- Given `SigA XXXX` という行がある
- Then `elements` が `[LevelRun(Bus, 1), Transition(BusCross), LevelRun(Bus, 1), Transition(BusCross), LevelRun(Bus, 1), Transition(BusCross), LevelRun(Bus, 1)]` となる
- And 最初の X は cross を生成せず body のみ、2 つ目以降は通常の BusCross + 新値 body
- And パーサーはエラーを出さない

## @not-implemented
### Scenario: `X==` (信号行頭の X) は valid、X body と後続 `=` が merge
- Given `SigA X==` という行がある
- Then `elements` が `[LevelRun(Bus, 3)]` となる (X body 1 単位 + 後続 `==` 2 単位が merge)
- And パーサーはエラーを出さない (X 信号行頭で cross 省略、Transition(BusCross) も生成しない)

## @not-implemented @negative
### Scenario: `?X=` は先頭 `?` でエラー
- Given `SigA ?X=` という行がある
- Then `ParseError::DontCareWithoutAnchor` となる

## @not-implemented
### Scenario: `~X_` (High → X → Low) は valid、暗黙の BusOpen / BusClose 遷移を補完
- Given `SigA ~X_` という行がある
- Then `elements` が `[LevelRun(High,1), Transition(BusOpen, High→Bus), LevelRun(Bus,1), Transition(BusClose, Bus→Low), LevelRun(Low,1)]` となる
- And X 自体は cross 遷移を生成しない (前 bus が無いため、BusOpen が代わりに挿入される)

## @not-implemented
### Scenario: `_X~` (Low → X → High) は valid
- Given `SigA _X~` という行がある
- Then `elements` が `[LevelRun(Low,1), Transition(BusOpen, Low→Bus), LevelRun(Bus,1), Transition(BusClose, Bus→High), LevelRun(High,1)]` となる

## @not-implemented
### Scenario: `_____X=====` (Low → X → Bus、X body は後続 `=====` に merge)
- Given `SigA _____X=====` という行がある
- Then `elements` が `[LevelRun(Low,5), Transition(BusOpen, Low→Bus), LevelRun(Bus,6)]` となる
- And X body 1 単位 + 後続 `=====` 5 単位が同値 Bus で merge され `LevelRun(Bus, 6)`

## @not-implemented
### Scenario: `=====X_____` (Bus → X → Low、X cross + body + BusClose)
- Given `SigA =====X_____` という行がある
- Then `elements` が `[LevelRun(Bus,5), Transition(BusCross), LevelRun(Bus,1), Transition(BusClose, Bus→Low), LevelRun(Low,5)]` となる
- And X は通常の cross + body、後続 non-bus には BusClose を挿入

---

## 補助記号

## @not-implemented
### Scenario: Gap (`:`)
- Given `SigA __:__` という行がある
- Then `elements` に `Gap` 要素が含まれ、前後の `LevelRun(Low,2)` 同士は **マージされない**

## @not-implemented
### Scenario: Guide (`|`)
- Given `SigA __|__` という行がある
- Then `elements` に `Guide` が含まれる
- And Guide は LevelRun のマージに影響しない (前後の `Low,2` は別 LevelRun のまま)

## @not-implemented
### Scenario: Highlight (`[ ]`)
- Given `SigA __[~~]__` という行がある
- Then `elements` に `HighlightStart`, `HighlightEnd` が適切な位置に挿入される

## @not-implemented @negative
### Scenario: 閉じない `[` はエラー
- Given `SigA __[~~__` という行がある
- Then `ParseError::UnclosedHighlight` となる

## @not-implemented @negative
### Scenario: 単独 `]` はエラー
- Given `SigA __~~]__` という行がある
- Then `ParseError::UnopenedHighlightEnd` となる

## @not-implemented @negative
### Scenario: ネストした `[` はエラー
- Given `SigA [_[~]]` という行がある
- Then `ParseError::UnclosedHighlight` または専用エラー (仕様で決定) となる

---

## レベル文字列中のテキスト文字 (筑波大 tchart-coffee 方式)

レベル文字列中の非レベル・非特殊文字は所属する連続同一レベル区間の中央に描画されるテキストになる (`tcml-format.md` §「レベル文字列中のテキスト文字」)。

## @not-implemented
### Scenario: 同区間内の単一テキスト文字 (連続性維持)
- Given `SigA __a__` という行がある
- When パーサーでパースする
- Then `Waveform.elements` に 4 単位 Low の `LevelRun` 1 つと所属するテキスト `Text("a")` が含まれる (テキスト文字は連続性を切らないため Low が分断されない)

## @not-implemented
### Scenario: 同区間内の複数テキスト断片は空白結合
- Given `SigA __a__b_` という行がある
- When パーサーでパースする
- Then 5 単位 Low 区間に所属する `Text("a b")` が含まれる (`a` と `b` の重なりを避けるため空白結合)

## @not-implemented
### Scenario: 異なる区間に属する複数テキスト
- Given `SigA __ack__~~done~~` という行がある
- When パーサーでパースする
- Then 4 単位 Low 区間に `Text("ack")`、4 単位 High 区間に `Text("done")` が含まれる

## @not-implemented
### Scenario: 行末トレーリングテキストは直前区間に所属
- Given `SigA ~~~~~~~~かきくけこ` という行がある
- When パーサーでパースする
- Then 8 単位 High 区間に `Text("かきくけこ")` が所属する (後続 level char 無しでも直前区間に帰属)

## @not-implemented
### Scenario: BusCross 直後のテキストは遷移先 Bus 区間に所属
- Given `SigA ==Xa==` という行がある
- When パーサーでパースする
- Then `==X=a=` と等価の解釈になり、X 後ろの 2 単位 Bus 区間に `Text("a")` が含まれる

## @not-implemented @negative
### Scenario: 行頭テキストは MissingInitialLevel
- Given `SigA a__~~` という行がある
- When パーサーでパースする
- Then `ParseError::MissingInitialLevel` となる

## @not-implemented
### Scenario: クォートで level 記号を literal 化 (BusCross 抑制)
- Given `SigA =="X"==` という行がある
- When パーサーでパースする
- Then 4 単位 Bus 連続区間に `Text("X")` が含まれ、BusCross は生成されない

## @not-implemented
### Scenario: クォートで空白を含む文字列
- Given `SigA __"hello world"__` という行がある
- When パーサーでパースする
- Then 4 単位 Low 区間に `Text("hello world")` が含まれる (空白込みの 1 文字列)

## @not-implemented
### Scenario: クォートで複数 level 記号を literal 化
- Given `SigA __"_~="__` という行がある
- When パーサーでパースする
- Then 4 単位 Low 区間に `Text("_~=")` が含まれる (内部記号は波形にならない)

## @not-implemented
### Scenario: クォートで特殊記号を literal 化
- Given `SigA __"[@|]"__` という行がある
- When パーサーでパースする
- Then 4 単位 Low 区間に `Text("[@|]")` が含まれる (Highlight / Anchor / Guide にならない)

## @not-implemented
### Scenario: bare と quoted の混在は空白結合
- Given `SigA __a"b c"d__` という行がある
- When パーサーでパースする
- Then 4 単位 Low 区間に `Text("a b c d")` が含まれる (3 断片を空白結合)

## @not-implemented
### Scenario: 連続性を切らないテキスト混在区間 merge
- Given `SigA =="a"=="b"==` という行がある
- When パーサーでパースする
- Then 6 単位 Bus 連続区間に `Text("a b")` が含まれる (テキストは連続性を切らないため merge される)

## @not-implemented
### Scenario: 単独 `/` はレベル文字列・信号名・パラメータ値で通常文字として受理
- Given `Sig/A _/_/` という信号行と `@title a/b/c` というパラメータ行
- When パーサーでパースする
- Then 信号名 `Sig/A` を持つ `SignalRow` が生成され、レベル文字列中の `/` は所属区間に対応する bare text として扱われる
- And `ParseError::InvalidLevelChar('/')` を返さない
- And `@title a/b/c` は `UserText("a/b/c")` として受理される

## @not-implemented
### Scenario: `#` はレベル文字列・信号名・パラメータ値で通常文字として受理 (回帰防止)
- Given `Sig#A _#_#` という信号行と `@title #ff0000` というパラメータ行
- When パーサーでパースする
- Then 信号名 `Sig#A` を持つ `SignalRow` が生成され、レベル文字列中の `#` は所属区間に対応する bare text として扱われる
- And `ParseError::InvalidLevelChar('#')` を返さない
- And `@title #ff0000` は `UserText("#ff0000")` として受理される

## @not-implemented
### Scenario: クォート内の `//` は literal text として保持される
- Given `SigA __"// note"__` という行と `"// a" _~_~` という信号名にクォート `//` を含む行
- When パーサーでパースする
- Then `__"// note"__` 行は 4 単位 Low 区間の中央に `Text("// note")` が含まれる (コメント切断されない)
- And `"// a" _~_~` 行は信号名 `// a` (literal) を持つ `SignalRow` が生成される

## @not-implemented
### Scenario: クォート内外で `//` が混在する場合はクォート外初出の `//` のみコメント切断
- Given `SigA __"// keep"__ // drop` という行がある
- When パーサーでパースする
- Then 4 単位 Low 区間の中央に `Text("// keep")` が含まれる (クォート内 `//` は literal として保持)
- And クォート閉じ後の `// drop` 以降は行末コメントとして破棄される

## @not-implemented @negative
### Scenario: クォート未閉鎖は UnclosedQuote
- Given `SigA __"hello__` という行がある
- When パーサーでパースする
- Then `ParseError::UnclosedQuote` となる

## @not-implemented @negative
### Scenario: クォート未閉鎖は `//` 検出より優先
- Given `SigA __"hello // world__` という行がある (クォート未閉じかつ `//` を含む)
- When パーサーでパースする
- Then `ParseError::UnclosedQuote` を返す (`//` 以降をコメントとして破棄せず、`UnclosedQuote` 優先)

## @not-implemented
### Scenario: 信号名中の `<` `>` は文字として通る
- Given `"<request>" _~_~` という行がある
- Then 信号名 `<request>` (文字どおり 9 文字) を持つ `SignalRow` が生成される

## @not-implemented
### Scenario: 矢印ラベル中の `<` `>` は文字として通る
- Given `@-> (@{a}, @{b}) <signal-set>` という行がある
- Then `Arrow.label = Some("<signal-set>")` (文字どおり 12 文字) となる

---

## アンカー埋め込み (`@{name}` / `@N`)

## @not-implemented
### Scenario: 名前付きアンカー
- Given `SigA _~@{edge}_` という行がある
- Then `elements` に `Anchor(Named("edge"))` が含まれる

## @not-implemented
### Scenario: 番号付きアンカー
- Given `SigA ___@1__` という行がある
- Then `elements` に `Anchor(Indexed(1))` が含まれる

## @not-implemented
### Scenario: 名前付きと番号付きは別名前空間
- Given `@{1}` と `@1` を同一ファイル内で使う
- Then 重複エラーにならない

## @not-implemented @negative
### Scenario: 同一名前付きアンカー重複
- Given 同一信号行内で `@{a}` を 2 回使う
- Then `ParseError::DuplicateAnchor` となる

## @not-implemented @negative
### Scenario: 同一番号付きアンカー重複
- Given `@1` を 2 回使う
- Then `ParseError::DuplicateAnchor` となる

## @not-implemented @negative
### Scenario: 不正アンカー名
- Given `@{1abc}` (数字始まり) を使う
- Then `ParseError::InvalidAnchorName` となる

##
### Scenario: 遷移後アンカーの x 座標が遷移幅を含む
- Given `@step 10` `@slant 2`、信号行 `SigA ___@{a}~~~~@{b}___` をパース・レイアウト
- When `layout()` を実行する
- Then アンカー `@{a}` の x 座標は 30 (Low×3、step×3)
- And アンカー `@{b}` の x 座標は 30 + 2 + 38 = 70 (Low×3 + Transition(slant=2) + High×4 で High 区間幅は `4×step - slant = 38`)

##
### Scenario: Bus 行の X 遷移後アンカーの x 座標が BusCross 幅を含む
- Given `@step 10`、`@slant 2`、信号行 `Bus =====@1X====@2` をパース・レイアウト (X 全体で 1 step を消費、cross 部 `slant=2` + body 部 `step - slant = 8`)
- When `layout()` を実行する
- Then アンカー `@1` の x 座標は 50 (Bus×5)
- And アンカー `@2` の x 座標は 50 + 2 + 48 = 100 (Bus×5 + BusCross(slant=2) + 後続 Bus×5 で Bus 区間幅は `5×step - slant = 48`、X body 1 単位 + `====` 4 単位がマージされた区間)

##
### Scenario: `step <= slant` をパースエラーで弾く
- Given `@step 2` `@slant 2` (step が slant 以下) のパラメータ行
- When パースを実行する
- Then `ParseError::InvalidStepSlant` が返る (level の hold 部分が 0 以下になり波形を構築できないため、属性 set 時点でエラー)
- And 同様に `@step 10` の後で `@slant 10` または `@slant 11` を指定した場合もエラー (検査は両属性のうち後に変更された側のタイミングで行う)

---

## パラメータ行

## @not-implemented
### Scenario: グローバルパラメータ `@fontsize`
- Given `@fontsize 16` というパラメータ行がある
- Then `ChartStyle.canvas.font.size` が `Px(16.0)` となる

## @not-implemented
### Scenario: 表記揺れ (大文字小文字) — `@FontSize`
- Given `@FontSize 16` というパラメータ行がある
- Then `@fontsize 16` と同じ動作をする

## @not-implemented
### Scenario: 表記揺れ (`-` / `_`) — `@font-size`, `@font_size`
- Given `@font-size 16` および `@font_size 16` というパラメータ行がある
- Then `@fontsize 16` と同じ動作をする

## @not-implemented
### Scenario: ローカルパラメータの途中変更
- Given `@step 10` の後に信号行 1 つ、続けて `@step 14` 信号行 1 つ
- Then 1 つ目の信号は step=10、2 つ目は step=14 でレイアウトされる
- And 旧名 `@w_hold` / `@w_transient` は廃止されており、`ParseError::UnknownParameter` で弾かれる

## @not-implemented
### Scenario: `@bg` は次行限り
- Given `@bg #f0f` の後に信号行 A、信号行 B
- Then A の `Line.background == Some(#f0f)`、B の `Line.background == None`

## @not-implemented
### Scenario: `@bg` は行種別を問わず次の 1 行に適用
- Given `@bg #ff0` の後に `@title "X"`、続けて信号行 A
- Then Title 行の `Line.background == Some(#ff0)`、信号行 A の `Line.background == None`

## @not-implemented
### Scenario: `@bg` は他のディレクティブを跨いで保持される
- Given `@bg #f0f` の後に `@bgcolor0 #eee`、続けて信号行 A
- Then 信号行 A の `Line.background == Some(#f0f)` (間の `@bgcolor0` で消費されない)

## @not-implemented
### Scenario: `@bg none` は保留中の値を破棄
- Given `@bg #f0f` の後に `@bg none`、続けて信号行 A
- Then 信号行 A の `Line.background == None`

## @not-implemented @negative
### Scenario: 未知パラメータ
- Given `@foobar 42` という行がある
- Then `ParseError::UnknownParameter` となる

## @not-implemented @negative
### Scenario: 不正な色値
- Given `@signal_color notacolor` という行がある
- Then `ParseError::InvalidColor` となる

---

## `@skip`

## @not-implemented
### Scenario: `@skip(2)` は 2 lh の SkipRow を生成
- Given `@skip(2)` という行がある
- Then `LineContent::Skip(SkipRow { amount: Length::Lh(2.0) })` が `lines` に追加される

## @not-implemented
### Scenario: `@skip(2.5)` は小数 lh を許可
- Given `@skip(2.5)` という行がある
- Then `Length::Lh(2.5)` で SkipRow が生成される

## @not-implemented
### Scenario: `@skip(20px)` は px 単位
- Given `@skip(20px)` という行がある
- Then `Length::Px(20.0)` で SkipRow が生成される

## @not-implemented
### Scenario: `@skip(0)` は無視される
- Given `@skip(0)` という行がある
- Then `lines` に SkipRow が **追加されない** (要素数が増えない)

## @not-implemented @negative
### Scenario: 負の skip はエラー
- Given `@skip(-1)` という行がある
- Then `ParseError::InvalidSkipAmount` となる

## @not-implemented @negative
### Scenario: パース不能な skip はエラー
- Given `@skip(abc)` という行がある
- Then `ParseError::InvalidSkipAmount` となる

---

## `@title`

## @not-implemented
### Scenario: 単一行タイトル
- Given `@title 同期回路` という行がある
- Then `LineContent::Title(TitleRow { text: UserText("同期回路") })` が `lines` に追加される

## @not-implemented
### Scenario: 複数行タイトル (`"..."`)
- Given `@title "クロック\n同期回路"` という行がある
- Then タイトル本文が複数行 `UserText` として保持される

## @not-implemented
### Scenario: 複数の `@title` が許可される
- Given ファイル中に `@title A` と `@title B` が出現する
- Then `lines` に Title が 2 つ含まれる

## @not-implemented
### Scenario: `@title #ff0000` をクォートなしで受理 (色値ユースケース)
- Given `@title #ff0000` という行がある
- When パーサーでパースする
- Then `LineContent::Title(TitleRow { text: UserText("#ff0000") })` が `lines` に追加される
- And `#` は新仕様で特殊扱いされないため、クォートで囲まなくても色値風文字列をタイトル本文として受理する

---

## `@titlealign`

## @not-implemented
### Scenario: `@titlealign center|left|right` を受理
- Given `@titlealign left` という行がある
- Then パーサーは `HorizontalAlign::Left` として保存し、後続 `@title` の `style.align` に反映する

## @not-implemented
### Scenario: 大文字小文字を区別しない
- Given `@titlealign CENTER` / `@titlealign Right` のような表記
- Then すべて該当 `HorizontalAlign` 値として受理される

## @not-implemented
### Scenario: 不正値はパースエラー
- Given `@titlealign top` / `@titlealign middle` のような未知値
- Then 行番号付きパースエラーとなる

## @not-implemented
### Scenario: 途中変更が後続 `@title` に適用
- Given `@title A` → `@titlealign right` → `@title B` という並び
- Then "A" は `HorizontalAlign::Center` (デフォルト)、"B" は `HorizontalAlign::Right`

---

## `@clock`

## @not-implemented
### Scenario: `@clock(pos)` の後の信号行はクロックとして展開
- Given `@clock(pos)` の後に `CLK` (本体空) という行があり、他信号の最大長が 8 単位
- When パーサーでパースし clock 展開パスを実行する
- Then `CLK` 信号の `waveform.elements` が `_~_~_~_~` 相当 (Low/High 各 1 単位 × 4 周期) となる

## @not-implemented
### Scenario: `@clock(neg, _=2, ~=3)` のパルス指定
- Given `@clock(neg, _=2, ~=3)` の後に空クロック信号、他信号の最大長 10 単位
- Then クロック信号は Low(2) → High(3) を 2 周期展開され、最後 5 単位余る → Low(2)+High(3) で埋める

## @not-implemented
### Scenario: 部分指定からの自動繰り返し
- Given `@clock(pos)` の後に `ck ~~__`、他信号最大長 8 単位
- Then `ck` の続きは最後の状態 `_` から `~_~_` を繰り返し追加し合計 8 単位となる

## @not-implemented @smoke
### Scenario: `@clock(pos)` で立ち上がりエッジに EdgeMark が生成される
- Given `@clock(pos)` の空クロック (4 単位、`_~_~`)
- Then `SignalRow.edge_marks` に立ち上がりエッジ数分 (= 2) の `EdgeMark` が追加される
- And 各 `EdgeMark.line_start = (x, y_low)`、`line_end = (x + slant, y_high)`
- And `Annotations.arrows` には clock 由来の Arrow が **入らない**

## @not-implemented
### Scenario: `@clock(neg)` で立ち下がりエッジに EdgeMark
- Given `@clock(neg)` の空クロック (4 単位)
- Then 各 `EdgeMark.line_start = (x, y_high)`、`line_end = (x + slant, y_low)`

## @not-implemented
### Scenario: `@clock(both)` は両エッジに EdgeMark
- Given `@clock(both)` の空クロック (4 単位)
- Then 立ち上がり 2 + 立ち下がり 2 = 4 つの `EdgeMark` が生成される

## @not-implemented
### Scenario: `@clock(none)` は EdgeMark なし
- Given `@clock(none)` の空クロック (4 単位)
- Then `SignalRow.edge_marks` は空、波形展開のみ実施

## @not-implemented
### Scenario: 属性順不問
- Given `@clock(_=2, neg, ~=3)` という行がある
- Then `@clock(neg, _=2, ~=3)` と同じ動作をする

## @not-implemented
### Scenario: `edge` 省略時はデフォルト `none`
- Given `@clock(_=2)` という行がある
- Then `@clock(none, _=2)` と同じ動作をする (`edge = ClockEdge::None`、`EdgeMark` 非生成)

## @not-implemented
### Scenario: `@clock` 単独 (引数なし) は `@clock(none)` と等価
- Given `@clock` (括弧なし) の後に空クロック信号
- Then `@clock(none)` と同じ動作をする

## @not-implemented
### Scenario: `@clock()` 空括弧は `@clock(none)` と等価
- Given `@clock()` の後に空クロック信号
- Then `@clock(none)` と同じ動作をする

## @not-implemented
### Scenario: `@clock` 個別オプション `mark_height` で上書き
- Given `@clockmark_height 5` の後に `@clock(pos, mark_height=8)` の空クロック
- Then 生成された各 `EdgeMark.style.height == 8`、`width / position / color` はグローバル値を継承

## @not-implemented
### Scenario: `mark_color` 未指定は現行 `signal_color` を継承
- Given `signal_color blue` 状態で `@clock(pos)` を宣言 (`mark_color` 指定なし、`@clockmark_color` 設定なし)
- Then 各 `EdgeMark.style.color == Color("blue")`

## @not-implemented
### Scenario: `clockmark_position` をグローバル設定で変更
- Given `@clockmark_position 0.0` 状態で `@clock(pos)` 空クロック
- Then 各 `EdgeMark.style.position == 0.0`

## @not-implemented
### Scenario: `clockmark_width` / `clockmark_height` のデフォルト値はそれぞれ 6 / 7.5
- Given `@clockmark_width` / `@clockmark_height` のいずれも未指定、`@step 20` の後に `@clock(pos)` 空クロック (step は縮小条件外)
- Then 各 `EdgeMark.style.width == Px(6)` かつ `EdgeMark.style.height == Px(7.5)` (グローバル設定表記載のデフォルト値で解決される)

## @not-implemented
### Scenario: `clockmark_width` デフォルト解決時、step が小さいと `min(6, step × 2/3)` で縮小される
- Given `@step 6` の後に `@clock(pos)` 空クロック (`@clockmark_width` 未指定)
- Then 各 `EdgeMark.style.width == Px(4)` (= `min(6, 6 × 2/3)`)
- And `EdgeMark.style.height == Px(7.5)` (height は縮小されない)

## @not-implemented
### Scenario: `clockmark_width` デフォルト解決時、step×2/3 ≥ 6 なら縮小されない
- Given `@step 15` の後に `@clock(pos)` 空クロック (`@clockmark_width` 未指定)
- Then 各 `EdgeMark.style.width == Px(6)` (= `min(6, 15 × 2/3)` のうち 6 を選択)

## @not-implemented
### Scenario: グローバル `@clockmark_width` 明示指定は step 連動縮小の対象外
- Given `@step 3` `@clockmark_width 8` の後に `@clock(pos)` 空クロック
- Then 各 `EdgeMark.style.width == Px(8)` (step×2/3 = 2 にはならず、ユーザ指定値がそのまま採用される)

## @not-implemented
### Scenario: グローバル `@clockmark_width` をデフォルト値と同値で明示しても縮小されない
- Given `@step 3` `@clockmark_width 6` の後に `@clock(pos)` 空クロック
- Then 各 `EdgeMark.style.width == Px(6)` (数値は同じでも「明示指定された」状態のため min(...) は適用されない)

## @not-implemented
### Scenario: ローカル `@clock(..., mark_width=<px>)` 指定は step 連動縮小の対象外
- Given `@step 3` の後に `@clock(pos, mark_width=12)` 空クロック (`@clockmark_width` グローバル未指定)
- Then 各 `EdgeMark.style.width == Px(12)`

## @not-implemented
### Scenario: ローカル `mark_width` 指定はグローバル未指定の縮小条件下でも縮小しない
- Given `@step 6` の後に `@clock(pos, mark_width=8)` 空クロック (`@clockmark_width` グローバル未指定)
- Then 各 `EdgeMark.style.width == Px(8)` (グローバル未指定で step が縮小条件を満たしても、ローカル明示が優先で縮小は適用されない)

## @not-implemented
### Scenario: ローカル `mark_width` 指定はグローバル明示より優先 (どちらも縮小なし)
- Given `@step 3` `@clockmark_width 6` の後に `@clock(pos, mark_width=10)` 空クロック
- Then 各 `EdgeMark.style.width == Px(10)` (ローカル明示がグローバル明示より優先)

## @not-implemented
### Scenario: `clockmark_height` は step が小さくても縮小されない
- Given `@step 3` の後に `@clock(pos)` 空クロック (`@clockmark_height` 未指定)
- Then 各 `EdgeMark.style.height == Px(7.5)` (デフォルト値そのまま、step 連動縮小は height には適用されない)
- And 同 mark の `width == Px(2)` (width だけ縮小される)

## @not-implemented
### Scenario: グローバル `@clockmark_height` 明示指定はそのまま採用
- Given `@step 3` `@clockmark_height 20` の後に `@clock(pos)` 空クロック (`@clockmark_width` 未指定)
- Then 各 `EdgeMark.style.height == Px(20)` (height は明示指定に従い、縮小ルールなし)
- And 同 mark の `width == Px(2)` (= `min(6, 3 × 2/3)`、width は依然としてデフォルト解決のため縮小)

## @not-implemented
### Scenario: per-row `@step` 途中変更で clock 行ごとに縮小値が再計算される
- Given 1 信号目: `@step 12` の後に `@clock(pos)` 空クロック、2 信号目: `@step 3` の後に別の `@clock(pos)` 空クロック (どちらも `@clockmark_width` 未指定)
- Then 1 信号目の各 `EdgeMark.style.width == Px(6)`、2 信号目の各 `EdgeMark.style.width == Px(2)` (clock 信号行の生成時点の `@step` を用いて縮小値が決まる)

## @not-implemented
### Scenario: グローバル `@clockmark_width` 明示後は `@step` を縮小条件に変えても縮小しない
- Given `@clockmark_width 8` の後に `@step 3` で `@clock(pos)` 空クロック
- Then 各 `EdgeMark.style.width == Px(8)` (グローバル明示済みのため step 値に関わらず縮小は発生しない)

## @not-implemented
### Scenario: `=` 前後の空白を許容する (H-1)
- Given `@clock(pos , _ = 2 , ~ =3)` という行がある
- Then `@clock(pos, _=2, ~=3)` と完全に同じ動作をする

---

## `@signal`

## @not-implemented
### Scenario: `@signal(overline)` で次行に上線フラグ
- Given `@signal(overline)` の後に `nReset _~~~`
- Then `nReset` の `SignalDecorations.name_overline` が `true` となる
- And その後の信号行では `name_overline` が `false` に戻る

## @not-implemented
### Scenario: `@overline_gap` / `@overline_thickness` をローカルパラメータとして受理
- Given `@overline_gap 5` `@overline_thickness 2` という行
- Then パーサーは現行の `overline_gap=Px(5)`、`overline_thickness=Px(2)` を保持する
- And 後続の `@signal(overline)` 信号の描画はこれらの値を使う

## @not-implemented @negative
### Scenario: 不正値はパースエラー
- Given `@overline_gap -1` または `@overline_thickness 0`
- Then 行番号付きパースエラー (負値・ゼロは禁止)

---

## `@->` 矢印

## @not-implemented
### Scenario: 最小構文
- Given アンカー `@{a}` `@{b}` を定義し、`@-> (@{a}, @{b})` を宣言
- Then `Annotations.arrows` に既定スタイルの `Arrow` が追加される

## @not-implemented
### Scenario: フル構文 (色・太さ・線種・ラベル)
- Given `@-> (@{a}, @{b}, red, 2px, dashed) 変化` という行
- Then `Arrow.style` が `color=red, width=2px, line=Dashed` となり `label=Some("変化")` となる

## @not-implemented
### Scenario: 属性順不問
- Given `@-> (@{a}, @{b}, dashed, red, 2px)` という行
- Then 上記と同じ `ArrowStyle` になる

## @not-implemented
### Scenario: 前方参照可
- Given `@-> (@{a}, @{b})` を宣言した後の行で `@{a}` `@{b}` を定義
- Then エラーにならず `Arrow` が生成される

## @not-implemented @negative
### Scenario: 未定義アンカー参照
- Given 定義されていない `@{x}` を `@->` で参照
- Then `ParseError::UndefinedAnchor` となる

## @not-implemented @negative
### Scenario: 同カテゴリ属性の重複
- Given `@-> (@{a}, @{b}, red, blue)` (色 2 個)
- Then `ParseError::DuplicateArrowAttribute` となる

## @not-implemented @negative
### Scenario: 判別不能トークン
- Given `@-> (@{a}, @{b}, foobar)` という行
- Then `ParseError::UnknownArrowAttribute` となる

---

### Scenario: `head=` の `=` 前後空白を許容する (H-1)
- Given `@-> (@{a}, @{b}, head = both)` という行
- Then `Arrow.style.head` が `Both` となる (`head=both` と等価)

---

## `@highlight_style`

### Scenario: `=` 前後の空白を許容する (H-1)
- Given `@highlight_style fill = "#8f8" stroke ="green" stroke-width= "1"` という行
- Then `fill=#8f8`, `stroke=green`, `stroke-width=1` と等価に解釈される

## `@dontcare_color`

### Scenario: 単一色値を取る (`@bgcolor0` と同じ書式)
- Given `@dontcare_color #c00` という行
- Then `Directive::DontcareColor(#c00)` として解釈される

### Scenario: 名前付き色も受け付ける
- Given `@dontcare_color red` という行
- Then `Directive::DontcareColor(red)` として解釈される

### Scenario: チャート途中で再宣言可能
- Given `@dontcare_color #c00`、信号行 1、`@dontcare_color #06c`、信号行 2 の順に並ぶチャート
- Then 信号行 1 は `#c00`、信号行 2 は `#06c` のハッチ色を持つ

---

## 文字書き込み行 (`%`)

## @not-implemented
### Scenario: `% x y text` で TextOverlay が追加される
- Given `% 100 50 注釈` という行がある
- Then `Annotations.overlays` に `TextOverlay { at: (100,50), text: "注釈" }` が追加される

---

## 統合シナリオ

## @not-implemented @smoke
### Scenario: 完全な TCML ドキュメント
- Given `tmp/image/user_message-2026-4-26_1.webp` 相当の TCML
  ```
  foo  _?~_~?_~_?XaX?==
  bar  -?==X?==X
  ```
  (※ a は適切な bus ラベル定義済み)
- When パーサーでパースする
- Then すべての `?` が直前アンカーから `DontCareAlong*` に正しく解決される
- And X は `Transition(BusCross)` として保持される
- And エラーにならない

---

## 観点A 補強: 単独仕様の境界・上書き・途中変更

### Scenario: コメント行末尾の余白文字は無視される
- Given `// foo   ` (末尾空白多数) という行
- When パーサーでパースする
- Then エラーにならず、`lines` に追加されない

### Scenario: パラメータ行の末尾空白を許容
- Given `@step 10   ` (末尾空白) という行
- When パーサーでパースする
- Then `@step 10` と等価に処理される

### Scenario: 信号名と波形の間に複数空白を許容
- Given `Clock     _~_~` (間に空白 5 個) という行
- When パーサーでパースする
- Then `SignalName("Clock")` + waveform `_~_~` として正しくパースされる

### Scenario: パラメータ名の表記揺れ全パターン受理 (`step`)
- Given `@step 10` / `@STEP 10` / `@Step 10` を別ファイルとして与える
- Then どの表記でもすべて step=10 として正規化される

### Scenario: パラメータ名表記揺れ (`-` と `_` 混在) — `signal_color` / `signal-color`
- Given `@signal_color red` と `@signal-color red` をそれぞれ別ファイルで与える
- Then どちらも同一の `signal_color = red` として保存される

### Scenario: `@clockmark_height` をローカルパラメータとして再宣言可能
- Given `@clockmark_height 5` の後に信号、続けて `@clockmark_height 8` の後に別信号
- Then 1 つ目の信号 (clock 拡張時) は height=5、2 つ目は height=8 で展開される

### Scenario: `@clock(pos)` の `mark_color` 大文字小文字違い属性キー
- Given `@clock(POS, MARK_COLOR=red, MARK-HEIGHT=4)` という行
- Then `pos` edge、`mark_color=red`、`mark_height=4` として解釈される (キー名の大小・`-`/`_` を区別しない)

### Scenario: `@bg` を 2 連続で書くと最後の値が有効
- Given `@bg #f0f` の直後に `@bg #0f0`、続けて信号行 A
- Then 信号行 A の `Line.background == Some(#0f0)` (最新値で上書き、`#f0f` は破棄)

### Scenario: `@bgcolor0` に 16 進 8 桁 (alpha 付き) を許容
- Given `@bgcolor0 #ff8800ff` という行
- Then `Color::parse("#ff8800ff")` で受理され、`#ff8800ff` として保存される

### Scenario: `@bgcolor0` で `none` を明示
- Given `@bgcolor0 none` という行
- Then 偶数行背景は出力されない (`Color::None` 相当の保存)

### Scenario: `@bgcolor1` 単独指定 (奇数行のみ塗る)
- Given `@bgcolor1 #eee` のみ (`@bgcolor0` 未指定)
- Then 偶数行は塗らず、奇数行のみ `#eee` で塗る

### Scenario: `@scale 2.0` のグローバル設定
- Given `@scale 2.0` という行
- Then `ChartStyle.canvas.scale == 2.0` として保存される

### Scenario: `@page-margin 0` でマージンなし
- Given `@page-margin 0` という行
- Then チャート全体の四方マージンが 0 px

### Scenario: `@scale` 途中変更はエラー (グローバル属性)
- Given `@scale 1.0` の後に信号、続けて `@scale 2.0`
- Then 後者の `@scale` は途中変更不可のグローバルパラメータとして拒否されるか、または最後の値で上書き (実装に応じて固定挙動を要求)

### Scenario: `@step` を 0 にするとエラー
- Given `@step 0` という行
- Then `ParseError::InvalidStepSlant` (step <= slant 検査で 0 < slant=2 なので true、エラー)

### Scenario: `@step` を負値にするとエラー
- Given `@step -5` という行
- Then パースエラー (負値拒否)

### Scenario: `@slant` を負値にするとエラー
- Given `@slant -1` という行
- Then パースエラー (負値拒否)

### Scenario: `@slant 0` を許容 (垂直エッジ)
- Given `@slant 0` という行
- Then `slant=0` で保存され、後続信号は垂直エッジで描画される

### Scenario: `step == slant` ちょうどはエラー
- Given `@step 2` `@slant 2` の組合せ
- Then `ParseError::InvalidStepSlant`

### Scenario: `@h_space` を負値にするとエラー
- Given `@h_space -1`
- Then パースエラー

### Scenario: `@fontsize` を 0 にするとエラー
- Given `@fontsize 0`
- Then パースエラー (0 以下禁止)

### Scenario: `@lineheight` を 0 にするとエラー
- Given `@lineheight 0`
- Then パースエラー

### Scenario: `@title` の引数が空 (`@title ""`)
- Given `@title ""` という行
- Then タイトル行が生成されるが本文は空文字列の `UserText`

### Scenario: `@title` の引数省略 (引数なし) はエラー
- Given `@title` (引数なし) という行
- Then パースエラー (引数必須)

### Scenario: `@skip` の引数省略はエラー
- Given `@skip` (引数なし) という行
- Then `ParseError::InvalidSkipAmount`

### Scenario: `@skip()` 空括弧はエラー
- Given `@skip()` という行
- Then `ParseError::InvalidSkipAmount`

### Scenario: `@skip(1lh)` 単位 `lh` を明示
- Given `@skip(1lh)` という行
- Then 仕様で受理されるか拒否されるか実装挙動を固定 (現仕様では単位なしが lh、`lh` サフィックスは明記なし → エラーにすべき)

### Scenario: `@skip(2px)` 大文字 `2PX`
- Given `@skip(2PX)` という行
- Then `Length::Px(2.0)` で受理 (px サフィックスは大小区別しない)

### Scenario: `@signal(overline)` を信号行ではなく Title 行の前に置いた場合
- Given `@signal(overline)` の直後に `@title "X"` (信号ではなく Title)
- Then `@signal` は次の信号行に対するもののため、Title では消費されない (持ち越し) — または "信号行限定" のため警告/エラー (実装挙動を固定)

### Scenario: `@signal(overline)` の重複指定
- Given `@signal(overline)` を 2 回連続で書いた直後の信号
- Then `name_overline = true` として 1 回扱い (冪等) または重複エラー (実装決定)

### Scenario: `@clock` で `_=0` (パルス幅 0) はエラー
- Given `@clock(pos, _=0, ~=1)`
- Then パースエラー (パルス幅 0 不可、`_=<n>` は正整数)

### Scenario: `@clock` で `~=` 値が浮動小数
- Given `@clock(pos, ~=2.5)`
- Then パースエラー (正整数のみ)

### Scenario: `@clock` の `mark_position` 範囲外
- Given `@clock(pos, mark_position=1.5)`
- Then パースエラー (`0.0..=1.0` 外)

### Scenario: `@clock` の `mark_height` 負値
- Given `@clock(pos, mark_height=-1)`
- Then パースエラー (正値必須)

### Scenario: `@clock` の `start=high` 開始相
- Given `@clock(pos, start=high)` の後に空クロック (chart_units=4)
- Then 展開結果は `~_~_` (High 始まり、Low/High 各 1 単位)

### Scenario: `@clock` の `edge` 大文字小文字違い (`POS` / `Pos`)
- Given `@clock(POS)` / `@clock(Pos)` という行
- Then どちらも `pos` と等価に解釈される

### Scenario: `@clock` で同 attr が 2 回出現
- Given `@clock(pos, _=2, _=3)`
- Then パースエラー (重複属性)

### Scenario: `@clock` の不明 attr
- Given `@clock(pos, foo=1)`
- Then パースエラー (未知 attr)

### Scenario: `@->` の `head=invalid`
- Given `@-> (@{a}, @{b}, head=middle)`
- Then パースエラー (`end`/`both`/`none` 以外)

### Scenario: `@->` 線種 `solid` 明示
- Given `@-> (@{a}, @{b}, solid)`
- Then 線種が `LineDashStyle::Solid` で保存

### Scenario: `@->` で太さに `1.5` (単位なし)
- Given `@-> (@{a}, @{b}, 1.5)`
- Then 太さ 1.5 px (単位省略可)

### Scenario: `@->` で `0.5px` 小数 px
- Given `@-> (@{a}, @{b}, 0.5px)`
- Then 太さ 0.5 px

### Scenario: `@->` で太さ 0 (描画されないか線として残るか)
- Given `@-> (@{a}, @{b}, 0px)`
- Then 仕様上の挙動を明示 (受理 or 拒否)

### Scenario: `@->` で色をカテゴリ重複
- Given `@-> (@{a}, @{b}, red, #f00)` (色 2 個)
- Then `ParseError::DuplicateArrowAttribute`

### Scenario: `@->` で線種カテゴリ重複
- Given `@-> (@{a}, @{b}, dashed, dotted)`
- Then `ParseError::DuplicateArrowAttribute`

### Scenario: `@->` でラベルにカンマ含む
- Given `@-> (@{a}, @{b}) hello, world`
- Then ラベルは `"hello, world"` 全文 (括弧外なのでカンマで分割しない)

### Scenario: `@->` で括弧内に空白が大量にある
- Given `@-> (   @{a}   ,   @{b}   ,   red   )`
- Then 端点 `@{a}` `@{b}`、色 red として正しく解釈される

### Scenario: `@->` のアンカー名と未参照アンカーの混在
- Given `@{a}` `@{b}` `@{unused}` を定義し、`@-> (@{a}, @{b})` のみ
- Then `@{unused}` はエラーにならず、`Annotations.arrows` には 1 個のみ追加される

### Scenario: `@dontcare_color none` で未色化扱い
- Given `@dontcare_color none`
- Then 受理されるか拒否されるかを固定 (色値なので `none` 受理が自然、ハッチ非表示)

### Scenario: `@highlight_style` で許可外 SVG 属性
- Given `@highlight_style onmouseover="alert(1)"` (許可外属性)
- Then ホワイトリストで弾かれパースエラー、または黙って drop

### Scenario: `@highlight_style` 値に空白を含む (引用必須)
- Given `@highlight_style fill="rgb(255, 128, 0)"`
- Then `fill=rgb(255, 128, 0)` として `key=value` 1 組で受理 (引用内の空白は値の一部)

### Scenario: アンカー名 `_under_score`
- Given `@{_under_score}` (先頭がアンダースコア)
- Then 正しく `[A-Za-z_][A-Za-z0-9_-]*` を満たし受理

### Scenario: アンカー名にハイフン
- Given `@{a-b-c}`
- Then 受理

### Scenario: アンカー番号 0 は受理 (numbered anchor 0)
- Given `Sig _~@0_~` という信号行を 1 回だけ書く
- Then `elements` に `Anchor(Indexed(0))` が含まれパースエラーにならない

### Scenario: アンカー番号が極端に大きい (例: `@99999`)
- Given `@99999` を信号行で 1 回定義し `@-> (@99999, @1)` で参照
- Then 受理される (上限なし)

### Scenario: アンカー番号 0 を `@->` 端点として参照
- Given `Sig _~@0__@1` と `@-> (@0, @1)`
- Then パースエラーにならず、矢印は numbered anchor 0 と 1 の 2 座標を結ぶ

### Scenario: アンカー番号 0 の同一行重複は `DuplicateAnchor`
- Given 同一信号行内で `@0` を 2 回使う (`Sig _@0_~@0_`)
- Then `ParseError::DuplicateAnchor`

### Scenario: アンカー番号 0 の別信号行重複は `DuplicateAnchor`
- Given `Sig1 _~@0_` と `Sig2 _~@0_` を同一ファイルに記述
- Then `ParseError::DuplicateAnchor` (信号行を跨いでも numbered anchor は単一名前空間)

### Scenario: 名前付き `@{0}` と番号付き `@0` は別アンカー (値 0 でも名前空間分離)
- Given `Sig _~@{0}__@0` と `@-> (@{0}, @0)`
- Then パースエラーにならず両者は別アンカーとして解決され、矢印は 2 つの異なる座標を結ぶ

### Scenario: 信号名が UTF-8 マルチバイト (日本語)
- Given `クロック _~_~`
- Then `SignalName("クロック")` として受理

### Scenario: 信号名にスペース (引用なし) はエラーにならず最初のトークンのみ
- Given `Clock A _~_~` (引用なし、空白で 3 トークン)
- Then 信号名 `Clock`、波形は `A` で始まりエラー (`A` は level 記号でない → `MissingInitialLevel`)

### Scenario: 引用信号名内に literal `\t` (エスケープなし)
- Given `"a\tb" _~` (`\t` は実際のタブ文字)
- Then `ParseError::InvalidSignalName` (制御文字は不許可)

### Scenario: 引用信号名内のエスケープシーケンス未定義 (`\x`)
- Given `"a\xb" _~` (未定義エスケープ)
- Then パースエラー or リテラルとして `\x` 残し (実装挙動を固定)

### Scenario: 改行のみのファイル
- Given `\n\n\n` (改行 3 個のみ)
- Then `lines` 空、エラーなし

### Scenario: BOM 付きファイル
- Given `\u{FEFF}# header\n` で始まる TCML
- Then BOM が無視され (または許容され) パースエラーにならない

### Scenario: CRLF 改行
- Given Windows 改行 (`\r\n`) を含む TCML
- Then LF と等価に扱われる

### Scenario: Tab 文字を行頭に持つ信号行
- Given `\tClock _~_~` (タブインデント)
- Then タブが空白扱いで信号名 `Clock` として正常受理 — または「行頭タブは無効」とするか実装決定

---

## 観点B 補強: 組合せエッジケース (parser 寄り)

### Scenario: per-row `@step` 途中変更 × `@clock(auto)` 展開
- Given 1 信号目: `@step 10` の `@clock(pos)` 空クロック (chart_units=8)、2 信号目: `@step 20` の通常信号 4 文字
- Then clock 展開は信号別 `step` を尊重し、各 SignalRow.geometry の signal_box 幅が異なる
- And chart_units 計算は「unit 数」基準 (px ではなく) のため、両信号の波形末端 px は揃わなくてよい

### Scenario: per-row `@step` 途中変更 × DontCare (`?`) 解決
- Given `@step 10` で `Sig1 _?_`、続けて `@step 20` で `Sig2 _?_`
- Then 両方とも DontCareAlongLow,2 として解決され、polygon 幅は `2 × step` (Sig1=20、Sig2=40)

### Scenario: per-row `@step` 途中変更 × アンカー位置
- Given `@step 10` で `Sig1 ___@1__`、`@step 20` で `Sig2 ___@2__`
- Then `@1.x` と `@2.x` が異なる (capwidth 加算後それぞれ `30` と `60` の累積)

### Scenario: per-row `@step` 途中変更 × `@->` ラベル位置
- Given `@step 10` で `@1` 定義、`@step 20` で `@2` 定義、`@-> (@1, @2)` 宣言
- Then 矢印中点 (ラベル配置位置) が両端の x 平均位置に正しく配置される

### Scenario: per-row `@step` 途中変更 × `%` 行
- Given `@step 10` 信号、`% 100 50 mark`、`@step 20` 信号
- Then `%` 行の x=100 はチャート絶対座標 (どの信号の step とも独立)

### Scenario: per-row `@step` 途中変更 × `@signal(overline)`
- Given `@step 10` の通常信号、`@step 20` の `@signal(overline) NReset _~_~`
- Then 信号 `NReset` が step=20 で展開され、上線が信号名最長行幅で 1 本引かれる

### Scenario: per-row `@slant` 途中変更 × DontCare 解決
- Given `@slant 0` で `Sig1 _?=`、`@slant 4` で `Sig2 _?=`
- Then DontCare polygon の左辺斜辺が両者で異なる (Sig1 は垂直、Sig2 は斜辺 4px)

### Scenario: per-row `@slant` 変更 × 直後の `@clock(pos)` EdgeMark
- Given `@slant 0` の通常信号、`@slant 4` の直後 `@clock(pos)` 空クロック
- Then `@clock(pos)` 信号の各 EdgeMark.line_end.x = line_start.x + 4 (新 slant 値で計算)

### Scenario: `@clock(auto, _=2, ~=2)` × per-row `@step` 変更 × WaveDrom (parser 観点)
- Given `@step 10` で `@clock(pos, _=2, ~=2)` 空クロック (chart_units=8)、parser まで実行
- Then SignalRow.waveform.elements が `[LevelRun(Low,2), Trans, LevelRun(High,2-1?), ...]` 形で正しく展開
- And EdgeMark 数は立ち上がり位置数と一致

### Scenario: `@clock(auto)` × `@->` 矢印 (clock body にアンカー埋め込み不可)
- Given `@clock(pos)` の空クロック (本体波形なし)
- Then アンカー (`@{x}` / `@N`) は本体に書けない (本体空のため) → 別信号のアンカーを矢印端点にしか使えない

### Scenario: `@clock(auto)` × 部分指定本体内アンカー
- Given `@clock(pos) ck _~@{rise}__` (本体に `@{rise}` を含む、続きは自動展開)
- Then `@{rise}` は適切な位置に登録され、後続自動展開部分には影響しない

### Scenario: 同名アンカー — 異なる信号行で重複定義
- Given `Sig1 _~@{a}_` と `Sig2 _~@{a}_` (同名 `@{a}` を 2 回定義)
- Then `ParseError::DuplicateAnchor` (信号行を跨いでも名前空間共通)

### Scenario: 同番号アンカー — 異なる信号行で重複定義
- Given `Sig1 _~@1_` と `Sig2 _~@1_`
- Then `ParseError::DuplicateAnchor`

### Scenario: 番号アンカーの飛び番 (`@1` `@5` のみ定義)
- Given 番号 1 と 5 のみ定義 (2,3,4 なし)
- Then エラーにならず両方登録される (連番要求なし)

### Scenario: `@->` で `@{1}` (名前付き "1") と `@1` (番号付き 1) を別端点として使用
- Given `Sig1 _~@{1}__@1` と `@-> (@{1}, @1)`
- Then 両者は別アンカーとして解決され、矢印は 2 つの異なる座標を結ぶ

### Scenario: 複数行信号名 × アンカー
- Given `"Sig\nA" _~@{a}_~`
- Then 信号名は 2 行、アンカー `@{a}` は波形 x 累積位置で登録される (信号名行数は x に影響しない)

### Scenario: 複数行信号名 × `@signal(overline)` × overline 幅
- Given `@signal(overline) "short\nveryLongLine" _~`
- Then 上線は最長行 (`veryLongLine`) のテキスト幅で引かれる (短い行幅ではない)

### Scenario: 信号名なし行 (空白文字のみ) は無視
- Given `   ` (空白のみの行)
- Then 空行扱いで `lines` に追加されない

### Scenario: 信号名中の `#` は通常文字として受理
- Given `Sig#A _~_~`
- Then 信号名 `Sig#A` として受理 (`#` は新仕様で特殊扱いされない通常文字のため、`InvalidLevelChar('#')` を返さない)

### Scenario: 行末の `# comment` は bare text として受理、行末の `// comment` は無視
- Given `Sig _~_~ # comment` と `Sig _~_~ // comment` の 2 行
- Then `Sig _~_~ # comment` 行は `#` 以降をレベル文字列末尾の bare text として受理 (`InvalidLevelChar('#')` を返さない)
- And `Sig _~_~ // comment` 行は `//` 以降が行末コメントとして無視され、波形は `_~_~` のみ

### Scenario: アンカー番号と名前付きを `@->` で混在
- Given `Sig _~@1__@{end}` と `@-> (@1, @{end})`
- Then 両者を解決して 1 本の矢印を生成

### Scenario: `@->` で同一端点 (`@{a} → @{a}`) を指定
- Given `@-> (@{a}, @{a})`
- Then 受理されるが線長 0 (実装挙動を固定する)

### Scenario: `@->` を 100 個以上定義
- Given アンカー 200 個と矢印 100 本
- Then エラーにならず全て登録される (上限なし、parser レベル)

### Scenario: `@bg` × `@clock(pos)` 空クロック信号
- Given `@bg #f0f` の直後に `@clock(pos)` の空クロック行
- Then クロック信号行の `Line.background == Some(#f0f)` (clock 信号も「次の 1 行」として消費)

### Scenario: `@bg` × `@signal(overline)` × 信号行
- Given `@bg #f0f` `@signal(overline)` `nReset _~__`
- Then 信号 `nReset` の `Line.background == Some(#f0f)` かつ `name_overline == true`
- And `@bg` と `@signal` はそれぞれ独立に保留され、同一信号行で両方消費される

### Scenario: `@bgcolor0/1` 偶奇カウントは `@bg` 指定行も含む
- Given SignalRow A、`@bg #f0f` SignalRow B、SignalRow C
- Then 偶奇カウントとしては A=0, B=1, C=2 (`@bg` 適用行も信号インデックスをカウント)

### Scenario: `@title` 行 × `@bgcolor0/1` カウント除外
- Given SignalRow A、TitleRow、SignalRow B
- Then A=偶数 (idx 0)、B=奇数 (idx 1)、Title はインデックス対象外

## ユーザ承認反映シナリオ (2026-05-10)

### Scenario: 信号名のみ・wave 空文字列が許容される
- Given 入力行 `SigA `
- When parse
- Then SignalRow が 1 個生成される
- And SignalRow.waveform.elements.len() == 0
- And ParseError は発生しない

### Scenario: 信号名のみ・wave 空文字列の SignalRow は chart_units に寄与しない
- Given 行 1 `SigA `、行 2 `SigB ___`
- Then chart_units == 3 (SigB のみが寄与)

### Scenario: `@skip` 括弧形式と空白区切り形式は等価
- Given `@skip(2)` と `@skip 2` をそれぞれ別チャートに記述
- Then 両者が生成する Skip 行の高さが等しい

### Scenario: `@skip 2.5` (空白区切り、小数)
- Given `@skip 2.5`
- Then SkipRow が 2.5 lh 高さで生成される

### Scenario: `@skip 20px` (空白区切り、px 単位)
- Given `@skip 20px`
- Then SkipRow が 20 px 高さで生成される

### Scenario: `@-> ` の属性に `color=red` を `key=value` 形式で指定
- Given `@-> (@{a}, @{b}, color=red)`
- Then Arrow.color == red

### Scenario: `@->` の属性 `width=2px` を `key=value` 形式で指定
- Given `@-> (@{a}, @{b}, width=2px)`
- Then Arrow.width == 2.0px

### Scenario: `@->` の属性 `style=dashed` を `key=value` 形式で指定
- Given `@-> (@{a}, @{b}, style=dashed)`
- Then Arrow.style == dashed

### Scenario: `@->` で positional と key=value を混在
- Given `@-> (@{a}, @{b}, red, width=2px, style=dashed, head=both)`
- Then すべて期待通りに反映される

### Scenario: `@-> ` の color=・width=・style= キーは大文字小文字無視
- Given `@-> (@{a}, @{b}, COLOR=red)`
- Then Arrow.color == red

### Scenario: `@-> ` の同カテゴリを positional + key=value で重複指定はエラー
- Given `@-> (@{a}, @{b}, red, color=blue)`
- Then ParseError::DuplicateArrowAttribute

### Scenario: `@overline` 単独で `@signal(overline)` と同義
- Given `@overline\nnReset _~`
- Then SignalDecorations.name_overline == true (nReset 行)

### Scenario: `@overline` も 1 行限り
- Given `@overline\nA _\nB ~`
- Then A 行は overline、B 行は overline なし

### Scenario: `@OVERLINE` (大文字) も alias として有効
- Given `@OVERLINE\nA _`
- Then A 行は overline

### Scenario: 全 key=value で `_` ↔ `-` 等価
- Given `@-> (@{a}, @{b}, head-end=...)` のような `head-end=` (本来 `head=end`?) はエラーだが、`@->` `head=end` と `head-end` は別記号なので対象外
- Given `@highlight_style stroke-width="1"` と `@highlight_style stroke_width="1"` を同チャート別行に書いた場合
- Then 両者は同一スタイル属性として扱われる

### Scenario: `@clock(_=2)` の `_` は等価ルール対象外
- Given `@clock(pos, -=2)` (Low の単位を 2 に、`-` で書こうとした)
- Then ParseError (`_=` は level 記号、`-=` は無効)

### Scenario: `@-> ` の `head=end` を `head-end` で書ける
- Given `@-> (@{a}, @{b}, head=end)` を `head=end` のキー名は実際は `head` 1 文字 → 等価ルール対象は複数文字キーのみ
- Given `@-> (@{a}, @{b}, COLOR=red)` (キー大文字)
- Then color = red と解釈される

---

## 観点A 補強: エラー復旧と多重エラー (parser)

### Scenario: 1 行に複数のパースエラーが含まれるとき先頭エラーのみ報告
- Given `@bg notacolor @titlealign sideways` のように同一行に 2 つの不正値
- Then ParseError は 1 件のみ (先頭) で行番号は当該行
- And 後続行のパースは続行される (recovery)

### Scenario: `@dontcare_color invalid` の直後に有効な行が続く
- Given `@dontcare_color zonk\nA _~_~`
- Then 1 件目はパースエラー、2 件目 (信号 A) は正常にパース
- And `errors().len() == 1`、`document.lines.len() == 1` (信号 A)

### Scenario: `@-> (@{未定義1})` × `@-> (@{未定義2})` 複数行で UnknownAnchor が複数報告
- Given `@-> (@{a}, @{b})\n@-> (@{c}, @{d})` (どのアンカーも未定義)
- Then ParseError::UnknownAnchor が 4 件 (`a`/`b`/`c`/`d` 各々)、または「行ごとに 1 件 × 2」
- And いずれにせよ 1 件で打ち切らない

### Scenario: パースエラー後のグローバルパラメータ反映
- Given `@scale notnum\n@scale 2.0\nA _~`
- Then 1 行目はエラー、2 行目で `scale = 2.0` が確定
- And 信号 A のレイアウトに `scale = 2.0` が反映される

### Scenario: 連続するエラー行のあとに空行・コメントを挟んで復帰
- Given `@bg ???\n@step xyz\n// comment\n\nA _`
- Then エラーは 2 件、コメント・空行は無視、A は正常パース

### Scenario: 同一行内 `@signal(overline)` 複数指定はエラー 1 件、他属性は無視
- Given `@signal(overline, overline)` (重複)
- Then ParseError::DuplicateSignalAttribute 1 件
- And 後続行は通常通りパース継続

### Scenario: パースエラーが返っても document の他フィールドは初期化される
- Given 任意のパースエラー
- Then `parse()` は `(document, errors)` を返し、document.params は default 適用済み

### Scenario: 数値オーバーフロー (`@step 99999999999999999`) はエラーで停止せず
- Given `@step 99999999999999999\nA _`
- Then ParseError::InvalidNumber、A 行はパース継続 (step はデフォルト適用)

### Scenario: `@->` の `head=invalid` は当該 `@->` のみ無効化、他 `@->` は生きる
- Given `@-> (@{a}, @{b}, head=foo)\n@-> (@{a}, @{b})`
- Then 1 件目はエラー、2 件目は ArrowDef として保持される

### Scenario: 多重エラー時の error 順序は文書順
- Given 行 5 → 行 3 → 行 7 の順でエラー (実際には文書順なので行 3 → 行 5 → 行 7)
- Then `errors[]` は line 番号昇順

---

## 観点B 補強: 空状態とゼロ値・極端値の境界 (parser)

### Scenario: 空ファイル (0 byte) のパース
- Given `""`
- Then `document.lines.is_empty()` かつ `errors.is_empty()`

### Scenario: `@title` だけの 0 信号ファイル
- Given `@title "T"\n` のみ
- Then `document.lines.len() == 1` (TitleRow のみ)、`signal` カウント 0、`errors.is_empty()`

### Scenario: `@scale 0` はパースエラー (グローバル属性)
- Given `@scale 0\nA _`
- Then ParseError、A 行は default scale で続行

### Scenario: `@scale 1000` (極大) は受理
- Given `@scale 1000`
- Then `params.scale == 1000.0`、エラーなし
- And レイアウト/SVG width 計算が overflow しない

### Scenario: `@fontsize 0.5` (極小) は受理
- Given `@fontsize 0.5`
- Then `params.font_size == 0.5`、後続信号名のテキスト幅計算もこの値で行う

### Scenario: `@fontsize 1.0` 境界
- Given `@fontsize 1.0`
- Then エラーなし、`font_size == 1.0`

### Scenario: `@step 1` 最小境界 (slant=0 と組合せ)
- Given `@step 1\n@slant 0\nA _~_~`
- Then エラーなし、各文字幅 = 1px

### Scenario: `@slant 0` で全 SingleEdge が垂直 (パース時点では layout 不変)
- Given `@slant 0\nA _~`
- Then パースエラーなし、`params.slant == 0.0`

### Scenario: 1 文字波形 `A _` のみ
- Given `A _`
- Then `document.lines.len() == 1`、SignalRow.elements に LevelRun(Low,1) 1 個
- And chart_units == 1

---

## 観点C 補強: アンカーと矢印の高度な組合せ (parser)

### Scenario: 同一信号内に複数アンカーが連続
- Given `A _@{x}@{y}@{z}~`
- Then 3 個のアンカー要素が 0 幅で累積位置同じ
- And `@{x}.x == @{y}.x == @{z}.x` (LevelRun の境界に一致)

### Scenario: `@->` 自己ループ (始終端が同じアンカー)
- Given `@-> (@{a}, @{a})` と `A _@{a}~`
- Then ArrowDef が 1 件作られる (両端同点)
- And パースエラーなし (描画段階の挙動は別 spec)

### Scenario: 矢印 0 個 (アンカーのみ定義)
- Given `A _@{a}~` のみ、`@->` なし
- Then `document.arrows.is_empty()`、エラーなし

### Scenario: 矢印 1 個の最小ケース
- Given `A _@{a}@{b}~` + `@-> (@{a}, @{b})`
- Then `document.arrows.len() == 1`

### Scenario: 矢印 100 個でも全て保持
- Given 100 個の `@->`
- Then `document.arrows.len() == 100`、エラーなし

### Scenario: clock EdgeMark が 0 件 (`@clock(none)` 単独)
- Given `@clock(none)\nclk _~_~`
- Then SignalRow.clock_edges.is_empty()

### Scenario: clock EdgeMark が 1 件 (1 エッジしかない波形)
- Given `@clock(pos)\nclk _~`
- Then clock_edges.len() == 1

### Scenario: clock EdgeMark が 50 件
- Given `@clock(pos)\nclk _~_~_~_~...` (50 立ち上がり)
- Then clock_edges.len() == 50

### Scenario: アンカー連続 + 矢印複数本がそれぞれを参照
- Given `A _@{a}@{b}@{c}~` + `@-> (@{a}, @{b})` + `@-> (@{b}, @{c})` + `@-> (@{a}, @{c})`
- Then 3 本の Arrow、全て参照成功

---

## 観点F 補強: WaveDrom 化の負側 (parser 側 fixture)

### Scenario: `@clock` なしの普通の `_~_~` 信号
- Given `clk _~_~_~`
- Then SignalRow として LevelRun ベースで保持される (clock 拡張は走らない)
- And to_wavejson 入力として渡せる (parser 観点)

### Scenario: `@->` から参照されないアンカーが多数あっても parser はエラーなし
- Given `A _@{a}@{b}@{c}@{d}~` で `@->` 1 本も定義しない
- Then エラーなし、anchor 4 個は document.anchors に登録される

---

## 観点N 補強: 国際化 / Unicode (parser)

## @not-implemented
### Scenario: 信号名に RTL 文字 (アラビア語)
- Given `العربية _~_~`
- Then SignalRow.name が RTL bytes を保持
- And ParseError なし

## @not-implemented
### Scenario: 信号名にヘブライ語 + 数字 (BiDi 混在)
- Given `שלום2 _~`
- Then 信号名が原文どおり保持される (BiDi マークなし)

## @not-implemented
### Scenario: 信号名に結合文字 (combining diacritics)
- Given `é _~` (e + combining acute → é)
- Then 信号名は 2 code point として保持、長さは byte 数に従う
- And NFC 正規化はしない (原文ママ)

## @not-implemented
### Scenario: 信号名にサロゲートペア (BMP 外絵文字)
- Given `🎉 _~` (U+1F389)
- Then 信号名は UTF-8 4 byte 1 char として保持
- And ParseError なし

## @not-implemented
### Scenario: 信号名に全角空白 (U+3000)
- Given `A　B _~` (A + 全角空白 + B)
- Then 信号名は `"A B"` ではなく `"A　B"` を保持
- Note: 半角空白で分割するならエラー、しないなら 1 トークン (spec 確認)

## @not-implemented
### Scenario: 信号名にノーブレークスペース (U+00A0)
- Given `A B _~`
- Then 半角空白とは異なる扱い (区切り文字にしない)

## @not-implemented
### Scenario: 信号名にゼロ幅スペース (U+200B)
- Given `A​B _~`
- Then 視覚上は連続だが 2 文字として保持

## @not-implemented
### Scenario: `@title` に NFD 分解形と NFC 結合形の差
- Given `@title "café"` (NFC) / `@title "café"` (NFD) の 2 ファイル
- Then 2 つは異なる byte 列として扱われる (parser は正規化しない)

## @not-implemented
### Scenario: BOM (U+FEFF) 先頭の TCML
- Given `﻿@title T\nA _`
- Then BOM 除去するか、エラーにするか仕様準拠
- Note: spec 未定義なら spec gap

## @not-implemented
### Scenario: コメント行に絵文字 + 結合文字を含む
- Given `// 🎉é コメント\nA _`
- Then エラーなく次行をパース、コメント本文は破棄

---

## 観点P 補強: エラー位置精度 (line / column)

## @not-implemented
### Scenario: 1 行目 1 列目にエラー
- Given `@invalid_directive\n` (先頭から不正)
- Then ParseError.line == 1、ParseError.col == 1

## @not-implemented
### Scenario: タブ文字を含む行での col 計算
- Given `\t\t@bad` (タブ 2 個 + 不正ディレクティブ)
- Then col が 仕様 (タブ展開規約) に従う
- Note: タブ展開規約が spec に未定義なら spec gap

## @not-implemented
### Scenario: 複数行クォート信号名内の改行で line 進行
- Given `"line1\nline2"` を含む信号定義 + 後続行に不正ディレクティブ
- Then 後続エラーの line 番号が改行を正しく加算した値

## @not-implemented
### Scenario: CRLF 改行ファイルでの line / col
- Given `@title T\r\n@bad\r\n`
- Then 2 行目のエラーは line == 2 (CRLF を 1 改行としてカウント)
- And col == 1

## @not-implemented
### Scenario: CR のみ (旧 Mac) 改行
- Given `@title T\r@bad\r`
- Then 仕様で CR 単独を改行として扱うかに従う
- Note: 仕様未定義なら spec gap

## @not-implemented
### Scenario: マルチバイト文字含む行の col は code point ベース
- Given `日本語 _~ @bad` (日本語 + 不正)
- Then col が code point 単位 (byte 数ではない)
- Note: 仕様確認 — col 単位の規約

## @not-implemented
### Scenario: 行末空白後のエラーで col が空白末を指す
- Given `A _~   ` の末尾に不正トークン
- Then col が空白の終端 + 1

## @not-implemented
### Scenario: 同一行に 2 つエラーがあっても先頭のみ報告 (line/col 一致)
- Given `@bad1 @bad2` 同一行
- Then errors.len() == 1、line/col は `@bad1` 先頭
- Note: iter1 で同様シナリオあり (1 行複数エラーは先頭のみ) — col の正確性に焦点

## @not-implemented
### Scenario: ファイル末尾 (EOF) 直前のエラー
- Given `@title` (引数なしで EOF)
- Then line == 1, col == EOF 直前 (or 1 + len)
- And ParseError 種別が "unexpected EOF" 系

---

## 観点M 補強: parser 数値精度 (negative)

## @not-implemented
### Scenario: `@scale 0.0001` 極小値の受理境界
- Given `@scale 0.0001`
- Then 仕様で許容下限を超えるならエラー、超えないなら受理
- Note: 下限が spec に未定義なら spec gap

## @not-implemented
### Scenario: `@scale 1e10` 指数表記
- Given `@scale 1e10`
- Then 指数表記を受理するかは仕様準拠
- Note: 仕様未定義なら spec gap

## @not-implemented
### Scenario: `@step 0` を拒否
- Given `@step 0`
- Then ParseError (0 は不正)

## @not-implemented
### Scenario: `@step -1` 負値を拒否
- Given `@step -1`
- Then ParseError (負値は不正)

## @not-implemented
### Scenario: `@slant -0.5` 負値の扱い
- Given `@slant -0.5`
- Then 仕様で負値を許可するかに従う (鏡像 transition か拒否か)
- Note: 仕様未定義なら spec gap

---

## `@step` 設定時の `@slant` 自動クランプ

`tcml-format.md` §ローカルパラメータ `@step` の規定:
- `@step` で新値を設定したとき、まだ `@slant` がユーザーにより明示されていなければ slant を `step / 2` に自動クランプ (小さな `@step` 値で既定 slant=5 と干渉して `ParseError::InvalidStepSlant` を発生させないため)。
- `@slant` が一度でも明示されていれば、以降の `@step` はクランプを行わない。
- `@slant` 明示済みで `step <= slant` となった場合はそのまま `ParseError::InvalidStepSlant`。

クランプ規則の解釈:
- クランプは「`step / 2` を上限とする」操作。すなわち新 slant = `min(現 slant, step / 2)`。現 slant が既に `step / 2` 以下ならクランプによって値は変わらない (拡大はしない)。

## @not-implemented
### Scenario: `@slant` 未明示で `@step` 設定 (小さな step) → slant が自動クランプされエラー回避
- Given デフォルト状態 (slant=5、`@slant` 未明示) のチャート
- When `@step 4` をパースする
- Then パースエラーは発生しない
- And パース後の `params.step == 4`
- And パース後の `params.slant == 2` (= `min(5, 4/2) = 2` に自動クランプ)

## @not-implemented
### Scenario: `@slant` 未明示で `@step` 設定 (step/2 ちょうど) → slant=step/2 でちょうど境界をクリア
- Given デフォルト状態 (slant=5、`@slant` 未明示) のチャート
- When `@step 10` をパースする
- Then パースエラーは発生しない
- And パース後の `params.step == 10`
- And パース後の `params.slant == 5` (= `min(5, 10/2) = 5`、クランプは発火するが値は不変)

## @not-implemented
### Scenario: `@slant` 未明示で `@step` 設定 (現 slant が step/2 より小) → slant 不変
- Given デフォルト状態 (slant=5、`@slant` 未明示) のチャート
- When `@step 100` をパースする
- Then パースエラーは発生しない
- And パース後の `params.step == 100`
- And パース後の `params.slant == 5` (= `min(5, 100/2) = 5`、現値が小さいので不変)

## @not-implemented @edge-case
### Scenario: `@slant` 未明示で `@step` 設定 (極小値) → slant も極小にクランプされエラー回避
- Given デフォルト状態 (slant=5、`@slant` 未明示) のチャート
- When `@step 0.0001` をパースする
- Then パースエラーは発生しない (step > 0 かつ slant が連動クランプされるため)
- And パース後の `params.step == 0.0001`
- And パース後の `params.slant == 0.00005` (= `min(5, 0.0001/2)`)
- Note: `@step` 自体が正値であること (0 / 負 / 非有限拒否) は別シナリオでカバー。本シナリオは正値の極小ケース。

## @not-implemented @negative
### Scenario: `@slant` 明示済みなら `@step` 設定でクランプされず、`step < slant` でエラー
- Given `@slant 5` を明示済み (slant=5、`@slant` 明示フラグが立っている) のチャート
- When `@step 4` をパースする
- Then `ParseError::InvalidStepSlant` が返る (slant=5 のままで step=4 <= slant=5 が成立)
- And `params.slant` はクランプによって 2 に変更されない (= クランプ抑止が効いていることの確認)

## @not-implemented @negative
### Scenario: `@slant` 明示済みなら `@step` 設定でクランプされず、`step == slant` でエラー
- Given `@slant 5` を明示済みのチャート
- When `@step 5` をパースする
- Then `ParseError::InvalidStepSlant` が返る (step == slant ちょうどは hold 部分が 0 になるためエラー)

## @not-implemented
### Scenario: `@slant` 明示済みなら `@step` 設定でクランプされない (エラーにならないケース)
- Given `@slant 3` を明示済みのチャート
- When `@step 10` をパースする
- Then パースエラーは発生しない
- And パース後の `params.step == 10`
- And パース後の `params.slant == 3` (= 明示値 3 のまま、`step / 2 = 5` への自動引き上げや変更は一切起きない)

## @not-implemented @edge-case
### Scenario: `@slant 0` 明示後の `@step` はクランプ抑止されるが step > 0 なら通る
- Given `@slant 0` を明示済みのチャート (`@slant` 明示フラグ ON、slant=0)
- When `@step 1` をパースする
- Then パースエラーは発生しない (step=1 > slant=0)
- And パース後の `params.slant == 0` (`@step` によって `step / 2 = 0.5` に書き換わらない)

## @not-implemented @negative
### Scenario: `@slant 0` 明示後の `@step 0` はクランプ抑止 + step 正値必須で弾かれる
- Given `@slant 0` を明示済みのチャート
- When `@step 0` をパースする
- Then `ParseError::InvalidLength` が返る (`@step` 自体の正値必須検査が先に発火)
- Note: 仕様上 `@step` は「正値かつ有限値必須」のため `InvalidLength` が優先される

## @not-implemented
### Scenario: `@step` を 2 回連続で設定 (両方とも `@slant` 未明示) → 毎回クランプ評価される
- Given デフォルト状態 (slant=5、`@slant` 未明示) のチャート
- When `@step 4` をパースする (1 回目)
- And 続けて `@step 100` をパースする (2 回目)
- Then 1 回目の後: `params.slant == 2` (= `min(5, 2)`)
- And 2 回目の後: `params.slant == 2` (= `min(2, 50)`、クランプは「下方向」のみで値を拡大しない)
- And `params.step == 100` (最終値)
- And いずれのステップでもパースエラーは発生しない

## @not-implemented
### Scenario: per-row `@step` 途中変更 × 自動クランプ (`@slant` 未明示)
- Given デフォルト状態 (slant=5、`@slant` 未明示)
- When `@step 4` → 信号行 `A _~_~` → `@step 100` → 信号行 `B _~_~` の順でパースする
- Then 信号 A のレイアウトは step=4, slant=2 で計算される
- And 信号 B のレイアウトは step=100, slant=2 で計算される (2 回目のクランプも下方向のみで slant は 2 のまま)
- And パースエラーは発生しない

## @not-implemented
### Scenario: 自動クランプで救済された slant がそのまま後続信号のレイアウトに反映される
- Given デフォルト状態 (slant=5、`@slant` 未明示)
- When `@step 6` をパースし、続いて信号行 `A ___~~~` をパースする
- Then `params.slant == 3` (= `min(5, 6/2) = 3` にクランプ)
- And 信号 A のレイアウトでは slant=3 で SingleEdge が描画される (`Low×3 + Transition(slant=3) + High×3` の幅構成)

## @not-implemented
### Scenario: `@step` 後に `@slant` を明示 → 以降の `@step` ではクランプ抑止される
- Given デフォルト状態 (slant=5、`@slant` 未明示)
- When `@step 4` (slant が 2 に自動クランプされる) → `@slant 3` (明示) → `@step 100` の順でパースする
- Then 最初の `@step 4` 後: `params.slant == 2`
- And `@slant 3` 後: `params.slant == 3` (明示フラグ ON)
- And 最後の `@step 100` 後: `params.slant == 3` (`step / 2 = 50` へのクランプは発火しない = 拡大しないし変更もしない)
- And パースエラーは発生しない

## @not-implemented
### Scenario: `@slant` 明示フラグは信号行を跨いで保持される
- Given デフォルト状態
- When `@slant 4` を明示 → 信号行 `A _~_~` → `@step 5` をパースする
- Then `@slant 4` 後: `params.slant == 4`、明示フラグ ON
- And `A _~_~` パース後も明示フラグは保持される
- And `@step 5` 後: `params.slant == 4` (クランプ抑止) かつ `params.step == 5`
- And `step > slant` (5 > 4) なのでパースエラーは発生しない

## @not-implemented @negative @edge-case
### Scenario: `@slant` 明示フラグは信号行を跨いで保持される (エラー側)
- Given デフォルト状態
- When `@slant 4` を明示 → 信号行 `A _~_~` → `@step 4` をパースする
- Then `@step 4` 設定時にクランプは抑止されている (`@slant` 明示フラグが信号行を跨いで保持されているため)
- And `step <= slant` (4 <= 4) が成立し `ParseError::InvalidStepSlant` が返る

## @not-implemented @smoke
### Scenario: デフォルト step=25 のままで何も設定しない場合はエラーにならない (基準確認)
- Given TCML の冒頭にパラメータ宣言なし、信号行 `A _~` のみ
- When パースする
- Then `params.step == 25`、`params.slant == 5`
- And パースエラーは発生しない (デフォルト同士 25 > 5 なので step > slant 成立)

## @not-implemented
### Scenario: `@step` 未設定状態で `@slant` のみ明示 → クランプ機構と独立に通常の `step > slant` 検査が走る
- Given デフォルト状態 (step=25)
- When `@slant 12` をパースする
- Then パースエラーは発生しない (step=25 > slant=12)
- And `params.slant == 12`、`@slant` 明示フラグ ON

## @not-implemented @negative
### Scenario: `@step` 未設定状態で `@slant` のみ明示し step <= slant となるケース
- Given デフォルト状態 (step=25)
- When `@slant 25` をパースする
- Then `ParseError::InvalidStepSlant` が返る (step=25 <= slant=25)
- Note: 自動クランプ機構は `@step` 設定時にのみ発火する。`@slant` 側設定時は従来通り `step <= slant` 検査が走る。

---

## ParseError 位置情報の必須化

`docs/spec/tcml-format.md` §エラー一覧 §位置情報の必須化 の検証。
すべての `ParseError` バリアントは `(line, col, length)` を保持する。`line` / `col` は 1 始まり、`length` は文字単位の範囲長。`length == 0` は「列 `col` の直前 / 直後の挿入点」を示す挿入位置エラー。

core 層では文字列フォーマットは規定しない (CLI / エディタ / Wasm の責務)。本セクションは core 側のデータ保持のみを検証する。

### 1 始まり (line / col)

## @not-implemented @smoke
### Scenario: line=1 col=1 のエラーで (1, 1, length) が保持される
- Given TCML の冒頭 (1 行目 1 桁目) に位置するトークンがエラーになる入力 (例: 信号行を 1 行目に置き、レベル文字列を空白 1 個から始める等)
- When パースする
- Then 返る `ParseError` の `line == 1`、`col == 1`、`length >= 0` (バリアントに応じた値) を持つ

## @not-implemented
### Scenario: 後続行 (line >= 2) のエラーは line=N で 1 始まり
- Given 1 行目が有効なコメントで、3 行目の 7 桁目から始まるトークンが `ParseError::InvalidLength` を起こす TCML (例: `@step xyz`)
- When パースする
- Then 返る `ParseError` の `line == 3`、`col == 7` (1 始まり)
- And `length` は `xyz` の文字長 3 と一致

## @not-implemented
### Scenario: 空行・コメント行を挟んでも line 番号が物理行で正しい
- Given 1 行目 `// comment` / 2 行目 空行 / 3 行目 `// comment` / 4 行目にエラー行がある TCML
- When パースする
- Then 返る `ParseError` の `line == 4` (空行・コメント行も物理行としてカウント)

## @not-implemented
### Scenario: CRLF 改行でも line がインクリメントされる
- Given CRLF 改行で 3 行目にエラーがある TCML
- When パースする
- Then 返る `ParseError` の `line == 3` (CRLF も LF 同等に 1 改行扱い)
- And `col` は CR を含まない論理桁を指す

### length の意味

## @not-implemented @smoke
### Scenario: length >= 1 のエラーは該当トークンの文字長と一致
- Given `@step xyz` のように `xyz` (3 文字) を引数とするエラー TCML
- When パースする
- Then 返る `ParseError` の `length == 3` (トークン `xyz` の文字長)

## @not-implemented @edge-case
### Scenario: length == 0 は挿入位置エラー (UnclosedQuote 行末)
- Given `SigA "hello` のように `"` が行末まで閉じない TCML
- When パースする
- Then 返る `ParseError::UnclosedQuote` の `length == 0`
- And `col` は行末の挿入点 (`hello` の末尾の直後) を指す

## @not-implemented @edge-case
### Scenario: length == 0 は EOF 直前の挿入位置でもよい
- Given TCML 最終行末尾 (改行なし) で `"` が閉じない `UnclosedQuote`
- When パースする
- Then 返る `ParseError` の `length == 0`
- And `line` は最終行の番号、`col` は最終行末尾の挿入点を指す

### 文字単位 (UTF-8 byte ではない)

## @not-implemented
### Scenario: マルチバイト文字を含む行で col は文字単位
- Given 日本語信号名 `同期信号 _~_~` を含む行で、レベル文字列内の不正文字がエラーになる TCML
- When パースする
- Then 返る `ParseError` の `col` は文字単位の桁 (`同`=1, `期`=2, `信`=3, `号`=4, ` `=5, ... の数え方)
- And `length` も文字単位で、エラートークン中のマルチバイト文字を 1 文字 1 単位として数える

## @not-implemented
### Scenario: タブ文字は core 層では 1 文字 1 単位
- Given タブを含むエラー行の TCML (例: `\t@step xyz`)
- When パースする
- Then 返る `ParseError` の `col` はタブを 1 文字として数えた論理桁
- Note: 4 スペースへの展開は CLI / エディタ側の表示処理。core 層は展開しない。

### バリアント別

## @not-implemented
### Scenario: すべての ParseError バリアントが (line, col, length) を持つ
- Given `docs/spec/tcml-format.md` §エラー一覧 に列挙される全バリアントを起こすそれぞれの TCML 入力
- When 各入力をパースする
- Then 各 `ParseError` のバリアントから `line` / `col` / `length` の 3 フィールドが取得できる (型レベルで全バリアント共通)
- And いずれのバリアントでも `line >= 1`、`col >= 1`、`length >= 0`

## @not-implemented
### Scenario: `ParseError::UnclosedQuote` の length と col
- Given `"Data Bus _~_~` (閉じない `"`、開始 col=1) という行
- When パースする
- Then `ParseError::UnclosedQuote` の `col` は開始 `"` の桁 (col=1) または行末挿入点のどちらかを示す (実装裁量だが文書化された片方に統一)
- And `length` は閉じない引用部全体の文字長 (length>=1) または行末挿入点表現 (length=0) のどちらかに統一

## @not-implemented
### Scenario: `ParseError::InvalidLength` の範囲は引数トークン全体
- Given `@step xyz` (`xyz` は col=7 から 3 文字)
- When パースする
- Then 返る `ParseError::InvalidLength` の `col == 7`、`length == 3` (引数トークン `xyz` 全体)

## @not-implemented
### Scenario: `ParseError::DontCareWithoutAnchor` の位置は `?` 自身
- Given `SigA ?==` (`?` は行先頭 `SigA ` の直後)
- When パースする
- Then `ParseError::DontCareWithoutAnchor` の `col` は `?` の桁を指す
- And `length == 1` (`?` 1 文字)

### error 本文 (core 側で固定文字列を持つ)

## @not-implemented
### Scenario: ParseError は error 本文文字列を持つ
- Given 任意のパースエラーを起こす TCML
- When パースする
- Then `ParseError` から「何が起きたか」を述べる error 本文文字列が取得できる (英語固定、i18n しない)
- And 文末にピリオド `.` を含まない

### 組み合わせ / エッジケース

## @not-implemented @edge-case
### Scenario: 1 行内に複数候補エラーがあっても最初の 1 件を返す
- Given 1 行内で 2 箇所のパースエラー候補がある TCML
- When パースする
- Then 返る `ParseError` は左から最初に検出されたものの位置 (line, col, length) を持つ
- Note: パーサーの停止点は実装裁量。最低保証は「位置情報が一貫している」こと

## @not-implemented @edge-case
### Scenario: BOM 付きでも col は BOM を除いた論理桁
- Given 先頭 BOM (U+FEFF) 付きで 1 行目にエラーがある TCML
- When パースする
- Then 返る `ParseError` の `col` は BOM を含めずに 1 始まりで数える (例: 1 行目先頭エラーで col=1)
- Note: BOM 扱いの仕様 (保持 / 除去) が未定義なら spec gap

## @not-implemented @edge-case
### Scenario: 行末 LF 直前のエラー位置
- Given `@step xyz\n` で `xyz` が行末 LF 直前 (col=7, length=3) のエラー
- When パースする
- Then `col == 7`、`length == 3` (LF は length に含まれない)

## @not-implemented @edge-case
### Scenario: 同一の (line, col, length) を持つエラーが繰り返しパースしても安定する
- Given 同一の不正 TCML を 2 回パースする
- When 各パース結果の `ParseError` を比較する
- Then `line` / `col` / `length` の 3 フィールドが完全に一致する (パース順序や hashmap 等で揺れない)

---

## `@ruler` 寄付モデル (パース時スナップショット)

`docs/spec/tcml-format.md` §「`@ruler` の詳細」の検証。各信号行/`@skip` 行は commit 時点で有効な `ruler` (on/off)・`ruler_color`・`step`・`units` をスナップショットし、`@ruler on` の行はサイドカー `Vec<RulerContribution { x: Px, color: Color }>` に `units + 1` 本ぶんの寄付情報を保持する。

### デフォルト値

## @not-implemented @smoke
### Scenario: `@ruler` のデフォルト値は `on`
- Given `@ruler` 系ディレクティブを 1 つも含まない TCML (例: `A _~_~`)
- When パーサーでパースする
- Then すべての信号行のサイドカー `ruler_contributions` は寄付ありの状態で生成される (各行 `units + 1` 本)
- And パース後の有効パラメータ `params.ruler == true` (on 相当)

## @not-implemented
### Scenario: `@ruler_color` のデフォルト値は `#a0a0a0`
- Given `@ruler on` のみ指定し `@ruler_color` を明示しない TCML
- When パーサーでパースする
- Then `@ruler on` 状態で commit された行の `ruler_contributions[*].color` はすべて `Color("#a0a0a0")`

### `@ruler on` / `@ruler off` の引数バリデーション

## @not-implemented @smoke
### Scenario: `@ruler on` が有効パラメータを切り替える
- Given `@ruler off` で off 状態にしている
- When `@ruler on` をパースする
- Then パースエラーは発生しない
- And パース後の `params.ruler == true`

## @not-implemented
### Scenario: `@ruler off` が有効パラメータを切り替える
- Given `@ruler on` 状態
- When `@ruler off` をパースする
- Then パースエラーは発生しない
- And パース後の `params.ruler == false`

## @not-implemented @negative
### Scenario: `@ruler` の引数が `on` / `off` 以外はエラー
- Given `@ruler maybe` という行
- When パーサーでパースする
- Then `ParseError` が返る (引数不正)
- And パース後の `params.ruler` は変化しない (= 既存値のまま)

## @not-implemented @negative
### Scenario: `@ruler` の引数欠落はエラー
- Given `@ruler` (引数なし) という行
- When パーサーでパースする
- Then `ParseError` が返る (引数欠落)

## @not-implemented @negative
### Scenario: `@ruler_color` の引数が不正な色形式はエラー
- Given `@ruler_color not-a-color` という行
- When パーサーでパースする
- Then `ParseError` が返る (`Color::parse` 失敗)
- And パース後の `params.ruler_color` は変化しない (= 既存値のまま)

### 寄付の発火条件 (どの行が寄付源か)

## @not-implemented @smoke
### Scenario: `@ruler on` 状態で commit された信号行は寄付する
- Given `@step 10`、`@ruler on`、信号行 `A _~_~_~` (units = 6)
- When パースする
- Then 信号行 A の `ruler_contributions` は 7 本 (`{i × 10 | 0 ≤ i ≤ 6}` = `[0, 10, 20, 30, 40, 50, 60]`)
- And 各寄付の `color` は `#a0a0a0` (デフォルト)

## @not-implemented
### Scenario: `@ruler on` 状態で commit された `@skip` 行も寄付する
- Given `@step 10`、`@ruler on`、`@skip(3)` 行
- When パースする
- Then `@skip(3)` 行の `ruler_contributions` は 4 本 (`{0, 10, 20, 30}`、units = 3、両端含む)

## @not-implemented
### Scenario: `@ruler off` 状態で commit された信号行は寄付しない
- Given `@step 10`、`@ruler on`、信号行 A、`@ruler off`、信号行 B
- When パースする
- Then 信号行 A の `ruler_contributions` は非空 (寄付あり)
- And 信号行 B の `ruler_contributions` は空 (寄付ゼロ)

## @not-implemented
### Scenario: `@title` 行は寄付源にならない
- Given `@ruler on`、`@title "Section"`、信号行 `A _~`
- When パースする
- Then `@title` 行にはサイドカー `ruler_contributions` が存在しない、または空
- And 信号行 A は通常通り寄付する

## @not-implemented
### Scenario: コメント行は寄付源にならない
- Given `@ruler on`、`// コメント`、信号行 `A _~`
- When パースする
- Then コメント行は `ChartDocument.lines` に含まれず、寄付も発生しない
- And 信号行 A のみが寄付する

## @not-implemented
### Scenario: ディレクティブ単独行は寄付源にならない
- Given `@ruler on`、`@bgcolor0 #eee`、信号行 `A _~`
- When パースする
- Then `@bgcolor0 #eee` の行自体は寄付情報を生成しない
- And 寄付するのは信号行 A のみ

### スナップショット (color / step / units)

## @not-implemented
### Scenario: `@ruler_color` は commit 時点の値でスナップショットされる
- Given `@step 10`、`@ruler on`、`@ruler_color #aaa`、信号行 A、`@ruler_color #bbb`、信号行 B
- When パースする
- Then 信号行 A の全寄付の `color` は `#aaa`
- And 信号行 B の全寄付の `color` は `#bbb`
- And 後の `@ruler_color #bbb` 宣言で行 A の寄付色は変化しない (スナップショット)

## @not-implemented
### Scenario: `@step` は commit 時点の値で寄付位置を計算する
- Given `@ruler on`、`@step 10`、信号行 A (units = 4)、`@step 25`、信号行 B (units = 4)
- When パースする
- Then 信号行 A の `ruler_contributions` の x 値は `{0, 10, 20, 30, 40}` (step=10 ベース)
- And 信号行 B の `ruler_contributions` の x 値は `{0, 25, 50, 75, 100}` (step=25 ベース)
- And 後の `@step 25` 宣言で行 A の寄付位置は変化しない (スナップショット)

## @not-implemented
### Scenario: `units` は commit 時点の信号行が持つ表示単位数を使う
- Given `@step 10`、`@ruler on`、信号行 `A _~_~` (4 文字 = units 4) と信号行 `B _~_~_~_~` (8 文字 = units 8)
- When パースする
- Then 信号行 A の寄付本数は 5 (= units + 1)、x 値は `{0, 10, 20, 30, 40}`
- And 信号行 B の寄付本数は 9 (= units + 1)、x 値は `{0, 10, 20, 30, 40, 50, 60, 70, 80}`

### `@ruler` × `@ruler_color` × 信号行の組み合わせ

## @not-implemented @edge-case
### Scenario: `@ruler off` 中に `@ruler_color` を変えても、後で `@ruler on` した行は新色で寄付
- Given `@ruler on`、`@ruler_color #aaa`、信号行 A、`@ruler off`、`@ruler_color #bbb`、`@ruler on`、信号行 B
- When パースする
- Then 信号行 A の寄付色は `#aaa`
- And 信号行 B の寄付色は `#bbb`
- And `@ruler off` 中の `@ruler_color #bbb` は記録されるが、寄付は発生しない

## @not-implemented @edge-case
### Scenario: `@ruler on` / `@ruler off` を複数回トグルしても各行の寄付は独立
- Given `@ruler on`、信号行 A、`@ruler off`、信号行 B、`@ruler on`、信号行 C
- When パースする
- Then 行 A は寄付あり、行 B は寄付ゼロ、行 C は寄付あり
- And `@ruler off` で行 A の過去寄付は破棄されない (サイドカーは行毎に独立)

## @not-implemented @edge-case
### Scenario: `@ruler off` 状態のデフォルトのみで、すべての行のサイドカーは空
- Given `@ruler` 系ディレクティブを一切宣言せず、信号行 A / B / C を含む TCML
- When パースする
- Then すべての信号行のサイドカー `ruler_contributions` は空
- And `@skip` 行があってもサイドカーは空

## @not-implemented
### Scenario: 同一行内では x 値が小さい順に並ぶ
- Given `@step 10`、`@ruler on`、信号行 `A _~_~_~`
- When パースする
- Then `ruler_contributions` の x 値は昇順 (`[0, 10, 20, 30, 40, 50, 60]`) で出現する
- Note: 仕様としての保証は「同一の x は 1 本に統合される」ことのみ。並び順は実装裁量だが、x 昇順が自然 (テストで明確化)。
