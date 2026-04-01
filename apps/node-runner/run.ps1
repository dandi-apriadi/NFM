# =============================================================================
# NFM Node Runner - Quick Launch Script (Windows PowerShell)
# =============================================================================

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$CoreDir = Join-Path $ScriptDir "..\..\core\blockchain"

# Optional flags:
#   .\run.ps1 --restart   Restart existing node process
#   .\run.ps1 --health    Health check only (no build/run)
#   .\run.ps1 --open      Open dashboard URL in browser
#   .\run.ps1 --json      Emit JSON output for health checks
#   .\run.ps1 --quiet     Suppress non-essential console output
#   .\run.ps1 --schema    Print JSON payload schema example
$RestartRequested = $args -contains "--restart"
$HealthOnly = $args -contains "--health"
$OpenDashboard = $args -contains "--open"
$JsonMode = $args -contains "--json"
$QuietMode = $args -contains "--quiet"
$SchemaMode = $args -contains "--schema"

function Write-Info {
    param(
        [string]$Message,
        [string]$Color = "Gray"
    )
    if (-not $QuietMode -and -not $JsonMode) {
        Write-Host $Message -ForegroundColor $Color
    }
}

function New-HealthPayload {
    param(
        [bool]$Ok,
        [string]$Mode,
        [string]$Timestamp,
        [Nullable[int]]$LatencyMs,
        [Nullable[int]]$Blocks,
        [Nullable[int]]$Peers,
        [Nullable[int]]$NodeProcessId,
        [string]$Status,
        [string]$Version,
        [object]$Error = $null
    )

    [ordered]@{
        ok = $Ok
        mode = $Mode
        endpoint = "http://127.0.0.1:3000/api/status"
        api_port = 3000
        p2p_port = 9000
        timestamp = $Timestamp
        latency_ms = $LatencyMs
        blocks = $Blocks
        chain_height = $Blocks
        peers = $Peers
        status = $Status
        version = $Version
        pid = $NodeProcessId
        error = $Error
    }
}

if ($QuietMode -or $JsonMode -or $SchemaMode) {
    # keep header hidden for automation modes
} else {
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  NFM Node Runner - Quick Launch" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
}

function Test-ApiHealth {
    param(
        [string]$Mode = "health",
        [Nullable[int]]$NodePid = $null
    )

    try {
        $sw = [System.Diagnostics.Stopwatch]::StartNew()
        $status = Invoke-RestMethod -Uri "http://127.0.0.1:3000/api/status" -Method Get -TimeoutSec 2
        $sw.Stop()
        $script:HealthPayload = New-HealthPayload `
            -Ok $true `
            -Mode $Mode `
            -Timestamp ((Get-Date).ToUniversalTime().ToString("o")) `
            -LatencyMs ([int]$sw.ElapsedMilliseconds) `
            -Blocks $status.blocks `
            -Peers $status.peers `
            -NodeProcessId $NodePid `
            -Status $status.status `
            -Version $status.version
        if (-not $JsonMode) {
            Write-Info "[HEALTH] API reachable. Blocks=$($status.blocks) Peers=$($status.peers) Status=$($status.status)" "Green"
        }
        if ($OpenDashboard) {
            Start-Process "http://127.0.0.1:3000"
            Write-Info "[OPEN] Dashboard opened in default browser." "Green"
        }
        return $true
    }
    catch {
        $script:HealthPayload = New-HealthPayload `
            -Ok $false `
            -Mode $Mode `
            -Timestamp ((Get-Date).ToUniversalTime().ToString("o")) `
            -LatencyMs $null `
            -Blocks $null `
            -Peers $null `
            -NodeProcessId $NodePid `
            -Status $null `
            -Version $null `
            -Error "API probe failed"
        if (-not $JsonMode) {
            Write-Info "[HEALTH] API probe failed at /api/status (node may be down or warming up)." "Yellow"
        }
        return $false
    }
}

# Check Docker mode
if ($args -contains "--docker") {
    Write-Info "[MODE] Docker" "Yellow"
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

if ($SchemaMode) {
    $schema = New-HealthPayload `
        -Ok $true `
        -Mode "health" `
        -Timestamp "2026-01-01T00:00:00Z" `
        -LatencyMs 5 `
        -Blocks 1 `
        -Peers 0 `
        -NodeProcessId $null `
        -Status "running" `
        -Version "1.0.0-mesh"
    $schema | ConvertTo-Json -Compress
    exit 0
}

if ($HealthOnly) {
    Write-Info "[MODE] Health Check" "Yellow"
    $healthy = Test-ApiHealth -Mode "health" -NodePid $null
    if ($JsonMode -and $script:HealthPayload) {
        $script:HealthPayload | ConvertTo-Json -Compress
    }
    if (-not $healthy) {
        exit 1
    }
    exit 0
}

# Check Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    if ($JsonMode) {
        [pscustomobject]@{ ok = $false; error = "Rust/Cargo not found" } | ConvertTo-Json -Compress
    } else {
        Write-Host "[ERROR] Rust/Cargo not found. Install from https://rustup.rs" -ForegroundColor Red
    }
    exit 1
}

Write-Info "[MODE] Native Rust" "Yellow"
Set-Location $CoreDir

$existingNode = Get-Process -Name "nfm-core-blockchain" -ErrorAction SilentlyContinue | Select-Object -First 1
if ($existingNode) {
    if ($RestartRequested) {
        Write-Info "[INFO] Existing nfm-core-blockchain process found (PID $($existingNode.Id)). Restart requested." "Yellow"
        Stop-Process -Id $existingNode.Id -Force
        Start-Sleep -Seconds 1
    }
    else {
        Write-Info "[INFO] NFM Node is already running (PID $($existingNode.Id))." "Green"
        Write-Info "       API: http://127.0.0.1:3000" "Green"
        Write-Info "       P2P: 127.0.0.1:9000" "Green"
        $null = Test-ApiHealth -Mode "native" -NodePid $existingNode.Id
        if ($JsonMode -and $script:HealthPayload) {
            $script:HealthPayload | ConvertTo-Json -Compress
        }
        Write-Info "       Use '.\run.ps1 --restart' to restart the node." "Yellow"
        exit 0
    }
}

$busyPorts = Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue |
    Where-Object { $_.LocalPort -in @(3000, 9000) }

if ($busyPorts) {
    $portEntries = @($busyPorts | Select-Object LocalPort, OwningProcess | Sort-Object LocalPort -Unique)
    $portReport = $busyPorts |
        Select-Object LocalAddress, LocalPort, OwningProcess |
        Sort-Object LocalPort |
        Format-Table -AutoSize | Out-String

    if ($JsonMode) {
        [ordered]@{
            ok = $false
            mode = "native"
            timestamp = (Get-Date).ToUniversalTime().ToString("o")
            error = "Required ports are already in use"
            conflicts = @($portEntries | ForEach-Object {
                [ordered]@{
                    port = [int]$_.LocalPort
                    pid = [int]$_.OwningProcess
                }
            })
        } | ConvertTo-Json -Compress
    } else {
        Write-Host "[ERROR] Required ports are already in use (3000/9000)." -ForegroundColor Red
        Write-Host $portReport
        Write-Host "[HINT] Stop the process above or run this script with --restart if it is nfm-core-blockchain." -ForegroundColor Yellow
    }
    exit 1
}

Write-Info "[BUILD] Compiling NFM Core..." "Green"
cargo build --release

Write-Info "[RUN] Starting NFM Node..." "Green"
& ".\target\release\nfm-core-blockchain.exe"
