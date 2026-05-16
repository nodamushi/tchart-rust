# Help generator

TCML のユーザー向けスタンドアロン HTML ヘルプ (`help/output/tcml-format.html`) を Python で生成する変換スクリプト。

## 使い方

```sh
python3 help/build.py
```

`source.py` の `DATA` を編集して再実行すると、HTML が再生成される。SVG はビルド時に `tchart` CLI を呼んで inline 埋め込みされる (オフライン完結)。

## ファイル

- `build.py` — 変換スクリプト本体。`source.py` を読んで `output/tcml-format.html` を出力する。
- `source.py` — 入力データ (Python dict)。文章・コード・テーブル・サンプル TCML 等をここに書く。
- `output/tcml-format.html` — 生成物 (CSS / JS / 画像すべて inline)。
- `README.md` — このファイル。

## `source.py` の構造

```python
DATA = {
    "title": "...",
    "subtitle": "...",
    "extension": ".tc",
    "footer": "...",
    "references": [
        {"name": "...", "url": "..."},
        ...
    ],
    "sections": [
        {
            "id": "...",        # アンカー
            "num": 1,           # セクション番号
            "title": "...",     # 見出し (HTML 可)
            "blocks": [...],    # 内容 (下記 block type 参照)
        },
        ...
    ],
}
```

## block type 一覧

| type | 用途 | 主要キー |
|------|------|---------|
| `text` | HTML 文字列をそのまま埋め込む | `content` |
| `heading` | h3 / h4 等の小見出し | `level` (3/4), `text` |
| `code` | コードブロック (シンタックスハイライトのみ、SVG 化しない) | `code` |
| `sample` | コードブロック + tchart で生成した SVG プレビューを並べる | `code` (実際に tchart に渡す TCML), 任意 `display` (表示用に上書き) |
| `table` | 表 | `headers`, `rows` |
| `error_table` | 表 (エラー名を赤で強調) | `headers`, `rows` |
| `references` | `DATA["references"]` を ul で出す | (なし) |

## 注記

- 「ただの変換スクリプト」なので、構造化やエラーハンドリングは雑。
- 失敗したら例外がそのまま出る。ユーザーが直す。
- スタイル (CSS) は `build.py` 内に直書き。デザイン変更時はそちらを編集。
- 仕様 `docs/spec/tcml-format.md` を改訂したら、対応する `source.py` の文章と `sample` も追従させて再ビルドすること。
