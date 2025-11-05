# LoLShorts Build Environment Verification Script
# Checks if all required tools and dependencies are installed

$ErrorActionPreference = "Continue"

Write-Host "🔍 LoLShorts Build Environment Verification" -ForegroundColor Cyan
Write-Host "===========================================" -ForegroundColor Cyan
Write-Host ""

$issues = @()
$warnings = @()

# Check Rust
Write-Host "Checking Rust..." -NoNewline
try {
    $rustVersion = rustc --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host " ✅" -ForegroundColor Green
        Write-Host "  $rustVersion" -ForegroundColor Gray
    } else {
        Write-Host " ❌" -ForegroundColor Red
        $issues += "Rust is not installed or not in PATH"
    }
} catch {
    Write-Host " ❌" -ForegroundColor Red
    $issues += "Rust is not installed. Install from: https://www.rust-lang.org/"
}

# Check Cargo
Write-Host "Checking Cargo..." -NoNewline
try {
    $cargoVersion = cargo --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host " ✅" -ForegroundColor Green
        Write-Host "  $cargoVersion" -ForegroundColor Gray
    } else {
        Write-Host " ❌" -ForegroundColor Red
        $issues += "Cargo is not installed"
    }
} catch {
    Write-Host " ❌" -ForegroundColor Red
    $issues += "Cargo is not installed"
}

# Check Node.js
Write-Host "Checking Node.js..." -NoNewline
try {
    $nodeVersion = node --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        $versionNumber = [version]($nodeVersion -replace 'v', '')
        if ($versionNumber.Major -ge 18) {
            Write-Host " ✅" -ForegroundColor Green
            Write-Host "  $nodeVersion" -ForegroundColor Gray
        } else {
            Write-Host " ⚠️" -ForegroundColor Yellow
            Write-Host "  $nodeVersion (recommend v18+)" -ForegroundColor Yellow
            $warnings += "Node.js version is below recommended v18+"
        }
    } else {
        Write-Host " ❌" -ForegroundColor Red
        $issues += "Node.js is not installed"
    }
} catch {
    Write-Host " ❌" -ForegroundColor Red
    $issues += "Node.js is not installed. Install from: https://nodejs.org/"
}

# Check npm
Write-Host "Checking npm..." -NoNewline
try {
    $npmVersion = npm --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host " ✅" -ForegroundColor Green
        Write-Host "  v$npmVersion" -ForegroundColor Gray
    } else {
        Write-Host " ❌" -ForegroundColor Red
        $issues += "npm is not installed"
    }
} catch {
    Write-Host " ❌" -ForegroundColor Red
    $issues += "npm is not installed"
}

# Check WiX Toolset
Write-Host "Checking WiX Toolset..." -NoNewline
try {
    $wixCandle = Get-Command candle -ErrorAction SilentlyContinue
    $wixLight = Get-Command light -ErrorAction SilentlyContinue

    if ($wixCandle -and $wixLight) {
        Write-Host " ✅" -ForegroundColor Green
        $candlePath = $wixCandle.Source
        Write-Host "  $candlePath" -ForegroundColor Gray
    } else {
        Write-Host " ❌" -ForegroundColor Red
        $issues += "WiX Toolset is not installed or not in PATH. Required for MSI installer."
        Write-Host "  Download from: https://wixtoolset.org/releases/" -ForegroundColor Red
    }
} catch {
    Write-Host " ❌" -ForegroundColor Red
    $issues += "WiX Toolset is not installed. Required for MSI installer."
}

# Check Visual Studio Build Tools
Write-Host "Checking Visual Studio Build Tools..." -NoNewline
$vswhere = "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
    $vsPath = & $vswhere -latest -property installationPath
    if ($vsPath) {
        Write-Host " ✅" -ForegroundColor Green
        Write-Host "  $vsPath" -ForegroundColor Gray
    } else {
        Write-Host " ⚠️" -ForegroundColor Yellow
        $warnings += "Visual Studio Build Tools may not be properly configured"
    }
} else {
    Write-Host " ⚠️" -ForegroundColor Yellow
    Write-Host "  vswhere.exe not found (may still work if build tools are installed)" -ForegroundColor Yellow
    $warnings += "Could not verify Visual Studio Build Tools installation"
}

# Check Tauri CLI
Write-Host "Checking Tauri CLI..." -NoNewline
try {
    $tauriVersion = cargo tauri --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host " ✅" -ForegroundColor Green
        Write-Host "  $tauriVersion" -ForegroundColor Gray
    } else {
        Write-Host " ⚠️" -ForegroundColor Yellow
        $warnings += "Tauri CLI not found. Install with: npm install -g @tauri-apps/cli"
    }
} catch {
    Write-Host " ⚠️" -ForegroundColor Yellow
    $warnings += "Tauri CLI not installed. Run: npm install -g @tauri-apps/cli"
}

# Check FFmpeg binaries
Write-Host "Checking FFmpeg binaries..." -NoNewline
$ffmpegPath = "..\bin\ffmpeg.exe"
$ffprobePath = "..\bin\ffprobe.exe"

if ((Test-Path $ffmpegPath) -and (Test-Path $ffprobePath)) {
    Write-Host " ✅" -ForegroundColor Green
    $ffmpegSize = (Get-Item $ffmpegPath).Length / 1MB
    $ffprobeSize = (Get-Item $ffprobePath).Length / 1MB
    Write-Host "  ffmpeg.exe:  $($ffmpegSize.ToString('F2')) MB" -ForegroundColor Gray
    Write-Host "  ffprobe.exe: $($ffprobeSize.ToString('F2')) MB" -ForegroundColor Gray
} else {
    Write-Host " ⚠️" -ForegroundColor Yellow
    Write-Host "  FFmpeg binaries not found in bin directory" -ForegroundColor Yellow
    $warnings += "Run prepare_ffmpeg.ps1 to download FFmpeg binaries before building"
}

# Check Node dependencies
Write-Host "Checking Node dependencies..." -NoNewline
$nodeModulesPath = "..\..\node_modules"
if (Test-Path $nodeModulesPath) {
    Write-Host " ✅" -ForegroundColor Green
    Write-Host "  node_modules directory exists" -ForegroundColor Gray
} else {
    Write-Host " ⚠️" -ForegroundColor Yellow
    $warnings += "node_modules not found. Run: npm install"
}

# Summary
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "Summary" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

if ($issues.Count -eq 0 -and $warnings.Count -eq 0) {
    Write-Host "✅ All checks passed! Environment is ready for building." -ForegroundColor Green
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Cyan
    Write-Host "  1. Ensure FFmpeg is prepared: .\prepare_ffmpeg.ps1" -ForegroundColor Gray
    Write-Host "  2. Build the app: cargo tauri build" -ForegroundColor Gray
    Write-Host ""
    exit 0
} else {
    if ($issues.Count -gt 0) {
        Write-Host "❌ Critical Issues Found:" -ForegroundColor Red
        foreach ($issue in $issues) {
            Write-Host "  • $issue" -ForegroundColor Red
        }
        Write-Host ""
    }

    if ($warnings.Count -gt 0) {
        Write-Host "⚠️  Warnings:" -ForegroundColor Yellow
        foreach ($warning in $warnings) {
            Write-Host "  • $warning" -ForegroundColor Yellow
        }
        Write-Host ""
    }

    if ($issues.Count -gt 0) {
        Write-Host "❌ Please fix critical issues before building." -ForegroundColor Red
        exit 1
    } else {
        Write-Host "⚠️  You may proceed, but consider addressing warnings for best results." -ForegroundColor Yellow
        exit 0
    }
}
