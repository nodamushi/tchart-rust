#!/usr/bin/env bash
set -e

export CARGO_HOME=${CARGO_HOME:-"/usr/local/cargo"}
export RUSTUP_HOME=${RUSTUP_HOME:-"/usr/local/rustup"}

apt-get update
apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ripgrep \
    fontconfig \
    fonts-dejavu \
    fonts-liberation \
    fonts-noto-core \
    fonts-comic-neue \
    fonts-lobster
apt-get clean
rm -rf /var/lib/apt/lists/*

echo "Installing WebAssembly target..."
rustup target add wasm32-unknown-unknown

echo "Installing cargo tools..."
cargo install cargo-expand cargo-make wasm-bindgen-cli cargo-audit cargo-llvm-cov

# wavedrom-cli is the official Node-based renderer used to generate the
# inline SVGs shown in help/output/tcml-format.html (the `tchart wavedrom`
# subcommand only emits WaveJSON, not images). Pin to v3 to match the
# version used when the help page was last regenerated.
if command -v npm >/dev/null 2>&1; then
    echo "Installing wavedrom-cli (npm)..."
    npm install -g wavedrom-cli@3
else
    echo "npm not found; skipping wavedrom-cli install. Install Node first." >&2
fi

echo "Cleaning up cargo registry cache to prevent permission issues..."

rm -rf ${CARGO_HOME}/registry
chmod -R 755 ${CARGO_HOME}/bin
chmod -R a+w ${CARGO_HOME}
