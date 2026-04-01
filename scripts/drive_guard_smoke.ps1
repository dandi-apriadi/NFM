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

Write-Host "[NFM-DRIVE] Running drive ownership guard checks" -ForegroundColor Cyan
Write-Host "[NFM-DRIVE] BaseUrl: $BaseUrl" -ForegroundColor Cyan

$statusRes = Invoke-JsonRequest -Method "GET" -Path "/api/status"
if ($statusRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Unable to read /api/status" -ForegroundColor Red
    exit 1
}

$owner = [string]$statusRes.Payload.node
if ([string]::IsNullOrWhiteSpace($owner)) {
    Write-Host "[FAIL] Missing node address from /api/status" -ForegroundColor Red
    exit 1
}

$fileName = "drive-guard-$(Get-Date -Format 'yyyyMMddHHmmss').txt"
$fileContent = "drive guard content $(Get-Date -Format o)"

$uploadRes = Invoke-JsonRequest -Method "POST" -Path "/api/drive/upload" -Body @{
    address = $owner
    name = $fileName
    content = $fileContent
    type = "TEXT"
    fragments = 1
}

if ($uploadRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Upload failed ($($uploadRes.StatusCode)): $($uploadRes.Raw)" -ForegroundColor Red
    exit 1
}

$fileId = [string]$uploadRes.Payload.file_id
if ([string]::IsNullOrWhiteSpace($fileId)) {
    Write-Host "[FAIL] Upload response missing file_id" -ForegroundColor Red
    exit 1
}

$ownerDownloadRes = Invoke-JsonRequest -Method "POST" -Path "/api/drive/download" -Body @{
    file_id = $fileId
    address = $owner
}

if ($ownerDownloadRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Owner download failed ($($ownerDownloadRes.StatusCode)): $($ownerDownloadRes.Raw)" -ForegroundColor Red
    exit 1
}

if (($ownerDownloadRes.Raw -notmatch [regex]::Escape($fileContent)) -and ($ownerDownloadRes.Raw -notmatch "content")) {
    Write-Host "[FAIL] Owner download response missing expected content" -ForegroundColor Red
    exit 1
}

$otherDownloadRes = Invoke-JsonRequest -Method "POST" -Path "/api/drive/download" -Body @{
    file_id = $fileId
    address = "nfm_non_owner_guard"
}

if ($otherDownloadRes.StatusCode -ne 403) {
    Write-Host "[FAIL] Non-owner download should be 403, got $($otherDownloadRes.StatusCode)" -ForegroundColor Red
    Write-Host "[FAIL] Body: $($otherDownloadRes.Raw)" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] Drive ownership guard verified (owner=200, non-owner=403)." -ForegroundColor Green
exit 0
