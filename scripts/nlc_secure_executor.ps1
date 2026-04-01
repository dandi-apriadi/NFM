param(
    [string]$ApiSecret = "",
    [string]$Intent = "",
    [string]$Address = "",
    [string]$BaseUrl = "http://127.0.0.1:3000",
    [switch]$Interactive = $false
)

$ErrorActionPreference = "Stop"

Write-Host "╔════════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║         NFM NLC Secure Executor (Signature Verified)          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

# --- STEP 1: Validate/Prompt for API Secret ---
if ([string]::IsNullOrWhiteSpace($ApiSecret)) {
    if ($Interactive) {
        Write-Host "[PROMPT] Enter API secret (will not echo): " -ForegroundColor Yellow -NoNewline
        $secureSecret = Read-Host -AsSecureString
        $ApiSecret = [System.Runtime.InteropServices.Marshal]::PtrToStringAuto([System.Runtime.InteropServices.Marshal]::SecureStringToCoTaskMemUnicode($secureSecret))
    } else {
        Write-Host "[ERROR] API secret not provided. Use -ApiSecret or -Interactive" -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "[INFO] API secret provided via parameter (length: $($ApiSecret.Length))" -ForegroundColor Green
}

# --- STEP 2: Validate/Prompt for Intent ---
if ([string]::IsNullOrWhiteSpace($Intent)) {
    if ($Interactive) {
        Write-Host "[PROMPT] Enter NLC intent (e.g., 'transfer 10 @alice'): " -ForegroundColor Yellow -NoNewline
        $Intent = Read-Host
    } else {
        Write-Host "[ERROR] Intent not provided. Use -Intent or -Interactive" -ForegroundColor Red
        exit 1
    }
}

# --- STEP 3: Validate/Prompt for Address ---
if ([string]::IsNullOrWhiteSpace($Address)) {
    $Address = "nfm_nlc_executor"
    Write-Host "[INFO] Using default address: $Address" -ForegroundColor Green
}

Write-Host ""
Write-Host "── Intent Execution Plan ────────────────────────────────────────" -ForegroundColor Cyan
Write-Host "Intent:      $Intent" -ForegroundColor White
Write-Host "Address:     $Address" -ForegroundColor White
Write-Host "Endpoint:    POST $BaseUrl/api/nlc" -ForegroundColor White
Write-Host "Secret:      ***$(10..($ApiSecret.Length-1) | % { '*' } -join '')" -ForegroundColor DarkGray
Write-Host ""

# --- STEP 4: Preview with /api/nlc/preview ---
Write-Host "[STEP 1/3] Previewing intent..." -ForegroundColor Cyan
try {
    $previewPayload = @{
        input = $Intent
        address = $Address
    } | ConvertTo-Json -Compress

    $previewResponse = Invoke-WebRequest -Uri "$BaseUrl/api/nlc/preview" `
        -Method Post `
        -ContentType "application/json" `
        -Body $previewPayload `
        -UseBasicParsing `
        -ErrorAction Stop

    $previewData = $previewResponse.Content | ConvertFrom-Json
    $preview = $previewData.preview
} catch {
    Write-Host "[ERROR] Preview failed: $_" -ForegroundColor Red
    exit 1
}

Write-Host "  ✓ Intent parsed successfully" -ForegroundColor Green
Write-Host "    - Action:     $($preview.action)" -ForegroundColor DarkGray
Write-Host "    - Executable: $($preview.executable)" -ForegroundColor DarkGray
if ($preview.amount) { Write-Host "    - Amount:     $($preview.amount) NVC" -ForegroundColor DarkGray }
if ($preview.target -or $preview.resolved_target) { Write-Host "    - Target:     $($preview.resolved_target ?? $preview.target)" -ForegroundColor DarkGray }

if (-not $preview.executable) {
    Write-Host "[ABORT] Intent is not executable. Reason: $($preview.reason)" -ForegroundColor Yellow
    exit 0
}

Write-Host ""

# --- STEP 5: Build Signed Request ---
Write-Host "[STEP 2/3] Building signed request..." -ForegroundColor Cyan

$ts = [int]([DateTime]::UtcNow - [DateTime]::UnixEpoch).TotalSeconds
$nonce = "exec-$(Get-Random -Minimum 100000 -Maximum 999999)"

$bodyObj = @{
    input = $Intent
    address = $Address
    ts = $ts
    nonce = $nonce
} | ConvertTo-Json -Compress

Write-Host "  ✓ Payload constructed (size: $($bodyObj.Length) bytes)" -ForegroundColor Green
Write-Host "    - ts:    $ts" -ForegroundColor DarkGray
Write-Host "    - nonce: $nonce" -ForegroundColor DarkGray

# Compute signature: SHA256(secret:/api/nlc:body)
$signaturePayload = "$($ApiSecret):/api/nlc:$($bodyObj)"
$hasher = [System.Security.Cryptography.SHA256]::Create()
$signatureBytes = $hasher.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($signaturePayload))
$signature = [System.BitConverter]::ToString($signatureBytes).Replace("-", "").ToLower()

Write-Host "  ✓ Signature computed (SHA256)" -ForegroundColor Green
Write-Host "    Sig: $($signature.Substring(0, 16))..." -ForegroundColor DarkGray

Write-Host ""

# --- STEP 6: Confirm Execution ---
Write-Host "[STEP 3/3] Confirming execution..." -ForegroundColor Cyan
Write-Host "  Ready to execute: $Intent" -ForegroundColor Yellow
Write-Host "  Type 'CONFIRM' to proceed (case-sensitive): " -ForegroundColor Yellow -NoNewline
$confirm = Read-Host
Write-Host ""

if ($confirm -ne "CONFIRM") {
    Write-Host "[ABORT] Execution cancelled by user." -ForegroundColor Yellow
    exit 0
}

# --- STEP 7: Execute Signed Request ---
Write-Host "[EXEC] Sending signed request to /api/nlc..." -ForegroundColor Cyan

try {
    $response = Invoke-WebRequest -Uri "$BaseUrl/api/nlc" `
        -Method Post `
        -ContentType "application/json" `
        -Body $bodyObj `
        -Headers @{ "x-nfm-signature" = $signature } `
        -UseBasicParsing `
        -ErrorAction Stop

    $result = $response.Content | ConvertFrom-Json
    
    Write-Host ""
    Write-Host "[SUCCESS] Intent executed!" -ForegroundColor Green
    Write-Host "  Status:  $($result.status)" -ForegroundColor Cyan
    Write-Host "  Message: $($result.message)" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Transaction Details:" -ForegroundColor Cyan
    Write-Host ($result | ConvertTo-Json -Depth 3 | Out-String) -ForegroundColor DarkGray
    
    exit 0
} catch {
    $webResponse = $_.Exception.Response
    $statusCode = if ($null -ne $webResponse) { [int]$webResponse.StatusCode } else { 0 }
    $errorBody = if ($null -ne $_.ErrorDetails) { $_.ErrorDetails.Message } else { $_.Exception.Message }
    
    Write-Host ""
    Write-Host "[ERROR] Execution failed with HTTP $statusCode" -ForegroundColor Red
    Write-Host "Response: $errorBody" -ForegroundColor Red
    Write-Host ""
    exit 1
}
