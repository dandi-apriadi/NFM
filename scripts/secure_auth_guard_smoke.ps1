param(
    [string]$BaseUrl = "http://127.0.0.1:3000"
)

$ErrorActionPreference = "Stop"

Write-Host "[NFM-SEC] Running secure transfer auth guard check" -ForegroundColor Cyan
Write-Host "[NFM-SEC] BaseUrl: $BaseUrl" -ForegroundColor Cyan

$payload = @{
    from = "nfm_auth_probe_sender"
    to = "nfm_auth_probe_target"
    amount = 0.01
    nonce = "auth-guard-check"
    timestamp = 1710000000
    signature = "invalid"
} | ConvertTo-Json

try {
    $response = Invoke-WebRequest -Uri "$BaseUrl/api/transfer/secure" -Method Post -ContentType "application/json" -Body $payload -UseBasicParsing
    $statusCode = [int]$response.StatusCode
    $rawContent = [string]$response.Content
}
catch {
    $webResponse = $_.Exception.Response
    if ($null -eq $webResponse) {
        throw
    }

    $statusCode = [int]$webResponse.StatusCode
    $rawContent = ""
    if ($null -ne $_.ErrorDetails -and -not [string]::IsNullOrWhiteSpace($_.ErrorDetails.Message)) {
        $rawContent = [string]$_.ErrorDetails.Message
    }
}

if ($statusCode -ne 403) {
    Write-Host "[FAIL] Expected HTTP 403 on invalid signature probe, got $statusCode" -ForegroundColor Red
    Write-Host "[FAIL] Body: $rawContent" -ForegroundColor Red
    exit 1
}

if (-not [string]::IsNullOrWhiteSpace($rawContent) -and $rawContent -notmatch "Forbidden") {
    Write-Host "[FAIL] 403 body does not contain expected forbidden marker." -ForegroundColor Red
    Write-Host "[FAIL] Body: $rawContent" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] Secure transfer endpoint rejects invalid signature with HTTP 403." -ForegroundColor Green
exit 0
