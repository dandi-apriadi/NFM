$secret = 'nfm_dev_secret_v0.5'
$url = '/api/staking/deposit'
$body = '{"amount":10.0,"address":"nfm_954bd86cf02170ef848197a5660d03a6"}'
$payload = "$($secret):$($url):$($body)"
$sha256 = [System.Security.Cryptography.SHA256]::Create()
$hash = $sha256.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($payload))
$sig = [System.BitConverter]::ToString($hash).Replace('-', '').ToLower()

Write-Host "Sending Staking Deposit..."
$response = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/staking/deposit' -Method Post -Headers @{'x-nfm-signature'=$sig; 'Content-Type'='application/json'} -Body $body
$response | ConvertTo-Json

Write-Host "`nWaiting 2s for block generation..."
Start-Sleep -Seconds 2

Write-Host "Latest Block Data:"
$latest = Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/blocks' | Select-Object -Last 1
$latest | ConvertTo-Json
