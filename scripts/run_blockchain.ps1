param(
    [switch]$Debug,
    [string]$ApiPort,
    [string]$P2PPort,
    [switch]$NoRun
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$manifestPath = Join-Path $repoRoot "core/blockchain/Cargo.toml"

if (-not (Test-Path $manifestPath)) {
    throw "Manifest not found: $manifestPath"
}

if ($ApiPort) {
    $env:NFM_API_PORT = $ApiPort
}

if ($P2PPort) {
    $env:NFM_P2P_PORT = $P2PPort
}

$mode = if ($Debug) { "debug" } else { "release" }
$cargoArgs = @("run", "--manifest-path", $manifestPath)
if (-not $Debug) {
    $cargoArgs += "--release"
}

Write-Host "[NFM] Repo root : $repoRoot" -ForegroundColor Cyan
Write-Host "[NFM] Manifest  : $manifestPath" -ForegroundColor Cyan
Write-Host "[NFM] Mode      : $mode" -ForegroundColor Cyan
if ($ApiPort) {
    Write-Host "[NFM] NFM_API_PORT=$ApiPort" -ForegroundColor DarkCyan
}
if ($P2PPort) {
    Write-Host "[NFM] NFM_P2P_PORT=$P2PPort" -ForegroundColor DarkCyan
}

$commandPreview = "cargo " + ($cargoArgs -join " ")
Write-Host "[NFM] Command   : $commandPreview" -ForegroundColor Yellow

if ($NoRun) {
    Write-Host "[NFM] NoRun enabled, command not executed." -ForegroundColor DarkYellow
    exit 0
}

Push-Location $repoRoot
try {
    & cargo @cargoArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
