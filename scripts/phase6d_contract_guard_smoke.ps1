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

Write-Host "[NFM-P6D] Running Phase 6D contract guard checks" -ForegroundColor Cyan
Write-Host "[NFM-P6D] BaseUrl: $BaseUrl" -ForegroundColor Cyan

$govRes = Invoke-JsonRequest -Method "GET" -Path "/api/governance/indicators"
if ($govRes.StatusCode -ne 200) {
    Write-Host "[FAIL] Governance indicators endpoint failed ($($govRes.StatusCode)): $($govRes.Raw)" -ForegroundColor Red
    exit 1
}

foreach ($required in @('quorum_target', 'active_proposals', 'quorum_progress', 'veto_risk_count', 'treasury_pool')) {
    if ($govRes.Raw -notmatch ('"' + [regex]::Escape($required) + '"')) {
        Write-Host "[FAIL] Governance indicators response missing field: $required" -ForegroundColor Red
        Write-Host "[FAIL] Body: $($govRes.Raw)" -ForegroundColor Red
        exit 1
    }
}

$kgRes = Invoke-JsonRequest -Method "GET" -Path "/api/kg/semantic"
if ($kgRes.StatusCode -ne 200) {
    Write-Host "[FAIL] KG semantic endpoint failed ($($kgRes.StatusCode)): $($kgRes.Raw)" -ForegroundColor Red
    exit 1
}

foreach ($required in @('concepts', 'nodes', 'category_counts')) {
    if ($kgRes.Raw -notmatch ('"' + [regex]::Escape($required) + '"')) {
        Write-Host "[FAIL] KG semantic response missing field: $required" -ForegroundColor Red
        Write-Host "[FAIL] Body: $($kgRes.Raw)" -ForegroundColor Red
        exit 1
    }
}

Write-Host "[PASS] Phase 6D contract guard verified (/api/governance/indicators + /api/kg/semantic)." -ForegroundColor Green
exit 0
