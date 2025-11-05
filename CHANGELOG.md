# Changelog

All notable changes to LoLShorts will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-11-05

### 🎉 Initial Production Release

First stable production release of LoLShorts - automatic League of Legends gameplay recording and editing application.

### Added

#### Core Features
- **Automatic Recording System**
  - LCU API integration for League client monitoring
  - Real-time game event detection via Live Client Data API
  - Hardware-accelerated video encoding (H.265 with NVENC/QSV/AMF)
  - Circular replay buffer (configurable 30s - 5min)
  - Automatic highlight detection with priority scoring

- **Event Detection**
  - Pentakills (Priority 5)
  - Quadrakills (Priority 4)
  - Baron Steals (Priority 4)
  - Dragon Steals (Priority 3)
  - Triple Kills (Priority 3)
  - Multi-Kills (Priority 2)
  - Single Kills (Priority 1)

- **Clip Management**
  - Comprehensive clip library with metadata
  - Smart filtering and sorting
  - Priority-based organization
  - Thumbnail generation
  - Storage usage monitoring
  - Automatic cleanup system

- **Video Editor**
  - Timeline-based editing interface
  - Drag-and-drop clip arrangement
  - Trim and cut functionality
  - Transition effects
  - Text overlays and annotations
  - YouTube Shorts export (9:16 format, max 60s)

- **Settings System**
  - Quality presets (Low, Medium, High, Ultra)
  - Resolution selection (720p - 2160p)
  - Frame rate options (30/60 FPS)
  - Bitrate control (1-20 Mbps)
  - Audio configuration (game audio + microphone)
  - Hotkey customization
  - Clip padding settings
  - Auto-save options

- **User Interface**
  - Dark theme inspired by League of Legends
  - Real-time status dashboard
  - LCU connection monitoring
  - Recording controls with hotkeys
  - Responsive desktop layout
  - Toast notifications

#### Backend (Rust + Tauri)
- LCU client with authentication
- Live client monitor for real-time events
- Game DVR controller with replay buffer
- Event detector with priority calculation
- Clip manager with metadata tracking
- Storage manager with file operations
- Settings persistence system
- Authentication and license validation
- Video processor with FFmpeg integration
- Thumbnail generator
- Error handling and logging system

#### Frontend (React + TypeScript)
- Dashboard page with status monitoring
- Clip Library with real backend integration
- Video Editor with timeline interface
- Settings page with comprehensive options
- Recording controls component
- Status display components
- shadcn/ui component library integration
- Zustand state management
- Toast notification system

#### Build & Deployment
- Production-optimized Rust build (LTO, stripped binaries)
- Frontend bundle optimization (536 KB, gzipped 161 KB)
- NSIS installer for Windows (3.8 MB)
- MSI installer for enterprise deployment (5.5 MB)
- GitHub Actions CI/CD workflow
- Automated release process

#### Documentation
- Comprehensive README with installation guide
- Production readiness documentation
- Production checklist
- Build guide
- Deployment guide
- Privacy policy
- Terms of service
- Riot Games compliance documentation
- Development guidelines (CLAUDE.md)

### Technical Details

**Build Information**:
- Build Date: 2025-11-05
- Build Time: 39.23 seconds
- Rust: 0 errors, 46 non-critical warnings
- TypeScript: Strict mode compilation success
- Binary Size: Optimized with LTO and stripping
- Bundle Size: 536.72 KB (161.04 KB gzipped)

**Platform Support**:
- Windows 10+ (x64)
- macOS support: Planned for v1.1.0
- Linux support: Planned for v1.1.0

**Dependencies**:
- Tauri 2.0
- React 18
- TypeScript 5.7
- FFmpeg (bundled)
- Rust 1.83+ (stable)

### Performance

- App startup: < 3 seconds (cold start)
- LCU connection: < 2 seconds
- Event detection latency: < 500ms
- Video processing: < 30s per minute of footage
- Memory usage: < 500MB idle, < 2GB during processing

### Security

- Input validation on all Tauri commands
- Path traversal prevention
- Secure LCU communication (HTTPS)
- No secrets in logs
- Local-only processing (no data collection)

### Known Limitations

- Windows only (macOS Boot Camp support planned for v1.1.0)
- Requires external FFmpeg binary (bundled with installer)
- English only (multi-language support in v1.2.0)

### Build Artifacts

- `LoLShorts_1.0.0_x64-setup.exe` (3.8 MB) - NSIS installer
- `LoLShorts_1.0.0_x64_en-US.msi` (5.5 MB) - MSI installer

---

## [Unreleased]

### Planned for v1.1.0 (Q1 2025)
- macOS support (Boot Camp)
- Cloud storage integration
- Direct upload to YouTube/TikTok
- Multi-language support (Korean, Chinese, Japanese)

### Planned for v1.2.0 (Q2 2025)
- **Auto-Edit Workflow** - Complete automated Shorts generation system
  - Multi-game clip selection with intelligent priority algorithm
  - Canvas-style background editor (Canva-like) for overlays and text
  - Duration-based auto-composition (60/120/180 seconds)
  - Background music integration with volume mixing controls
  - Template save/load system for reusable designs
- AI-powered highlight detection improvements
- Advanced editing features (slow motion replays, dynamic zoom on kills, event-triggered animations, visual filters)
- Auto-caption generation

### Planned for v2.0.0 (Q3 2025)
- Support for other Riot Games (Valorant, Teamfight Tactics, Legends of Runeterra)
- Live streaming integration
- Mobile companion app

---

## Version History

- [1.0.0] - 2025-11-05 - Initial Production Release

---

[1.0.0]: https://github.com/KhakiSkech/lolshorts/releases/tag/v1.0.0
[Unreleased]: https://github.com/KhakiSkech/lolshorts/compare/v1.0.0...HEAD
