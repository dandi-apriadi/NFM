$secret = 'nfm_dev_secret_v0.5'
$url = '/api/nlc'
$body = '{"input":"transfer 10 @alice","address":"nfm_34750f98bd59fcfc946da45aaabe933b"}'
$payload = "$($secret):$($url):$($body)"
$sha256 = [System.Security.Cryptography.SHA256]::Create()
$hash = $sha256.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($payload))
$sig = [System.BitConverter]::ToString($hash).Replace("-","").ToLower()

$headers = @{ "X-NFM-Signature" = $sig }
Invoke-RestMethod -Uri "http://127.0.0.1:3000$url" -Method Post -Body $body -Headers $headers -ContentType "application/json"
