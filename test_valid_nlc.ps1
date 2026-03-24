$secret = 'nfm_dev_secret_v0.5'
$url = '/api/nlc'
$body = '{"input":"stake 50","address":"nfm_34750f98bd59fcfc946da45aaabe933b"}'
$payload = "$($secret):$($url):$($body)"
$sha256 = [System.Security.Cryptography.SHA256]::Create()
$hash = $sha256.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($payload))
$sig = [System.BitConverter]::ToString($hash).Replace('-', '').ToLower()
$headers = @{
    'x-nfm-signature' = $sig
    'Content-Type' = 'application/json'
}
Invoke-RestMethod -Uri 'http://127.0.0.1:3000/api/nlc' -Method Post -Headers $headers -Body $body | ConvertTo-Json
