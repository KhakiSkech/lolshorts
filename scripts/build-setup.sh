#!/bin/bash

# LoLShorts Cross-Platform Build Setup Script
# Sets up build environment for Windows, macOS, and Linux

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect platform
PLATFORM=$(uname -s)
ARCH=$(uname -m)

echo -e "${BLUE}🚀 LoLShorts Build Setup${NC}"
echo "Platform: $PLATFORM"
echo "Architecture: $ARCH"
echo ""

# Function to print status
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Platform-specific setup
setup_windows() {
    print_status "Setting up Windows build environment..."

    # Check for Chocolatey
    if ! command -v choco &> /dev/null; then
        print_warning "Chocolatey not found. Installing..."
        powershell -Command "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))"
    fi

    # Install required tools
    choco install -y nodejs git wixtoolset visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

    # Setup Rust
    if ! command -v cargo &> /dev/null; then
        print_warning "Rust not found. Installing..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Add Rust targets
    rustup target add x86_64-pc-windows-msvc

    print_status "Windows build environment setup complete!"
}

setup_macos() {
    print_status "Setting up macOS build environment..."

    # Check for Homebrew
    if ! command -v brew &> /dev/null; then
        print_warning "Homebrew not found. Installing..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi

    # Install required tools
    brew install node git rust create-dmg

    # Install Xcode command line tools
    xcode-select --install || print_warning "Xcode command line tools already installed"

    # Add Rust targets
    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin

    print_status "macOS build environment setup complete!"
}

setup_linux() {
    print_status "Setting up Linux build environment..."

    # Detect distribution
    if [ -f /etc/debian_version ]; then
        # Debian/Ubuntu
        sudo apt-get update
        sudo apt-get install -y \
            build-essential \
            curl \
            git \
            nodejs \
            npm \
            pkg-config \
            libgtk-3-dev \
            libwebkit2gtk-4.0-dev \
            libappindicator3-dev \
            librsvg2-dev \
            patchelf \
            libssl-dev
    elif [ -f /etc/fedora-release ]; then
        # Fedora
        sudo dnf install -y \
            gcc \
            gcc-c++ \
            curl \
            git \
            nodejs \
            npm \
            pkgconfig \
            gtk3-devel \
            webkit2gtk3-devel \
            librsvg2-devel \
            openssl-devel
    else
        print_error "Unsupported Linux distribution"
        exit 1
    fi

    # Install Rust
    if ! command -v cargo &> /dev/null; then
        print_warning "Rust not found. Installing..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    # Add Rust targets
    rustup target add x86_64-unknown-linux-gnu
    rustup target add x86_64-pc-windows-gnu

    print_status "Linux build environment setup complete!"
}

# Setup FFmpeg binaries
setup_ffmpeg() {
    print_status "Setting up FFmpeg binaries..."

    case $PLATFORM in
        CYGWIN*|MINGW*|MSYS*)
            # Windows
            if [ ! -f "src-tauri/binaries/ffmpeg.exe" ]; then
                print_status "Downloading FFmpeg for Windows..."
                mkdir -p src-tauri/binaries
                curl -L "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip" -o ffmpeg.zip
                unzip ffmpeg.zip -d temp_ffmpeg
                cp temp_ffmpeg/ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe src-tauri/binaries/
                cp temp_ffmpeg/ffmpeg-master-latest-win64-gpl/bin/ffprobe.exe src-tauri/binaries/
                rm -rf temp_ffmpeg ffmpeg.zip
            fi
            ;;
        Darwin*)
            # macOS
            if [ ! -f "src-tauri/binaries/ffmpeg" ]; then
                print_status "Downloading FFmpeg for macOS..."
                mkdir -p src-tauri/binaries
                curl -L "https://evermeet.cx/ffmpeg/ffmpeg-6.1.zip" -o ffmpeg.zip
                unzip ffmpeg.zip -d src-tauri/binaries/
                chmod +x src-tauri/binaries/ffmpeg
                rm ffmpeg.zip

                # Also download ffprobe
                curl -L "https://evermeet.cx/ffmpeg/ffprobe-6.1.zip" -o ffprobe.zip
                unzip ffprobe.zip -d src-tauri/binaries/
                chmod +x src-tauri/binaries/ffprobe
                rm ffprobe.zip
            fi
            ;;
        Linux*)
            # Linux
            if [ ! -f "src-tauri/binaries/ffmpeg" ]; then
                print_status "Downloading FFmpeg for Linux..."
                mkdir -p src-tauri/binaries
                curl -L "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz" -o ffmpeg.tar.xz
                tar -xf ffmpeg.tar.xz
                cp ffmpeg-master-latest-linux64-gpl/bin/ffmpeg src-tauri/binaries/
                cp ffmpeg-master-latest-linux64-gpl/bin/ffprobe src-tauri/binaries/
                chmod +x src-tauri/binaries/ffmpeg src-tauri/binaries/ffprobe
                rm -rf ffmpeg-master-latest-linux64-gpl ffmpeg.tar.xz
            fi
            ;;
    esac

    print_status "FFmpeg setup complete!"
}

# Setup Node.js dependencies
setup_node_deps() {
    print_status "Installing Node.js dependencies..."

    # Install dependencies
    npm ci

    # Install development dependencies
    npm install -g @playwright/test

    # Install Playwright browsers
    npx playwright install --with-deps

    print_status "Node.js dependencies installed!"
}

# Verify setup
verify_setup() {
    print_status "Verifying build setup..."

    # Check Rust
    if command -v cargo &> /dev/null; then
        RUST_VERSION=$(rustc --version)
        print_status "Rust: $RUST_VERSION"
    else
        print_error "Rust not found"
        return 1
    fi

    # Check Node.js
    if command -v node &> /dev/null; then
        NODE_VERSION=$(node --version)
        print_status "Node.js: $NODE_VERSION"
    else
        print_error "Node.js not found"
        return 1
    fi

    # Check npm
    if command -v npm &> /dev/null; then
        NPM_VERSION=$(npm --version)
        print_status "npm: $NPM_VERSION"
    else
        print_error "npm not found"
        return 1
    fi

    # Check FFmpeg
    if [ -f "src-tauri/binaries/ffmpeg" ] || [ -f "src-tauri/binaries/ffmpeg.exe" ]; then
        print_status "FFmpeg: Found"
    else
        print_error "FFmpeg not found"
        return 1
    fi

    # Check if we can build
    print_status "Testing build configuration..."
    cargo check --manifest-path src-tauri/Cargo.toml

    print_status "Build setup verification complete!"
}

# Main setup logic
main() {
    case $PLATFORM in
        CYGWIN*|MINGW*|MSYS*)
            setup_windows
            ;;
        Darwin*)
            setup_macos
            ;;
        Linux*)
            setup_linux
            ;;
        *)
            print_error "Unsupported platform: $PLATFORM"
            exit 1
            ;;
    esac

    setup_ffmpeg
    setup_node_deps
    verify_setup

    print_status "🎉 LoLShorts build environment is ready!"
    echo ""
    echo "Next steps:"
    echo "  • Run tests: npm test"
    echo "  • Start development: npm run tauri:dev"
    echo "  • Build for production: npm run tauri:build"
}

# Run main function
main "$@"