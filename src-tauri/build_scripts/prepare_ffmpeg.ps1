# LoLShorts FFmpeg Preparation Script
# This script downloads and prepares FFmpeg for bundling with the installer
# Run this before building the production installer

$ErrorActionPreference = "Stop"

# Configuration
$FFMPEG_VERSION = "7.1"
$DOWNLOAD_URL = "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
$TEMP_DIR = ".\temp_ffmpeg"
$BIN_DIR = "..\bin"

Write-Host "🎬 LoLShorts FFmpeg Preparation" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan
Write-Host ""

# Create bin directory if it doesn't exist
if (-not (Test-Path $BIN_DIR)) {
    Write-Host "📁 Creating bin directory..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $BIN_DIR | Out-Null
}

# Check if FFmpeg already exists
$ffmpegPath = Join-Path $BIN_DIR "ffmpeg.exe"
$ffprobePath = Join-Path $BIN_DIR "ffprobe.exe"

if ((Test-Path $ffmpegPath) -and (Test-Path $ffprobePath)) {
    Write-Host "✅ FFmpeg binaries already exist in bin directory" -ForegroundColor Green

    # Verify versions
    $ffmpegVersion = & $ffmpegPath -version 2>&1 | Select-Object -First 1
    Write-Host "📦 Current version: $ffmpegVersion" -ForegroundColor Cyan

    $response = Read-Host "Do you want to re-download FFmpeg? (y/N)"
    if ($response -ne "y") {
        Write-Host "✅ Using existing FFmpeg binaries" -ForegroundColor Green
        exit 0
    }
}

# Create temp directory
Write-Host "📁 Creating temporary directory..." -ForegroundColor Yellow
if (Test-Path $TEMP_DIR) {
    Remove-Item -Recurse -Force $TEMP_DIR
}
New-Item -ItemType Directory -Path $TEMP_DIR | Out-Null

# Download FFmpeg
Write-Host "⬇️  Downloading FFmpeg..." -ForegroundColor Yellow
Write-Host "   URL: $DOWNLOAD_URL" -ForegroundColor Gray
$zipPath = Join-Path $TEMP_DIR "ffmpeg.zip"

try {
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $zipPath -UseBasicParsing
    Write-Host "✅ Downloaded successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to download FFmpeg: $_" -ForegroundColor Red
    exit 1
}

# Extract archive
Write-Host "📦 Extracting archive..." -ForegroundColor Yellow
try {
    Expand-Archive -Path $zipPath -DestinationPath $TEMP_DIR -Force
    Write-Host "✅ Extracted successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Failed to extract archive: $_" -ForegroundColor Red
    exit 1
}

# Find ffmpeg.exe and ffprobe.exe
Write-Host "🔍 Locating FFmpeg binaries..." -ForegroundColor Yellow
$ffmpegSource = Get-ChildItem -Path $TEMP_DIR -Recurse -Filter "ffmpeg.exe" | Select-Object -First 1
$ffprobeSource = Get-ChildItem -Path $TEMP_DIR -Recurse -Filter "ffprobe.exe" | Select-Object -First 1

if (-not $ffmpegSource -or -not $ffprobeSource) {
    Write-Host "❌ Could not find FFmpeg binaries in downloaded archive" -ForegroundColor Red
    exit 1
}

# Copy to bin directory
Write-Host "📋 Copying binaries to bin directory..." -ForegroundColor Yellow
Copy-Item -Path $ffmpegSource.FullName -Destination $BIN_DIR -Force
Copy-Item -Path $ffprobeSource.FullName -Destination $BIN_DIR -Force

# Verify binaries
Write-Host "✅ Verifying binaries..." -ForegroundColor Green
$newFfmpegVersion = & $ffmpegPath -version 2>&1 | Select-Object -First 1
Write-Host "   $newFfmpegVersion" -ForegroundColor Cyan

# Cleanup
Write-Host "🧹 Cleaning up temporary files..." -ForegroundColor Yellow
Remove-Item -Recurse -Force $TEMP_DIR

# Show size info
$ffmpegSize = (Get-Item $ffmpegPath).Length / 1MB
$ffprobeSize = (Get-Item $ffprobePath).Length / 1MB
Write-Host ""
Write-Host "📊 Binary Sizes:" -ForegroundColor Cyan
Write-Host "   ffmpeg.exe:  $($ffmpegSize.ToString('F2')) MB" -ForegroundColor Gray
Write-Host "   ffprobe.exe: $($ffprobeSize.ToString('F2')) MB" -ForegroundColor Gray
Write-Host "   Total:       $(($ffmpegSize + $ffprobeSize).ToString('F2')) MB" -ForegroundColor Gray

Write-Host ""
Write-Host "✅ FFmpeg preparation complete!" -ForegroundColor Green
Write-Host "   Binaries are ready for bundling in: $BIN_DIR" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Run: cargo tauri build" -ForegroundColor Gray
Write-Host "  2. Find installers in: src-tauri\target\release\bundle\" -ForegroundColor Gray
