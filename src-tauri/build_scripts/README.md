# LoLShorts Build Scripts

Automated scripts for preparing the production build environment.

## Scripts Overview

### `verify_environment.ps1`
**Purpose**: Verify all build prerequisites are installed and properly configured

**What it checks**:
- ✅ Rust and Cargo installation
- ✅ Node.js (v18+) and npm
- ✅ WiX Toolset (for MSI installer)
- ✅ Visual Studio Build Tools
- ✅ Tauri CLI
- ✅ FFmpeg binaries presence
- ✅ Node dependencies (node_modules)

**Usage**:
```powershell
.\verify_environment.ps1
```

**Output**:
- Green ✅ - All requirements met
- Yellow ⚠️ - Warning (non-critical)
- Red ❌ - Critical issue (must fix)

---

### `prepare_ffmpeg.ps1`
**Purpose**: Download and prepare FFmpeg binaries for bundling with the installer

**What it does**:
1. Checks if FFmpeg already exists in `../bin/`
2. Downloads latest FFmpeg GPL build from GitHub
3. Extracts `ffmpeg.exe` and `ffprobe.exe`
4. Copies binaries to `../bin/` directory
5. Verifies binaries work correctly
6. Cleans up temporary files

**Usage**:
```powershell
.\prepare_ffmpeg.ps1
```

**Requirements**:
- Internet connection
- ~150 MB free disk space
- PowerShell 5.1+

**Output Location**:
- `../bin/ffmpeg.exe` (~75-80 MB)
- `../bin/ffprobe.exe` (~70-75 MB)

---

## Quick Start

### First-Time Setup

1. **Verify environment**:
```powershell
cd src-tauri\build_scripts
.\verify_environment.ps1
```

2. **Fix any critical issues** reported by verification script

3. **Prepare FFmpeg**:
```powershell
.\prepare_ffmpeg.ps1
```

4. **Build the app**:
```powershell
cd ..\..
cargo tauri build
```

### Subsequent Builds

If FFmpeg is already prepared:
```powershell
cargo tauri build
```

### Clean Build

```powershell
# Clean previous builds
cargo clean

# Re-verify environment
cd src-tauri\build_scripts
.\verify_environment.ps1

# Rebuild
cd ..\..
cargo tauri build
```

## Build Output

After successful build, installers are located in:
```
src-tauri/target/release/bundle/
├── nsis/
│   └── LoLShorts_0.1.0_x64-setup.exe
├── msi/
│   └── LoLShorts_0.1.0_x64_en-US.msi
└── LoLShorts.exe
```

## Troubleshooting

### "WiX toolset not found"
**Solution**: Download and install WiX Toolset v3.14+
- Download: https://wixtoolset.org/releases/
- Add to PATH: `C:\Program Files (x86)\WiX Toolset v3.14\bin`
- Restart PowerShell

### "FFmpeg download failed"
**Solution**:
- Check internet connection
- Try manual download: https://github.com/BtbN/FFmpeg-Builds/releases
- Place `ffmpeg.exe` and `ffprobe.exe` in `../bin/`

### "Node modules not found"
**Solution**:
```powershell
cd ../..
npm install
```

### "Rust compilation errors"
**Solution**:
```powershell
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

## Advanced Usage

### Build Specific Installer Type

```powershell
# MSI only
cargo tauri build --bundles msi

# NSIS only
cargo tauri build --bundles nsis

# Portable exe only
cargo tauri build --bundles app
```

### Development Build

```powershell
# Run in dev mode (no installer creation)
npm run tauri:dev
```

### Custom FFmpeg Source

If you need a specific FFmpeg version:

1. Download from https://ffmpeg.org/download.html
2. Extract `ffmpeg.exe` and `ffprobe.exe`
3. Copy to `../bin/`
4. Run `verify_environment.ps1` to confirm

## CI/CD Integration

These scripts can be used in GitHub Actions or other CI systems:

```yaml
- name: Verify Build Environment
  run: |
    cd src-tauri/build_scripts
    .\verify_environment.ps1

- name: Prepare FFmpeg
  run: |
    cd src-tauri/build_scripts
    .\prepare_ffmpeg.ps1

- name: Build Release
  run: cargo tauri build
```

## Additional Resources

- [BUILD_GUIDE.md](../../BUILD_GUIDE.md) - Complete build documentation
- [Tauri Documentation](https://tauri.app/v2/guides/)
- [WiX Toolset Documentation](https://wixtoolset.org/documentation/)
- [FFmpeg Documentation](https://ffmpeg.org/documentation.html)

## Support

For build issues:
- Check [BUILD_GUIDE.md](../../BUILD_GUIDE.md) for detailed troubleshooting
- Run `verify_environment.ps1` to diagnose environment issues
- Check GitHub Issues for known build problems
