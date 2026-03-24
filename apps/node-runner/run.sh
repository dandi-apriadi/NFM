#!/usr/bin/env bash
# =============================================================================
# NFM Node Runner - Quick Launch Script (Linux/Mac)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CORE_DIR="$SCRIPT_DIR/../../core/blockchain"

echo "=========================================="
echo "  NFM Node Runner - Quick Launch"
echo "=========================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "[ERROR] Rust/Cargo not found. Install from https://rustup.rs"
    exit 1
fi

# Check if Docker is preferred
if [ "${1:-}" = "--docker" ]; then
    echo "[MODE] Docker"
    cd "$CORE_DIR"
    docker build -t nfm-node .
    docker run -it --rm \
        --name nfm-node \
        -p 3000:3000 \
        -p 9000:9000 \
        -v nfm-data:/home/nfm/nfm_main.db \
        nfm-node
    exit 0
fi

echo "[MODE] Native Rust"
cd "$CORE_DIR"

echo "[BUILD] Compiling NFM Core..."
cargo build --release 2>&1

echo "[RUN] Starting NFM Node..."
./target/release/nfm-core-blockchain
