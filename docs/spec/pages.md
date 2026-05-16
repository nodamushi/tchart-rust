# GitHub Pages 配信仕様

## 概要

`main` ブランチの内容から GitHub Pages に静的サイトをデプロイする。
公開対象は次の 2 種類のみで、装飾されたトップページは作らない。

| URL | 内容 | ソース |
|------|------|------|
| `/` | Web エディタ (`tchart-editor`) の vite build 結果 | `tchart-editor/dist/` |
| `/help/tcml-format.html` | 日本語ヘルプ | `help/output/tcml-format.html` |
| `/help/tcml-format.en.html` | 英語ヘルプ | `help/output/tcml-format.en.html` |

エディタの `Help` ボタンが開く iframe `src` も `./help/tcml-format.html` を指すため、
配信レイアウトはこの相対パス前提で固定する。

## ワークフロー

`.github/workflows/pages.yml`。

| 項目 | 設定 |
|------|------|
| トリガ | `push` (`branches: [main]`) と `workflow_dispatch` のみ |
| `pull_request_target` | **使わない** (フォーク PR からの実行を避ける) |
| `permissions` | `contents: read` / `pages: write` / `id-token: write` の 3 つに最小化 |
| 並列実行 | `concurrency: pages` で同時に 1 ジョブのみ |
| 実行環境 | `ubuntu-latest` (Pages デプロイには `actions/configure-pages` を使用) |

## ジョブ構成

1. `build` ジョブ
   1. `actions/checkout@v5` でリポジトリを取得。
   2. Rust toolchain (`stable`) をセットアップし `wasm32-unknown-unknown` ターゲットを追加。
   3. `wasm-pack` をインストール。
   4. `pnpm` をセットアップ (バージョン固定、`--frozen-lockfile`)。
   5. `wasm-pack build tchart-web --target web` で `tchart-web/pkg/` を生成。
   6. `cargo build --bin tchart` で help/build.py が呼ぶ tchart CLI を用意し、`python3 help/build.py` でスタンドアロン help HTML (ja / en) を生成。
   7. `pnpm install --frozen-lockfile` → `pnpm build` で editor のシングルファイル HTML (`tchart-editor/dist/index.html`) を生成。help は `?raw` import で editor バンドルに inline 同梱されるので、editor の HTML 1 枚で完結する。
   8. ステージング `dist-pages/`:
      - `tchart-editor/dist/index.html` → `dist-pages/index.html` (シングルページ editor)
      - `help/output/tcml-format.html` → `dist-pages/help/tcml-format.html` (公開直リンク用の日本語 help、editor とは独立)
      - `help/output/tcml-format.en.html` → `dist-pages/help/tcml-format.en.html` (英語 help)
   9. `actions/upload-pages-artifact@v3` で `dist-pages/` を `github-pages` artifact としてアップロード。
2. `deploy` ジョブ
   1. `needs: build` で順序依存を明示。
   2. `actions/deploy-pages@v4` で artifact を Pages に展開。
   3. `environment: { name: github-pages, url: ${{ steps.deploy.outputs.page_url }} }` で URL を露出。

## ローカル再現手順

```bash
wasm-pack build tchart-web --target web
cargo build --bin tchart
python3 help/build.py
( cd tchart-editor && pnpm install --frozen-lockfile && pnpm build )
mkdir -p dist-pages/help
cp tchart-editor/dist/index.html dist-pages/index.html
cp help/output/tcml-format.html help/output/tcml-format.en.html dist-pages/help/
```

最終成果物は次の 3 ファイル: `dist-pages/index.html` (シングルページ editor)、
`dist-pages/help/tcml-format.html` (日本語 help)、`dist-pages/help/tcml-format.en.html` (英語 help)。
editor 単体で配布したい場合は `tchart-editor/dist/index.html` を任意の場所に
置けば、ダブルクリックで `file://` から開いて使える (外部参照ゼロ)。

## 非対応

- 装飾されたトップページや個別ランディング HTML は生成しない。
- ベンチマーク / カバレッジ等の付随アーティファクトは Pages に乗せない。
- リリースバイナリ (Windows / Linux 等) の配布は別ワークフローで扱う。
