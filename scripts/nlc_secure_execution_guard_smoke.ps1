param(
    [string]$BaseUrl = "http://127.0.0.1:3000",
    [string]$ApiSecret = "nfm_dev_secret_v0.5"
)

$ErrorActionPreference = "Stop"

Write-Host "[NLC-EXEC] Running NLC signed execution guard..." -ForegroundColor Cyan

# Fetch app state to find a funded address
Write-Host "[CHECK 0] Fetching funded test address from app state..." -ForegroundColor Cyan

try {
    $appStateResponse = Invoke-WebRequest -Uri "$BaseUrl/api/app/state" `
        -Method Get `
        -ContentType "application/json" `
        -UseBasicParsing -ErrorAction Stop
    
    $appState = $appStateResponse.Content | ConvertFrom-Json
    $testAddress = $appState.user_profile.nfmAddress
    Write-Host "[CHECK 0] ✓ Using funded address: $testAddress" -ForegroundColor Green
} catch {
    Write-Host "[FAIL] Failed to fetch app state for funded address: $_" -ForegroundColor Red
    exit 1
}
Write-Host "[CHECK 1] Testing valid signed transfer intent..." -ForegroundColor Cyan

$ts = [int]([DateTime]::UtcNow - [DateTime]::UnixEpoch).TotalSeconds
$nonce = "test-nlc-$(Get-Random -Minimum 100000 -Maximum 999999)"

$bodyObj = @{
    input = "transfer 5.5 @alice_test_recipient"
    address = $testAddress
    ts = $ts
    nonce = $nonce
} | ConvertTo-Json -Compress

# Compute signature
$signaturePayload = "$($ApiSecret):/api/nlc:$($bodyObj)"
$hasher = [System.Security.Cryptography.SHA256]::Create()
$signatureBytes = $hasher.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($signaturePayload))
$signature = [System.BitConverter]::ToString($signatureBytes).Replace("-", "").ToLower()

try {
    $response = Invoke-WebRequest -Uri "$BaseUrl/api/nlc" `
        -Method Post `
        -ContentType "application/json" `
        -Body $bodyObj `
        -Headers @{ "x-nfm-signature" = $signature } `
        -UseBasicParsing
    
    $statusCode = [int]$response.StatusCode
    $result = $response.Content | ConvertFrom-Json
} catch {
    $webResponse = $_.Exception.Response
    $statusCode = if ($null -ne $webResponse) { [int]$webResponse.StatusCode } else { 0 }
    $result = if ($null -ne $_.ErrorDetails) { $_.ErrorDetails.Message | ConvertFrom-Json -ErrorAction SilentlyContinue } else { $null }
}

if ($statusCode -ne 200) {
    Write-Host "[FAIL] Expected HTTP 200 for valid signed intent, got $statusCode" -ForegroundColor Red
    Write-Host "[FAIL] Response: $($result | ConvertTo-Json)" -ForegroundColor Red
    exit 1
}

if ($result.status -ne "success") {
    Write-Host "[FAIL] Expected status.success for valid signed intent" -ForegroundColor Red
    Write-Host "[FAIL] Response: $($result | ConvertTo-Json)" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] ✓ Valid signed transfer accepted (HTTP 200, status=success)" -ForegroundColor Green
Write-Host "        Message: $($result.message)" -ForegroundColor DarkGray

# Test 2: Invalid signature (wrong secret)
Write-Host "[CHECK 2] Testing invalid signature rejection..." -ForegroundColor Cyan

$wrongSecret = "wrong_secret_xyz"
$wrongSignaturePayload = "$($wrongSecret):/api/nlc:$($bodyObj)"
$wrongSignatureBytes = $hasher.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($wrongSignaturePayload))
$wrongSignature = [System.BitConverter]::ToString($wrongSignatureBytes).Replace("-", "").ToLower()

try {
    $response = Invoke-WebRequest -Uri "$BaseUrl/api/nlc" `
        -Method Post `
        -ContentType "application/json" `
        -Body $bodyObj `
        -Headers @{ "x-nfm-signature" = $wrongSignature } `
        -UseBasicParsing
    
    Write-Host "[FAIL] Expected HTTP 403 for invalid signature, but got 200" -ForegroundColor Red
    exit 1
} catch {
    $webResponse = $_.Exception.Response
    $statusCode = if ($null -ne $webResponse) { [int]$webResponse.StatusCode } else { 0 }
}

if ($statusCode -ne 403) {
    Write-Host "[FAIL] Expected HTTP 403 for invalid signature, got $statusCode" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] ✓ Invalid signature rejected with HTTP 403" -ForegroundColor Green

# Test 3: Missing signature header
Write-Host "[CHECK 3] Testing missing signature header rejection..." -ForegroundColor Cyan

try {
    $response = Invoke-WebRequest -Uri "$BaseUrl/api/nlc" `
        -Method Post `
        -ContentType "application/json" `
        -Body $bodyObj `
        -UseBasicParsing
    
    Write-Host "[FAIL] Expected HTTP 403 for missing signature, but got 200" -ForegroundColor Red
    exit 1
} catch {
    $webResponse = $_.Exception.Response
    $statusCode = if ($null -ne $webResponse) { [int]$webResponse.StatusCode } else { 0 }
}

if ($statusCode -ne 403) {
    Write-Host "[FAIL] Expected HTTP 403 for missing signature, got $statusCode" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] ✓ Missing signature rejected with HTTP 403" -ForegroundColor Green

# Test 4: Verify preview still works (unprotected)
Write-Host "[CHECK 4] Testing NLC preview endpoint (unprotected)..." -ForegroundColor Cyan

try {
    $previewPayload = @{
        input = "stake 100"
        address = $testAddress
    } | ConvertTo-Json -Compress

    $response = Invoke-WebRequest -Uri "$BaseUrl/api/nlc/preview" `
        -Method Post `
        -ContentType "application/json" `
        -Body $previewPayload `
        -UseBasicParsing
    
    $result = $response.Content | ConvertFrom-Json
    if ($null -eq $result.preview -or -not (Get-Member -InputObject $result.preview -Name "action" -ErrorAction SilentlyContinue)) {
        Write-Host "[FAIL] NLC preview did not return expected preview.action structure" -ForegroundColor Red
        Write-Host "[FAIL] Response: $($result | ConvertTo-Json)" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "[FAIL] NLC preview endpoint failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] ✓ NLC preview endpoint works without signature" -ForegroundColor Green
Write-Host "        Parsed action: $($result.preview.action)" -ForegroundColor DarkGray

Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Green
Write-Host "║ [NFM-EXEC] All NLC signed execution guards passed!             ║" -ForegroundColor Green
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Green

exit 0
