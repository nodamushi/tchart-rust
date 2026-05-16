# Web エディタ仕様

## 概要

ブラウザ上で TCML を編集し、リアルタイムに SVG プレビューを確認できる Web エディタ。
`tchart-web` (WASM) を利用して TCML → SVG 変換をクライアントサイドで実行する。

サーバーサイド不要。Vite で静的にビルド・配信できる構成とする。

## 画面レイアウト

2分割画面 (左右配置)。

```
┌─────────────────────────────────────────────────┐
│ [toolbar]                                       │
│  [Save SVG] [Save PNG]                          │
├────────────────────┬────────────────────────────┤
│                    │                            │
│   TCML Editor      │   SVG Preview              │
│   (code-input +    │   (rendered SVG)           │
│    Prism overlay)  │                            │
│                    │                            │
│                    │                            │
│                    │                            │
│                    │                            │
└────────────────────┴────────────────────────────┘
```

### 左ペイン: TCML エディタ

- [`@webcoder49/code-input`](https://github.com/WebCoder49/code-input) (Web Component) で `<textarea>` をラップし、その上に [Prism.js](https://prismjs.com/) ベースのシンタックスハイライトを重ねる構成。
- 内部の `<textarea>` がそのまま編集対象なので、IME 入力 / Ctrl+S / コピー&ペースト / Undo Redo / 文字選択 / カーソル移動などのブラウザ標準キー操作は何も追加実装せずに動作する。
- 等幅フォント (`monospace`) で表示する (code-input 側 CSS で textarea と overlay のフォントを完全に揃え、文字位置がずれないようにする)。
- タブキーによるインデント入力をサポート (デフォルトのフォーカス移動は code-input が抑制する)。
- 初期値としてサンプル TCML を表示する (現状と同じ初期 TCML 文字列を `<code-input value="...">` に流し込む)。
- エディタの内容は `codeInputEl.value` で取得・設定する (`HTMLInputElement` と同じ API)。`input` イベントもブラウザ標準どおり発火するため、リアルタイムプレビュー機構 (現状の `handleInput` / 300ms デバウンス) は textarea 時代の実装をそのまま流用できる。
- 左ペインは、内容が実際にあふれた場合にのみスクロールバーを表示します。あふれていないのにスクロールバーを表示してはいけません。

#### シンタックスハイライト

Prism.js の `Prism.languages` に TCML 用文法を登録する (自前定義、~30 行)。
ハイライト対象は次の 5 要素とし、それ以外は通常色のままとする (色は実装裁量、本仕様では Prism のセマンティックトークン名のみを規定する)。

| 要素 | 例 | Prism トークン名 |
|------|----|------------------|
| `@` ディレクティブ | `@step` / `@slant` / `@clock` / `@title` / `@->` / `@signal` 等の `@` で始まる識別子 | `keyword` |
| 文字列リテラル | `"hello world"` / `"複数行\n信号名"` | `string` |
| 信号名 | タイミング記述行の行頭にある識別子 (レベル文字以外の文字列) | `variable` |
| コメント | `//` から行末まで (クォート内を除く) | `comment` |
| 数値リテラル | `25` / `1.5` / `12px` の数値部分 | `number` |

文法定義の細部は実装裁量で良いが、最低限「クォート文字列の中の `//` をコメント扱いしない」「`@->` を 1 個のキーワードとして扱う (`@-` で切らない)」の 2 点だけは満たすこと。
波形文字 (`_~=-X?:|[]`) は無装飾の通常色で残す (個別色分けは過剰と判断して非対象)。

#### エラー位置の下線表示

パースエラーが返ったとき、エラー範囲 (`line` / `column` / `length`) に **赤の波線下線** をエディタ内で重ねて表示する。

実装方針 (lint 専用ライブラリは使わない):

- code-input の overlay 要素 (`<pre><code>` でハイライトされた DOM) の上に、絶対配置の 1 個の `<div class="tcml-error-underline">` を置く。
- 位置は「行頭から `column - 1` 文字ぶんの x オフセット」「対象行の y オフセット」「`max(length, 1)` 文字ぶんの幅」で算出する (等幅フォントなので 1 文字幅 = `measureText("M")` か CSS の `ch` 単位で求められる)。
- スタイル: `<div>` の中に幅ぶんの透明な NBSP テキスト 1 本を入れ、その文字に `text-decoration: underline wavy var(--error-color)` を当てて波線を描く。`wavy` は CSS の `border-style` ではサポートされておらず `text-decoration-style` でのみ有効なので、波線は必ずテキスト装飾経由で描画する。文字自体は `color: transparent` で見えないようにし、選択や caret 操作の邪魔をしないよう外側 `<div>` で `pointer-events: none` を維持する。色は status 行の赤と揃える (`--error-color`)。
- エラーが解消した瞬間に下線を消す。複数エラーが返るケースは将来対応で、本タスクのスコープでは **先頭 1 件のみ** を表示する (`renderTcml` も先頭 1 件しか返さないため整合)。
- `title` 属性に `message` を入れ、下線にマウスホバーすると tooltip でメッセージが見えるようにする。
- `column` は文字単位 (バイト数ではない) なので、x 位置計算は文字数ベースで行うこと。`renderTcml` の `error.line` / `error.column` / `error.length` の単位の定義は [`web.md` §`renderTcml`](web.md#rendertclinput-string-options-renderoptions-renderresult) を参照。

### 右ペイン: SVG プレビュー

- `tchart-web` (WASM) でレンダリングした SVG を表示
- エディタの内容変更時に自動で再レンダリング (デバウンス: 300ms)
- パースエラー時はステータス行 (`#status`, toolbar 下部) にエラー文言 (`renderTcml` が返す `error.message` の英語本文) を赤字で表示し、右ペインのプレビューは直前の有効な SVG をそのまま保持する。位置情報 (`line` / `column` / `length`) はエディタ内の波線下線 (§シンタックスハイライト/§エラー位置の下線表示) で示す
- SVG はスクロール・拡大縮小可能

### ツールバー

上部に配置。toolbar 全体の高さは控えめにし (寸法は実装裁量)、画面の縦領域をエディタ/プレビューに最大限譲る。背景は単色べた塗りではなく装飾的にする (グラデーション等)。エディタ・プレビューと色が衝突しない落ち着いた色味とする。

レイアウトは横方向に 3 ゾーンで配置する:

| ゾーン | 内容 |
|--------|------|
| 左 | アクションボタン群 (Load / Save SVG / Save PNG / WaveDrom / Help) |
| 中央 | テキストロゴ `tchart rust editor` を 1 行で表示 |
| 右 | privacy-note (後述) と License ボタン |

各アクションボタンは「インライン SVG アイコン + テキストラベル」の形で表示する。アイコンはラベルの左に並べ、同一ボタン内に置く。SVG はリソース読み込みではなくバンドル内のインライン `<svg>` として描画する。

アイコン要件:

- 機能ごとに **silhouette を視覚的に区別できる** こと。全ボタンが矩形ベースの似た形にならないよう、輪郭・形状の差で識別可能にする (例: 円、巻物、矢印付き、文字埋め込み等を組み合わせる)。
- 同じテンプレート (例: floppy disk) を流用するボタンであっても、フォーマット名等を埋め込むなどして **互いに視覚的に区別** できる SVG を用意する。Save SVG と Save PNG は別々の SVG パスを持つこと。
- 具体図案 (例: Load = フォルダ + import 矢印 / Save SVG = floppy + 文字 "SVG" / Save PNG = floppy + 文字 "PNG" / Help = 円 + `?` / License = 巻物または文書) は実装裁量。

例外: **WaveDrom ボタンはアイコンを付けずテキストのみとする**。WaveDrom は第三者プロジェクト名であり、こちら側でビジュアルアイデンティティを与えると当該プロジェクトに対する誤った視覚的代理表現になり得るため、テキストのみで表示する。

ボタンと動作の対応:

| ボタン | 動作 |
|--------|------|
| Load | ローカルの `.tc` / `.svg` / `.png` を 1 ボタンで選び、editor 側が中身の先頭バイトで PNG / SVG / プレーンテキスト (`.tc`) を判定し、`.tc` ならそのまま UTF-8 として textarea に流し込み、SVG / PNG なら対応する wasm 抽出関数で埋め込み TCML を取り出して流し込む。プレビューは即時再描画 |
| Save SVG | 現在の SVG を `.svg` ファイルとしてダウンロード |
| Save PNG | 現在の SVG を Canvas 経由で PNG に変換し、`tchart-source` iTXt チャンクに現在の TCML ソースを埋めて `.png` ファイルとしてダウンロード |
| WaveDrom | 現在の TCML を WaveJSON に変換し `tchart.json` としてダウンロード。変換時の警告は status 行に表示 |
| Help | TCML ヘルプ HTML をモーダルダイアログ + `iframe` で開く |
| License | バンドルされている外部ライブラリのライセンス全文一覧をモーダルダイアログで開く (詳細は後述) |

toolbar 右ゾーンに「このページは外部に情報を送信しません」(JA) / 「No data is sent externally」(EN) を 1 行常時表示する。文言は `navigator.language` で `ja*` か否かを判定して切り替え、`#privacy-note` 要素として置く。表示サイズは控えめにする (寸法は実装裁量)。

#### License モーダル

`License` ボタンクリックで開くモーダル。Help モーダルと同じくダイアログ層に表示し、Esc キーまたは閉じるボタンで閉じる。

内容:

- 表示は **「同一ライセンス本文」単位でグループ化** する。同じ本文を共有する複数ライブラリは 1 つのグループにまとめ、**ライセンス本文はグループにつき 1 回だけ** 表示する。MIT / BSD 等で同じ permission notice 本文が多数のライブラリに重複するのを避けることが目的。
- 各グループは次の要素を持つ:
    - ライセンス識別子 (SPDX) ラベル (例: `MIT`、`Apache-2.0`、`BSD-3-Clause`、`MIT OR Apache-2.0` 等)
    - グループ本文 (該当ライセンスの全文)
    - グループに属する **ライブラリ一覧**。各ライブラリ行に「ライブラリ名」「バージョン」「個別の Copyright 表記」(MIT/BSD の "copyright notice shall be included" 要件は、本文ではなくこの行で個別に満たす) を出す
- 対象は実行時にユーザーへ配布される依存のみ:
    - TypeScript 側: `package.json` の `dependencies` (`@webcoder49/code-input`, `prismjs`, `tchart-web` ローカル)
    - WASM 側: `tchart-web` ビルドが取り込む Rust ランタイムクレート (`cargo metadata` から `normal` 依存を抽出)
- 本プロジェクト自体 (`tchart-core` / `tchart-cli` / `tchart-web` / `tchart-editor`) の著作権・ライセンス表記は出さない (プロジェクト方針: 0BSD でユーザー注意義務を要求しない)。
- ライセンス一覧データはビルド時に自動生成し、ランタイムでは静的データとして読み込む (外部ネットワーク取得を行わない / プライバシー宣言と整合)。生成方法・配置・ファイル形式は実装裁量。グループ化のキーは「ライセンス本文の正規化文字列」とする (空白正規化等の最小限の正規化のみ、文面そのものを書き換えない)。
- モーダル内のスクロールはモーダル内で完結すること (背後の toolbar / editor / preview をスクロールさせない)。

## 機能詳細

### リアルタイムプレビュー

1. ユーザーがエディタの内容を変更する (code-input がラップしている textarea の `input` イベント)
2. 300ms のデバウンス後、`renderTcml(text)` (wasm) を呼び出す
3. 戻り値 `{ svg?, error? }` を判定する:
   - `svg` が返れば右ペインを差し替え、ステータス行をクリアし、エディタ内の波線下線も消す
   - `error` が返れば:
     - ステータス行 (`#status`) に `error.message` を赤字で表示
     - エディタ内に `error.line` / `error.column` / `error.length` の範囲を波線下線で表示 (§エラー位置の下線表示)
     - 右ペインのプレビューは直前の有効な SVG を破棄せずそのまま残す
4. 一度も成功レンダリングが行われていない初期状態でパースに失敗した場合のみ、右ペインに「(no preview)」相当の空表示でよい。直前の有効な SVG が無いケースの扱いはこの初期状態に限る。
5. ステータス行はパースエラー / Load 失敗 / WaveDrom 警告で共有し、新しいメッセージで上書きする。

### SVG ダウンロード

1. 現在レンダリング済みの SVG 文字列を取得
2. `Blob` を生成し `URL.createObjectURL` でダウンロードリンクを作成
3. ファイル名: `tchart.svg`
4. 直前に有効な SVG が一度も生成されていない初期状態の間のみボタンを無効化する。パースエラー中でも前回の有効な SVG が保持されていれば保存可能

### PNG ダウンロード

1. 現在の SVG 文字列から `Image` オブジェクトを生成
2. `Canvas` に描画（4倍解像度: SVG の幅・高さ × 4）
3. `canvas.toBlob('image/png')` で PNG を生成
4. tchart-web の wasm 関数 `embed_tcml_source_in_png(bytes, source)` を呼び、生 PNG バイト列に現在のエディタ TCML を `tchart-source` keyword の `iTXt` チャンクとして埋めた新しい PNG バイト列を取得する。iTXt のフォーマット詳細・CRC 計算・挿入位置はすべて wasm 側 (Rust の `png` クレート) が責任を持ち、editor 側は構造を意識しない
5. ファイル名: `tchart.png`
6. 直前に有効な SVG が一度も生成されていない初期状態の間のみボタンを無効化する。パースエラー中でも前回の有効な SVG が保持されていれば保存可能

### Load

1. `Load` ボタンが hidden な `<input type="file" accept=".tc,text/plain,.svg,image/svg+xml,.png,image/png">` 1 個をプロキシ起動する
2. 選ばれたファイルの先頭 8 バイトを `arrayBuffer()` で読み、PNG シグネチャ (`89 50 4e 47 0d 0a 1a 0a`) と一致したら PNG として扱う。先頭バイト列に `<svg`、`<?xml`、または XML namespace 宣言を含む場合 (大文字小文字無視、空白を許容) は SVG として扱う。それ以外はプレーンテキスト (`.tc`) として扱う (拡張子は補助的にしか使わない)
3. PNG なら全バイト列を wasm `extract_tcml_source_from_png(bytes)` に渡す
4. SVG なら全バイト列を UTF-8 でデコードし、wasm `extract_tcml_source(svg)` を呼ぶ
5. プレーンテキストなら全バイト列を UTF-8 でデコードし、そのまま TCML として扱う (wasm 抽出関数は呼ばない)
6. 取得した TCML を textarea に流し込み、`handleInput` でプレビューを即時再描画
7. status 行に `Loaded <ファイル名>` を表示
8. 抽出失敗 (該当チャンクなし / wasm が `undefined` を返した等) は status 行に `Load failed: <理由>` をエラー表示し、textarea は変更しない

### WaveDrom 出力

1. 現在のエディタ内容を `to_wavejson(text)` (`tchart-web` wasm export) に渡す
2. 成功時: `Blob` (`application/json`) を生成し `tchart.json` としてダウンロード
3. 警告 (`warnings: string[]`) が返ってきた場合はステータス行 (toolbar 下部) にメッセージを表示。1 件なら全文、複数なら "N warnings" の要約
4. パースエラー時: ステータス行にエラー文言を出してダウンロードはしない
5. 現在のエディタ内容がパースエラー状態の間は WaveDrom ボタンを無効化する（直前の有効な SVG が右ペインに残っていても、現テキストがエラーなら無効）

### Help モーダル

1. `Help` ボタン押下でモーダルダイアログを開く
2. 中身は `<iframe>` だが、内容は **`srcdoc` 属性で埋め込む** (`src` 属性は使用しない、URL fetch しない)。`?raw` import でバンドル時に埋め込んだ help HTML 文字列をそのまま `iframe.srcdoc` に流し込む
3. モーダルは Esc キー / 背景クリック / 右上 `×` ボタンのいずれかで閉じる
4. 多重 open しない (既に開いていれば何もしない)
5. help HTML は `?raw` import で editor バンドルに同梱される (`help/output/tcml-format.html` と `tcml-format.en.html` の両方)。要件: **`dist/index.html` 1 ファイルのみで完結し、ブラウザの初回ロード後に外部 CSS / JS / wasm / HTML を一切 fetch しない** (wasm は base64 data URL として inline、help HTML も同梱)
6. モーダル右上に `JA / EN` トグルボタンを置き、クリックで `srcdoc` を入れ替えて言語を切り替える。ブラウザ言語が `ja*` なら初期表示は日本語、それ以外は英語

### 初期サンプル

エディタの初期値として以下の TCML を表示:

```tcml
// Sample
@step 15
@slant 3

@clock(pos)
Clock  
Data   =D0====XD1====XD2====
Enable ____~~~~________
```

## 技術スタック

| 項目 | 選定 | 理由 |
|------|------|------|
| ビルドツール | Vite | 高速な HMR、WASM 対応が容易 |
| 言語 | TypeScript | 型安全性 |
| WASM | `tchart-web/pkg/` | 既存の `wasm-pack build` 出力を利用 |
| CSS | Plain CSS | 依存最小化。フレームワーク不要な規模 |
| エディタ | `@webcoder49/code-input` + `prismjs` | textarea を保持する薄い Web Component。IME / 既存キー操作はブラウザ標準のまま動く。バンドル増は ~20KB (Prism コア + code-input) と軽量 |
| UI フレームワーク | なし | code-input + div のみ。React 等は過剰 |

## 開発サーバー

```bash
# WASM ビルド (事前に必要)
wasm-pack build tchart-web --target web

# エディタ起動
cd tchart-editor
pnpm install
pnpm dev
```

`npm run dev` で Vite 開発サーバーが起動し、ブラウザで `http://localhost:5173` にアクセスするとエディタが表示される。

### Vite 設定

- `tchart-web/pkg/` を依存として参照する
- WASM ファイルの MIME type を正しく配信する (`application/wasm`)
- `vite-plugin-wasm` を使用して WASM のインポートを簡素化

## ビルド (本番)

```bash
cd tchart-editor
pnpm build
```

`dist/` に静的ファイルが生成される。任意の HTTP サーバーで配信可能。

## 対応ブラウザ

`docs/spec/web.md` と同一:

- Chrome / Chromium 最新版
- Firefox 最新版
- Edge 最新版

Safari は動作確認対象外。
