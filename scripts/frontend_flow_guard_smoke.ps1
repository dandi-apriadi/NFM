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

function Assert-ContainsField {
    param(
        [string]$Raw,
        [string]$Field,
        [string]$Context
    )

    if ($Raw -notmatch ('"' + [regex]::Escape($Field) + '"')) {
        Write-Host "[FAIL] $Context missing field: $Field" -ForegroundColor Red
        exit 1
    }
}

Write-Host "[NFM-FE] Running frontend flow contract guard" -ForegroundColor Cyan
Write-Host "[NFM-FE] BaseUrl: $BaseUrl" -ForegroundColor Cyan

# 1) Core app state for Dashboard/Explorer/Marketplace/Governance/Drive
$appState = Invoke-JsonRequest -Method "GET" -Path "/api/app/state"
if ($appState.StatusCode -ne 200) {
    Write-Host "[FAIL] /api/app/state failed ($($appState.StatusCode)): $($appState.Raw)" -ForegroundColor Red
    exit 1
}
foreach ($field in @('status','blocks','transactions','market_items','proposals','drive_files','kg_concepts','api_docs')) {
    Assert-ContainsField -Raw $appState.Raw -Field $field -Context "/api/app/state"
}

# 2) Marketplace contract
$auctionList = Invoke-JsonRequest -Method "GET" -Path "/api/auction/list"
if ($auctionList.StatusCode -ne 200) {
    Write-Host "[FAIL] /api/auction/list failed ($($auctionList.StatusCode)): $($auctionList.Raw)" -ForegroundColor Red
    exit 1
}
foreach ($field in @('status','count','auctions')) {
    Assert-ContainsField -Raw $auctionList.Raw -Field $field -Context "/api/auction/list"
}

# 3) Drive contract
$driveFiles = Invoke-JsonRequest -Method "GET" -Path "/api/drive/files"
if ($driveFiles.StatusCode -ne 200) {
    Write-Host "[FAIL] /api/drive/files failed ($($driveFiles.StatusCode)): $($driveFiles.Raw)" -ForegroundColor Red
    exit 1
}
foreach ($field in @('status','count','files')) {
    Assert-ContainsField -Raw $driveFiles.Raw -Field $field -Context "/api/drive/files"
}

# 4) Governance + KG phase 6D read models used by frontend
$govIndicators = Invoke-JsonRequest -Method "GET" -Path "/api/governance/indicators"
if ($govIndicators.StatusCode -ne 200) {
    Write-Host "[FAIL] /api/governance/indicators failed ($($govIndicators.StatusCode)): $($govIndicators.Raw)" -ForegroundColor Red
    exit 1
}
foreach ($field in @('quorum_target','active_proposals','quorum_progress','veto_risk_count','treasury_pool')) {
    Assert-ContainsField -Raw $govIndicators.Raw -Field $field -Context "/api/governance/indicators"
}

$kgSemantic = Invoke-JsonRequest -Method "GET" -Path "/api/kg/semantic"
if ($kgSemantic.StatusCode -ne 200) {
    Write-Host "[FAIL] /api/kg/semantic failed ($($kgSemantic.StatusCode)): $($kgSemantic.Raw)" -ForegroundColor Red
    exit 1
}
foreach ($field in @('concepts','nodes','category_counts')) {
    Assert-ContainsField -Raw $kgSemantic.Raw -Field $field -Context "/api/kg/semantic"
}

# 5) DevPortal NLC preview contract
$nlcPreview = Invoke-JsonRequest -Method "POST" -Path "/api/nlc/preview" -Body @{
    input = "send 12.5 @nfm_preview_target"
}
if ($nlcPreview.StatusCode -ne 200) {
    Write-Host "[FAIL] /api/nlc/preview failed ($($nlcPreview.StatusCode)): $($nlcPreview.Raw)" -ForegroundColor Red
    exit 1
}
foreach ($field in @('status','preview','action','executable')) {
    Assert-ContainsField -Raw $nlcPreview.Raw -Field $field -Context "/api/nlc/preview"
}

Write-Host "[PASS] Frontend flow contract guard verified (Dashboard/Explorer/Marketplace/Governance/Drive)." -ForegroundColor Green
exit 0
