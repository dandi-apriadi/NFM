#!/bin/bash
# ============================================================
# NFM Node Runner — Bootstrap Script
# Auto-detects hardware dan menjalankan node dengan profil optimal.
# ============================================================

set -e

echo "=========================================="
echo "  NFM Node Runner — Hardware Detection"
echo "=========================================="

# Detect CPU cores
CPU_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "unknown")
echo "  CPU Cores: $CPU_CORES"

# Detect total RAM (MB)
if command -v free &>/dev/null; then
    RAM_MB=$(free -m | awk '/^Mem:/{print $2}')
elif command -v sysctl &>/dev/null; then
    RAM_BYTES=$(sysctl -n hw.memsize 2>/dev/null || echo 0)
    RAM_MB=$((RAM_BYTES / 1048576))
else
    RAM_MB="unknown"
fi
echo "  Total RAM: ${RAM_MB} MB"

# Detect GPU (NVIDIA)
GPU_NAME="none"
GPU_VRAM_MB=0
if command -v nvidia-smi &>/dev/null; then
    GPU_NAME=$(nvidia-smi --query-gpu=name --format=csv,noheader 2>/dev/null | head -1 || echo "none")
    GPU_VRAM_MB=$(nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1 || echo "0")
fi
echo "  GPU: $GPU_NAME"
echo "  GPU VRAM: ${GPU_VRAM_MB} MB"

# Detect if running on battery (mobile)
ON_BATTERY=false
if [ -f /sys/class/power_supply/BAT0/status ]; then
    BAT_STATUS=$(cat /sys/class/power_supply/BAT0/status)
    if [ "$BAT_STATUS" = "Discharging" ]; then
        ON_BATTERY=true
    fi
fi
echo "  On Battery: $ON_BATTERY"

# Auto-select profile
PROFILE="${NFM_PROFILE:-auto}"

if [ "$PROFILE" = "auto" ]; then
    if [ "$ON_BATTERY" = true ]; then
        PROFILE="quiet"
        echo "  [AUTO] Battery detected → Quiet mode"
    elif [ "$RAM_MB" != "unknown" ] && [ "$RAM_MB" -lt 2048 ]; then
        PROFILE="quiet"
        echo "  [AUTO] Low RAM (< 2GB) → Quiet mode"
    elif [ "$GPU_VRAM_MB" -gt 4000 ]; then
        PROFILE="turbo"
        echo "  [AUTO] Strong GPU detected → Turbo mode"
    else
        PROFILE="balanced"
        echo "  [AUTO] Standard hardware → Balanced mode"
    fi
fi

echo ""
echo "  Selected Profile: $PROFILE"
echo "=========================================="

# Export for NFM Core
export NFM_PROFILE="$PROFILE"
export NFM_CPU_CORES="$CPU_CORES"
export NFM_RAM_MB="$RAM_MB"
export NFM_GPU_VRAM_MB="$GPU_VRAM_MB"

# Run the node
echo "  Starting NFM Node..."
cd "$(dirname "$0")/../../core/blockchain"

if command -v cargo &>/dev/null; then
    cargo run --release
else
    echo "  ERROR: Rust/Cargo not found. Please install Rust first."
    echo "  Visit: https://rustup.rs"
    exit 1
fi
