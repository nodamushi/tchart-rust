# パースエラーの位置とメッセージ要件

`docs/spec/tcml-format.md §不正な入力 (パースエラー)` と §エラーがユーザーに示すべき情報 を補強し、各不正入力に対して **どこを (line / column / length)** と **メッセージに何を含むか** を具体的なシナリオで固定する。

`tcml-parser.feature.md` (文法/正常系) に対して、本ファイルは「壊れた入力に対してユーザーに何を見せるべきか」を規定する。実装側の enum 構造・型名は記述しない (それは実装の責務)。

## 全体ルール

1. column / length は **Unicode スカラ単位** (1 文字 = 1 ch、バイト単位ではない)、1 始まり。
2. column は **問題のトークンの先頭** を指す。`@directive` 全体ではなく、エラーの原因となった具体的なトークン (属性、値、文字、識別子) を指す。
3. length は **問題のトークンの文字数**。`0` は「列 column の直前 / 直後の挿入点」エラー (該当文字幅を持たない、典型例: 末尾未閉のクォート) に限り許される。
4. メッセージは英語固定。可能な限り問題のトークン文字列を二重引用符で含める。文末にピリオドは付けない。
5. 各シナリオで `Given <input>` / `Then column = N` / `And length = M` / `And メッセージに "..." を含む` を最低限規定する。

各シナリオの末尾に `@not-implemented` タグが付いているものは本 feature 追加時点でまだ実装が伴っていない (典型例: メッセージに特定の文言を要求するが実装は一般メッセージしか返さない)。fix が入ったタイミングでタグを外す。

---

## `@clock(...)` 属性エラー

### Scenario: `@clock(_=3,~3)` で `~3` が指摘される
- Given `@clock(_=3,~3)↵Clock` の入力
- Then line = 1, column = 12 ← `~` の位置
- And length = 2 ← `~3` の文字数
- And メッセージに `~3` を含む

### Scenario: `@clock(pos, foo=1)` で `foo=1` が指摘される
- Given `@clock(pos, foo=1)↵Clock` の入力
- Then column = 13、length = 5
- And メッセージに `foo=1` を含む

### Scenario: `@clock(_=abc)` で値の型不一致が指摘される
- Given `@clock(_=abc)↵Clock` の入力
- Then column = 8、length = 5
- And メッセージに `_=abc` を含む

### Scenario: `@clock(_=0)` で `_` がゼロのときは指摘される
- Given `@clock(_=0)↵Clock` の入力
- Then column = 8、length = 3

### Scenario: `@clock(start=foo)` で未知 phase が指摘される
- Given `@clock(start=foo)↵Clock` の入力
- Then column = 8、length = 9

### Scenario: `@clock(pos, pos)` で edge 重複の 2 つめが指摘される
- Given `@clock(pos, pos)↵Clock` の入力
- Then column = 13 ← **2 番目** の `pos`
- And length = 3

### Scenario: `@clock(_=3, _=5)` で `_` 重複の 2 つめが指摘される
- Given `@clock(_=3, _=5)↵Clock` の入力
- Then column = 13 ← 2 番目の `_`
- And length = 3

### Scenario: `@clock(_=3,~3` で `)` 欠落が指摘される
- Given `@clock(_=3,~3↵Clock` (閉じ paren 欠落) の入力
- Then column = 7 ← `(` の位置
- And length = 7 ← `(_=3,~3` 全体
- And メッセージに `(_=3,~3` を含む

---

## `@signal(...)` 属性エラー

### Scenario: `@signal(unknownkey)` で未知属性名が指摘される
- Given `@signal(unknownkey)↵Sig _` の入力
- Then column = 9 ← `unknownkey` の先頭
- And length = 10
- And メッセージに `unknownkey` を含む

### Scenario: `@signal(overline, foo)` で 2 つめの `foo` が指摘される
- Given `@signal(overline, foo)↵Sig _` の入力
- Then column = 19 ← `foo` の先頭 (`(`=8、`overline,`=9..17、空白=18、`foo`=19)
- And length = 3
- And メッセージに `foo` を含む

### Scenario: `@signal(overline, overline)` で重複が指摘される
- Given `@signal(overline, overline)↵Sig _` の入力
- Then column = 19 ← 2 つめ `overline`
- And length = 8
- And メッセージに `overline` を含む

### Scenario: `@signal(overline` で `)` 欠落が指摘される
- Given `@signal(overline↵Sig _` の入力
- Then メッセージに「閉じ paren が無い」旨を含む @not-implemented

---

## `@->` (矢印) エラー

### Scenario: `@-> @{a}, @{b}` で paren 欠落が指摘される
- Given `Sig _@{a}~@{b}_↵@-> @{a}, @{b}` の入力
- Then 引数全体が underline される (length ≥ 1)
- And メッセージに「parens が必要」旨を含む @not-implemented

### Scenario: `@-> (@{a})` で endpoint 不足が指摘される
- Given `Sig _@{a}_↵@-> (@{a})` の入力
- Then メッセージに「endpoint が 2 つ必要」旨を含む @not-implemented

### Scenario: `@-> (@{a}, @{b}, foo=bar)` で未知属性が指摘される
- Given `Sig _@{a}~@{b}_↵@-> (@{a}, @{b}, foo=bar)` の入力
- Then column は `foo=bar` の先頭、length = 7
- And メッセージに `foo=bar` を含む

### Scenario: `@-> (..., color=red, color=blue)` で重複が 2 つめを指す
- Given アンカー定義 + `@-> (@{a}, @{b}, color=red, color=blue)` の入力
- Then column は 2 番目 `color=blue` の位置、length = 10

---

## アンカー (`@{name}` / `@N`) エラー

### Scenario: アンカー重複が 2 番目の `@{a}` を指す
- Given `Sig _@{a}~@{a}_` の入力
- Then column は 2 番目 `@{a}` の `@` の位置
- And length = 4 ← `@{a}` の文字数

### Scenario: 未定義アンカー参照が `@{undef}` 全体を指す
- Given `Sig _~_↵@-> (@{undef}, @{x})↵Sig2 _@{x}_` の入力
- Then column は `@{undef}` の `@` の位置
- And length = 8 ← `@{undef}` の文字数
- And メッセージに `@{undef}` (またはアンカー名 `undef`) を含む

### Scenario: アンカー名に不正文字が含まれるとその 1 文字を指す
- Given `Sig _@{abc!def}_` の入力
- Then column は `!` の位置 (`@`=col 6, `@{`=col 6..7, `abc`=col 8..10, `!`=col 11)
- And length = 1

---

## レベル文字 / 波形

### Scenario: 制御文字レベルが該当 1 文字を指す
- Given `Sig _\x01_` の入力
- Then column = `\x01` の位置、length = 1

### Scenario: 先頭テキスト文字が該当 1 文字を指す
- Given `Sig abc` の入力 (`a` がレベル前 bare text)
- Then column = `a` の位置、length = 1

### Scenario: 閉じない `[` が `[` の位置を指す
- Given `Sig _[~_` の入力
- Then column = `[` の位置、length = 1

### Scenario: 余分な `]` が `]` を指す
- Given `Sig _~]_` の入力
- Then column = `]` の位置、length = 1

### Scenario: 開きクォート未閉じが `"` の位置を指す
- Given `Sig _ "abc␃` の入力 (EOF まで閉じない)
- Then column = `"` の位置、length = 0 ← 挿入点

---

## `@title` エラー

### Scenario: `@title` 引数なし
- Given `@title↵Sig _` の入力
- Then メッセージに「引数が必要」旨を含む

### Scenario: `@title "未閉じ` (クォート未閉じ)
- Given `@title "未閉じ↵Sig _` の入力
- Then column = 開きクォート `"` の位置、length = 0

### Scenario: `@title "abc\x01"` の制御文字が `\x01` を指す
- Given `@title "abc\x01def"↵Sig _` の入力
- Then column = `\x01` の位置 (`"`=col 8 → `\x01` は `"abc` の後で col 12)
- And length = 1

---

## `@skip(...)` エラー

### Scenario: `@skip 5` (paren なし)
- Given `@skip 5↵Sig _` の入力
- Then メッセージに「paren が必要」旨を含む @not-implemented

### Scenario: `@skip(abc)` 非数値
- Given `@skip(abc)↵Sig _` の入力
- Then column は `abc` の先頭、length = 3
- And メッセージに `abc` を含む

### Scenario: `@skip(-3)` 負値
- Given `@skip(-3)↵Sig _` の入力
- Then メッセージに `-3` を含む

---

## `@overline` エラー

### Scenario: `@overline foo` 引数つき
- Given `@overline foo↵Sig _` の入力
- Then column は `foo` の先頭、length = 3
- And メッセージに `foo` を含む

---

## オーバーレイ `% x y text`

### Scenario: `% abc 2 text` で x が非数値
- Given `% abc 2 text↵Sig _` の入力
- Then column は `abc` の先頭、length = 3
- And メッセージに `abc` を含む

### Scenario: `% 1 abc text` で y が非数値
- Given 上記入力
- Then column は y 側 `abc` の先頭、length = 3

### Scenario: `%` 引数皆無
- Given `%↵Sig _` の入力
- Then メッセージに「x y が必要」旨を含む @not-implemented

---

## 値型パラメータ `@<name> <value>`

### Scenario: 未知パラメータ名
- Given `@unknown 1↵Sig _` の入力
- Then column = `@` の位置、length = `unknown` の文字数 (7)
- And メッセージに `@unknown` を含む

### Scenario: `@step abc` 非数値
- Given `@step abc↵Sig _` の入力
- Then column は `abc` の先頭、length = 3
- And メッセージに `abc` を含む

### Scenario: `@step nan` 非有限
- Given `@step nan↵Sig _` の入力
- Then メッセージに「非有限」旨を含む

### Scenario: `@step -1` 負値
- Given `@step -1↵Sig _` の入力
- Then メッセージに `-1` を含む

### Scenario: `@step 5↵@slant 10↵Sig _` で `slant > step`
- Given 上記入力
- Then メッセージに `5` と `10` の両方を含む

---

## 色

### Scenario: `@bgcolor0 #zzz` 不正 hex digit
- Given `@bgcolor0 #zzz↵Sig _` の入力
- Then column は最初の `z` の位置、length = 1

### Scenario: `@bgcolor0 #1z3` 不正 hex digit
- Given `@bgcolor0 #1z3↵Sig _` の入力
- Then column は `z` の位置、length = 1

### Scenario: `@bgcolor0 pinkish` 未知色名
- Given `@bgcolor0 pinkish↵Sig _` の入力
- Then column = `p` の位置、length = `pinkish` の文字数 (7)

### Scenario: `@bgcolor0 #12345` hex 桁不正
- Given `@bgcolor0 #12345↵Sig _` の入力
- Then column = `#` の位置、length = 6 ← `#12345` 全体

---

## 信号名

### Scenario: 空名
- Given `␣_~↵` (1 文字目空白) の入力
- Then column = 信号名想定位置、length = 0 ← 挿入点

### Scenario: 制御文字を含む名がその 1 文字を指す
- Given `Sig\x01 _~↵` の入力
- Then column = `\x01` の位置 (col 4)、length = 1

---

## メモ

- 本ファイルは「実装が伴ったシナリオ」+ 「直近実装予定のシナリオ (`@not-implemented` タグ付き)」のみを公式記載とする。
- 仕様 (TCML 形式) と本ファイル (テスト) の境界: 仕様は **何が不正か** だけを述べる。本ファイルは **その不正を検出したとき、ユーザーには何が見えるか** を、観察可能な動作 (column / length / メッセージに含む文字列) で固定する。
- 内部 enum 構造・型名・payload 形状は実装の責務であり、ここには書かない。
