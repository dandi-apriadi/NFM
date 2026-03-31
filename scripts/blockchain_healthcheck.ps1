param(
    [string]$BaseUrl = "http://127.0.0.1:3000",
    [int]$TimeoutSec = 30,
    [int]$IntervalMs = 1000,
    [string[]]$Paths = @("/api/status", "/api/p2p/status", "/api/app/state")
)

$ErrorActionPreference = "Stop"

if ($TimeoutSec -lt 1) {
    throw "TimeoutSec must be >= 1"
}
if ($IntervalMs -lt 100) {
    throw "IntervalMs must be >= 100"
}
if (-not $Paths -or $Paths.Count -eq 0) {
    throw "At least one API path is required"
}

$deadline = (Get-Date).AddSeconds($TimeoutSec)
$remaining = @{}
foreach ($p in $Paths) {
    if ([string]::IsNullOrWhiteSpace($p) -or -not $p.StartsWith('/')) {
        throw "Invalid path: '$p' (must start with '/')"
    }
    $remaining[$p] = $true
}

Write-Host "[NFM-HEALTH] Base URL  : $BaseUrl" -ForegroundColor Cyan
Write-Host "[NFM-HEALTH] Timeout   : ${TimeoutSec}s" -ForegroundColor Cyan
Write-Host "[NFM-HEALTH] Interval  : ${IntervalMs}ms" -ForegroundColor Cyan
Write-Host "[NFM-HEALTH] Endpoints : $($Paths -join ', ')" -ForegroundColor Cyan

while ((Get-Date) -lt $deadline) {
    foreach ($path in @($remaining.Keys)) {
        $url = "$BaseUrl$path"
        try {
            $res = Invoke-WebRequest -Uri $url -Method Get -TimeoutSec 5 -UseBasicParsing
            if ($res.StatusCode -ge 200 -and $res.StatusCode -lt 300) {
                Write-Host "[OK] $path -> $($res.StatusCode)" -ForegroundColor Green
                $remaining.Remove($path) | Out-Null
            }
        }
        catch {
            # Keep retrying until timeout.
        }
    }

    if ($remaining.Count -eq 0) {
        Write-Host "[NFM-HEALTH] All required endpoints are healthy." -ForegroundColor Green
        exit 0
    }

    Start-Sleep -Milliseconds $IntervalMs
}

Write-Host "[NFM-HEALTH] Timeout reached. Unhealthy endpoints:" -ForegroundColor Red
foreach ($path in $remaining.Keys) {
    Write-Host "  - $path" -ForegroundColor Red
}
exit 1
