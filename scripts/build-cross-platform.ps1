# LoLShorts Cross-Platform Build Script for Windows
# Automated build script for CI/CD and local development

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("debug", "release")]
    [string]$BuildType = "release",

    [Parameter(Mandatory=$false)]
    [ValidateSet("windows", "macos", "linux", "all")]
    [string]$Platform = "windows",

    [Parameter(Mandatory=$false)]
    [switch]$SkipTests,

    [Parameter(Mandatory=$false)]
    [switch]$SkipSigning,

    [Parameter(Mandatory=$false)]
    [switch]$Clean
)

# Import helper functions
. "$PSScriptRoot\build-helpers.ps1"

# Configuration
$ErrorActionPreference = "Stop"

Write-Host "🚀 LoLShorts Cross-Platform Build Script" -ForegroundColor Blue
Write-Host "Build Type: $BuildType"
Write-Host "Platform: $Platform"
Write-Host ""

# Initialize build environment
Initialize-BuildEnvironment

# Clean build if requested
if ($Clean) {
    Write-Host "🧹 Cleaning build artifacts..." -ForegroundColor Yellow
    Clean-BuildArtifacts
}

# Install dependencies
Write-Host "📦 Installing dependencies..." -ForegroundColor Yellow
Install-Dependencies

# Prepare FFmpeg binaries
Write-Host "🎬 Preparing FFmpeg binaries..." -ForegroundColor Yellow
Prepare-FFmpeg

# Run tests unless skipped
if (-not $SkipTests) {
    Write-Host "🧪 Running tests..." -ForegroundColor Yellow
    Run-Tests
}

# Build for specified platforms
switch ($Platform) {
    "windows" {
        Build-Windows -BuildType $BuildType -SkipSigning:$SkipSigning
    }
    "macos" {
        Build-macOS -BuildType $BuildType -SkipSigning:$SkipSigning
    }
    "linux" {
        Build-Linux -BuildType $BuildType
    }
    "all" {
        Build-Windows -BuildType $BuildType -SkipSigning:$SkipSigning
        Build-macOS -BuildType $BuildType -SkipSigning:$SkipSigning
        Build-Linux -BuildType $BuildType
    }
}

# Generate build report
Generate-BuildReport

Write-Host "🎉 Build completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Build artifacts:"
Write-Host "  Windows: $(Get-Location)\src-tauri\target\$($BuildType)\bundle\"
Write-Host "  macOS: $(Get-Location)\src-tauri\target\$($BuildType)\bundle\macos\"
Write-Host "  Linux: $(Get-Location)\src-tauri\target\$($BuildType)\bundle\"
Write-Host ""