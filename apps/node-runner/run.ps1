# =============================================================================
# NFM Node Runner - Quick Launch Script (Windows PowerShell)
# =============================================================================

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$CoreDir = Join-Path $ScriptDir "..\..\core\blockchain"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  NFM Node Runner - Quick Launch" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Check Docker mode
if ($args -contains "--docker") {
    Write-Host "[MODE] Docker" -ForegroundColor Yellow
    Set-Location $CoreDir
    docker build -t nfm-node .
    docker run -it --rm `
        --name nfm-node `
        -p 3000:3000 `
        -p 9000:9000 `
        -v nfm-data:/home/nfm/nfm_main.db `
        nfm-node
    exit 0
}

# Check Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "[ERROR] Rust/Cargo not found. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}

Write-Host "[MODE] Native Rust" -ForegroundColor Yellow
Set-Location $CoreDir

Write-Host "[BUILD] Compiling NFM Core..." -ForegroundColor Green
cargo build --release

Write-Host "[RUN] Starting NFM Node..." -ForegroundColor Green
& ".\target\release\nfm-core-blockchain.exe"
