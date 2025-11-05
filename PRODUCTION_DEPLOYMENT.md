# LoLShorts Production Deployment Guide

**Version**: 1.0
**Last Updated**: 2025-01-13
**Target Audience**: DevOps, Release Managers, Developers

---

## 📋 Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [Environment Setup](#environment-setup)
3. [Build Process](#build-process)
4. [Testing Requirements](#testing-requirements)
5. [Release Process](#release-process)
6. [Post-Deployment Validation](#post-deployment-validation)
7. [Rollback Procedures](#rollback-procedures)
8. [Monitoring and Alerts](#monitoring-and-alerts)
9. [Troubleshooting](#troubleshooting)

---

## 🔍 Pre-Deployment Checklist

### Code Quality
- [ ] All tests passing (unit, integration, E2E)
- [ ] No Clippy warnings (`cargo clippy`)
- [ ] No ESLint errors (`npm run lint`)
- [ ] Code formatted (`cargo fmt`, `npm run format`)
- [ ] Security audit clean (`cargo audit`, `npm audit`)

### Documentation
- [ ] CHANGELOG.md updated with release notes
- [ ] README.md version updated
- [ ] API documentation current
- [ ] User guides updated for new features
- [ ] Legal documents reviewed (TOS, Privacy Policy, Riot Compliance)

### Versioning
- [ ] Version bumped in `Cargo.toml`
- [ ] Version bumped in `package.json`
- [ ] Version bumped in `src-tauri/tauri.conf.json`
- [ ] Git tag created (`v*.*.*`)

### Infrastructure
- [ ] Supabase production environment ready
- [ ] Database migrations tested
- [ ] CDN configured for update delivery
- [ ] Backup systems verified

### Legal & Compliance
- [ ] Terms of Service up to date
- [ ] Privacy Policy GDPR/CCPA compliant
- [ ] Riot Games compliance verified
- [ ] License files correct (MIT + GPL notice)

---

## 🛠️ Environment Setup

### Required Tools

**Development Machine**:
- Windows 10+ (64-bit)
- Rust 1.70+ (`rustup`)
- Node.js 18+ LTS
- npm or pnpm
- Visual Studio Build Tools 2019+
- WiX Toolset 3.14+

**CI/CD Environment**:
- GitHub Actions (configured in `.github/workflows/`)
- Access to GitHub Secrets:
  - `TAURI_PRIVATE_KEY`: For update signing
  - `TAURI_KEY_PASSWORD`: Private key password
  - `SUPABASE_URL`: Production Supabase URL
  - `SUPABASE_ANON_KEY`: Production Supabase anonymous key

### Environment Variables

**Production**:
```bash
# Supabase
VITE_SUPABASE_URL=https://your-project.supabase.co
VITE_SUPABASE_ANON_KEY=your-anon-key

# Tauri (for auto-updates)
TAURI_PRIVATE_KEY=path/to/private.key
TAURI_KEY_PASSWORD=your-password

# Build Configuration
TAURI_BUILD_TARGET=release
TAURI_BUNDLE_IDENTIFIER=com.lolshorts.app
```

**Staging**:
```bash
# Use staging Supabase project
VITE_SUPABASE_URL=https://your-staging-project.supabase.co
VITE_SUPABASE_ANON_KEY=your-staging-anon-key
```

---

## 🏗️ Build Process

### 1. Clean Build Environment

```powershell
# Clear Rust build cache
cargo clean

# Clear Node modules
Remove-Item node_modules -Recurse -Force
Remove-Item package-lock.json

# Fresh install
npm install
```

### 2. Prepare FFmpeg Binaries

```powershell
cd src-tauri\build_scripts
.\prepare_ffmpeg.ps1

# Verify FFmpeg binaries
Test-Path ..\bin\ffmpeg.exe   # Should be True
Test-Path ..\bin\ffprobe.exe  # Should be True
```

### 3. Build Application

```powershell
# Production build
npm run tauri build

# Expected output:
# - src-tauri/target/release/bundle/msi/*.msi (~150-200MB)
# - src-tauri/target/release/bundle/nsis/*-setup.exe (~150-200MB)
```

### 4. Verify Build Artifacts

```powershell
# Run installer validation
.\tests\installer\validate-installer.ps1 -InstallerType Both

# Check installer sizes
Get-ChildItem src-tauri\target\release\bundle\msi\*.msi | Format-Table Name, @{Label="Size (MB)"; Expression={[math]::Round($_.Length/1MB, 2)}}
Get-ChildItem src-tauri\target\release\bundle\nsis\*-setup.exe | Format-Table Name, @{Label="Size (MB)"; Expression={[math]::Round($_.Length/1MB, 2)}}
```

---

## 🧪 Testing Requirements

### Pre-Release Testing Matrix

| Test Type | Environment | Duration | Required Pass Rate |
|-----------|-------------|----------|-------------------|
| Unit Tests | CI | 5-10 min | 100% |
| Integration Tests | CI | 10-15 min | 100% |
| E2E Tests | CI | 15-20 min | 95%+ |
| Security Audit | CI | 5 min | 0 critical |
| Installer Tests | Windows 10 | 10 min | 100% |
| Installer Tests | Windows 11 | 10 min | 100% |
| Manual QA | Multiple PCs | 2-4 hours | No blockers |

### Manual QA Checklist

**Installation**:
- [ ] MSI installer installs cleanly on Windows 10
- [ ] MSI installer installs cleanly on Windows 11
- [ ] NSIS installer works on both OS versions
- [ ] Uninstaller removes all files
- [ ] Upgrade from previous version works

**Core Functionality**:
- [ ] LCU connection works
- [ ] Recording starts/stops correctly
- [ ] Replay buffer functions
- [ ] Clips are captured automatically
- [ ] Manual clip saving works
- [ ] Screenshot capture works

**Authentication**:
- [ ] Login works with valid credentials
- [ ] Signup creates new accounts
- [ ] Logout clears session
- [ ] Token refresh happens automatically
- [ ] Session persists across restarts

**PRO Features** (requires PRO account):
- [ ] Clip extraction works
- [ ] YouTube Shorts composition works
- [ ] Thumbnail generation works
- [ ] Advanced editing features accessible

**Performance**:
- [ ] App starts in <3 seconds
- [ ] LCU connection in <2 seconds
- [ ] No memory leaks during 1-hour session
- [ ] CPU usage <30% during recording
- [ ] Memory usage <500MB idle, <2GB recording

**UI/UX**:
- [ ] All pages load correctly
- [ ] Navigation works smoothly
- [ ] Responsive design functions
- [ ] Dark/light mode (if applicable)
- [ ] No console errors

---

## 🚀 Release Process

### Automated Release (via GitHub Actions)

1. **Create Release Tag**:
```bash
git checkout main
git pull origin main

# Create annotated tag
git tag -a v0.1.0 -m "Release v0.1.0"

# Push tag to trigger release workflow
git push origin v0.1.0
```

2. **Monitor GitHub Actions**:
- Go to: https://github.com/LoLShorts/lolshorts/actions
- Watch "Release" workflow
- Verify all jobs complete successfully

3. **Verify Release Assets**:
- Go to: https://github.com/LoLShorts/lolshorts/releases
- Check release includes:
  - MSI installer
  - NSIS installer
  - checksums.txt
  - Release notes

### Manual Release (if needed)

1. **Build Locally**:
```powershell
npm run tauri build
```

2. **Test Installers**:
```powershell
.\tests\installer\validate-installer.ps1
```

3. **Create GitHub Release Manually**:
- Go to: https://github.com/LoLShorts/lolshorts/releases/new
- Tag: `v0.1.0`
- Title: `LoLShorts 0.1.0`
- Upload installers and checksums
- Publish release

4. **Generate Checksums**:
```powershell
Get-FileHash .\src-tauri\target\release\bundle\msi\*.msi -Algorithm SHA256
Get-FileHash .\src-tauri\target\release\bundle\nsis\*-setup.exe -Algorithm SHA256
```

---

## ✅ Post-Deployment Validation

### Immediate Checks (within 1 hour)

- [ ] Download installers from GitHub Release
- [ ] Verify SHA256 checksums match
- [ ] Install on clean Windows 10 machine
- [ ] Install on clean Windows 11 machine
- [ ] Run full QA smoke tests
- [ ] Check auto-update manifest is accessible
- [ ] Monitor error reporting (if configured)

### 24-Hour Checks

- [ ] Review crash reports
- [ ] Check Supabase usage metrics
- [ ] Monitor user feedback channels
- [ ] Verify no critical bugs reported
- [ ] Check download statistics

### 7-Day Checks

- [ ] Analyze usage patterns
- [ ] Review performance metrics
- [ ] Check for memory leaks in long sessions
- [ ] Gather user feedback
- [ ] Plan hotfix if needed

---

## ⏪ Rollback Procedures

### Scenario 1: Critical Bug Discovered

1. **Immediate Actions**:
   - Update release notes with warning
   - Consider unpublishing release (if very critical)
   - Notify users via Discord/social media

2. **Quick Hotfix**:
```bash
# Create hotfix branch
git checkout -b hotfix/v0.1.1 v0.1.0

# Fix critical bug
# ... make changes ...

# Test fix
cargo test
npm run test

# Create hotfix release
git tag -a v0.1.1 -m "Hotfix: Critical bug fix"
git push origin v0.1.1
```

3. **Notify Users**:
   - Auto-update will deliver fix
   - Manual download link for urgent cases

### Scenario 2: Major Regression

1. **Revert to Previous Version**:
   - Users can download previous release
   - Update manifest to point to stable version
   - Communicate issue transparently

2. **Investigation**:
   - Reproduce issue in test environment
   - Create detailed bug report
   - Fix in develop branch
   - Thorough testing before next release

---

## 📊 Monitoring and Alerts

### Key Metrics to Monitor

**Application Health**:
- Crash rate (<1% acceptable)
- Startup time (<3 seconds target)
- Memory usage (<500MB idle)
- CPU usage (<30% during recording)

**User Engagement**:
- Daily active users (DAU)
- Recording sessions per day
- Clips captured per session
- PRO feature usage rate

**Technical Metrics**:
- Supabase API response time
- Database query performance
- FFmpeg processing time
- LCU connection success rate

### Alerting Thresholds

| Metric | Warning | Critical |
|--------|---------|----------|
| Crash Rate | >1% | >5% |
| API Error Rate | >2% | >10% |
| Startup Time | >5s | >10s |
| Memory Usage | >1GB | >2GB |

### Monitoring Tools

**Recommended Stack**:
- **Crash Reporting**: Sentry or Rollbar
- **Analytics**: Amplitude or Mixpanel (privacy-respecting)
- **APM**: DataDog or New Relic
- **Logging**: Logtail or Papertrail
- **Uptime**: Pingdom or StatusCake

---

## 🔧 Troubleshooting

### Common Deployment Issues

#### Issue: Build Fails with FFmpeg Missing

**Symptom**: Build error "FFmpeg binaries not found"

**Solution**:
```powershell
cd src-tauri\build_scripts
.\prepare_ffmpeg.ps1
```

#### Issue: WiX Toolset Not Found

**Symptom**: MSI build fails

**Solution**:
```powershell
choco install wixtoolset -y
$env:PATH += ";C:\Program Files (x86)\WiX Toolset v3.14\bin"
```

#### Issue: Installer Size Too Small

**Symptom**: Installer <100MB (missing FFmpeg)

**Solution**:
- Verify FFmpeg binaries in `src-tauri/bin/`
- Check `tauri.conf.json` includes `externalBin`
- Re-run build

#### Issue: Auto-Update Not Working

**Symptom**: Users not receiving updates

**Solution**:
1. Verify `updater-manifest.json` is accessible
2. Check `TAURI_PRIVATE_KEY` signature
3. Verify CDN/update server is reachable
4. Check Tauri updater configuration

#### Issue: Supabase Connection Fails

**Symptom**: "Authentication failed" errors

**Solution**:
1. Verify `VITE_SUPABASE_URL` is correct
2. Check `VITE_SUPABASE_ANON_KEY` is valid
3. Ensure Supabase project is not paused
4. Check RLS policies allow access

---

## 📞 Support Contacts

**Technical Issues**:
- Email: dev@lolshorts.com
- Discord: #dev-support
- GitHub Issues: https://github.com/LoLShorts/lolshorts/issues

**Release Management**:
- Release Manager: [Name]
- DevOps Lead: [Name]
- Emergency Contact: [Phone]

**Third-Party Services**:
- Supabase Support: https://supabase.com/support
- Riot Games Developer Support: https://developer.riotgames.com/

---

## 📝 Deployment History

| Version | Date | Type | Notes |
|---------|------|------|-------|
| v0.1.0 | 2025-01-13 | Initial | First production release |
| v0.1.1 | TBD | Hotfix | (Planned if needed) |

---

## ✅ Sign-Off

Before deploying to production, the following stakeholders must approve:

- [ ] **Technical Lead**: Code review complete, quality standards met
- [ ] **QA Lead**: All tests passing, manual QA complete
- [ ] **Product Manager**: Features complete, user stories satisfied
- [ ] **Legal/Compliance**: Legal documents reviewed, Riot compliance verified
- [ ] **DevOps/Release Manager**: Infrastructure ready, rollback plan in place

---

**Last Reviewed**: 2025-01-13
**Next Review**: 2025-02-13 (or before next major release)
