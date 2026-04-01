param(
    [string]$BaseUrl = "http://127.0.0.1:3000",
    [int]$Repeat = 1
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
        $response = Invoke-WebRequest `
            -Uri $uri `
            -Method $Method `
            -ContentType "application/json" `
            -Body $jsonBody `
            -UseBasicParsing

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
        } catch {
            $payload = $rawContent
        }
    }

    return [PSCustomObject]@{
        StatusCode = $statusCode
        Payload    = $payload
        Raw        = $rawContent
        Path       = $Path
    }
}

function Assert-Status {
    param(
        [Parameter(Mandatory = $true)]$Result,
        [Parameter(Mandatory = $true)][int[]]$Allowed
    )

    if ($Allowed -notcontains $Result.StatusCode) {
        $body = if ($Result.Raw) { $Result.Raw } else { "<empty>" }
        throw "[$($Result.Path)] expected status $($Allowed -join ',') but got $($Result.StatusCode). Body: $body"
    }
}

Write-Host "=== NFM App Actions Smoke Test ===" -ForegroundColor Cyan
Write-Host "Base URL: $BaseUrl" -ForegroundColor DarkCyan
Write-Host "Repeat: $Repeat" -ForegroundColor DarkCyan

if ($Repeat -lt 1) {
    throw "Repeat must be >= 1"
}

$stateBefore = Invoke-JsonRequest -Method "GET" -Path "/api/app/state"
Assert-Status -Result $stateBefore -Allowed @(200)

$userAddress = [string]$stateBefore.Payload.user_profile.nfmAddress
$startingBalance = [double]$stateBefore.Payload.user_profile.balance
$createdProposalIds = @()

if ([string]::IsNullOrWhiteSpace($userAddress)) {
    throw "user_profile.nfmAddress is empty in /api/app/state"
}

Write-Host "User address: $userAddress" -ForegroundColor Yellow
Write-Host "Initial balance: $startingBalance" -ForegroundColor Yellow

for ($i = 1; $i -le $Repeat; $i++) {
    Write-Host "-- Iteration $i/$Repeat --" -ForegroundColor Magenta

    # 1) Wallet transfer
    $transferAmount = 1.25
    $transferTarget = "nfm_smoke_receiver_$i"
    $transferRes = Invoke-JsonRequest -Method "POST" -Path "/api/app/wallet/transfer" -Body @{
        from   = $userAddress
        to     = $transferTarget
        amount = $transferAmount
    }
    Assert-Status -Result $transferRes -Allowed @(200)
    Write-Host "[OK] Wallet transfer" -ForegroundColor Green

    # 2) Create governance proposal
    $proposalTitle = "Smoke Proposal $(Get-Date -Format 'yyyyMMdd-HHmmss')-it$i"
    $proposalRes = Invoke-JsonRequest -Method "POST" -Path "/api/app/governance/proposal" -Body @{
        title       = $proposalTitle
        description = "Automated smoke proposal"
        proposer    = $userAddress
    }
    Assert-Status -Result $proposalRes -Allowed @(200)
    $proposalId = [string]$proposalRes.Payload.proposal_id
    if ([string]::IsNullOrWhiteSpace($proposalId)) {
        throw "proposal_id missing in governance proposal response"
    }
    $createdProposalIds += $proposalId
    Write-Host "[OK] Governance proposal created: $proposalId" -ForegroundColor Green

    # 3) Vote proposal (backend may require reputation > 0)
    $voteRes = Invoke-JsonRequest -Method "POST" -Path "/api/app/governance/vote" -Body @{
        proposal_id = $proposalId
        approve     = $true
        voter       = $userAddress
    }
    if ($voteRes.StatusCode -eq 200) {
        Write-Host "[OK] Governance vote accepted" -ForegroundColor Green
    } elseif ($voteRes.StatusCode -eq 400 -and ($voteRes.Raw -match "No reputation")) {
        Write-Host "[OK] Governance vote rejected as expected (No reputation)" -ForegroundColor DarkYellow
    } else {
        Assert-Status -Result $voteRes -Allowed @(200)
    }

    # 4) Claim quest
    $questRes = Invoke-JsonRequest -Method "POST" -Path "/api/app/quest/claim" -Body @{
        quest_id = "q-2"
        address  = $userAddress
    }
    if ($questRes.StatusCode -eq 200) {
        Write-Host "[OK] Quest claim" -ForegroundColor Green
    } elseif ($questRes.StatusCode -eq 400 -and ($questRes.Raw -match "Completed" -or $questRes.Raw -match "sudah")) {
        Write-Host "[OK] Quest claim skipped (already completed)" -ForegroundColor DarkYellow
    } else {
        Assert-Status -Result $questRes -Allowed @(200)
    }

    # 5) Mystery extract
    $mysteryRes = Invoke-JsonRequest -Method "POST" -Path "/api/app/mystery/extract" -Body @{
        address = $userAddress
    }
    Assert-Status -Result $mysteryRes -Allowed @(200)
    Write-Host "[OK] Mystery extract" -ForegroundColor Green

    # 6) Market purchase
    $purchaseRes = Invoke-JsonRequest -Method "POST" -Path "/api/app/market/purchase" -Body @{
        address = $userAddress
        item_id = "smoke-item-$i"
        price   = 1.0
    }
    Assert-Status -Result $purchaseRes -Allowed @(200)
    Write-Host "[OK] Market purchase" -ForegroundColor Green
}

$stateAfter = Invoke-JsonRequest -Method "GET" -Path "/api/app/state"
Assert-Status -Result $stateAfter -Allowed @(200)

$endingProposals = @($stateAfter.Payload.proposals).Count
$visibleIds = @($stateAfter.Payload.proposals | ForEach-Object {
    $id = [string]$_.id
    if ([string]::IsNullOrWhiteSpace($id)) { return $null }
    if ($id.StartsWith("prop-")) { return $id.Substring(5) }
    return $id
})

foreach ($expected in $createdProposalIds) {
    if ($visibleIds -notcontains $expected) {
        throw "Created proposal id $expected is not visible in /api/app/state proposals payload"
    }
}

$endingBalance = [double]$stateAfter.Payload.user_profile.balance
Write-Host "Final balance: $endingBalance" -ForegroundColor Yellow
Write-Host "Final proposal count: $endingProposals" -ForegroundColor Yellow

Write-Host "=== SMOKE TEST PASSED ===" -ForegroundColor Cyan
