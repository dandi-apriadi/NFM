param(
    [string]$ApiPort = "3000",
    [string]$P2PPort = "9000",
    [int]$HealthTimeoutSec = 45,
    [switch]$StartExplorer,
    [switch]$StopNodeAfterCheck
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$runScript = Join-Path $PSScriptRoot "run_blockchain.ps1"
$healthScript = Join-Path $PSScriptRoot "blockchain_healthcheck.ps1"
$explorerDir = Join-Path $repoRoot "apps/nfm-explorer"

if (-not (Test-Path $runScript)) {
    throw "Missing script: $runScript"
}
if (-not (Test-Path $healthScript)) {
    throw "Missing script: $healthScript"
}

$baseUrl = "http://127.0.0.1:$ApiPort"
$explorerProc = $null

$nodeArgs = @(
    "-NoLogo",
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", "`"$runScript`"",
    "-ApiPort", $ApiPort,
    "-P2PPort", $P2PPort
)

Write-Host "[NFM-BOOT] Starting blockchain node..." -ForegroundColor Cyan
$nodeProc = Start-Process -FilePath "pwsh" -ArgumentList $nodeArgs -WorkingDirectory $repoRoot -PassThru
Write-Host "[NFM-BOOT] Node PID: $($nodeProc.Id)" -ForegroundColor DarkCyan

$healthOk = $false
try {
    & $healthScript -BaseUrl $baseUrl -TimeoutSec $HealthTimeoutSec
    if ($LASTEXITCODE -ne 0) {
        throw "Health check failed"
    }
    $healthOk = $true
    Write-Host "[NFM-BOOT] Node is healthy at $baseUrl" -ForegroundColor Green

    if ($StartExplorer) {
        if (-not (Test-Path $explorerDir)) {
            throw "Explorer directory not found: $explorerDir"
        }

        $nodeModulesDir = Join-Path $explorerDir "node_modules"
        if (-not (Test-Path $nodeModulesDir)) {
            Write-Host "[NFM-BOOT] Explorer dependencies missing. Running npm install..." -ForegroundColor Cyan
            Push-Location $explorerDir
            try {
                & npm install
                if ($LASTEXITCODE -ne 0) {
                    throw "npm install failed in $explorerDir"
                }
            }
            finally {
                Pop-Location
            }
        }

        Write-Host "[NFM-BOOT] Starting explorer dev server..." -ForegroundColor Cyan
        $explorerArgs = @(
            "-NoLogo",
            "-NoProfile",
            "-ExecutionPolicy", "Bypass",
            "-Command", "Set-Location '$explorerDir'; npm run dev"
        )
        $explorerProc = Start-Process -FilePath "pwsh" -ArgumentList $explorerArgs -WorkingDirectory $repoRoot -PassThru
        Write-Host "[NFM-BOOT] Explorer PID: $($explorerProc.Id)" -ForegroundColor DarkCyan
    }
}
catch {
    Write-Host "[NFM-BOOT] Bootstrap failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($explorerProc -and -not $explorerProc.HasExited) {
        Stop-Process -Id $explorerProc.Id -Force
        Write-Host "[NFM-BOOT] Explorer process stopped due to bootstrap failure." -ForegroundColor DarkYellow
    }
    if (-not $nodeProc.HasExited) {
        Stop-Process -Id $nodeProc.Id -Force
        Write-Host "[NFM-BOOT] Node process stopped due to bootstrap failure." -ForegroundColor DarkYellow
    }
    exit 1
}

if ($StopNodeAfterCheck) {
    if ($explorerProc -and -not $explorerProc.HasExited) {
        Stop-Process -Id $explorerProc.Id -Force
        Write-Host "[NFM-BOOT] Explorer process stopped after successful check." -ForegroundColor DarkYellow
    }
    if (-not $nodeProc.HasExited) {
        Stop-Process -Id $nodeProc.Id -Force
        Write-Host "[NFM-BOOT] Node process stopped after successful check." -ForegroundColor DarkYellow
    }
    exit 0
}

Write-Host "[NFM-BOOT] Bootstrap complete. Node process remains running." -ForegroundColor Green
if (-not $healthOk) {
    exit 1
}
exit 0
