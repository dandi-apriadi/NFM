param(
    [string]$ApiPort = "3010",
    [string]$P2PPort = "9010",
    [int]$HealthTimeoutSec = 90,
    [switch]$KeepNodeRunning
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$integrationScript = Join-Path $PSScriptRoot "integration_e2e_smoke.ps1"

if (-not (Test-Path $integrationScript)) {
    throw "Missing script: $integrationScript"
}

Write-Host "[NFM-GATE] Step 1/2: cargo build --release" -ForegroundColor Cyan
Push-Location (Join-Path $repoRoot "core/blockchain")
try {
    $running = Get-Process -Name "nfm-core-blockchain" -ErrorAction SilentlyContinue
    if ($running) {
        Write-Host "[NFM-GATE] Stopping running nfm-core-blockchain process to avoid release binary lock..." -ForegroundColor DarkYellow
        $running | Stop-Process -Force
        Start-Sleep -Seconds 1
    }

    & cargo build --release
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build --release failed"
    }
}
finally {
    Pop-Location
}

Write-Host "[NFM-GATE] Step 2/2: integration policy checks" -ForegroundColor Cyan
$integrationArgs = @(
    "-NoLogo",
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", "`"$integrationScript`"",
    "-ApiPort", $ApiPort,
    "-P2PPort", $P2PPort,
    "-HealthTimeoutSec", [string]$HealthTimeoutSec,
    "-RunTransferFeeGuard",
    "-TransferFeeGuardIncludeAcceptedCase",
    "-RunSecureAuthGuard",
    "-RunDriveGuard",
    "-RunIdentityGuard",
    "-RunPhase6DGuard",
    "-RunFrontendFlowGuard",
    "-RunBrainCurriculumGuard",
    "-RunNLCSecureExecutionGuard"
)
if ($KeepNodeRunning) {
    $integrationArgs += "-KeepNodeRunning"
}

& pwsh @integrationArgs
if ($LASTEXITCODE -ne 0) {
    throw "Integration policy checks failed"
}

Write-Host "[NFM-GATE] PASS: compile + policy checks completed." -ForegroundColor Green
exit 0
