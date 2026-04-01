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

Write-Host "[NFM-BRAIN] Running brain curriculum guard checks" -ForegroundColor Cyan
Write-Host "[NFM-BRAIN] BaseUrl: $BaseUrl" -ForegroundColor Cyan

$statusRes = Invoke-JsonRequest -Method "GET" -Path "/api/status"
if ($statusRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Unable to read /api/status" -ForegroundColor Red
    exit 1
}

$proposer = [string]$statusRes.Payload.node
if ([string]::IsNullOrWhiteSpace($proposer)) {
    Write-Host "[FAIL] Missing node address from /api/status" -ForegroundColor Red
    exit 1
}

$intent = "start_learning_window"

$proposeRes = Invoke-JsonRequest -Method "POST" -Path "/api/brain/curriculum/propose" -Body @{
    address = $proposer
    intent = $intent
    model_version = "nfm-brain-guard"
}
if ($proposeRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Curriculum propose failed ($($proposeRes.StatusCode)): $($proposeRes.Raw)" -ForegroundColor Red
    exit 1
}
if ($proposeRes.Raw -notmatch '"status":"success"') {
    Write-Host "[FAIL] Curriculum propose missing success marker: $($proposeRes.Raw)" -ForegroundColor Red
    exit 1
}

$voteId = [int]$proposeRes.Payload.intent_vote_id
if ($voteId -le 0) {
    Write-Host "[FAIL] Missing valid intent_vote_id from propose response: $($proposeRes.Raw)" -ForegroundColor Red
    exit 1
}

$activeRes = Invoke-JsonRequest -Method "GET" -Path "/api/brain/curriculum/active"
if ($activeRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Curriculum active fetch failed ($($activeRes.StatusCode)): $($activeRes.Raw)" -ForegroundColor Red
    exit 1
}
if ($activeRes.Raw -notmatch 'windows') {
    Write-Host "[FAIL] Curriculum active payload missing windows field: $($activeRes.Raw)" -ForegroundColor Red
    exit 1
}

$voteRes = Invoke-JsonRequest -Method "POST" -Path "/api/brain/curriculum/vote" -Body @{
    vote_id = $voteId
    address = $proposer
    approve = $true
    execute_now = $false
}
if ($voteRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Curriculum vote failed ($($voteRes.StatusCode)): $($voteRes.Raw)" -ForegroundColor Red
    exit 1
}
if ($voteRes.Raw -notmatch '"status":"success"') {
    Write-Host "[FAIL] Curriculum vote payload missing success marker: $($voteRes.Raw)" -ForegroundColor Red
    exit 1
}

$leaderboardRes = Invoke-JsonRequest -Method "GET" -Path "/api/brain/reputation/leaderboard"
if ($leaderboardRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Leaderboard fetch failed ($($leaderboardRes.StatusCode)): $($leaderboardRes.Raw)" -ForegroundColor Red
    exit 1
}
if ($leaderboardRes.Raw -notmatch 'leaderboard') {
    Write-Host "[FAIL] Leaderboard payload missing leaderboard field: $($leaderboardRes.Raw)" -ForegroundColor Red
    exit 1
}

Write-Host "[PASS] Brain curriculum guard verified (propose + active + vote + leaderboard)." -ForegroundColor Green
exit 0
