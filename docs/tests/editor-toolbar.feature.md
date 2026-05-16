# Web エディタ ツールバー / License モーダル

エディタ画面上部 toolbar の 3 ゾーン配置・各アクションボタンの構造・装飾背景、
および License ボタンで開かれる外部ライブラリライセンス一覧モーダルのテスト仕様。

対応仕様: `docs/spec/editor.md` §ツールバー / §License モーダル

---

## ツールバー レイアウト (3 ゾーン)

## @not-implemented @smoke
### Scenario: toolbar が左 / 中央 / 右の 3 ゾーンに分かれて配置される
- Given エディタページを開く
- When toolbar の DOM 構造を取得する
- Then toolbar 内に「左ゾーン (アクションボタン群)」「中央ゾーン (テキストロゴ)」「右ゾーン (privacy-note + License)」の 3 個の領域コンテナが存在する
- And 左ゾーンの右端 x 座標が中央ゾーンの左端 x 座標以下である
- And 中央ゾーンの右端 x 座標が右ゾーンの左端 x 座標以下である

## @implemented
### Scenario: 左ゾーンに Load / Save SVG / Save PNG / WaveDrom / Help が並ぶ
- Given エディタページを開く
- When toolbar 左ゾーン内のボタン要素を列挙する
- Then "Load" / "Save SVG" / "Save PNG" / "WaveDrom" / "Help" の各ボタンが左ゾーン内に存在する
- And これらのボタンはいずれも toolbar 右ゾーン・中央ゾーンには存在しない

## @not-implemented
### Scenario: 中央ゾーンにテキストロゴ `tchart rust editor` が 1 行で表示される
- Given エディタページを開く
- When toolbar 中央ゾーン内のテキストを取得する
- Then `tchart rust editor` の文字列が中央ゾーンに表示される
- And ロゴ要素の表示が 1 行に収まる (改行されない)

## @implemented
### Scenario: 右ゾーンに privacy-note と License ボタンが同居する
- Given エディタページを開く
- When toolbar 右ゾーン内の要素を列挙する
- Then `#privacy-note` 要素が右ゾーン内に存在する
- And "License" ボタンが右ゾーン内に存在する
- And "License" ボタンは toolbar 左ゾーン・中央ゾーンには存在しない

---

## アクションボタンの構造 (インライン SVG アイコン + テキストラベル)

## @implemented @smoke
### Scenario: 各アクションボタンの内部に `<svg>` 要素とテキストラベルが共存する
- Given エディタページを開く
- When "Load" / "Save SVG" / "Save PNG" / "Help" / "License" の各ボタンを取得する (WaveDrom は除く: 第三者ブランド配慮でテキストのみ)
- Then 各ボタンは内部に少なくとも 1 個の `<svg>` 要素を含む
- And 各ボタンは内部にテキストラベル (button のテキスト内容が非空) を含む
- And SVG 要素はいずれも外部リソース参照ではなくインライン `<svg>` として DOM 上に存在する (`<img src=...>` や `<use href="external.svg#...">` ではない)

## @implemented
### Scenario: WaveDrom ボタンはテキストのみで描画される
- Given エディタページを開く
- When "WaveDrom" ボタン要素を取得する
- Then ボタン内部に `<svg>` 要素が存在しない
- And ボタンのテキスト内容に `WaveDrom` が含まれる

## @implemented
### Scenario: アイコンはラベルの左に配置される
- Given エディタページを開く
- When 各アクションボタン内の `<svg>` とテキストラベル要素の位置を確認する (WaveDrom 除く)
- Then `<svg>` 要素の右端 x 座標がテキストラベルの左端 x 座標以下である

## @not-implemented @edge-case
### Scenario: アクションボタンのアイコン SVG はバンドル内インラインで外部 fetch を発生させない
- Given エディタページをロード直後の状態
- When toolbar 上の各ボタンが描画される
- Then ブラウザ DevTools の Network タブで、アイコン画像 (`.svg` / `.png` 等) を取得するための追加 HTTP リクエストが発生しない

---

## ツールバー背景の装飾

## @implemented
### Scenario: toolbar 背景が単色べた塗りではない装飾を持つ
- Given エディタページを開く
- When toolbar 要素の computed style の背景指定を取得する
- Then `background-image` がグラデーション (`linear-gradient` / `radial-gradient` / `conic-gradient` のいずれか) を含む、または背景に複数の色レイヤーが重なっている等の装飾が確認できる
- And 単一色のみの `background-color` 指定だけでは終わっていない

---

## License ボタンの動作 (モーダル open)

## @implemented @smoke
### Scenario: License ボタンクリックでモーダルが開く
- Given エディタが初期化済みである
- When toolbar 右ゾーンの "License" ボタンをクリックする
- Then モーダルダイアログ要素が DOM に表示される
- And モーダル内に「外部ライブラリのライセンス一覧」を示すコンテンツが描画される

## @implemented
### Scenario: License モーダルは Esc キーで閉じる
- Given License モーダルが開いている
- When Esc キーを押す
- Then モーダルが閉じる (DOM から消えるか hidden 状態になる)

## @implemented
### Scenario: License モーダルは閉じるボタンで閉じる
- Given License モーダルが開いている
- When モーダル内の閉じるボタン (例: ヘッダー右端の `×`) を押す
- Then モーダルが閉じる

## @implemented
### Scenario: License モーダルが多重に開かれない
- Given License モーダルが既に開いている
- When 再度 "License" ボタンをクリックする
- Then モーダル DOM はそのまま 1 つだけ存在する

---

## License モーダル コンテンツ要件 (同一本文グループ化)

## @implemented @smoke
### Scenario: モーダルは同一ライセンス本文単位にグループ化された表示になっている
- Given License モーダルを開く
- When モーダル内のグループ要素を列挙する
- Then 各グループは SPDX ラベル / ライセンス本文 (1 回) / そのライセンスで配布されているライブラリ一覧 から構成される
- And 同一ライセンス本文が複数グループに分かれて重複表示されない

## @not-implemented
### Scenario: 各ライブラリ行にはライブラリ名・バージョン・個別 Copyright 表記が表示される
- Given License モーダルを開く
- When 各グループ内のライブラリ行を列挙する
- Then 各行にライブラリ名が表示される
- And 各行にバージョン文字列が表示される
- And 各行に当該ライブラリ固有の Copyright 表記 (`Copyright (c) ...` 等の 1 行) が表示される

## @implemented
### Scenario: TypeScript 側の `dependencies` がモーダルに含まれる
- Given License モーダルを開く
- When モーダル内のライブラリ行を列挙する
- Then `@webcoder49/code-input` の行が含まれる
- And `prismjs` の行が含まれる

## @implemented
### Scenario: WASM 側 Rust ランタイムクレートがモーダルに含まれる
- Given License モーダルを開く
- When モーダル内のライブラリ行を列挙する
- Then `tchart-web` ビルドが取り込む Rust ランタイム依存 (cargo metadata の normal 依存) のクレートが少なくとも 1 件含まれる

## @implemented @negative
### Scenario: 本プロジェクト自身の表記がモーダルに含まれない
- Given License モーダルを開く
- When モーダル内のライブラリ行を列挙する
- Then `tchart-core` / `tchart-cli` / `tchart-web` / `tchart-editor` のいずれも、本プロジェクトのクレート / パッケージとしては表示されない

---

## License モーダル スクロール隔離

## @not-implemented
### Scenario: モーダル内のスクロールが背後をスクロールさせない
- Given License モーダルが開き、モーダル内のコンテンツが縦方向にあふれてスクロール可能な状態である
- And モーダルを開いた直後の背後 (toolbar / editor / preview) のスクロール位置を記録する
- When モーダル内コンテンツを下端までホイール / タッチパッドでスクロールする
- Then モーダル内のスクロール位置だけが変化する
- And 背後 (toolbar / editor / preview) のスクロール位置は記録した値から変化しない

## @implemented @edge-case
### Scenario: モーダル open 中にページ全体のスクロールがロックされる
- Given License モーダルが開いている
- When ページ全体 (モーダル外側) をホイールでスクロールしようとする
- Then 背後コンテンツ (toolbar / editor / preview) のスクロール位置が変化しない

---

## License データのビルド時静的化

## @not-implemented @smoke
### Scenario: License モーダル open 時にネットワーク fetch が発生しない
- Given エディタページの初回ロードが完了している
- And ブラウザ DevTools の Network タブをクリアする
- When "License" ボタンを押してモーダルを開く
- Then モーダルが描画されるまでの間、新規 HTTP リクエストが 1 件も発生しない

## @not-implemented
### Scenario: License 一覧データはビルド時生成済みでバンドルに同梱される
- Given `pnpm build` で生成された `dist/` を任意の静的サーバーで配信する
- When ブラウザでエディタを開き License モーダルを表示する
- Then `dist/index.html` 1 ファイルのみへの初回ロードで完結し、ライセンス一覧取得のための追加 fetch (例: `licenses.json` / `npm registry` / `crates.io` 等) が発生しない

---

## 組合せ / リグレッション

## @not-implemented @regression
### Scenario: toolbar 改修後も既存のアクション (Save SVG / Save PNG / Load / WaveDrom / Help) が動作する
- Given エディタが初期化済みで有効な SVG がプレビューされている
- When 左ゾーンの Save SVG / Save PNG / Load / WaveDrom / Help を順に押下する
- Then いずれも従来どおりの結果 (ダウンロード / モーダル open 等) が得られる
- And アイコン + テキストラベル構造への変更はクリックイベント配信を妨げない

## @not-implemented @regression
### Scenario: License モーダル open 中でも privacy-note 表示が維持される
- Given License モーダルが開いている
- When `#privacy-note` を取得する
- Then モーダル open 前と同じ文言 (`navigator.language` に応じた JA / EN 文) が右ゾーンに表示され続けている

## @implemented
### Scenario: Help モーダルと License モーダルが互いに独立して open / close できる
- Given Help モーダルが開いている
- When "License" ボタンをクリックする
- Then License モーダルが開く、または Help モーダルを先に閉じる挙動になる (同時に重なって両方表示されたままにはならない)
- And 続いて Esc / 閉じるボタンで開いているモーダルを閉じると、エディタ操作が通常状態に戻る

## @implemented @edge-case
### Scenario: License モーダルを連続して open / close しても DOM が増殖しない
- Given License モーダルを 5 回連続で open → close する
- When 最終状態のモーダル要素を DOM から検索する
- Then License モーダルに対応する要素は最大 1 個しか存在しない (close 後は 0 個になり、open 後も 1 個を超えない)
