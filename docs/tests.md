# Test Cases

## テスト方針

自動テスト (cargo test / wasm-pack test) を主とし、目視確認は補助的な参考にとどめる。

**波形の接続性（遷移後に線がつながっているか）は目視ではなく座標ベースの自動テストで検証する。**
layout-engine および svg-rendering の波形接続性シナリオで
「遷移終端座標 = 直後要素の開始座標」を数値として確認する。目視のみでの合否判定は不可。

### 目視確認手順

SVG レンダラー (`tchart-core/src/svg.rs`) を変更した場合、以下を実施すること:

1. サンプルを CLI で再生成する:
   ```bash
   cargo build --release -p tchart-cli
   for f in docs/images/*.tc; do
     ./target/release/tchart "$f" -o "${f%.tc}.svg"
   done
   ```
2. PNG に変換して画像を確認する:
   ```bash
   for f in docs/images/*.svg; do
     rsvg-convert -z 4 "$f" -o "${f%.svg}.png"
   done
   ```
3. 各ファイルを目視チェックする:

   **sample.tc**: 複合デモ
   - 信号名・ラベルテキストが信号線の縦中央に揃っている
   - バスラベル (D0/D1/D2) がバス帯線に被っていない
   - 拡大時に波形の線がつながっている (隙間がない)

   **all_transitions.tc**: 遷移パターン
   - 各遷移の斜線が正しい方向に描画されている
   - BusBusX: × クロスハッチのみ (余分な線なし)
   - 非Bus間X: 対応する斜線のみ (× なし)

   **gap.tc**: Gap (信号断絶) パターン
   - 前後の波形が切断されている (対角線でつながっていない)

   **vertical_line.tc**: 縦線パターン
   - 縦線が赤色で信号線より細い
   - 縦線がチャートの上下に少し飛び出している
   - 全信号を貫通している

   **labels.tc**: ラベルパターン
   - 各セグメントのラベルが区間中央に配置されている

   **multiline.tc**: 複数行信号名パターン
   - 複数行信号名が波形の中央に揃っている
   - capwidth が最長信号名に合わせて自動計算されている

   **highlight_dontcare.tc** (新規): ハイライト・不定値パターン
   - 不定値がグレーの矩形で描画されている
   - ハイライト区間が黄色の矩形で描画されている
   - 背景色が波形部分のみに適用されている

   **highlight_full_height.tc** (新規): `[]` ハイライトの全行縦断確認
   - `[..]` が書かれた信号行に関わらず矩形が全信号行を縦断している
   - 矩形上下が `page_margin/2` 分はみ出している

   **async_clock.tc** (新規): 非同期クロックパターン
   - 途中で step が変わった信号が異なる横幅で描画されている

| Feature | File | Status Summary |
|---|---|---|
| TCML パーサー | [tcml-parser.feature.md](tests/tcml-parser.feature.md) | ⬜ 92 |
| レイアウトエンジン | [layout-engine.feature.md](tests/layout-engine.feature.md) | ⬜ 33 |
| SVG レンダリング | [svg-rendering.feature.md](tests/svg-rendering.feature.md) | ⬜ 51 |
| CLI | [cli.feature.md](tests/cli.feature.md) | ⬜ 23 |
| CLI フォント管理 | [cli-font.feature.md](tests/cli-font.feature.md) | ⬜ 19 |
| Web (WASM) | [web-wasm.feature.md](tests/web-wasm.feature.md) | ⬜ 7 |
| 統合テスト | [integration.feature.md](tests/integration.feature.md) | ⬜ 23 |
| Web エディタ | [editor.feature.md](tests/editor.feature.md) | ✅ 12 |
| Web エディタ ツールバー / License | [editor-toolbar.feature.md](tests/editor-toolbar.feature.md) | ✅ 17 / ⬜ 9 |
| WaveDrom 変換 | [wavedrom.feature.md](tests/wavedrom.feature.md) | ⬜ 41 |

**合計**: ✅ 29 / ⬜ 298 シナリオ

> ⬜ タグは Gherkin 仕様としては記述済だが、`@implemented` への昇格は対応する Rust テストの存在確認後に行う (コード上で実装済の機能でもテスト未整備なら `@not-implemented` のまま)。

## 凡例

| Symbol | Tag |
|---|---|
| ✅ | `@implemented` |
| ⬜ | `@not-implemented` |
| 🚧 | `@wip` |
| ⏭ | `@skip` |
| 🔴 | `@known-broken` |
