# tchart プロジェクト固有 Rust ルール

`docs/coding/rust.md` (汎用 Rust ルール) の補足。プロジェクト固有の判断や例外の置き場所。

## pub field の追加例外 (rust.md の closed list を補完)

現時点なし。

(将来このプロジェクトで「このドメイン型は pub field のままでよい」と合意した場合、ここに型名と理由を列挙する。`rust.md` 側ではなくこのファイルに追記。)

## その他のプロジェクト固有規約

### Tests: XML / SVG must go through a real parser

When tests inspect XML/SVG output, parse it with a trusted external XML parser added as a dev-dependency. Locate elements and read attributes through the parsed DOM, not the raw string.

Forbidden:

- Using `str::find` / `contains` / `matches` / `split` etc. on the raw XML/SVG to locate tags or extract attribute values.
- Hand-rolled XML parsers or tokenizers (wrapping them in a helper does not make it OK — the ban is on the technique).

Once a value has been extracted by the parser (e.g. an attribute string like `points="1,2 3,4"`), normal string operations on that value are fine.
