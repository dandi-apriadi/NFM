$secret = 'nfm_dev_secret_v0.5'
$url = '/api/nlc'

# First get initial status
$initial = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/status'
$initial_balance = $initial.balance
$initial_fees = $initial.total_fees

Write-Host "Initial Balance: $initial_balance"
Write-Host "Initial Protocol Fees: $initial_fees"

# --- TEST 1: INVALID NLC COMMAND (Should NOT charge gas) ---
Write-Host "`n[TEST 1] Sending invalid NLC command 'stake xyz'..."
$body_invalid = '{"input":"stake xyz","address":"nfm_e85a48fad1443fc6f8585cd5cbe2c8cd"}'
$payload_invalid = "$($secret):$($url):$($body_invalid)"
$sha256 = [System.Security.Cryptography.SHA256]::Create()
$hash_invalid = $sha256.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($payload_invalid))
$sig_invalid = [System.BitConverter]::ToString($hash_invalid).Replace('-', '').ToLower()

try {
    Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/nlc' -Method Post -Headers @{'x-nfm-signature'=$sig_invalid; 'Content-Type'='application/json'} -Body $body_invalid
} catch {
    Write-Host "Expected Error: $($_.Exception.Message)"
}

Start-Sleep -Seconds 1
$after_invalid = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/status'
Write-Host "Balance after invalid: $($after_invalid.balance)"
Write-Host "Fees after invalid: $($after_invalid.total_fees)"

if ($after_invalid.balance -eq $initial_balance) {
    Write-Host "SUCCESS: No gas fee deducted for invalid command! ✓" -ForegroundColor Green
} else {
    Write-Host "FAILURE: Balance changed for invalid command! ✗" -ForegroundColor Red
}

# --- TEST 2: VALID NLC COMMAND (Should charge gas AND stake) ---
Write-Host "`n[TEST 2] Sending valid NLC command 'stake 50'..."
$body_valid = '{"input":"stake 50","address":"nfm_e85a48fad1443fc6f8585cd5cbe2c8cd"}'
$payload_valid = "$($secret):$($url):$($body_valid)"
$hash_valid = $sha256.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($payload_valid))
$sig_valid = [System.BitConverter]::ToString($hash_valid).Replace('-', '').ToLower()

$res = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/nlc' -Method Post -Headers @{'x-nfm-signature'=$sig_valid; 'Content-Type'='application/json'} -Body $body_valid
Write-Host "Response: $($res | ConvertTo-Json -Compress)"

Start-Sleep -Seconds 1
$after_valid = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/status'
Write-Host "Balance after valid: $($after_valid.balance) (Expected: $($initial_balance - 50 - 0.5) approx)"
Write-Host "Fees after valid: $($after_valid.total_fees)"

if ($after_valid.balance -lt $after_invalid.balance) {
    Write-Host "SUCCESS: Balance deducted for valid command! ✓" -ForegroundColor Green
} else {
    Write-Host "FAILURE: Balance NOT deducted! ✗" -ForegroundColor Red
}
