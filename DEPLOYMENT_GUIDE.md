# LoLShorts Production Deployment Guide

**Version**: 1.0.0
**Target Platform**: Windows 10/11
**Last Updated**: 2025-01-05

---

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [FFmpeg Binary Setup](#ffmpeg-binary-setup)
3. [Code Signing Setup](#code-signing-setup)
4. [Building for Production](#building-for-production)
5. [Installer Configuration](#installer-configuration)
6. [Distribution](#distribution)
7. [Post-Release](#post-release)

---

## 🔧 Prerequisites

### Required Tools

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | 1.75+ | Backend compilation |
| Node.js | 18.x LTS | Frontend build |
| pnpm | 8.x | Package management |
| Tauri CLI | 2.0+ | App bundling |
| WiX Toolset | 3.11+ | MSI installer |
| NSIS | 3.08+ | NSIS installer |

### Installation Commands

```bash
# Install Rust
winget install Rustlang.Rust.MSVC

# Install Node.js LTS
winget install OpenJS.NodeJS.LTS

# Install pnpm
npm install -g pnpm

# Install Tauri CLI
cargo install tauri-cli --version "^2.0.0"

# Install WiX Toolset
winget install WiX.Toolset

# Install NSIS
winget install NSIS.NSIS
```

### Verify Installation

```bash
rustc --version   # Should be 1.75+
node --version    # Should be v18.x
pnpm --version    # Should be 8.x
cargo tauri --version  # Should be 2.x
```

---

## 🎬 FFmpeg Binary Setup

### Option 1: Download Pre-Built Binary (Recommended)

1. **Download FFmpeg**
   ```
   URL: https://github.com/BtbN/FFmpeg-Builds/releases
   File: ffmpeg-master-latest-win64-gpl.zip
   ```

2. **Extract Binary**
   ```bash
   # Extract ffmpeg.exe from the downloaded archive
   # Copy to project directory
   mkdir -p src-tauri/binaries
   cp ffmpeg.exe src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
   ```

3. **Verify Binary**
   ```bash
   .\src-tauri\binaries\ffmpeg-x86_64-pc-windows-msvc.exe -version
   ```

### Option 2: Build FFmpeg from Source

**Only if you need custom FFmpeg configuration**

```bash
# Clone FFmpeg
git clone https://git.ffmpeg.org/ffmpeg.git
cd ffmpeg

# Configure with required codecs
./configure --enable-gpl --enable-libx264 --enable-libx265 \
            --enable-nvenc --disable-shared --enable-static

# Build (takes 30-60 minutes)
make -j$(nproc)

# Copy binary
cp ffmpeg.exe ../LoLShorts/src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
```

### FFmpeg Binary Naming Convention

Tauri requires platform-specific naming:

| Platform | Binary Name |
|----------|-------------|
| Windows x64 | `ffmpeg-x86_64-pc-windows-msvc.exe` |
| Windows ARM | `ffmpeg-aarch64-pc-windows-msvc.exe` |

### tauri.conf.json Configuration

The project is already configured:

```json
{
  "bundle": {
    "externalBin": [
      "binaries/ffmpeg"
    ]
  }
}
```

Tauri automatically appends the platform suffix during build.

---

## 🔐 Code Signing Setup

### Windows Code Signing Certificate

**Production Requirements:**
- EV (Extended Validation) Code Signing Certificate
- Hardware Security Module (HSM) or USB token
- Annual cost: ~$300-500

### Obtaining a Certificate

1. **Purchase from CA**
   - DigiCert
   - Sectigo
   - GlobalSign

2. **Company Verification**
   - Requires legal business entity
   - Dun & Bradstreet (DUNS) number
   - 3-5 business days verification

3. **Certificate Delivery**
   - USB token shipped via courier
   - Install on signing machine

### Configure Signing in Tauri

**Option 1: Certificate Thumbprint (Recommended)**

```json
// tauri.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT_HERE",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

**Option 2: PFX File**

```json
{
  "bundle": {
    "windows": {
      "signCommand": "signtool sign /f path/to/cert.pfx /p PASSWORD /fd sha256 /tr http://timestamp.digicert.com /td sha256 %1"
    }
  }
}
```

### Get Certificate Thumbprint

```powershell
# List installed certificates
Get-ChildItem -Path Cert:\CurrentUser\My

# Get specific certificate thumbprint
(Get-ChildItem -Path Cert:\CurrentUser\My | Where-Object {$_.Subject -match "LoLShorts"}).Thumbprint
```

### Timestamping (CRITICAL)

**Always use timestamping:**
- Allows installers to remain valid after certificate expires
- Free service provided by CAs
- Required for production

Recommended timestamp servers:
- DigiCert: `http://timestamp.digicert.com`
- Sectigo: `http://timestamp.sectigo.com`
- GlobalSign: `http://timestamp.globalsign.com`

### Testing Without Certificate (Development Only)

```bash
# Build unsigned installer for testing
cargo tauri build --no-bundle

# Or skip signing with environment variable
$env:TAURI_SKIP_SIGNING="1"
cargo tauri build
```

**⚠️ WARNING**: Unsigned installers will show Windows SmartScreen warnings!

---

## 🏗️ Building for Production

### Pre-Build Checklist

- [ ] FFmpeg binary in `src-tauri/binaries/`
- [ ] Code signing certificate configured (or skip for testing)
- [ ] All tests passing: `cargo test && npm test`
- [ ] No compiler warnings: `cargo clippy`
- [ ] Frontend lint clean: `pnpm lint`
- [ ] Version bumped in `tauri.conf.json`, `Cargo.toml`, `package.json`
- [ ] CHANGELOG.md updated
- [ ] LICENSE file present

### Build Commands

```bash
# 1. Install dependencies
pnpm install
cd src-tauri && cargo fetch && cd ..

# 2. Run tests
cargo test --workspace
pnpm test

# 3. Build production bundle
cargo tauri build

# Output will be in: src-tauri/target/release/bundle/
```

### Build Artifacts

| File | Type | Size (approx) | Use Case |
|------|------|---------------|----------|
| `LoLShorts_1.0.0_x64_en-US.msi` | MSI Installer | 80 MB | Enterprise deployment |
| `LoLShorts_1.0.0_x64-setup.exe` | NSIS Installer | 75 MB | Consumer distribution |
| `LoLShorts.exe` | Portable | 70 MB | No-install version |

### Build Configuration

**Release Mode Optimizations** (`Cargo.toml`):

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

**Frontend Production Build** (`vite.config.ts`):

```typescript
export default defineConfig({
  build: {
    target: 'es2021',
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
        drop_debugger: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          ui: ['@radix-ui/react-dialog', '@radix-ui/react-dropdown-menu'],
        },
      },
    },
  },
});
```

---

## 📦 Installer Configuration

### MSI Installer (WiX)

**Features:**
- Enterprise-friendly
- Group Policy deployment
- Silent installation support
- Upgrade/downgrade control

**Custom Configuration**:

Create `src-tauri/wix/main.wxs`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="LoLShorts" Version="1.0.0"
           Manufacturer="LoLShorts Team" UpgradeCode="YOUR-GUID-HERE">

    <Package InstallerVersion="200" Compressed="yes" InstallScope="perUser" />

    <Feature Id="MainApplication" Title="LoLShorts" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>

    <!-- Desktop shortcut -->
    <Icon Id="icon.ico" SourceFile="icons/icon.ico"/>
    <Property Id="ARPPRODUCTICON" Value="icon.ico" />
  </Product>
</Wix>
```

### NSIS Installer

**Features:**
- Smaller file size
- Custom UI themes
- Language selection
- Per-user installation

**Advantages:**
- No admin privileges required
- Faster installation
- Better for consumer distribution

---

## 🌐 Distribution

### Distribution Checklist

- [ ] Code signed installer
- [ ] Virus scanned (VirusTotal)
- [ ] Tested on clean Windows 10/11
- [ ] SmartScreen reputation built (submit to Microsoft)
- [ ] Release notes prepared
- [ ] User documentation ready

### Hosting Options

| Platform | Type | Best For |
|----------|------|----------|
| GitHub Releases | Free | Open source |
| AWS S3 + CloudFront | Paid | High traffic |
| DigitalOcean Spaces | Paid | Cost-effective |
| BunnyCDN | Paid | Global CDN |

### GitHub Releases Workflow

```bash
# 1. Tag release
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# 2. Create GitHub release
gh release create v1.0.0 \
  src-tauri/target/release/bundle/msi/*.msi \
  src-tauri/target/release/bundle/nsis/*.exe \
  --title "LoLShorts v1.0.0" \
  --notes-file CHANGELOG.md

# 3. Attach installers
gh release upload v1.0.0 src-tauri/target/release/bundle/**/*.{msi,exe}
```

### Windows SmartScreen

**New applications will trigger SmartScreen warnings.**

**Solutions:**
1. **Microsoft SmartScreen Submission**
   - Submit signed installer to Microsoft
   - Build reputation over time (months)
   - Free but slow

2. **EV Code Signing Certificate**
   - Instant SmartScreen reputation
   - More expensive ($300-500/year)
   - Recommended for professional releases

3. **Accept the Warning Period**
   - Users click "More info" → "Run anyway"
   - Reputation builds over downloads
   - Free but user friction

---

## 🔄 Auto-Update Configuration

### Tauri Updater Setup

```json
// tauri.conf.json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.lolshorts.com/{{target}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

### Generate Update Keys

```bash
# Generate update signing keys
tauri signer generate -w ~/.tauri/lolshorts.key

# Add public key to tauri.conf.json
# Keep private key secure (use in CI/CD)
```

### Update Endpoint Response

```json
{
  "version": "1.0.1",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2025-01-10T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "BASE64_SIGNATURE",
      "url": "https://releases.lolshorts.com/LoLShorts_1.0.1_x64-setup.exe"
    }
  }
}
```

---

## 🧪 Post-Release

### Monitoring

1. **Crash Reports**
   - Implement Sentry or similar
   - Monitor error rates
   - Quick hotfix deployment

2. **Performance Metrics**
   - Backend telemetry (opt-in)
   - Usage analytics
   - Feature adoption

3. **User Feedback**
   - In-app feedback form
   - GitHub Issues
   - Discord community

### Hotfix Process

```bash
# 1. Fix critical bug
git checkout -b hotfix/1.0.1

# 2. Update version
# - Cargo.toml: 1.0.0 → 1.0.1
# - package.json: 1.0.0 → 1.0.1
# - tauri.conf.json: 1.0.0 → 1.0.1

# 3. Build and test
cargo tauri build
# Test installer

# 4. Release
git tag -a v1.0.1 -m "Hotfix: Critical crash fix"
gh release create v1.0.1 ...
```

---

## 📊 Release Checklist

**Pre-Release:**
- [ ] Version bumped in all files
- [ ] CHANGELOG.md updated
- [ ] All tests passing
- [ ] FFmpeg binary bundled
- [ ] Code signed
- [ ] Tested on clean Windows

**Release:**
- [ ] Git tag created
- [ ] GitHub release published
- [ ] Installers uploaded
- [ ] Update endpoint configured
- [ ] Documentation updated

**Post-Release:**
- [ ] Monitoring active
- [ ] User feedback tracked
- [ ] Performance metrics reviewed
- [ ] Next version planned

---

## 🆘 Troubleshooting

### Common Build Issues

**FFmpeg Not Found**
```
Error: External binary 'binaries/ffmpeg' not found
```
**Solution**: Ensure `ffmpeg-x86_64-pc-windows-msvc.exe` exists in `src-tauri/binaries/`

**Signing Failed**
```
Error: Code signing failed
```
**Solution**: Check certificate thumbprint, or set `TAURI_SKIP_SIGNING=1` for testing

**NSIS Build Error**
```
Error: NSIS not found in PATH
```
**Solution**: Install NSIS and add to PATH, or use MSI only: `--bundles msi`

### Performance Issues

**Large Installer Size**
- Compress assets
- Use `strip = true` in Cargo.toml
- Optimize frontend bundle

**Slow Startup**
- Enable lazy loading
- Reduce initial dependencies
- Use release build for testing

---

## 📞 Support

- **Documentation**: https://lolshorts.com/docs
- **GitHub Issues**: https://github.com/lolshorts/lolshorts/issues
- **Discord**: https://discord.gg/lolshorts
- **Email**: support@lolshorts.com

---

**Last Updated**: 2025-01-05
**Document Version**: 1.0.0
