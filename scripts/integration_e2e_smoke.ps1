param(
    [string]$ApiPort = "3000",
    [string]$P2PPort = "9000",
    [int]$HealthTimeoutSec = 60,
    [int]$SmokeRepeat = 1,
    [switch]$RunTransferFeeGuard,
    [switch]$TransferFeeGuardIncludeAcceptedCase,
    [switch]$RunSecureAuthGuard,
    [switch]$RunDriveGuard,
    [switch]$RunIdentityGuard,
    [switch]$RunPhase6DGuard,
    [switch]$RunFrontendFlowGuard,
    [switch]$RunBrainCurriculumGuard,
    [switch]$RunNLCSecureExecutionGuard,
    [switch]$KeepNodeRunning,
    [string]$ArtifactDir = "artifacts/integration-smoke"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$runScript = Join-Path $PSScriptRoot "run_blockchain.ps1"
$healthScript = Join-Path $PSScriptRoot "blockchain_healthcheck.ps1"
$smokeScript = Join-Path $PSScriptRoot "app_actions_smoke.ps1"
$transferFeeGuardScript = Join-Path $PSScriptRoot "transfer_fee_guard_smoke.ps1"
$secureAuthGuardScript = Join-Path $PSScriptRoot "secure_auth_guard_smoke.ps1"
$driveGuardScript = Join-Path $PSScriptRoot "drive_guard_smoke.ps1"
$identityGuardScript = Join-Path $PSScriptRoot "identity_guard_smoke.ps1"
$phase6dGuardScript = Join-Path $PSScriptRoot "phase6d_contract_guard_smoke.ps1"
$frontendFlowGuardScript = Join-Path $PSScriptRoot "frontend_flow_guard_smoke.ps1"
$brainCurriculumGuardScript = Join-Path $PSScriptRoot "brain_curriculum_guard_smoke.ps1"
$nlcSecureExecutionGuardScript = Join-Path $PSScriptRoot "nlc_secure_execution_guard_smoke.ps1"
$artifactRoot = Join-Path $repoRoot $ArtifactDir
$startedAt = Get-Date
$errorMessage = $null

foreach ($required in @($runScript, $healthScript, $smokeScript, $transferFeeGuardScript, $secureAuthGuardScript, $driveGuardScript, $identityGuardScript, $phase6dGuardScript, $frontendFlowGuardScript, $brainCurriculumGuardScript, $nlcSecureExecutionGuardScript)) {
    if (-not (Test-Path $required)) {
        throw "Missing script: $required"
    }
}

$baseUrl = "http://127.0.0.1:$ApiPort"
$dbPath = "nfm_e2e_${ApiPort}.db"

$nodeArgs = @(
    "-NoLogo",
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", "`"$runScript`"",
    "-ApiPort", $ApiPort,
    "-P2PPort", $P2PPort,
    "-DbPath", $dbPath,
    "-UsePrebuiltBinary"
)

Write-Host "[NFM-E2E] Starting blockchain node..." -ForegroundColor Cyan
$nodeProc = Start-Process -FilePath "pwsh" -ArgumentList $nodeArgs -WorkingDirectory $repoRoot -PassThru
Write-Host "[NFM-E2E] Node PID: $($nodeProc.Id)" -ForegroundColor DarkCyan

$ok = $false
try {
    & $healthScript -BaseUrl $baseUrl -TimeoutSec $HealthTimeoutSec
    if ($LASTEXITCODE -ne 0) {
        throw "Health check failed"
    }

    Write-Host "[NFM-E2E] Running app action smoke test..." -ForegroundColor Cyan
    & $smokeScript -BaseUrl $baseUrl -Repeat $SmokeRepeat
    if ($LASTEXITCODE -ne 0) {
        throw "Smoke test failed"
    }

    if ($RunTransferFeeGuard) {
        Write-Host "[NFM-E2E] Running transfer fee guard smoke test..." -ForegroundColor Cyan
        if ($TransferFeeGuardIncludeAcceptedCase) {
            & $transferFeeGuardScript -BaseUrl $baseUrl -IncludeAcceptedCase
        }
        else {
            & $transferFeeGuardScript -BaseUrl $baseUrl
        }

        if ($LASTEXITCODE -ne 0) {
            throw "Transfer fee guard smoke test failed"
        }
    }

    if ($RunSecureAuthGuard) {
        Write-Host "[NFM-E2E] Running secure auth guard smoke test..." -ForegroundColor Cyan
        & $secureAuthGuardScript -BaseUrl $baseUrl
        if ($LASTEXITCODE -ne 0) {
            throw "Secure auth guard smoke test failed"
        }
    }

    if ($RunDriveGuard) {
        Write-Host "[NFM-E2E] Running drive ownership guard smoke test..." -ForegroundColor Cyan
        & $driveGuardScript -BaseUrl $baseUrl
        if ($LASTEXITCODE -ne 0) {
            throw "Drive ownership guard smoke test failed"
        }
    }

    if ($RunIdentityGuard) {
        Write-Host "[NFM-E2E] Running identity elite shield guard smoke test..." -ForegroundColor Cyan
        & $identityGuardScript -BaseUrl $baseUrl
        if ($LASTEXITCODE -ne 0) {
            throw "Identity guard smoke test failed"
        }
    }

    if ($RunPhase6DGuard) {
        Write-Host "[NFM-E2E] Running Phase 6D contract guard smoke test..." -ForegroundColor Cyan
        & $phase6dGuardScript -BaseUrl $baseUrl
        if ($LASTEXITCODE -ne 0) {
            throw "Phase 6D contract guard smoke test failed"
        }
    }

    if ($RunFrontendFlowGuard) {
        Write-Host "[NFM-E2E] Running frontend flow guard smoke test..." -ForegroundColor Cyan
        & $frontendFlowGuardScript -BaseUrl $baseUrl
        if ($LASTEXITCODE -ne 0) {
            throw "Frontend flow guard smoke test failed"
        }
    }

    if ($RunBrainCurriculumGuard) {
        Write-Host "[NFM-E2E] Running brain curriculum guard smoke test..." -ForegroundColor Cyan
        & $brainCurriculumGuardScript -BaseUrl $baseUrl
        if ($LASTEXITCODE -ne 0) {
            throw "Brain curriculum guard smoke test failed"
        }
    }

    if ($RunNLCSecureExecutionGuard) {
        Write-Host "[NFM-E2E] Running NLC secure execution guard smoke test..." -ForegroundColor Cyan
        & $nlcSecureExecutionGuardScript -BaseUrl $baseUrl
        if ($LASTEXITCODE -ne 0) {
            throw "NLC secure execution guard smoke test failed"
        }
    }

    Write-Host "[NFM-E2E] Integration smoke test PASSED." -ForegroundColor Green
    $ok = $true
}
catch {
    Write-Host "[NFM-E2E] Integration smoke test FAILED: $($_.Exception.Message)" -ForegroundColor Red
    $errorMessage = $_.Exception.Message
    $ok = $false
}
finally {
    if (-not $KeepNodeRunning) {
        if ($nodeProc -and -not $nodeProc.HasExited) {
            Stop-Process -Id $nodeProc.Id -Force
            Write-Host "[NFM-E2E] Node process stopped." -ForegroundColor DarkYellow
        }
    }
    else {
        Write-Host "[NFM-E2E] KeepNodeRunning enabled, node left running." -ForegroundColor DarkYellow
    }

    New-Item -ItemType Directory -Force -Path $artifactRoot | Out-Null
    $finishedAt = Get-Date
    $durationSec = [Math]::Round(($finishedAt - $startedAt).TotalSeconds, 2)
    $summary = [PSCustomObject]@{
        status = if ($ok) { "passed" } else { "failed" }
        started_at = $startedAt.ToString("o")
        finished_at = $finishedAt.ToString("o")
        duration_sec = $durationSec
        base_url = $baseUrl
        api_port = $ApiPort
        p2p_port = $P2PPort
        db_path = $dbPath
        health_timeout_sec = $HealthTimeoutSec
        smoke_repeat = $SmokeRepeat
        run_transfer_fee_guard = [bool]$RunTransferFeeGuard
        transfer_fee_guard_include_accepted_case = [bool]$TransferFeeGuardIncludeAcceptedCase
        run_secure_auth_guard = [bool]$RunSecureAuthGuard
        run_drive_guard = [bool]$RunDriveGuard
        run_identity_guard = [bool]$RunIdentityGuard
        run_phase6d_guard = [bool]$RunPhase6DGuard
        run_frontend_flow_guard = [bool]$RunFrontendFlowGuard
        run_brain_curriculum_guard = [bool]$RunBrainCurriculumGuard
        run_nlc_secure_execution_guard = [bool]$RunNLCSecureExecutionGuard
        keep_node_running = [bool]$KeepNodeRunning
        node_pid = if ($nodeProc) { $nodeProc.Id } else { $null }
        error = $errorMessage
    }

    $jsonPath = Join-Path $artifactRoot "summary.json"
    $txtPath = Join-Path $artifactRoot "summary.txt"
    $summary | ConvertTo-Json -Depth 4 | Set-Content -Path $jsonPath -Encoding UTF8

    $txt = @(
        "status=$($summary.status)",
        "started_at=$($summary.started_at)",
        "finished_at=$($summary.finished_at)",
        "duration_sec=$($summary.duration_sec)",
        "base_url=$($summary.base_url)",
        "api_port=$($summary.api_port)",
        "p2p_port=$($summary.p2p_port)",
        "health_timeout_sec=$($summary.health_timeout_sec)",
        "smoke_repeat=$($summary.smoke_repeat)",
        "run_transfer_fee_guard=$($summary.run_transfer_fee_guard)",
        "transfer_fee_guard_include_accepted_case=$($summary.transfer_fee_guard_include_accepted_case)",
        "run_secure_auth_guard=$($summary.run_secure_auth_guard)",
        "run_drive_guard=$($summary.run_drive_guard)",
        "run_identity_guard=$($summary.run_identity_guard)",
        "run_phase6d_guard=$($summary.run_phase6d_guard)",
        "run_frontend_flow_guard=$($summary.run_frontend_flow_guard)",
        "run_brain_curriculum_guard=$($summary.run_brain_curriculum_guard)",
        "run_nlc_secure_execution_guard=$($summary.run_nlc_secure_execution_guard)",
        "keep_node_running=$($summary.keep_node_running)",
        "node_pid=$($summary.node_pid)",
        "error=$($summary.error)"
    ) -join [Environment]::NewLine
    Set-Content -Path $txtPath -Value $txt -Encoding UTF8
    Write-Host "[NFM-E2E] Wrote artifacts to $artifactRoot" -ForegroundColor DarkCyan
}

if ($ok) {
    exit 0
}
exit 1
