# NFM Blockchain Reset Script
# This will WIPE ALL DATA and start from Genesis.

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  NFM BLOCKCHAIN SYSTEM RESET" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Stop the node
Write-Host "[1/3] Stopping NFM Core node..." -ForegroundColor Yellow
$process = Get-Process "nfm-core-blockchain" -ErrorAction SilentlyContinue
if ($process) {
    Stop-Process -Name "nfm-core-blockchain" -Force
    Start-Sleep -Seconds 2
    Write-Host "      Node stopped successfully." -ForegroundColor Green
} else {
    Write-Host "      Node is not running." -ForegroundColor Gray
}

# 2. Delete database files
Write-Host "[2/3] Clearing persistent databases..." -ForegroundColor Yellow
$dbFiles = @(
    "nfm_main.db",
    "nfm_wallets.db",
    "nfm_main.db_governance.db",
    "temp_gov.db"
)

foreach ($db in $dbFiles) {
    if (Test-Path $db) {
        Remove-Item -Path $db -Recurse -Force
        Write-Host "      Deleted: $db" -ForegroundColor Green
    } else {
        Write-Host "      Skipped (not found): $db" -ForegroundColor Gray
    }
}

# 3. Restart the node
Write-Host "[3/3] System ready. Restarting node for Genesis initialization..." -ForegroundColor Yellow
Write-Host "      Run 'cargo run' or use your node-runner script." -ForegroundColor Cyan

Write-Host "`nRESET COMPLETE. Blockchain is now empty." -ForegroundColor Green
