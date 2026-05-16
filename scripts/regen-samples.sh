#!/usr/bin/env bash
# docs/images/*.tc から PNG / SVG を再生成する。
#
# 使い方:
#   scripts/regen-samples.sh              # 全サンプル
#   scripts/regen-samples.sh multiline    # 単一ファイル (multiline.tc)
#
# フォントは TCHART_FONT 環境変数があれば優先、なければ DejaVuSans を既定として使う。
set -euo pipefail

cd "$(dirname "$0")/.."

FONT="${TCHART_FONT:-/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf}"
TARGETS=()

if [[ $# -eq 0 ]]; then
  while IFS= read -r -d '' f; do
    TARGETS+=("$f")
  done < <(find docs/images -maxdepth 1 -name '*.tc' -print0)
else
  for name in "$@"; do
    f="docs/images/${name%.tc}.tc"
    if [[ ! -f "$f" ]]; then
      echo "error: $f not found" >&2
      exit 1
    fi
    TARGETS+=("$f")
  done
fi

cargo build -q -p tchart-cli --release

# 複数 .tc を `tchart batch` でまとめてレンダリング (フォントを 1 回だけロードして共有)。
# 全 TARGETS は docs/images/ 配下なので -o docs/images/ で <STEM>.svg / <STEM>.png に出力される。
echo "regen: ${#TARGETS[@]} files → docs/images/*.svg"
./target/release/tchart batch svg "${TARGETS[@]}" --font "$FONT" -o docs/images/
echo "regen: ${#TARGETS[@]} files → docs/images/*.png"
./target/release/tchart batch png "${TARGETS[@]}" --font "$FONT" -o docs/images/
