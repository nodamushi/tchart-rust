"""TCML help HTML の入力データ (日本語 / 英語)。

build.py がこの DATA_JA / DATA_EN を読んで以下を出力する:
    help/output/tcml-format.html      (日本語)
    help/output/tcml-format.en.html   (英語)

NOTE: 2026-05-10 spec audit による tcml-format.md の追記項目 (HTML テンプレート反映は後続コミット):
- @overline (alias of @signal(overline))
- @skip 空白区切り形式
- @-> の color=/width=/style= key=value 形式
- 全 key=value での `_` ↔ `-` 等価ルール (`@clock(_=N)` の `_` は除外)
- @clockmark_color の静的焼き込み挙動
- @title 行の高さに h_space 加算
- ParseError 一覧: InvalidAnchorName / ClockMissingEdge 追加
- h_space デフォルト値表記の統一 (10 px)

各 section の blocks の中で:
- type="text"          : HTML 文字列をそのまま埋め込む
- type="heading"       : h3/h4 等の小見出し
- type="code"          : TCML コードブロック (シンタックスハイライト付き、tchart-cli は呼ばない)
- type="sample"        : TCML コード + 隣に tchart-cli で生成した SVG を inline
- type="table"         : ヘッダ行 + データ行のテーブル
- type="error_table"   : エラー名を赤強調するテーブル
- type="references"    : references リストを ul で出す
- type="wavedrom_sample": TCML + WaveJSON + 両ツールの SVG を比較表示する複合ブロック
"""

# ---------- 共通 (UI 文字列以外) ----------

REFERENCES_JA = [
    {
        "name": "タイミングチャート清書ツール (東北学院大)",
        "url": "https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html",
    },
    {
        "name": "tchart-coffee (筑波大)",
        "url": "https://dora.bk.tsukuba.ac.jp/~takeuchi/?%E3%82%BD%E3%83%95%E3%83%88%E3%82%A6%E3%82%A7%E3%82%A2%2F%E3%82%BF%E3%82%A4%E3%83%9F%E3%83%B3%E3%82%B0%E3%83%81%E3%83%A3%E3%83%BC%E3%83%88%E6%B8%85%E6%9B%B8%E3%82%B5%E3%83%BC%E3%83%93%E3%82%B9",
    },
]

REFERENCES_EN = [
    {
        "name": "Original tchart (Tohoku Gakuin University)",
        "url": "https://www.mech.tohoku-gakuin.ac.jp/rde/contents/library/tchart/indexframe.html",
    },
    {
        "name": "tchart-coffee (University of Tsukuba)",
        "url": "https://dora.bk.tsukuba.ac.jp/~takeuchi/?%E3%82%BD%E3%83%95%E3%83%88%E3%82%A6%E3%82%A7%E3%82%A2%2F%E3%82%BF%E3%82%A4%E3%83%9F%E3%83%B3%E3%82%B0%E3%83%81%E3%83%A3%E3%83%BC%E3%83%88%E6%B8%85%E6%9B%B8%E3%82%B5%E3%83%BC%E3%83%93%E3%82%B9",
    },
]

# ---------- 日本語版 ----------

DATA_JA = {
    "lang": "ja",
    "title": "TCML",
    "subtitle": "Timing Chart Markup Language — テキストでタイミングチャートを記述する簡易言語",
    "extension": ".tc",
    "labels": {
        "title_suffix": "フォーマット仕様",
        "extension_label": "拡張子",
        "toc": "目次",
        "wavedrom_tcml_input": "TCML 入力",
        "wavedrom_tcml_render": "TCML 描画結果",
        "wavedrom_json_output": "WaveJSON 出力",
        "wavedrom_render": "WaveDrom 描画結果",
        "lang_switch_label": "English",
        "lang_switch_href": "tcml-format.en.html",
    },
    "references": REFERENCES_JA,
    "sections": [
        {
            "id": "overview",
            "num": 1,
            "title": "概要",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>TCML (Timing Chart Markup Language) は、タイミングチャートをテキストで記述するための言語です。シンプルな ASCII 風記法で、信号の Low / High / バス / 不定値・遷移・矢印・クロック展開などを表現できます。</p>",
                },
                {"type": "text", "content": "<p>本実装は以下の実装・定義を参考にさせていただいております。この場での謝辞を述べさせていただきます。ありがとうございます。</p>"},
                {"type": "references"},
                {"type": "text", "content": "<p>詳細かつ美しい図を生成するには <a href=\"https://wavedrom.com/\">WaveDrom</a> を利用すべきです。本ツールで生成される図を DataSheet に記載すべきではありません。</p>"},
                {
                    "type": "text",
                    "content": "<p>しかし、<a href=\"https://wavedrom.com/\">WaveDrom</a> の記法は難しく、なんとなく考えながら記述するには不向きです。東北学院大学の熊谷先生、筑波大学の武内先生らが考案してきたタイミングチャートのほうが思考段階において、圧倒的に楽です。</p>",
                },
                {
                    "type": "text",
                    "content": "<p>実際に著者が利用していた際に、こんなことが書けたらな、と数年前に思っていたことを実現するために GW を作って作られました。なお、著者本人はもうこの手の設計をしてないので使う予定がないんですがね。</p>",
                },
            ],
        },
        {
            "id": "start",
            "num": 2,
            "title": "TCMLについて",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>左側に信号名を、右側にレベル文字列を空白で区切って記述します。</p>",
                },
                {
                    "type": "text",
                    "content": "<p><code>_</code> は Low、<code>~</code> は High、<code>=</code> は Bus、<code>?</code> は不定値を表します。下記のコードを <code>tchart</code> CLI に渡すと右の SVG が生成されます。</p>",
                },
                {
                    "type": "text",
                    "content": "<p>下記のコードを <code>tchart</code> CLI に渡すと右の SVG が生成されます。</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 8
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

Clock   _~_~_~_~_~_~
"Data
-Data"  =<D0>====X<D1>====X<D2>====
Enable  ____~~~~____
Output  _______~~~~~____
""",
                },
            ],
        },
        {
            "id": "linetypes",
            "num": 3,
            "title": "行の種類",
            "blocks": [
                {
                    "type": "table",
                    "headers": ["種類", "先頭文字", "説明"],
                    "rows": [
                        ["コメント行", "<code>//</code>", "無視されます。<code>//</code> は行頭・行中いずれでもクォート外で出現するとそれ以降が行末まで破棄されます。単独の <code>/</code> や <code>#</code> は通常文字として扱います。"],
                        [
                            "パラメータ行",
                            "<code>@</code>",
                            "パラメータ設定 / 行ディレクティブ (<code>@title</code> / <code>@skip</code> / <code>@clock</code> / <code>@-&gt;</code>) / 信号属性 (<code>@signal</code>) のいずれかです。",
                        ],
                        [
                            "文字書き込み行",
                            "<code>%</code>",
                            "指定座標に文字列を配置します。",
                        ],
                        [
                            "タイミング記述行",
                            "その他",
                            "信号名とレベル文字列です。",
                        ],
                    ],
                },
            ],
        },
        {
            "id": "signal-name",
            "num": 4,
            "title": "信号名",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>有効な UTF-8 文字列を指定します。制御文字を含めることはできません (<code>\\n</code> のみ例外)。空文字も不可です。</p>",
                },
                {"type": "heading", "level": 3, "text": "複数行信号名"},
                {
                    "type": "text",
                    "content": "<p>信号名を <code>\"</code> で囲むと、内部に改行を含めることができます。</p>",
                },
                {
                    "type": "code",
                    "code": """"Data
Bus" =<D0>====X<D1>====
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>開始の <code>\"</code> は行頭にある必要があります。</li><li>閉じの <code>\"</code> の後には空白を挟んでレベル文字列が続きます。</li></ul>",
                },
                {"type": "heading", "level": 4, "text": "エスケープ (\"...\" 内のみ)"},
                {
                    "type": "table",
                    "headers": ["シーケンス", "意味"],
                    "rows": [
                        ["<code>\\\"</code>", "リテラルの <code>\"</code>"],
                        ["<code>\\n</code>", "改行"],
                        ["<code>\\\\</code>", "リテラルの <code>\\</code>"],
                    ],
                },
            ],
        },
        {
            "id": "levels",
            "num": 5,
            "title": "レベル記号",
            "blocks": [
                {
                    "type": "table",
                    "headers": ["記号", "意味", "形状"],
                    "rows": [
                        ["<code>_</code>", "Low", "下端の単線"],
                        ["<code>~</code>", "High", "上端の単線"],
                        ["<code>-</code>", "HiZ", "中央の破線"],
                        ["<code>=</code>", "Bus", "上下 2 本のレール"],
                        [
                            "<code>?</code>",
                            "Don't care",
                            "塗り潰し + 内部に直前レベル位置の線",
                        ],
                    ],
                },
                {"type": "heading", "level": 4, "text": "例"},
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 12
@slant 2

Low       ________
High      ~~~~~~~~
HiZ       --------
Bus       =<A>====
DontCare  ___?????
""",
                },
                {
                    "type": "text",
                    "content": "<p>同じ記号が連続する場合 (<code>__</code>, <code>~~</code>, <code>??</code> 等) は 1 つの区間としてまとめて扱われます。</p>",
                },
            ],
        },
        {
            "id": "aux",
            "num": 6,
            "title": "補助記号",
            "blocks": [
                {
                    "type": "table",
                    "headers": ["記号", "意味", "x 進行"],
                    "rows": [
                        [
                            "<code>:</code>",
                            "Gap (1 単位の空白、信号連続性を断絶)",
                            "<code>step</code>",
                        ],
                        [
                            "<code>X</code>",
                            "Bus 値変化 (BusCross)",
                            "<code>step</code> (cross 部 <code>slant</code> + body 部 <code>step - slant</code>、または信号行頭等で cross 省略時は body のみ <code>step</code>)",
                        ],
                        [
                            "<code>?</code>",
                            "Don't care マーカー (周辺 bus 区間が不定値)",
                            "0",
                        ],
                        ["<code>|</code>", "縦線 (ガイド線)", "0"],
                        ["<code>[</code> / <code>]</code>", "ハイライト開始/終了", "0"],
                        [
                            "<code>@{name}</code> / <code>@N</code>",
                            "アンカー",
                            "0",
                        ],
                    ],
                },
                {
                    "type": "text",
                    "content": "<p><code>X</code> は cross 遷移 + body (Bus 1 単位、新値) の 2 部品。X 全体で level char 1 個ぶん = <code>step</code> 幅を占め、cross 部が <code>slant</code>、body 部が <code>step - slant</code> になります。前に bus 信号が無い場合 (信号行頭等) は cross が省略され body のみ (幅 <code>step</code>)。</p><p><code>?</code> は幅 0 のマーカー。周辺の連続レベル区間を不定値領域として塗ります。</p>",
                },
                {"type": "heading", "level": 4, "text": "例"},
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 12
@slant 2

Gap        ____:____
BusX       =<A>====X<B>====
Guide      _~__|__~_
Highlight  __[~~~~]__
""",
                },
            ],
        },
        {
            "id": "dontcare",
            "num": 7,
            "title": "不定値 <code>?</code>",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>領域内の線位置は <strong>直前のレベル</strong>から決定されます。</p>",
                },
                {
                    "type": "table",
                    "headers": ["直前アンカー", "領域内の線"],
                    "rows": [
                        ["<code>_</code> Low", "下端"],
                        ["<code>~</code> High", "上端"],
                        ["<code>-</code> HiZ", "中央"],
                        ["<code>=</code> Bus", "bus 包絡 (上下 2 本)"],
                        ["<code>X</code>", "bus 包絡"],
                    ],
                },
                {"type": "heading", "level": 3, "text": "Bus 内 ? の塗り形状"},
                {
                    "type": "text",
                    "content": "<p>Bus 文脈の <code>?</code> (<code>=?=</code> 等) は、前後の波形境界に応じた多角形で塗られます。信号 1 行ぶんの表示エリアからはみ出すことはありません。</p>",
                },
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 12
@slant 3

@title "DontCareAlongBus shapes"

// `@dontcare_color` を途中で書き換えることでハッチ色は行ごとに切り替えられる。
// (デフォルト #bbb / #c00 / #06c / #080 の 4 色を順に使う例)

// デフォルト色 (#bbb)
=?=         ====?====

@dontcare_color #c00
// 両側 Low: /=\\ 形状 (赤)
_=?=_       ____====?====____

// 両側 High: \\=/ 形状 (赤)
~=?=~       ~~~~====?====~~~~

@dontcare_color #06c
// 片側のみ Low (左斜辺、右垂直) (青)
_=?=        ____====?====

// 片側のみ Low (左垂直、右斜辺) (青)
=?=_        ====?====____

@dontcare_color #080
// 片側 High / 片側 Low (混在) (緑)
~=?=_       ~~~~====?====____

// HiZ 絡み: 左 HiZ、右 Bus continue (緑)
-=?=        ----====?====

// Bus + ラベル付き (緑)
=?=L        ==<A>==?====

// Bus (緑)
=?=X        ==X==?==X==
""",
                },
                {"type": "heading", "level": 3, "text": "? のエラー条件"},
                {
                    "type": "text",
                    "content": "<ul><li>信号行が <code>?</code> で始まる、あるいは <code>:</code> <code>|</code> <code>[</code> <code>]</code> <code>@{...}</code> <code>@N</code> のような幅 0 の要素しか先行していない場合はエラーになります (<code>?</code> はどのレベルを don't care にすべきか判定できないため)。</li><li>例: <code>foo ?==</code>、<code>bar ???</code>、<code>baz ?_~</code>、<code>qux :?_~</code>、<code>quux @{a}?_~</code> はすべてエラーです。</li></ul>",
                },
                {"type": "heading", "level": 3, "text": "X / X? パターン"},
                {
                    "type": "table",
                    "headers": ["パターン", "解釈"],
                    "rows": [
                        ["<code>=X=</code>", "Bus(1) + X(cross + body, 新値) + Bus(1)"],
                        ["<code>=X?</code>", "Bus(1) + X(cross + body) + ? (X body が dontcare)"],
                        [
                            "<code>=X?=</code>",
                            "? 領域 = X body + 後 <code>=</code> (2 単位ぶん)",
                        ],
                        [
                            "<code>=?X=</code>",
                            "? 領域 = 前 <code>=</code> (1 単位)、X body + 後 <code>=</code> は新値 Bus",
                        ],
                        [
                            "<code>=X?X=</code>",
                            "? 領域 = X1 body 1 単位、polygon は X1 cross 中点 〜 X2 cross 中点 (六角形)",
                        ],
                        [
                            "<code>~X_</code>",
                            "High + BusOpen + Bus(1, X body) + BusClose + Low (X 隣接 non-bus でも valid)",
                        ],
                        [
                            "<code>XXXX</code>",
                            "信号行頭の連続 X。1 つ目は cross 省略、2 つ目以降は通常の BusCross",
                        ],
                        ["<code>?X=</code>", "エラー (先頭 <code>?</code>)"],
                    ],
                },
            ],
        },
        {
            "id": "label",
            "num": 8,
            "title": "レベル文字列中のラベル <code>&lt;...&gt;</code>",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>レベル区間や遷移にテキストラベルを付与します。</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2

BusLabel   =<A>====X<B>====X<C>====
LowLabel   ____<L>____
HighLabel  ~~~~<H>~~~~
HiZLabel   ----<Z>----
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>ラベル文字列に制御文字 (<code>\\n</code> を除く) は含められません。</li><li><code>&lt;</code> 自体を表示したい場合は <code>\\&lt;</code>、<code>&gt;</code> は <code>\\&gt;</code>、<code>\\</code> は <code>\\\\</code> でエスケープします。</li></ul>",
                },
            ],
        },
        {
            "id": "anchor-arrow",
            "num": 9,
            "title": "アンカー <code>@{name}</code> と矢印 <code>@-&gt;</code>",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>波形上の特定の点を <strong>アンカー</strong> として 0 幅マーカーで記録し、<code>@-&gt;</code> 行で 2 つのアンカーを矢印として結びます。アンカー単独では描画されず、矢印の端点として参照されたときに線が引かれます。</p>",
                },
                {"type": "heading", "level": 3, "text": "最小例"},
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 10
@slant 2

Req   ___@{s}~~~~~~
Ack   ________@{a}~~~

@-> (@{s}, @{a}, red) request
""",
                },
                {"type": "heading", "level": 3, "text": "アンカーの規則"},
                {
                    "type": "text",
                    "content": "<ul><li><code>@{name}</code> または <code>@N</code> (1 以上の整数) で指定します。0 幅マーカーで x 進行に影響しません。</li><li>名前付きと番号付きは別の名前空間として扱われます (<code>@{1}</code> と <code>@1</code> は別物です)。</li><li>同一 ID を重複定義するとエラーになります。</li><li>アンカー名に使える文字は半角英数とアンダースコア・ハイフンで、先頭は英字またはアンダースコアです (正規表現で書くと <code>[A-Za-z_][A-Za-z0-9_-]*</code>)。</li><li><code>?</code> の塗り範囲を決めるときアンカーは無視されません (アンカー単独では <code>?</code> の直前レベルにはなりません)。</li></ul>",
                },
                {"type": "heading", "level": 3, "text": "矢印の書式"},
                {
                    "type": "text",
                    "content": "<p><code>@-&gt; (&lt;始端&gt;, &lt;終端&gt; [, &lt;属性&gt;, ...]) [&lt;テキスト&gt;]</code></p>",
                },
                {
                    "type": "table",
                    "headers": ["カテゴリ", "例", "判別"],
                    "rows": [
                        [
                            "色",
                            "<code>red</code>, <code>#f0f</code>, <code>#ff8800</code>",
                            "<code>Color::parse</code> 成功",
                        ],
                        [
                            "太さ",
                            "<code>2</code>, <code>2px</code>, <code>1.5px</code>",
                            "数値 (単位 <code>px</code> 省略可)",
                        ],
                        [
                            "線種",
                            "<code>solid</code>, <code>dashed</code>, <code>dotted</code>",
                            "キーワード",
                        ],
                        [
                            "矢印頭",
                            "<code>head=end</code>, <code>head=both</code>, <code>head=none</code>",
                            "<code>head=</code> プレフィクス",
                        ],
                    ],
                },
                {
                    "type": "text",
                    "content": "<p>デフォルトは色 = <code>signal_color</code>、太さ = <code>1px</code>、線種 = <code>solid</code>、矢印頭 = <code>end</code> です。属性は順不同で書けますが、同じカテゴリ (色なら色、太さなら太さ) を 2 回指定するとエラーになります。</p>",
                },
                {"type": "heading", "level": 3, "text": "色・太さ・線種・前方参照を組み合わせた例"},
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

Req     ___@{s}~~~~@{e}___
Ack     _______@{a}~~~~

@-> (@{s}, @{a}) request
@-> (@{e}, @{a}, dashed) ack

Bus     =<A>====@1X<B>====@2X<C>====@3
Flag    ____@4~~~~@5___

@-> (@1, @4, red, 2px) A
@-> (@2, @5, blue, head=both) B
@-> (@3, @4, green, dotted) forward ref

Data    __@{d1}~~~~@{d2}___
Out     ___@{o1}~~~~@{o2}__

@-> (@{d1}, @{o1}, #ff8800, solid) start
@-> (@{d2}, @{o2}, dashed, head=none) end
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>前方参照が可能です。<code>@-&gt;</code> 行は TCML 中の任意の位置に書くことができます。</li><li>ラベル等のスタイル (<code>font</code>, <code>signal_color</code> 等) は <code>@-&gt;</code> 行を記述した位置のローカル設定が適用されます。</li><li>未定義のアンカーを参照するとエラーになります。</li><li>矢印同士が重ならないよう自動配置することはしません (書き手側で位置をずらしてください)。</li></ul>",
                },
            ],
        },
        {
            "id": "params",
            "num": 10,
            "title": "パラメータ",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>書式は <code>@&lt;パラメータ名&gt; &lt;値&gt;</code> です。名前は大文字小文字を区別せず、ハイフンとアンダースコアは等価に扱われます (<code>@fontsize</code> と <code>@font-size</code> と <code>@FONT_SIZE</code> はすべて同じです)。</p>",
                },
                {
                    "type": "heading",
                    "level": 3,
                    "text": "グローバルパラメータ (途中変更不可)",
                },
                {
                    "type": "table",
                    "headers": ["名前", "デフォルト", "説明"],
                    "rows": [
                        ["<code>fontsize</code>", "<code>14</code>", "フォントサイズ (px)。レイアウトの基準となります。"],
                        ["<code>lineheight</code>", "<code>1.2</code>", "波形高さの係数 (= <code>fontsize × lineheight</code>)。"],
                        ["<code>capwidth</code>", "<code>0</code>", "信号名欄の幅 (px)。0 のときは自動計算されます。"],
                        ["<code>namepad</code>", "<code>8</code>", "信号名右端と波形左端の余白 (px)。"],
                        ["<code>scale</code>", "<code>1.0</code>", "SVG 全体のスケール係数。"],
                        ["<code>page-margin</code>", "<code>10</code>", "チャート四方の固定余白 (px)。"],
                        ["<code>bgcolor0</code>", "<code>none</code>", "偶数行の背景色。"],
                        ["<code>bgcolor1</code>", "<code>none</code>", "奇数行の背景色。"],
                    ],
                },
                {
                    "type": "heading",
                    "level": 3,
                    "text": "ローカルパラメータ (途中変更可、それ以降に適用)",
                },
                {
                    "type": "table",
                    "headers": ["名前", "デフォルト", "説明"],
                    "rows": [
                        ["<code>step</code>", "<code>25</code>", "level char 1 個ぶんの x 進行幅 (px)。直前に遷移ありの場合、その遷移は本 <code>step</code> 幅の先頭 <code>slant</code> 部分として描画される。<code>step &lt;= slant</code> はパースエラー。<code>@step</code> 設定時に <code>@slant</code> がまだ明示されていなければ slant を <code>step / 2</code> に自動クランプ (既定 slant=5px と小さな <code>@step</code> 値の衝突回避)。"],
                        ["<code>slant</code>", "<code>5</code>", "遷移幅 (px)。0 で垂直エッジになります。SingleEdge / BusOpen / BusClose / BusCross すべてに適用。<code>@slant</code> を一度でも書くとそれ以降の <code>@step</code> は slant を自動調整しません (明示優先)。"],
                        ["<code>h_space</code>", "<code>10</code>", "信号行の上下余白合計 (px)。旧名 <code>signal_gap</code> も同義として受理。"],
                        ["<code>font</code>", "<code>sans-serif</code>", "フォントファミリー (空白入りは <code>\"</code> で囲みます。カンマ区切りでフォールバック順を指定できます)。"],
                        ["<code>signal_color</code>", "<code>black</code>", "信号線の色。"],
                        ["<code>signal_width</code>", "<code>1</code>", "信号線の幅 (px)。"],
                        ["<code>guide_color</code>", "<code>red</code>", "縦線の色。"],
                        ["<code>guide_width</code>", "<code>0.6</code>", "縦線の幅 (px)。"],
                        ["<code>bg</code>", "<code>none</code>", "次の 1 行の背景色 (ローカル上書き)。"],
                        ["<code>highlight_style</code>", "<code>fill=\"#ff8\" stroke=\"none\"</code>", "ハイライト矩形のスタイル。"],
                        ["<code>dontcare_color</code>", "<code>#bbb</code>", "<code>?</code> ハッチ線色。<code>@dontcare_color #c00</code> のように単一の色値を指定するとそれ以降の行の色が切り替わる (途中で再宣言可)。"],
                        ["<code>titlealign</code>", "<code>center</code>", "<code>@title</code> の横揃え (<code>center</code> / <code>left</code> / <code>right</code>)。"],
                        ["<code>clockmark_position</code>", "<code>0.5</code>", "クロック三角形マーカーの頂点位置 (線方向比 0.0..=1.0)。"],
                        ["<code>clockmark_height</code>", "<code>7.5</code>", "クロック三角形マーカーの高さ (px)。"],
                        ["<code>clockmark_width</code>", "<code>6</code>", "クロック三角形マーカーの底辺の幅 (px)。デフォルト値で解決されるときのみ step 連動縮小 <code>min(6, step × 2/3)</code> が適用される (<code>@clockmark_width</code> または <code>mark_width</code> を明示指定したときは縮小しない)。"],
                        ["<code>clockmark_color</code>", "signal_color 継承", "クロック三角形マーカーの塗り色。未指定なら現在有効な signal_color を継承する。"],
                        ["<code>overline_gap</code>", "<code>2</code>", "信号名上線とテキスト cap-top の隙間 (px)。"],
                        ["<code>overline_thickness</code>", "<code>1</code>", "信号名上線の太さ (px)。"],
                        ["<code>ruler</code>", "<code>on</code>", "背景に薄い縦線 (ガイド線) を引くかどうか。値は <code>on</code> / <code>off</code>。詳細は <a href=\"#ruler\">§<code>@ruler</code></a> 参照"],
                        ["<code>ruler_color</code>", "<code>#a0a0a0</code>", "ruler 線の色。途中で変えても、すでに描画されている縦線の色は変わらない"],
                    ],
                },
            ],
        },
        {
            "id": "bg",
            "num": 11,
            "title": "背景色",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>偶奇で塗り分ける <code>@bgcolor0</code> / <code>@bgcolor1</code> (グローバル) と、次の 1 行だけを上書きする <code>@bg</code> (ローカル) があります。</p>",
                },
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 10
@bgcolor0 #eef6ff
@bgcolor1 #fff4ee

A    _~_~_~_~
B    ~_~_~_~_

@bg #ffe4cc
Local _~_~_~_~

After _~_~_~_~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li><code>@bg</code> は次の 1 行 (Signal / Skip / Title) を消費したらリセットされます (<code>@bg none</code> で破棄もできます)。</li><li><code>@bg</code> 指定行では <code>bgcolor0/1</code> を重ね描きしません。</li><li><code>Skip</code> 行と <code>Title</code> 行は <code>bgcolor0/1</code> の偶奇カウントから除外されます。</li></ul>",
                },
                {
                    "type": "heading",
                    "level": 3,
                    "text": "@highlight_style / @dontcare_color",
                },
                {
                    "type": "code",
                    "code": """@highlight_style fill="#8f8" stroke="green" stroke-width="1"
@dontcare_color #c00
""",
                },
                {
                    "type": "text",
                    "content": "<p><code>@highlight_style</code> は SVG 属性を <code>key=\"value\"</code> 形式で空白区切り指定します。<code>@dontcare_color</code> は単一の色値 (<code>@bgcolor0</code> 等と同じ書式) を取り、ハッチ線色を切り替えます。</p>",
                },
            ],
        },
        {
            "id": "skip",
            "num": 12,
            "title": "<code>@skip</code> — 空白行",
            "blocks": [
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

Clock   _~_~_~_~_~_~

@skip(1)

Data    =<A>====X<B>====

@skip(2)

Control ____~~~~____

@skip(0.5)

Flag    ~~____~~____

@skip(20px)

Out     _~~~~___~~~~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>単位なしの数値は <code>lh</code> (line-height 単位) として解釈されます。</li><li>数値 + <code>px</code> でピクセル指定もできます。</li><li>負値や数値として解釈できない値はエラーになります。</li><li>0 は許容しますが、空白行は出力されません (何もしないのと同じ)。</li></ul>",
                },
            ],
        },
        {
            "id": "title",
            "num": 13,
            "title": "<code>@title</code> — タイトル行",
            "blocks": [
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

@title "Default Center Title"

A     _~_~_~_~

@titlealign left
@title "Left Aligned"

B     _~_~_~_~

@titlealign right
@title "Right Aligned"

C     _~_~_~_~

@titlealign center
@title "Back to Center"

D     _~_~_~_~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>引数文字列をタイトル行として描画します。複数行は <code>\"...\"</code> で引用します (信号名と同じエスケープ規則です)。</li><li>1 ファイル中に複数回出現させることができます。</li><li><code>Title</code> 行は <code>bgcolor0/1</code> の偶奇カウントから除外されます。</li><li><code>@titlealign</code> は <code>center</code> / <code>left</code> / <code>right</code> から選び、設定後に出現するすべての <code>@title</code> に適用されます。</li></ul>",
                },
            ],
        },
        {
            "id": "clock",
            "num": 14,
            "title": "<code>@clock</code> — クロック自動展開",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>直後の信号行をクロック信号として展開します。本体が空または部分的なら、最後の状態から <code>pulse</code> を繰り返し展開し、立ち上がり / 立ち下がりに三角形マーカーを自動挿入します。auto 展開の units 数は <code>round(max(他信号の終端ピクセル x) / 当該行の step)</code> で算出され、<code>@step</code> を行ごとに変えても右端がおよそ揃います。「他信号」は auto 拡張対象でない explicit な信号行 (普通の信号行と部分指定 clock の explicit 部分) のみを指し、全信号が auto の場合は target=0 (空波形) になります。複数の auto 行は互いに独立に同じ explicit 集合を参照します。</p>",
                },
                {
                    "type": "sample",
                    "code": """@clock(pos)
ClkPos  _~_~_~

@clock(neg)
ClkNeg  _~_~_~

@clock(both)
ClkBoth _~_~_~

@clock(pos, _=2, ~=1)
ClkWide
""",
                },
                {
                    "type": "text",
                    "content": "<p>書式: <code>@clock</code> / <code>@clock()</code> / <code>@clock([&lt;edge&gt;] [, _=&lt;n&gt;] [, ~=&lt;n&gt;] [, start=&lt;low|high&gt;] [, mark_position=&lt;f32&gt;] [, mark_height=&lt;px&gt;] [, mark_width=&lt;px&gt;] [, mark_color=&lt;color&gt;])</code>。<code>@clock</code> 単独および <code>@clock()</code> は <code>@clock(none)</code> と等価。</p>",
                },
                {
                    "type": "table",
                    "headers": ["属性", "値", "説明"],
                    "rows": [
                        ["<code>edge</code>", "<code>pos</code> / <code>neg</code> / <code>both</code> / <code>none</code>", "三角形マーカーの対象 (省略時 <code>none</code>)。"],
                        ["<code>_=&lt;n&gt;</code>", "正整数", "Low の単位時間数 (省略時 1)。"],
                        ["<code>~=&lt;n&gt;</code>", "正整数", "High の単位時間数 (省略時 1)。"],
                        ["<code>start</code>", "<code>low</code> / <code>high</code>", "開始相 (省略時 <code>low</code>)。"],
                        ["<code>mark_position</code>", "<code>0.0..=1.0</code>", "三角形の頂点位置。"],
                        ["<code>mark_height</code>", "正値", "三角形の高さ。"],
                        ["<code>mark_width</code>", "正値", "三角形の底辺の幅。"],
                        ["<code>mark_color</code>", "色", "塗り色 (省略時 <code>signal_color</code> を継承)。"],
                    ],
                },
                {
                    "type": "text",
                    "content": "<ul><li>属性順は不問です。属性キー名は大文字小文字を区別せず、<code>-</code> と <code>_</code> は等価に扱われます。</li><li><code>@-&gt;</code> 矢印とは <strong>完全に別系統</strong> です。clock 由来のマーカーは <code>@-&gt;</code> の Arrow には混入しません。</li></ul>",
                },
            ],
        },
        {
            "id": "signal-attr",
            "num": 15,
            "title": "<code>@signal</code> — 信号属性",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>直後の信号行に属性を適用します (1 回限りで、適用後にリセットされます)。現在は <code>overline</code> のみ提供しています。</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

@signal(overline)
nReset    ~~~~__~~~~

@signal(overline)
nWrite    ~~__~~__~~

Enable    ____~~~~____

@signal(overline)
"nChip
Enable"   ~~~~____~~~~

Out       __~~~~____~~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li><code>overline</code>: 信号名に上線 (負論理表記) を引きます。複数行信号名では <strong>最上行のみ</strong>に 1 本、幅は<strong>全行のうち最長行</strong>に合わせます。</li><li>位置・太さは <code>@overline_gap</code> / <code>@overline_thickness</code> で制御します。SVG 出力は <code>text-decoration</code> 属性ではなく独立した <code>&lt;line&gt;</code> 要素として描画されます。</li></ul>",
                },
            ],
        },
        {
            "id": "ruler",
            "num": 16,
            "title": "<code>@ruler</code> — 背景の縦線",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>各 step 区切りに薄い縦の破線を引いて、信号の立ち上がり・立ち下がりが揃って見えるようにします。デフォルトでオンになっており、特に書かなくても表示されます。要らないときは <code>@ruler off</code> で消せます。色を変えたいときは <code>@ruler_color</code> を使います。</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 15
@slant 3
@ruler_color #c0c0c0

@clock(pos)
CLK
Data   =D0====XD1====XD2====
Enable ____~~~~________
""",
                },
                {
                    "type": "text",
                    "content": "<p>使い方:</p><ul><li><code>@ruler on</code> / <code>@ruler off</code> — 以降の行の縦線を表示・非表示に切り替えます。</li><li><code>@ruler_color &lt;色&gt;</code> — 以降に書く行の縦線の色を変えます (デフォルトは <code>#a0a0a0</code>)。途中で変えても、すでに表示されている縦線の色は変わりません。</li></ul>",
                },
            ],
        },
        {
            "id": "overlay",
            "num": 17,
            "title": "<code>%</code> — 文字書き込み行",
            "blocks": [
                {
                    "type": "code",
                    "code": "% <x座標> <y座標> <文字列>\n",
                },
                {
                    "type": "text",
                    "content": "<p>指定座標 (px) にテキストを overlay として配置します。座標はチャート左上を原点とします。</p>",
                },
            ],
        },
        {
            "id": "kvrules",
            "num": 18,
            "title": "<code>key=value</code> 規則",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>TCML 中で <code>=</code> を使う属性記述 (<code>@clock(...)</code> の各オプション、<code>@-&gt;</code> の <code>head=</code>、<code>@highlight_style</code> 等) は<strong>すべて</strong>同じ規則に従います。</p>",
                },
                {
                    "type": "text",
                    "content": "<ul><li><code>=</code> の前後の空白は<strong>任意</strong>です。<code>key=value</code> / <code>key =value</code> / <code>key= value</code> / <code>key = value</code> はすべて等価です。</li><li>キー名・値はそれぞれ両端の空白を除いた上で評価されます。</li><li>値に空白を含めたい場合は <code>\"...\"</code> で囲みます。</li></ul>",
                },
            ],
        },
        {
            "id": "wavedrom",
            "num": 19,
            "title": "WaveDrom への引き継ぎ",
            "blocks": [
                {
                    "type": "text",
                    "content": (
                        "<p>本ツール (tchart) は、テキスト編集中にタイミングチャートを思考とともに書き留めるための簡便な道具です。"
                        "仕上げの図をデータシート・発表資料・論文といった公の場に載せる段階では、"
                        "本格的なタイミングチャート描画ツールである <a href=\"https://wavedrom.com/\">WaveDrom</a> をお使いください。"
                        "本ツールは皆さまの最終的な発表の場までご一緒できる<em>真の仲間</em>ではありません。</p>"
                    ),
                },
                {
                    "type": "text",
                    "content": (
                        "<p>ここまでの作業が無駄になることはありません。"
                        "<code>tchart wavedrom</code> サブコマンドで <code>.tc</code> ファイルを "
                        "<a href=\"https://github.com/wavedrom/schema/blob/master/WaveJSON.md\">WaveJSON</a> 形式に変換できます。"
                        "完全互換ではなく、WaveDrom が描画可能な要素 (信号レベル・Bus データ・クロック・アンカー/矢印など) のみを近似マッピングし、"
                        "WaveDrom 側に対応物のないスタイル系 (背景色・フォント・上線・ハイライト等) は黙って落とします。"
                        "TCML での思考をそのまま WaveDrom の世界に持ち込んで、続きの旅をお進めください。</p>"
                    ),
                },
                {
                    "type": "code",
                    "code": "tchart wavedrom chart.tc            # → chart.json\ntchart wavedrom chart.tc -o out.json\n",
                },
                {
                    "type": "text",
                    "content": "<p>変換例として、以下に 3 ケースを掲載しています。各ケースで TCML 入力・<code>tchart svg</code> による TCML 描画結果・<code>tchart wavedrom</code> による WaveJSON 出力・<code>wavedrom-cli</code> による WaveDrom 描画結果を並べて比較できるようにしています。</p>",
                },
                {
                    "type": "heading",
                    "level": 4,
                    "text": "ケース 1: Gap (<code>:</code>) で連続性を切る",
                },
                {
                    "type": "text",
                    "content": "<p>TCML の Gap <code>:</code> は 1 unit 空白で連続性を断絶します。WaveDrom の <code>|</code> (直前 level を 1 unit 延長しつつ視覚的 break を描く) にマップされます。完全に同じ絵にはなりませんが、連続性が切れるというセマンティクスは保存されます。</p>",
                },
                {
                    "type": "wavedrom_sample",
                    "code": "@title 連続性の断絶\nsig1   ~_~_:~_~_\nsig2   ====:====\n",
                    "json": (
                        "{\n"
                        '  "head": { "text": "連続性の断絶" },\n'
                        '  "signal": [\n'
                        '    { "name": "sig1", "wave": "1010|1010" },\n'
                        '    { "name": "sig2", "wave": "=...|=..." }\n'
                        "  ]\n"
                        "}\n"
                    ),
                    "tcml_svg_file": "tcml-demo-gap.svg",
                    "wavedrom_svg_file": "wavedrom-demo-gap.svg",
                },
                {
                    "type": "heading",
                    "level": 4,
                    "text": "ケース 2: バス値の切替 (<code>X</code> = BusCross)",
                },
                {
                    "type": "text",
                    "content": "<p><code>X</code> を挟むとバス区間がそこで区切られ、WaveDrom 出力でも別セグメント (<code>=</code>) になって、それぞれに対応する <code>data</code> エントリが生成されます。X の位置で値が変わる典型的なバスのタイミングを描けます。</p>",
                },
                {
                    "type": "wavedrom_sample",
                    "code": "@title バス値の切替\nclk    ~_~_~_~_\ndata   ==A=X=B=X=C\n",
                    "json": (
                        "{\n"
                        '  "head": { "text": "バス値の切替" },\n'
                        '  "signal": [\n'
                        '    { "name": "clk",  "wave": "10101010" },\n'
                        '    { "name": "data", "wave": "=..=..=.",\n'
                        '      "data": ["A", "B", "C"] }\n'
                        "  ]\n"
                        "}\n"
                    ),
                    "tcml_svg_file": "tcml-demo-bus-x.svg",
                    "wavedrom_svg_file": "wavedrom-demo-bus-x.svg",
                },
                {
                    "type": "heading",
                    "level": 4,
                    "text": "ケース 3: アンカー <code>@{name}</code> と信号間の矢印 <code>@-&gt;</code>",
                },
                {
                    "type": "text",
                    "content": "<p>波形中に埋め込んだ 0 幅アンカー (<code>@{request}</code> 等) を始端・終端として <code>@-&gt;</code> で矢印を引きます。複数アンカーを別の信号に置けば、信号間をまたぐ依存関係 (例: request → ack → complete) を矢印で示せます。WaveDrom 出力では各信号の <code>node</code> プロパティ (波形と同じ長さの文字列、アンカー位置に文字、それ以外は <code>.</code>) と top-level <code>edge</code> 配列にマップされます。</p>",
                },
                {
                    "type": "text",
                    "content": (
                        "<p><strong>変換時に欠落する情報があります。</strong> "
                        "WaveDrom の <code>node</code> 識別子は 1 文字 (<code>a</code>–<code>z</code>, <code>A</code>–<code>Z</code> の最大 52 個) なので、TCML 側で付けた英字交じりのアンカー名 "
                        "(例: <code>@{request}</code>) は <strong>出現順に通し番号化された単一文字</strong> (<code>a</code> など) に置き換わり、元の名前は復元できません。"
                        "アンカー数が 52 を超える場合は超過分の矢印が落とされ、stderr に警告が 1 行出力されます。"
                        "矢印の色・太さ・線種の細部も WaveDrom 側に対応物がないため落とされ、矢印の存在・端点・ラベルのみが保存されます。</p>"
                    ),
                },
                {
                    "type": "wavedrom_sample",
                    "code": "@title 信号間にまたがる矢印\nclk    ~_~_~_~_\nreq    _@{request}~~~~~~_\nack    ___@{ack_received}~~~~_\ndone   ______@{complete}~_\n@-> (@{request}, @{ack_received}) ack\n@-> (@{ack_received}, @{complete}) done\n",
                    "json": (
                        "{\n"
                        '  "head": { "text": "信号間にまたがる矢印" },\n'
                        '  "signal": [\n'
                        '    { "name": "clk",  "wave": "10101010" },\n'
                        '    { "name": "req",  "wave": "01.....0", "node": ".a......" },\n'
                        '    { "name": "ack",  "wave": "0..1...0", "node": "...b...." },\n'
                        '    { "name": "done", "wave": "0.....10", "node": "......c." }\n'
                        "  ],\n"
                        '  "edge": ["a->b ack", "b->c done"]\n'
                        "}\n"
                    ),
                    "tcml_svg_file": "tcml-demo-arrow.svg",
                    "wavedrom_svg_file": "wavedrom-demo-arrow.svg",
                },
            ],
        },
    ],
}

# ---------- 英語版 ----------

DATA_EN = {
    "lang": "en",
    "title": "TCML",
    "subtitle": "Timing Chart Markup Language — a small text language for sketching timing charts",
    "extension": ".tc",
    "labels": {
        "title_suffix": "format reference",
        "extension_label": "Extension",
        "toc": "Contents",
        "wavedrom_tcml_input": "TCML input",
        "wavedrom_tcml_render": "TCML rendering",
        "wavedrom_json_output": "WaveJSON output",
        "wavedrom_render": "WaveDrom rendering",
        "lang_switch_label": "日本語",
        "lang_switch_href": "tcml-format.html",
    },
    "references": REFERENCES_EN,
    "sections": [
        {
            "id": "overview",
            "num": 1,
            "title": "Overview",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>TCML (Timing Chart Markup Language) is a small text language for describing timing charts. With a simple ASCII-style notation it can express signal Low / High / bus / don't-care levels, transitions, arrows between signals, automatic clock expansion, and more.</p>",
                },
                {"type": "text", "content": "<p>This implementation builds on the prior art listed below. The author owes much to these works:</p>"},
                {"type": "references"},
                {"type": "text", "content": "<p>For polished, publication-quality figures use <a href=\"https://wavedrom.com/\">WaveDrom</a>. Charts produced by this tool are not intended for datasheets.</p>"},
                {
                    "type": "text",
                    "content": "<p>That said, <a href=\"https://wavedrom.com/\">WaveDrom</a>'s notation is a bit involved and not the easiest thing to write while you are still thinking. The TCML-style notation pioneered by Prof. Kumagai (Tohoku Gakuin University) and Prof. Takeuchi (University of Tsukuba) is much friendlier in the thinking-and-sketching phase.</p>",
                },
                {
                    "type": "text",
                    "content": "<p>This tool was a long-weekend project to scratch the author's own itch from a few years ago, when designing hardware that could have used a notation like this. The author no longer does that kind of design, so honestly it isn't even for the author anymore.</p>",
                },
            ],
        },
        {
            "id": "start",
            "num": 2,
            "title": "Getting started",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Each timing line has a signal name on the left and a level string on the right, separated by whitespace.</p>",
                },
                {
                    "type": "text",
                    "content": "<p><code>_</code> is Low, <code>~</code> is High, <code>=</code> is Bus, and <code>?</code> is don't-care. Feed the source below to the <code>tchart</code> CLI and you get the SVG shown to the right.</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 8
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

Clock   _~_~_~_~_~_~
"Data
-Data"  =<D0>====X<D1>====X<D2>====
Enable  ____~~~~____
Output  _______~~~~~____
""",
                },
            ],
        },
        {
            "id": "linetypes",
            "num": 3,
            "title": "Line types",
            "blocks": [
                {
                    "type": "table",
                    "headers": ["Kind", "Leading char", "Description"],
                    "rows": [
                        ["Comment", "<code>//</code>", "Ignored. <code>//</code> begins a line comment whether it appears at the start of a line or mid-line; everything from <code>//</code> through end-of-line is discarded (only inside quotes (<code>\"...\"</code>) does <code>//</code> remain literal). A single <code>/</code> or a bare <code>#</code> are plain characters."],
                        [
                            "Parameter / directive",
                            "<code>@</code>",
                            "Either a parameter setting, a row directive (<code>@title</code> / <code>@skip</code> / <code>@clock</code> / <code>@-&gt;</code>), or a per-signal attribute (<code>@signal</code>).",
                        ],
                        [
                            "Overlay text",
                            "<code>%</code>",
                            "Places a text label at the given chart coordinates.",
                        ],
                        [
                            "Timing line",
                            "Anything else",
                            "A signal name followed by a level string.",
                        ],
                    ],
                },
            ],
        },
        {
            "id": "signal-name",
            "num": 4,
            "title": "Signal names",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Any valid UTF-8 string. Control characters are not permitted (with <code>\\n</code> as the only exception inside quoted names). Empty names are rejected.</p>",
                },
                {"type": "heading", "level": 3, "text": "Multi-line names"},
                {
                    "type": "text",
                    "content": "<p>Surround a name with <code>\"</code> to embed newlines:</p>",
                },
                {
                    "type": "code",
                    "code": """"Data
Bus" =<D0>====X<D1>====
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>The opening <code>\"</code> must sit at the start of the line.</li><li>The closing <code>\"</code> is followed by whitespace and then the level string.</li></ul>",
                },
                {"type": "heading", "level": 4, "text": "Escapes (only inside \"...\")"},
                {
                    "type": "table",
                    "headers": ["Sequence", "Meaning"],
                    "rows": [
                        ["<code>\\\"</code>", "Literal <code>\"</code>"],
                        ["<code>\\n</code>", "Newline"],
                        ["<code>\\\\</code>", "Literal <code>\\</code>"],
                    ],
                },
            ],
        },
        {
            "id": "levels",
            "num": 5,
            "title": "Level symbols",
            "blocks": [
                {
                    "type": "table",
                    "headers": ["Symbol", "Meaning", "Shape"],
                    "rows": [
                        ["<code>_</code>", "Low", "Single line at the bottom"],
                        ["<code>~</code>", "High", "Single line at the top"],
                        ["<code>-</code>", "HiZ", "Dashed line in the middle"],
                        ["<code>=</code>", "Bus", "Two parallel rails"],
                        [
                            "<code>?</code>",
                            "Don't care",
                            "Filled hatch with a line at the previous level's position",
                        ],
                    ],
                },
                {"type": "heading", "level": 4, "text": "Example"},
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 12
@slant 2

Low       ________
High      ~~~~~~~~
HiZ       --------
Bus       =<A>====
DontCare  ___?????
""",
                },
                {
                    "type": "text",
                    "content": "<p>Repeated identical symbols (<code>__</code>, <code>~~</code>, <code>??</code>, …) are merged into one segment.</p>",
                },
            ],
        },
        {
            "id": "aux",
            "num": 6,
            "title": "Auxiliary symbols",
            "blocks": [
                {
                    "type": "table",
                    "headers": ["Symbol", "Meaning", "x advance"],
                    "rows": [
                        [
                            "<code>:</code>",
                            "Gap (one unit of whitespace, breaks signal continuity)",
                            "<code>step</code>",
                        ],
                        [
                            "<code>X</code>",
                            "Bus value change (BusCross)",
                            "<code>step</code> (cross part <code>slant</code> + body part <code>step - slant</code>; or body only of width <code>step</code> when the cross is omitted at the start of a signal)",
                        ],
                        [
                            "<code>?</code>",
                            "Don't-care marker (paints surrounding bus segment as don't-care)",
                            "0",
                        ],
                        ["<code>|</code>", "Vertical guide line", "0"],
                        ["<code>[</code> / <code>]</code>", "Highlight start / end", "0"],
                        [
                            "<code>@{name}</code> / <code>@N</code>",
                            "Anchor",
                            "0",
                        ],
                    ],
                },
                {
                    "type": "text",
                    "content": "<p><code>X</code> is composed of two parts: a cross transition followed by a body (one bus unit at the new value). It occupies the width of one level char (= <code>step</code>); the cross gets <code>slant</code>, the body gets <code>step - slant</code>. When there is no preceding bus signal (e.g. at the start of a signal line) the cross is omitted and the body alone takes the full <code>step</code>.</p><p><code>?</code> is a zero-width marker. It paints the surrounding contiguous level segment as a don't-care region.</p>",
                },
                {"type": "heading", "level": 4, "text": "Example"},
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 12
@slant 2

Gap        ____:____
BusX       =<A>====X<B>====
Guide      _~__|__~_
Highlight  __[~~~~]__
""",
                },
            ],
        },
        {
            "id": "dontcare",
            "num": 7,
            "title": "Don't care <code>?</code>",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>The line drawn inside a don't-care region follows the <strong>preceding level</strong>.</p>",
                },
                {
                    "type": "table",
                    "headers": ["Preceding anchor", "Line drawn inside the region"],
                    "rows": [
                        ["<code>_</code> Low", "Bottom"],
                        ["<code>~</code> High", "Top"],
                        ["<code>-</code> HiZ", "Middle"],
                        ["<code>=</code> Bus", "Bus envelope (top + bottom)"],
                        ["<code>X</code>", "Bus envelope"],
                    ],
                },
                {"type": "heading", "level": 3, "text": "Don't care fill shapes inside a bus"},
                {
                    "type": "text",
                    "content": "<p><code>?</code> in a bus context (e.g. <code>=?=</code>) is filled with a polygon shaped to fit the surrounding waveform boundaries. It never extends past the height of a single signal row.</p>",
                },
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 12
@slant 3

@title "DontCareAlongBus shapes"

// `@dontcare_color` can be flipped between rows to change the hatch color.
// (Cycle through #bbb / #c00 / #06c / #080 below.)

// Default color (#bbb)
=?=         ====?====

@dontcare_color #c00
// Both sides Low: /=\\ shape (red)
_=?=_       ____====?====____

// Both sides High: \\=/ shape (red)
~=?=~       ~~~~====?====~~~~

@dontcare_color #06c
// Low only on the left (left slant, right vertical) (blue)
_=?=        ____====?====

// Low only on the right (left vertical, right slant) (blue)
=?=_        ====?====____

@dontcare_color #080
// Mixed High / Low on either side (green)
~=?=_       ~~~~====?====____

// HiZ on the left, bus continues on the right (green)
-=?=        ----====?====

// Bus + label (green)
=?=L        ==<A>==?====

// Bus (green)
=?=X        ==X==?==X==
""",
                },
                {"type": "heading", "level": 3, "text": "Error conditions for <code>?</code>"},
                {
                    "type": "text",
                    "content": "<ul><li>If a signal line begins with <code>?</code>, or only zero-width elements (<code>:</code>, <code>|</code>, <code>[</code>, <code>]</code>, <code>@{...}</code>, <code>@N</code>) precede it, that is an error: <code>?</code> cannot infer which level to fill.</li><li>For example: <code>foo ?==</code>, <code>bar ???</code>, <code>baz ?_~</code>, <code>qux :?_~</code>, <code>quux @{a}?_~</code> are all errors.</li></ul>",
                },
                {"type": "heading", "level": 3, "text": "X / X? patterns"},
                {
                    "type": "table",
                    "headers": ["Pattern", "Interpretation"],
                    "rows": [
                        ["<code>=X=</code>", "Bus(1) + X (cross + body, new value) + Bus(1)"],
                        ["<code>=X?</code>", "Bus(1) + X (cross + body) + ? (X body becomes don't-care)"],
                        [
                            "<code>=X?=</code>",
                            "Don't-care region = X body + the trailing <code>=</code> (2 units total)",
                        ],
                        [
                            "<code>=?X=</code>",
                            "Don't-care region = the leading <code>=</code> (1 unit); the X body and trailing <code>=</code> are bus at the new value",
                        ],
                        [
                            "<code>=X?X=</code>",
                            "Don't-care region = X1 body of 1 unit; the polygon spans X1 cross-midpoint to X2 cross-midpoint (hexagonal)",
                        ],
                        [
                            "<code>~X_</code>",
                            "High + BusOpen + Bus(1, X body) + BusClose + Low (X is valid even when the neighbours are non-bus)",
                        ],
                        [
                            "<code>XXXX</code>",
                            "A run of X's at the start of a signal. The first X drops its cross; subsequent X's behave as ordinary BusCross.",
                        ],
                        ["<code>?X=</code>", "Error (leading <code>?</code>)"],
                    ],
                },
            ],
        },
        {
            "id": "label",
            "num": 8,
            "title": "Labels in level strings <code>&lt;...&gt;</code>",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Attach a text label to a level segment or a transition.</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2

BusLabel   =<A>====X<B>====X<C>====
LowLabel   ____<L>____
HighLabel  ~~~~<H>~~~~
HiZLabel   ----<Z>----
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>Labels cannot contain control characters (other than <code>\\n</code>).</li><li>Escape with <code>\\&lt;</code> for <code>&lt;</code>, <code>\\&gt;</code> for <code>&gt;</code>, and <code>\\\\</code> for <code>\\</code>.</li></ul>",
                },
            ],
        },
        {
            "id": "anchor-arrow",
            "num": 9,
            "title": "Anchors <code>@{name}</code> and arrows <code>@-&gt;</code>",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>An <strong>anchor</strong> is a zero-width marker that records a particular point on a waveform. An <code>@-&gt;</code> line connects two anchors as an arrow. Anchors alone draw nothing; a line is rendered only when an arrow references them as an endpoint.</p>",
                },
                {"type": "heading", "level": 3, "text": "Minimal example"},
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 10
@slant 2

Req   ___@{s}~~~~~~
Ack   ________@{a}~~~

@-> (@{s}, @{a}, red) request
""",
                },
                {"type": "heading", "level": 3, "text": "Anchor rules"},
                {
                    "type": "text",
                    "content": "<ul><li>Use <code>@{name}</code> or <code>@N</code> (a positive integer). Anchors are zero-width and do not advance x.</li><li>Named and numbered anchors live in separate namespaces (<code>@{1}</code> and <code>@1</code> are different).</li><li>Defining the same id twice is an error.</li><li>Anchor names accept ASCII letters, digits, underscore, and hyphen, and must start with a letter or underscore (regex <code>[A-Za-z_][A-Za-z0-9_-]*</code>).</li><li>Anchors are <strong>not</strong> transparent for the <code>?</code> lookup: an anchor by itself does not become the level used to determine a following <code>?</code>.</li></ul>",
                },
                {"type": "heading", "level": 3, "text": "Arrow syntax"},
                {
                    "type": "text",
                    "content": "<p><code>@-&gt; (&lt;from&gt;, &lt;to&gt; [, &lt;attribute&gt;, ...]) [&lt;text&gt;]</code></p>",
                },
                {
                    "type": "table",
                    "headers": ["Category", "Examples", "Disambiguation"],
                    "rows": [
                        [
                            "Color",
                            "<code>red</code>, <code>#f0f</code>, <code>#ff8800</code>",
                            "Anything <code>Color::parse</code> accepts",
                        ],
                        [
                            "Width",
                            "<code>2</code>, <code>2px</code>, <code>1.5px</code>",
                            "Numeric (the <code>px</code> suffix is optional)",
                        ],
                        [
                            "Style",
                            "<code>solid</code>, <code>dashed</code>, <code>dotted</code>",
                            "Keyword",
                        ],
                        [
                            "Arrowhead",
                            "<code>head=end</code>, <code>head=both</code>, <code>head=none</code>",
                            "<code>head=</code> prefix",
                        ],
                    ],
                },
                {
                    "type": "text",
                    "content": "<p>Defaults: color = <code>signal_color</code>, width = <code>1px</code>, style = <code>solid</code>, arrowhead = <code>end</code>. Attributes can appear in any order; specifying two attributes from the same category (two colours, two widths, etc.) is an error.</p>",
                },
                {"type": "heading", "level": 3, "text": "Combining color, width, style, and forward references"},
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

Req     ___@{s}~~~~@{e}___
Ack     _______@{a}~~~~

@-> (@{s}, @{a}) request
@-> (@{e}, @{a}, dashed) ack

Bus     =<A>====@1X<B>====@2X<C>====@3
Flag    ____@4~~~~@5___

@-> (@1, @4, red, 2px) A
@-> (@2, @5, blue, head=both) B
@-> (@3, @4, green, dotted) forward ref

Data    __@{d1}~~~~@{d2}___
Out     ___@{o1}~~~~@{o2}__

@-> (@{d1}, @{o1}, #ff8800, solid) start
@-> (@{d2}, @{o2}, dashed, head=none) end
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>Forward references are allowed; an <code>@-&gt;</code> line may appear anywhere in the file.</li><li>Style values such as <code>font</code> or <code>signal_color</code> are taken from the local settings in effect at the position of the <code>@-&gt;</code> line.</li><li>Referring to an undefined anchor is an error.</li><li>Arrows are not auto-routed to avoid overlap; the author is expected to place them so they don't collide.</li></ul>",
                },
            ],
        },
        {
            "id": "params",
            "num": 10,
            "title": "Parameters",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Syntax: <code>@&lt;parameter-name&gt; &lt;value&gt;</code>. Names are case-insensitive and treat <code>-</code> and <code>_</code> as equivalent (<code>@fontsize</code>, <code>@font-size</code>, and <code>@FONT_SIZE</code> are all the same).</p>",
                },
                {
                    "type": "heading",
                    "level": 3,
                    "text": "Global parameters (cannot be changed mid-file)",
                },
                {
                    "type": "table",
                    "headers": ["Name", "Default", "Description"],
                    "rows": [
                        ["<code>fontsize</code>", "<code>14</code>", "Font size (px). The layout reference unit."],
                        ["<code>lineheight</code>", "<code>1.2</code>", "Multiplier for waveform row height (= <code>fontsize × lineheight</code>)."],
                        ["<code>capwidth</code>", "<code>0</code>", "Width of the signal-name column (px). Auto-computed when 0."],
                        ["<code>namepad</code>", "<code>8</code>", "Gap between the right edge of the name and the left edge of the waveform (px)."],
                        ["<code>scale</code>", "<code>1.0</code>", "Overall SVG scale factor."],
                        ["<code>page-margin</code>", "<code>10</code>", "Fixed margin around the chart (px)."],
                        ["<code>bgcolor0</code>", "<code>none</code>", "Background colour for even rows."],
                        ["<code>bgcolor1</code>", "<code>none</code>", "Background colour for odd rows."],
                    ],
                },
                {
                    "type": "heading",
                    "level": 3,
                    "text": "Local parameters (may be changed mid-file; new value applies from that point onward)",
                },
                {
                    "type": "table",
                    "headers": ["Name", "Default", "Description"],
                    "rows": [
                        ["<code>step</code>", "<code>25</code>", "X advance per level char (px). When there is a preceding transition, that transition is rendered as the leading <code>slant</code> portion of this <code>step</code>. <code>step &lt;= slant</code> is a parse error. If <code>@slant</code> has not been set explicitly yet, an <code>@step</code> directive auto-clamps slant to <code>step / 2</code> so small <code>@step</code> values do not collide with the default 5 px slant."],
                        ["<code>slant</code>", "<code>5</code>", "Transition width (px). Set to 0 for vertical edges. Applies to SingleEdge / BusOpen / BusClose / BusCross alike. Once <code>@slant</code> appears anywhere, later <code>@step</code> directives no longer auto-adjust slant (explicit user value wins)."],
                        ["<code>h_space</code>", "<code>10</code>", "Total vertical padding for a signal row (px). The legacy name <code>signal_gap</code> is also accepted."],
                        ["<code>font</code>", "<code>sans-serif</code>", "Font family. Quote with <code>\"</code> if it contains spaces; comma-separated lists are honoured as fallback chains."],
                        ["<code>signal_color</code>", "<code>black</code>", "Signal line colour."],
                        ["<code>signal_width</code>", "<code>1</code>", "Signal line width (px)."],
                        ["<code>guide_color</code>", "<code>red</code>", "Vertical guide line colour."],
                        ["<code>guide_width</code>", "<code>0.6</code>", "Vertical guide line width (px)."],
                        ["<code>bg</code>", "<code>none</code>", "Background colour for the next row only (local override)."],
                        ["<code>highlight_style</code>", "<code>fill=\"#ff8\" stroke=\"none\"</code>", "Highlight rectangle style."],
                        ["<code>dontcare_color</code>", "<code>#bbb</code>", "Hatch colour for <code>?</code>. A single colour value such as <code>@dontcare_color #c00</code> applies from that point onward; redeclare to switch again."],
                        ["<code>titlealign</code>", "<code>center</code>", "Horizontal alignment for <code>@title</code> (<code>center</code> / <code>left</code> / <code>right</code>)."],
                        ["<code>clockmark_position</code>", "<code>0.5</code>", "Position of the clock triangle marker's apex along the edge (0.0..=1.0)."],
                        ["<code>clockmark_height</code>", "<code>7.5</code>", "Clock marker height (px)."],
                        ["<code>clockmark_width</code>", "<code>6</code>", "Clock marker base width (px). The step-linked shrink <code>min(6, step × 2/3)</code> applies only when the value is resolved from this default (i.e. neither <code>@clockmark_width</code> nor a per-call <code>mark_width</code> is set)."],
                        ["<code>clockmark_color</code>", "Inherits signal_color", "Clock marker fill colour. Inherits the current <code>signal_color</code> when unset."],
                        ["<code>overline_gap</code>", "<code>2</code>", "Gap between the overline and the cap-top of the signal name (px)."],
                        ["<code>overline_thickness</code>", "<code>1</code>", "Overline thickness (px)."],
                        ["<code>ruler</code>", "<code>on</code>", "Whether to draw faint vertical ruler lines in the background. Values: <code>on</code> / <code>off</code>. See <a href=\"#ruler\">§<code>@ruler</code></a>"],
                        ["<code>ruler_color</code>", "<code>#a0a0a0</code>", "Colour of the ruler lines. Changing it does not recolour ruler lines that have already been drawn"],
                    ],
                },
            ],
        },
        {
            "id": "bg",
            "num": 11,
            "title": "Background colour",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>The global <code>@bgcolor0</code> / <code>@bgcolor1</code> alternate row colours, while the local <code>@bg</code> overrides only the next row.</p>",
                },
                {
                    "type": "sample",
                    "code": """@fontsize 14
@step 10
@bgcolor0 #eef6ff
@bgcolor1 #fff4ee

A    _~_~_~_~
B    ~_~_~_~_

@bg #ffe4cc
Local _~_~_~_~

After _~_~_~_~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li><code>@bg</code> is consumed by the next row (Signal / Skip / Title) and then resets. <code>@bg none</code> discards the pending value explicitly.</li><li>Rows that have a pending <code>@bg</code> do not also paint <code>bgcolor0/1</code>.</li><li><code>Skip</code> and <code>Title</code> rows are excluded from the even/odd counter for <code>bgcolor0/1</code>.</li></ul>",
                },
                {
                    "type": "heading",
                    "level": 3,
                    "text": "@highlight_style / @dontcare_color",
                },
                {
                    "type": "code",
                    "code": """@highlight_style fill="#8f8" stroke="green" stroke-width="1"
@dontcare_color #c00
""",
                },
                {
                    "type": "text",
                    "content": "<p><code>@highlight_style</code> takes whitespace-separated <code>key=\"value\"</code> SVG attributes. <code>@dontcare_color</code> takes a single colour value (same notation as <code>@bgcolor0</code> &amp;c.) and switches the hatch colour from then on.</p>",
                },
            ],
        },
        {
            "id": "skip",
            "num": 12,
            "title": "<code>@skip</code> — blank rows",
            "blocks": [
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

Clock   _~_~_~_~_~_~

@skip(1)

Data    =<A>====X<B>====

@skip(2)

Control ____~~~~____

@skip(0.5)

Flag    ~~____~~____

@skip(20px)

Out     _~~~~___~~~~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>A bare number is interpreted as <code>lh</code> (line-height units).</li><li>Append <code>px</code> to specify pixels instead.</li><li>Negative or unparseable values are an error.</li><li>Zero is allowed, but no blank row is emitted (same as omitting the directive).</li></ul>",
                },
            ],
        },
        {
            "id": "title",
            "num": 13,
            "title": "<code>@title</code> — title rows",
            "blocks": [
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

@title "Default Center Title"

A     _~_~_~_~

@titlealign left
@title "Left Aligned"

B     _~_~_~_~

@titlealign right
@title "Right Aligned"

C     _~_~_~_~

@titlealign center
@title "Back to Center"

D     _~_~_~_~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li>The argument string is rendered as a title row. Multi-line titles are quoted with <code>\"...\"</code> following the same escape rules as signal names.</li><li><code>@title</code> may appear multiple times in a single file.</li><li>Title rows are excluded from the even/odd counter for <code>bgcolor0/1</code>.</li><li><code>@titlealign</code> takes <code>center</code> / <code>left</code> / <code>right</code> and applies to every <code>@title</code> emitted after it.</li></ul>",
                },
            ],
        },
        {
            "id": "clock",
            "num": 14,
            "title": "<code>@clock</code> — automatic clock expansion",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Treats the next signal row as a clock. If the body is empty or partial it is padded out from the last state by repeating <code>pulse</code>, and triangle markers are placed automatically on the rising / falling edges. The auto-expanded length is <code>round(max(other signals' end pixel x) / this row's step)</code> units, so the right edge stays approximately aligned even when <code>@step</code> varies row by row. \"Other signals\" excludes auto-expanded rows (only explicit rows and the explicit part of partial clocks count); when every signal is auto-expanded the target is 0 (empty wave). Multiple auto rows independently reference the same explicit set.</p>",
                },
                {
                    "type": "sample",
                    "code": """@clock(pos)
ClkPos  _~_~_~

@clock(neg)
ClkNeg  _~_~_~

@clock(both)
ClkBoth _~_~_~

@clock(pos, _=2, ~=1)
ClkWide
""",
                },
                {
                    "type": "text",
                    "content": "<p>Syntax: <code>@clock</code> / <code>@clock()</code> / <code>@clock([&lt;edge&gt;] [, _=&lt;n&gt;] [, ~=&lt;n&gt;] [, start=&lt;low|high&gt;] [, mark_position=&lt;f32&gt;] [, mark_height=&lt;px&gt;] [, mark_width=&lt;px&gt;] [, mark_color=&lt;color&gt;])</code>. Bare <code>@clock</code> and <code>@clock()</code> are equivalent to <code>@clock(none)</code>.</p>",
                },
                {
                    "type": "table",
                    "headers": ["Attribute", "Value", "Description"],
                    "rows": [
                        ["<code>edge</code>", "<code>pos</code> / <code>neg</code> / <code>both</code> / <code>none</code>", "Where to place triangle markers (default <code>none</code>)."],
                        ["<code>_=&lt;n&gt;</code>", "Positive integer", "Time units in Low (default 1)."],
                        ["<code>~=&lt;n&gt;</code>", "Positive integer", "Time units in High (default 1)."],
                        ["<code>start</code>", "<code>low</code> / <code>high</code>", "Initial phase (default <code>low</code>)."],
                        ["<code>mark_position</code>", "<code>0.0..=1.0</code>", "Position of the marker's apex."],
                        ["<code>mark_height</code>", "Positive value", "Marker height."],
                        ["<code>mark_width</code>", "Positive value", "Marker base width."],
                        ["<code>mark_color</code>", "Colour", "Fill colour (inherits <code>signal_color</code> when unset)."],
                    ],
                },
                {
                    "type": "text",
                    "content": "<ul><li>Attributes may appear in any order. Keys are case-insensitive, and <code>-</code> / <code>_</code> are equivalent.</li><li>Clock markers are <strong>completely independent</strong> of <code>@-&gt;</code> arrows; clock-derived markers never leak into Arrow output.</li></ul>",
                },
            ],
        },
        {
            "id": "signal-attr",
            "num": 15,
            "title": "<code>@signal</code> — per-signal attributes",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Applies an attribute to the next signal row only. The attribute resets immediately after that row. Currently only <code>overline</code> is provided.</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 10
@slant 2
@bgcolor0 #f8f8ff
@bgcolor1 #f0f0f0

@signal(overline)
nReset    ~~~~__~~~~

@signal(overline)
nWrite    ~~__~~__~~

Enable    ____~~~~____

@signal(overline)
"nChip
Enable"   ~~~~____~~~~

Out       __~~~~____~~
""",
                },
                {
                    "type": "text",
                    "content": "<ul><li><code>overline</code>: draws an overline above the signal name (active-low convention). For multi-line names, only <strong>the topmost line</strong> gets the overline, with width fixed to <strong>the longest line</strong>.</li><li>Position and thickness are controlled by <code>@overline_gap</code> / <code>@overline_thickness</code>. The SVG output is an explicit <code>&lt;line&gt;</code> element rather than a <code>text-decoration</code> attribute.</li></ul>",
                },
            ],
        },
        {
            "id": "ruler",
            "num": 16,
            "title": "<code>@ruler</code> — background vertical guide lines",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Draws faint dashed vertical lines at every step boundary so that you can read rising and falling edges that line up across rows. It is on by default, so the lines appear even without any directive. Use <code>@ruler off</code> to hide them, and <code>@ruler_color</code> to change the colour.</p>",
                },
                {
                    "type": "sample",
                    "code": """@step 15
@slant 3
@ruler_color #c0c0c0

@clock(pos)
CLK
Data   =D0====XD1====XD2====
Enable ____~~~~________
""",
                },
                {
                    "type": "text",
                    "content": "<p>Usage:</p><ul><li><code>@ruler on</code> / <code>@ruler off</code> — show or hide the ruler lines for rows that follow.</li><li><code>@ruler_color &lt;colour&gt;</code> — change the ruler colour for rows that follow (default <code>#a0a0a0</code>). Lines already drawn keep their original colour.</li></ul>",
                },
            ],
        },
        {
            "id": "overlay",
            "num": 17,
            "title": "<code>%</code> — overlay text rows",
            "blocks": [
                {
                    "type": "code",
                    "code": "% <x> <y> <text>\n",
                },
                {
                    "type": "text",
                    "content": "<p>Places a text overlay at the given pixel coordinates. The origin is the chart's top-left corner.</p>",
                },
            ],
        },
        {
            "id": "kvrules",
            "num": 18,
            "title": "<code>key=value</code> rules",
            "blocks": [
                {
                    "type": "text",
                    "content": "<p>Every <code>=</code> attribute in TCML (the options to <code>@clock(...)</code>, <code>head=</code> for <code>@-&gt;</code>, <code>@highlight_style</code>, …) follows the <strong>same</strong> rules.</p>",
                },
                {
                    "type": "text",
                    "content": "<ul><li>Whitespace around <code>=</code> is <strong>optional</strong>: <code>key=value</code>, <code>key =value</code>, <code>key= value</code>, and <code>key = value</code> are equivalent.</li><li>Both key and value have leading/trailing whitespace stripped before evaluation.</li><li>Use <code>\"...\"</code> when the value needs to contain spaces.</li></ul>",
                },
            ],
        },
        {
            "id": "wavedrom",
            "num": 19,
            "title": "Handing off to WaveDrom",
            "blocks": [
                {
                    "type": "text",
                    "content": (
                        "<p>tchart is a small tool for jotting down timing diagrams alongside your thoughts while editing. "
                        "When the diagrams need to ship — datasheets, slides, papers — please switch to "
                        "<a href=\"https://wavedrom.com/\">WaveDrom</a>, a real timing-chart tool. "
                        "tchart is not a <em>true companion</em> for the final stage of your work.</p>"
                    ),
                },
                {
                    "type": "text",
                    "content": (
                        "<p>The work you have already done is not wasted. The <code>tchart wavedrom</code> subcommand converts a <code>.tc</code> file to "
                        "<a href=\"https://github.com/wavedrom/schema/blob/master/WaveJSON.md\">WaveJSON</a>. "
                        "The mapping is approximate, not exhaustive: only constructs WaveDrom can render (signal levels, bus data, clocks, anchors / arrows, …) are translated, "
                        "and styling that has no WaveDrom equivalent (background colour, fonts, overlines, highlights, …) is silently dropped. "
                        "Carry your TCML thinking into WaveDrom and continue the journey there.</p>"
                    ),
                },
                {
                    "type": "code",
                    "code": "tchart wavedrom chart.tc            # → chart.json\ntchart wavedrom chart.tc -o out.json\n",
                },
                {
                    "type": "text",
                    "content": "<p>Three example cases are shown below. Each lists the TCML input, the <code>tchart svg</code> rendering, the <code>tchart wavedrom</code> WaveJSON output, and the <code>wavedrom-cli</code> rendering of that JSON, side by side.</p>",
                },
                {
                    "type": "heading",
                    "level": 4,
                    "text": "Case 1: breaking continuity with Gap (<code>:</code>)",
                },
                {
                    "type": "text",
                    "content": "<p>TCML's <code>:</code> Gap is a one-unit blank that breaks signal continuity. It maps to WaveDrom's <code>|</code>, which extends the previous level by one unit while drawing a visual break. The pictures are not identical, but the &ldquo;continuity is broken here&rdquo; semantic is preserved.</p>",
                },
                {
                    "type": "wavedrom_sample",
                    "code": "@title 連続性の断絶\nsig1   ~_~_:~_~_\nsig2   ====:====\n",
                    "json": (
                        "{\n"
                        '  "head": { "text": "連続性の断絶" },\n'
                        '  "signal": [\n'
                        '    { "name": "sig1", "wave": "1010|1010" },\n'
                        '    { "name": "sig2", "wave": "=...|=..." }\n'
                        "  ]\n"
                        "}\n"
                    ),
                    "tcml_svg_file": "tcml-demo-gap.svg",
                    "wavedrom_svg_file": "wavedrom-demo-gap.svg",
                },
                {
                    "type": "heading",
                    "level": 4,
                    "text": "Case 2: changing bus values (<code>X</code> = BusCross)",
                },
                {
                    "type": "text",
                    "content": "<p>An <code>X</code> splits the bus segment, so each side of <code>X</code> becomes its own <code>=</code> segment in the WaveDrom output, and a corresponding <code>data</code> entry is generated for each. This expresses the typical &ldquo;bus value changes here&rdquo; pattern.</p>",
                },
                {
                    "type": "wavedrom_sample",
                    "code": "@title バス値の切替\nclk    ~_~_~_~_\ndata   ==A=X=B=X=C\n",
                    "json": (
                        "{\n"
                        '  "head": { "text": "バス値の切替" },\n'
                        '  "signal": [\n'
                        '    { "name": "clk",  "wave": "10101010" },\n'
                        '    { "name": "data", "wave": "=..=..=.",\n'
                        '      "data": ["A", "B", "C"] }\n'
                        "  ]\n"
                        "}\n"
                    ),
                    "tcml_svg_file": "tcml-demo-bus-x.svg",
                    "wavedrom_svg_file": "wavedrom-demo-bus-x.svg",
                },
                {
                    "type": "heading",
                    "level": 4,
                    "text": "Case 3: anchors <code>@{name}</code> and inter-signal arrows <code>@-&gt;</code>",
                },
                {
                    "type": "text",
                    "content": "<p>Embed zero-width anchors (e.g. <code>@{request}</code>) in the waveform and connect any two with <code>@-&gt;</code>. Spread anchors across multiple signals to express dependencies that span signals (e.g. request → ack → complete). In the WaveDrom output, anchors map to per-signal <code>node</code> strings (same length as the wave; an anchor character at the anchor position, otherwise <code>.</code>) and a top-level <code>edge</code> array.</p>",
                },
                {
                    "type": "text",
                    "content": (
                        "<p><strong>Some information is lost in the conversion.</strong> "
                        "WaveDrom <code>node</code> identifiers are single characters (<code>a</code>–<code>z</code>, <code>A</code>–<code>Z</code>, at most 52 of them), "
                        "so any descriptive anchor name on the TCML side (such as <code>@{request}</code>) is replaced by a <strong>single character assigned in order of appearance</strong> "
                        "(e.g. <code>a</code>) and the original name cannot be recovered. "
                        "If there are more than 52 anchors, the surplus arrows are dropped and a warning is printed to stderr. "
                        "Arrow colour, width, and style nuances are also dropped (WaveDrom has no equivalent); only the existence of the arrow, its endpoints, and its label survive.</p>"
                    ),
                },
                {
                    "type": "wavedrom_sample",
                    "code": "@title 信号間にまたがる矢印\nclk    ~_~_~_~_\nreq    _@{request}~~~~~~_\nack    ___@{ack_received}~~~~_\ndone   ______@{complete}~_\n@-> (@{request}, @{ack_received}) ack\n@-> (@{ack_received}, @{complete}) done\n",
                    "json": (
                        "{\n"
                        '  "head": { "text": "信号間にまたがる矢印" },\n'
                        '  "signal": [\n'
                        '    { "name": "clk",  "wave": "10101010" },\n'
                        '    { "name": "req",  "wave": "01.....0", "node": ".a......" },\n'
                        '    { "name": "ack",  "wave": "0..1...0", "node": "...b...." },\n'
                        '    { "name": "done", "wave": "0.....10", "node": "......c." }\n'
                        "  ],\n"
                        '  "edge": ["a->b ack", "b->c done"]\n'
                        "}\n"
                    ),
                    "tcml_svg_file": "tcml-demo-arrow.svg",
                    "wavedrom_svg_file": "wavedrom-demo-arrow.svg",
                },
            ],
        },
    ],
}

# Backward-compat alias for any external caller that still references DATA.
DATA = DATA_JA
