param(
    [switch]$Debug,
    [string]$ApiPort,
    [string]$P2PPort,
    [string]$DbPath,
    [switch]$UsePrebuiltBinary,
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

if ($DbPath) {
    $env:NFM_DB_PATH = $DbPath
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
if ($DbPath) {
    Write-Host "[NFM] NFM_DB_PATH=$DbPath" -ForegroundColor DarkCyan
}

if ($UsePrebuiltBinary) {
    Write-Host "[NFM] Use prebuilt binary mode enabled." -ForegroundColor DarkCyan
}

$commandPreview = "cargo " + ($cargoArgs -join " ")
Write-Host "[NFM] Command   : $commandPreview" -ForegroundColor Yellow

if ($NoRun) {
    Write-Host "[NFM] NoRun enabled, command not executed." -ForegroundColor DarkYellow
    exit 0
}

if ($UsePrebuiltBinary -and -not $Debug) {
    $binaryPath = Join-Path $repoRoot "core/blockchain/target/release/nfm-core-blockchain.exe"
    if (-not (Test-Path $binaryPath)) {
        throw "Prebuilt binary not found: $binaryPath"
    }

    Write-Host "[NFM] Executing prebuilt binary: $binaryPath" -ForegroundColor Yellow
    Push-Location $repoRoot
    try {
        & $binaryPath
        exit $LASTEXITCODE
    }
    finally {
        Pop-Location
    }
}

Push-Location $repoRoot
try {
    & cargo @cargoArgs
    exit $LASTEXITCODE
}
finally {
    Pop-Location
}
