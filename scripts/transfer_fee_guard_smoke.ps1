param(
    [string]$BaseUrl = "http://127.0.0.1:3000",
    [string]$Sender = "nfm_zero_balance_smoke_guard",
    [string]$Receiver = "nfm_receiver_smoke_guard",
    [double]$Amount = 1.0,
    [switch]$IncludeAcceptedCase
)

$ErrorActionPreference = "Stop"

function Get-Status {
    param([string]$Url)
    return Invoke-RestMethod -Uri "$Url/api/status" -Method Get
}

function Post-TransferIntent {
    param(
        [string]$Url,
        [string]$From,
        [string]$To,
        [double]$Value
    )

    $payload = @{ from = $From; to = $To; amount = $Value } | ConvertTo-Json

    try {
        $response = Invoke-WebRequest -Uri "$Url/api/transfer/create" -Method Post -ContentType "application/json" -Body $payload -UseBasicParsing
        return [pscustomobject]@{
            StatusCode = [int]$response.StatusCode
            Content = [string]$response.Content
        }
    }
    catch {
        $webResponse = $_.Exception.Response
        if ($null -eq $webResponse) {
            throw
        }

        $body = ""

        $hasGetResponseStream = $null -ne ($webResponse.PSObject.Methods | Where-Object { $_.Name -eq "GetResponseStream" })
        if ($hasGetResponseStream) {
            $stream = $webResponse.GetResponseStream()
            if ($null -ne $stream) {
                $reader = New-Object System.IO.StreamReader($stream)
                $body = $reader.ReadToEnd()
                $reader.Close()
            }
        }
        elseif ($null -ne $webResponse.Content) {
            try {
                $body = [string]$webResponse.Content.ReadAsStringAsync().Result
            }
            catch {
                $body = ""
            }
        }

        if ([string]::IsNullOrWhiteSpace($body) -and $null -ne $_.ErrorDetails -and -not [string]::IsNullOrWhiteSpace($_.ErrorDetails.Message)) {
            $body = [string]$_.ErrorDetails.Message
        }

        return [pscustomobject]@{
            StatusCode = [int]$webResponse.StatusCode
            Content = [string]$body
        }
    }
}

Write-Host "[NFM-TEST] Running transfer fee guard smoke test" -ForegroundColor Cyan
Write-Host "[NFM-TEST] BaseUrl: $BaseUrl" -ForegroundColor Cyan

$before = Get-Status -Url $BaseUrl
$failResp = Post-TransferIntent -Url $BaseUrl -From $Sender -To $Receiver -Value $Amount
$after = Get-Status -Url $BaseUrl

$failStatus = [int]$failResp.StatusCode
$failBody = $failResp.Content

if ($failStatus -ne 400) {
    Write-Host "[FAIL] Expected HTTP 400 for zero-balance sender, got $failStatus" -ForegroundColor Red
    Write-Host "[FAIL] Body: $failBody" -ForegroundColor Red
    exit 1
}

if ($after.mempool_count -ne $before.mempool_count) {
    Write-Host "[FAIL] Mempool changed after rejected transfer intent. before=$($before.mempool_count) after=$($after.mempool_count)" -ForegroundColor Red
    Write-Host "[FAIL] Body: $failBody" -ForegroundColor Red
    exit 1
}

if (-not [string]::IsNullOrWhiteSpace($failBody) -and $failBody -notmatch "Insufficient balance to pay Gas Fee") {
    Write-Host "[FAIL] Error message does not contain gas fee guard evidence." -ForegroundColor Red
    Write-Host "[FAIL] Body: $failBody" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] Rejected invalid sender with gas fee error and kept mempool unchanged." -ForegroundColor Green

if (-not $IncludeAcceptedCase) {
    exit 0
}

$live = Get-Status -Url $BaseUrl
$fundedSender = [string]$live.node
if ([string]::IsNullOrWhiteSpace($fundedSender)) {
    Write-Host "[FAIL] Could not read funded sender from /api/status.node" -ForegroundColor Red
    exit 1
}

$acceptedBefore = $live
$acceptedTo = "nfm_receiver_smoke_guard_ok"
$acceptedResp = Post-TransferIntent -Url $BaseUrl -From $fundedSender -To $acceptedTo -Value 0.01
$acceptedAfter = Get-Status -Url $BaseUrl

$acceptedStatus = [int]$acceptedResp.StatusCode
if ($acceptedStatus -lt 200 -or $acceptedStatus -ge 300) {
    Write-Host "[FAIL] Expected 2xx for funded sender, got $acceptedStatus" -ForegroundColor Red
    Write-Host "[FAIL] Body: $($acceptedResp.Content)" -ForegroundColor Red
    exit 1
}

if ($acceptedAfter.mempool_count -lt $acceptedBefore.mempool_count) {
    Write-Host "[FAIL] Mempool count unexpectedly decreased after accepted case." -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] Accepted-case request returned 2xx (funded sender path validated)." -ForegroundColor Green
exit 0
