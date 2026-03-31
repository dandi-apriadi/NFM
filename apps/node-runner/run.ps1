# =============================================================================
# NFM Node Runner - Quick Launch Script (Windows PowerShell)
# =============================================================================

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$CoreDir = Join-Path $ScriptDir "..\..\core\blockchain"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  NFM Node Runner - Quick Launch" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# Optional restart flag: .\run.ps1 --restart
$RestartRequested = $args -contains "--restart"

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

$existingNode = Get-Process -Name "nfm-core-blockchain" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($existingNode) {
    if ($RestartRequested) {
        Write-Host "[INFO] Existing nfm-core-blockchain process found (PID $($existingNode.Id)). Restart requested." -ForegroundColor Yellow
        Stop-Process -Id $existingNode.Id -Force
        Start-Sleep -Seconds 1
    }
    else {
        Write-Host "[INFO] NFM Node is already running (PID $($existingNode.Id))." -ForegroundColor Green
        Write-Host "       API: http://127.0.0.1:3000" -ForegroundColor Green
        Write-Host "       P2P: 127.0.0.1:9000" -ForegroundColor Green
        Write-Host "       Use '.\run.ps1 --restart' to restart the node." -ForegroundColor Yellow
        exit 0
    }
}

$busyPorts = Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue |
    Where-Object { $_.LocalPort -in @(3000, 9000) }

if ($busyPorts) {
    $portReport = $busyPorts |
        Select-Object LocalAddress, LocalPort, OwningProcess |
        Sort-Object LocalPort |
        Format-Table -AutoSize | Out-String

    Write-Host "[ERROR] Required ports are already in use (3000/9000)." -ForegroundColor Red
    Write-Host $portReport
    Write-Host "[HINT] Stop the process above or run this script with --restart if it is nfm-core-blockchain." -ForegroundColor Yellow
    exit 1
}

Write-Host "[BUILD] Compiling NFM Core..." -ForegroundColor Green
cargo build --release

Write-Host "[RUN] Starting NFM Node..." -ForegroundColor Green
& ".\target\release\nfm-core-blockchain.exe"
