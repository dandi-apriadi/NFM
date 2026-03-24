# ============================================================
# NFM Node Runner — Bootstrap Script (Windows PowerShell)
# Auto-detects hardware dan menjalankan node dengan profil optimal.
# ============================================================

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  NFM Node Runner - Hardware Detection" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# -----------------------------------------------------------
# 1. Detect CPU
# -----------------------------------------------------------
try {
    $cpu = Get-CimInstance -ClassName Win32_Processor -ErrorAction Stop
    $CPU_NAME = $cpu.Name.Trim()
    $CPU_CORES = $cpu.NumberOfCores
    $CPU_THREADS = $cpu.NumberOfLogicalProcessors
} catch {
    $CPU_NAME = "Unknown"
    $CPU_CORES = (Get-ComputerInfo).CsNumberOfProcessors
    $CPU_THREADS = [Environment]::ProcessorCount
}
Write-Host "  CPU       : $CPU_NAME" -ForegroundColor White
Write-Host "  Cores     : $CPU_CORES cores / $CPU_THREADS threads" -ForegroundColor White

# -----------------------------------------------------------
# 2. Detect RAM
# -----------------------------------------------------------
try {
    $os = Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction Stop
    $RAM_TOTAL_MB = [math]::Round($os.TotalVisibleMemorySize / 1024)
    $RAM_FREE_MB  = [math]::Round($os.FreePhysicalMemory / 1024)
} catch {
    $RAM_TOTAL_MB = 0
    $RAM_FREE_MB  = 0
}
Write-Host "  RAM       : $RAM_TOTAL_MB MB total / $RAM_FREE_MB MB free" -ForegroundColor White

# -----------------------------------------------------------
# 3. Detect GPU & VRAM
# -----------------------------------------------------------
$GPU_NAME = "none"
$GPU_VRAM_MB = 0

try {
    $gpus = Get-CimInstance -ClassName Win32_VideoController -ErrorAction Stop
    foreach ($gpu in $gpus) {
        $name = $gpu.Name
        $vram = [math]::Round($gpu.AdapterRAM / 1MB)

        # AdapterRAM bisa overflow (negatif/0) untuk GPU > 4GB, coba via registry
        if ($vram -le 0 -or $vram -gt 65536) {
            try {
                $regPath = "HKLM:\SYSTEM\ControlSet001\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}"
                $subkeys = Get-ChildItem $regPath -ErrorAction Stop
                foreach ($key in $subkeys) {
                    $props = Get-ItemProperty $key.PSPath -ErrorAction SilentlyContinue
                    if ($props.'HardwareInformation.qwMemorySize') {
                        $regVram = [math]::Round([uint64]$props.'HardwareInformation.qwMemorySize' / 1MB)
                        if ($regVram -gt $vram) { $vram = $regVram }
                    }
                    if ($props.'HardwareInformation.MemorySize') {
                        $regVram2 = [math]::Round([uint64]$props.'HardwareInformation.MemorySize' / 1MB)
                        if ($regVram2 -gt $vram) { $vram = $regVram2 }
                    }
                }
            } catch { }
        }

        # Pilih GPU dengan VRAM terbesar
        if ($vram -gt $GPU_VRAM_MB) {
            $GPU_NAME = $name
            $GPU_VRAM_MB = $vram
        }
    }
} catch {
    $GPU_NAME = "Not detected"
    $GPU_VRAM_MB = 0
}

# Fallback: coba nvidia-smi jika tersedia
if ($GPU_VRAM_MB -eq 0) {
    try {
        $nvsmi = & nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits 2>$null
        if ($nvsmi) {
            $parts = $nvsmi.Split(",")
            $GPU_NAME = $parts[0].Trim()
            $GPU_VRAM_MB = [int]$parts[1].Trim()
        }
    } catch { }
}

Write-Host "  GPU       : $GPU_NAME" -ForegroundColor White
Write-Host "  GPU VRAM  : $GPU_VRAM_MB MB" -ForegroundColor White

# -----------------------------------------------------------
# 4. Detect Battery Status
# -----------------------------------------------------------
$ON_BATTERY = $false
try {
    $battery = Get-CimInstance -ClassName Win32_Battery -ErrorAction Stop
    if ($battery) {
        $batteryPercent = $battery.EstimatedChargeRemaining
        $batteryStatus = $battery.BatteryStatus
        # BatteryStatus: 1=Discharging, 2=AC Power
        if ($batteryStatus -eq 1) {
            $ON_BATTERY = $true
        }
        Write-Host "  Battery   : ${batteryPercent}% $(if($ON_BATTERY){'(Discharging)'}else{'(Charging/AC)'})" -ForegroundColor White
    } else {
        Write-Host "  Battery   : Desktop (No battery)" -ForegroundColor White
    }
} catch {
    Write-Host "  Battery   : Not detected" -ForegroundColor White
}

# -----------------------------------------------------------
# 5. Auto-select Performance Profile
# -----------------------------------------------------------
Write-Host ""

$PROFILE = if ($env:NFM_PROFILE) { $env:NFM_PROFILE } else { "auto" }

if ($PROFILE -eq "auto") {
    if ($ON_BATTERY) {
        $PROFILE = "quiet"
        Write-Host "  [AUTO] Battery mode detected -> Quiet" -ForegroundColor Yellow
    }
    elseif ($RAM_TOTAL_MB -lt 2048) {
        $PROFILE = "quiet"
        Write-Host "  [AUTO] Low RAM (< 2GB) -> Quiet" -ForegroundColor Yellow
    }
    elseif ($GPU_VRAM_MB -ge 4000) {
        $PROFILE = "turbo"
        Write-Host "  [AUTO] Strong GPU ($GPU_VRAM_MB MB VRAM) -> Turbo" -ForegroundColor Green
    }
    elseif ($GPU_VRAM_MB -ge 2000 -and $RAM_TOTAL_MB -ge 8192) {
        $PROFILE = "balanced"
        Write-Host "  [AUTO] Good specs -> Balanced" -ForegroundColor Green
    }
    else {
        $PROFILE = "balanced"
        Write-Host "  [AUTO] Standard hardware -> Balanced" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  Selected Profile: $PROFILE" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# -----------------------------------------------------------
# 6. Export & Run
# -----------------------------------------------------------
$env:NFM_PROFILE = $PROFILE
$env:NFM_CPU_CORES = $CPU_CORES
$env:NFM_RAM_MB = $RAM_TOTAL_MB
$env:NFM_GPU_VRAM_MB = $GPU_VRAM_MB

Write-Host ""
Write-Host "  Starting NFM Node with profile: $PROFILE ..." -ForegroundColor White

$blockchainPath = Join-Path $PSScriptRoot "..\..\core\blockchain"

if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Push-Location $blockchainPath
    cargo run --release
    Pop-Location
} else {
    Write-Host "  ERROR: Rust/Cargo not found. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}
