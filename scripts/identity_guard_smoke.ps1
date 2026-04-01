param(
    [string]$BaseUrl = "http://127.0.0.1:3000"
)

$ErrorActionPreference = "Stop"

function Invoke-JsonRequest {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Path,
        [object]$Body = $null
    )

    $uri = "$BaseUrl$Path"
    $jsonBody = if ($null -ne $Body) { $Body | ConvertTo-Json -Depth 8 } else { $null }

    try {
        $response = Invoke-WebRequest -Uri $uri -Method $Method -ContentType "application/json" -Body $jsonBody -UseBasicParsing
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

        $hasGetResponseStream = $null -ne ($webResponse.PSObject.Methods | Where-Object { $_.Name -eq "GetResponseStream" })
        if ($hasGetResponseStream) {
            $stream = $webResponse.GetResponseStream()
            if ($null -ne $stream) {
                $reader = New-Object System.IO.StreamReader($stream)
                $rawContent = $reader.ReadToEnd()
                $reader.Close()
            }
        }
        elseif ($null -ne $webResponse.Content) {
            try {
                $rawContent = [string]$webResponse.Content.ReadAsStringAsync().Result
            }
            catch {
                $rawContent = ""
            }
        }

        if ([string]::IsNullOrWhiteSpace($rawContent) -and $null -ne $_.ErrorDetails -and -not [string]::IsNullOrWhiteSpace($_.ErrorDetails.Message)) {
            $rawContent = [string]$_.ErrorDetails.Message
        }
    }

    $payload = $null
    if (-not [string]::IsNullOrWhiteSpace($rawContent)) {
        try {
            $payload = $rawContent | ConvertFrom-Json
        }
        catch {
            $payload = $rawContent
        }
    }

    return [pscustomobject]@{
        StatusCode = $statusCode
        Payload = $payload
        Raw = $rawContent
        Path = $Path
    }
}

Write-Host "[NFM-ID] Running identity elite shield guard checks" -ForegroundColor Cyan
Write-Host "[NFM-ID] BaseUrl: $BaseUrl" -ForegroundColor Cyan

$statusRes = Invoke-JsonRequest -Method "GET" -Path "/api/status"
if ($statusRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Unable to read /api/status" -ForegroundColor Red
    exit 1
}

$seller = [string]$statusRes.Payload.node
if ([string]::IsNullOrWhiteSpace($seller)) {
    Write-Host "[FAIL] Missing node address from /api/status" -ForegroundColor Red
    exit 1
}

$eliteBidder = "nfm_identity_elite_guard"
$regularUser = "nfm_identity_regular_guard"

# Baseline: regular user should not have elite shield
$regularRes = Invoke-JsonRequest -Method "GET" -Path "/api/identity/$regularUser"
if ($regularRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Baseline identity read failed ($($regularRes.StatusCode)): $($regularRes.Raw)" -ForegroundColor Red
    exit 1
}
if ($regularRes.Raw -notmatch '"elite_shield":false') {
    Write-Host "[FAIL] Expected non-elite baseline identity, got: $($regularRes.Raw)" -ForegroundColor Red
    exit 1
}

# Create mythic auction
$createRes = Invoke-JsonRequest -Method "POST" -Path "/api/auction/create" -Body @{
    seller = $seller
    name = "Identity Guard Mythic"
    rarity = "MYTHIC"
    power_multiplier = 2.4
    starting_price = 10.0
    duration_hours = 24
}

if ($createRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Auction create failed ($($createRes.StatusCode)): $($createRes.Raw)" -ForegroundColor Red
    exit 1
}

$auctionId = [int]$createRes.Payload.auction_id
if ($auctionId -le 0) {
    Write-Host "[FAIL] Invalid auction_id from create response: $($createRes.Raw)" -ForegroundColor Red
    exit 1
}

# Fund elite bidder through app transfer endpoint
$fundRes = Invoke-JsonRequest -Method "POST" -Path "/api/app/wallet/transfer" -Body @{
    from = $seller
    to = $eliteBidder
    amount = 50.0
}
if ($fundRes.StatusCode -lt 200 -or $fundRes.StatusCode -ge 300) {
    Write-Host "[FAIL] Failed to fund elite bidder ($($fundRes.StatusCode)): $($fundRes.Raw)" -ForegroundColor Red
    exit 1
}

$bidRes = Invoke-JsonRequest -Method "POST" -Path "/api/auction/bid" -Body @{
    auction_id = $auctionId
    bidder = $eliteBidder
    amount = 20.0
}
if ($bidRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Auction bid failed ($($bidRes.StatusCode)): $($bidRes.Raw)" -ForegroundColor Red
    exit 1
}

$settleRes = Invoke-JsonRequest -Method "POST" -Path "/api/auction/settle" -Body @{
    auction_id = $auctionId
}
if ($settleRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Auction settle failed ($($settleRes.StatusCode)): $($settleRes.Raw)" -ForegroundColor Red
    exit 1
}

$identityRes = Invoke-JsonRequest -Method "GET" -Path "/api/identity/$eliteBidder"
if ($identityRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Elite identity read failed ($($identityRes.StatusCode)): $($identityRes.Raw)" -ForegroundColor Red
    exit 1
}

if ($identityRes.Raw -notmatch '"elite_shield":true') {
    Write-Host "[FAIL] Expected elite_shield=true for elite bidder, got: $($identityRes.Raw)" -ForegroundColor Red
    exit 1
}

if ($identityRes.Raw -notmatch 'ELITE_VERIFIED') {
    Write-Host "[FAIL] Expected ELITE_VERIFIED status, got: $($identityRes.Raw)" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] Identity guard verified (baseline non-elite + elite shield awarded after mythic win)." -ForegroundColor Green
exit 0
