param(
    [string]$ApiPort = "3000",
    [string]$P2PPort = "9000",
    [int]$HealthTimeoutSec = 60,
    [int]$SmokeRepeat = 1,
    [switch]$KeepNodeRunning,
    [string]$ArtifactDir = "artifacts/integration-smoke"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$runScript = Join-Path $PSScriptRoot "run_blockchain.ps1"
$healthScript = Join-Path $PSScriptRoot "blockchain_healthcheck.ps1"
$smokeScript = Join-Path $PSScriptRoot "app_actions_smoke.ps1"
$artifactRoot = Join-Path $repoRoot $ArtifactDir
$startedAt = Get-Date
$errorMessage = $null

foreach ($required in @($runScript, $healthScript, $smokeScript)) {
    if (-not (Test-Path $required)) {
        throw "Missing script: $required"
    }
}

$baseUrl = "http://127.0.0.1:$ApiPort"

$nodeArgs = @(
    "-NoLogo",
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", "`"$runScript`"",
    "-ApiPort", $ApiPort,
    "-P2PPort", $P2PPort
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
        health_timeout_sec = $HealthTimeoutSec
        smoke_repeat = $SmokeRepeat
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
