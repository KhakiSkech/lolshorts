# LoLShorts Development Plan

**Last Updated**: 2025-11-06
**Status**: Planning Complete - Ready for Implementation

---

## 📅 Development Timeline

```
┌─────────────────────────────────────────────────────────────────┐
│                         2025 Development Roadmap                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Q1 (Jan-Mar)          Q2 (Apr-Jun)          Q3 (Jul-Sep)      │
│  ────────────          ────────────          ────────────      │
│   v1.1.0                v1.2.0 🎯             v1.3.0           │
│  Maintenance          PRIORITY              macOS + TikTok      │
│                                                                 │
│  • Cloud storage      • Auto-Edit           • macOS support    │
│  • Multi-language     • YouTube Upload      • TikTok upload    │
│  • Performance        • AI metadata         • Multi-account    │
│  • Bug fixes          • Canvas editor       • Scheduling       │
│                       • Music mixing                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎯 Version 1.2.0: Complete Creation Suite (PRIORITY)

**Target Release**: Q2 2025 (April-June)
**Development Duration**: 15 weeks
**Status**: Specification Complete - Ready to Start

### Phase A: Auto-Edit Workflow (8 weeks)
📋 **Specification**: `AUTO_EDIT_SPEC.md`

#### Week 1-2: Backend Foundation
- [ ] Create `AutoComposer` struct in `src-tauri/src/video/auto_composer.rs`
- [ ] Implement clip selection algorithm (`auto_select_clips`)
- [ ] Implement FFmpeg template rendering functions
- [ ] Add Tauri commands: `auto_select_clips`, `start_auto_edit`, `get_auto_edit_progress`
- [ ] Write unit tests for composition logic

**Deliverable**: Backend can select and concatenate clips

#### Week 3-4: Canvas Editor
- [ ] Create `CanvasEditor` React component with drag-and-drop
- [ ] Implement background layer selection (color/gradient/image)
- [ ] Implement text element creation and editing
- [ ] Implement image/logo upload and positioning
- [ ] Add template save/load functionality
- [ ] Backend: Implement `save_canvas_template`, `load_canvas_template`
- [ ] Backend: Implement FFmpeg overlay filter generation

**Deliverable**: Users can create Canva-style overlays

#### Week 5: Audio Mixing
- [ ] Create `AudioMixer` React component
- [ ] Implement MP3 file upload
- [ ] Add volume sliders for game audio + music
- [ ] Backend: Implement audio mixing with FFmpeg
- [ ] Add audio fade-in/fade-out (3s)
- [ ] Implement audio preview functionality

**Deliverable**: Background music integration works

#### Week 6: UI Integration
- [ ] Create `AutoEditPanel` main component
- [ ] Add multi-game selection UI
- [ ] Integrate clip auto-selection display
- [ ] Add duration selector (60/120/180s)
- [ ] Create progress bar with status messages
- [ ] Add "Start Auto Edit" button with validation
- [ ] Implement error handling + user feedback

**Deliverable**: Complete auto-edit workflow functional

#### Week 7-8: Testing & Optimization
- [ ] E2E testing of complete auto-edit workflow
- [ ] Performance optimization (FFmpeg settings)
- [ ] Memory leak testing (long video processing)
- [ ] Error recovery testing (FFmpeg failures)
- [ ] UI/UX refinement based on testing
- [ ] Documentation and user guide

**Deliverable**: Production-ready auto-edit feature

### Phase B: YouTube Upload (7 weeks)
📋 **Specification**: `YOUTUBE_UPLOAD_SPEC.md`

#### Week 9: OAuth Authentication
- [ ] Setup Google Cloud Console project
- [ ] Obtain OAuth2 client credentials
- [ ] Implement OAuth flow (authorization URL generation)
- [ ] Create callback server (localhost:9090)
- [ ] Implement token exchange
- [ ] Implement Windows Credential Manager storage
- [ ] Create YouTubeConnect component
- [ ] Add connection status to Settings page

**Deliverable**: Users can connect YouTube accounts

#### Week 10-11: Video Upload Core
- [ ] Integrate google-youtube3 Rust crate
- [ ] Implement YouTubeClient with upload method
- [ ] Implement resumable upload with chunking
- [ ] Add progress tracking system
- [ ] Create UploadModal component
- [ ] Create UploadProgress component
- [ ] Add "Upload to YouTube" button to Editor

**Deliverable**: Videos upload to YouTube with progress

#### Week 12: Metadata Generation
- [ ] Implement title generation algorithm
- [ ] Implement description generation
- [ ] Implement tag generation
- [ ] Create MetadataEditor component
- [ ] Add AI-powered suggestions (optional)
- [ ] Implement metadata preview

**Deliverable**: AI-powered metadata generation works

#### Week 13: Thumbnail Management
- [ ] Auto-extract thumbnail from video (middle frame)
- [ ] Implement custom thumbnail upload
- [ ] Create ThumbnailSelector component
- [ ] Add thumbnail preview
- [ ] Implement thumbnail upload to YouTube

**Deliverable**: Thumbnail management functional

#### Week 14: Upload History
- [ ] Create upload history database table
- [ ] Implement UploadHistory component
- [ ] Fetch video stats from YouTube API (views, likes)
- [ ] Add "Open on YouTube" links
- [ ] Implement upload retry functionality

**Deliverable**: Upload history and management complete

#### Week 15: Testing & Polish
- [ ] E2E testing of complete upload flow
- [ ] Error handling for network failures
- [ ] Error handling for quota limits
- [ ] Token refresh testing
- [ ] Performance optimization
- [ ] UI/UX polish

**Deliverable**: Production-ready YouTube upload

---

## 🖥️ Version 1.3.0: macOS Support (Post v1.2.0)

**Target Release**: Q3 2025 (July-September)
**Prerequisites**: v1.2.0 complete (Auto-Edit + YouTube)

### Technical Approach
```
┌──────────────────────────────────────────────┐
│  macOS (Boot Camp) - Windows Environment    │
├──────────────────────────────────────────────┤
│                                              │
│  Boot Camp Partition                         │
│  ├─ Windows 10/11                           │
│  ├─ League of Legends (Windows)             │
│  └─ LoLShorts.app                           │
│                                              │
│  Requirements:                               │
│  • Intel Mac (required for Boot Camp)       │
│  • Windows license                           │
│  • 100GB+ partition for Windows + LoL       │
│                                              │
│  Limitations:                                │
│  ✗ M1/M2 Macs NOT supported (no Boot Camp)  │
│  ✗ Apple Silicon incompatible                │
│  ✓ Intel Macs fully supported               │
└──────────────────────────────────────────────┘
```

### Implementation Plan (6 weeks)

#### Week 1-2: Tauri Configuration
- [ ] Update `tauri.conf.json` for macOS target
- [ ] Create macOS-specific build scripts
- [ ] Setup code signing certificates
- [ ] Configure macOS entitlements
- [ ] Test build on Intel Mac

#### Week 3-4: Platform-Specific Adaptations
- [ ] Adapt file path handling (Windows vs Unix)
- [ ] Update FFmpeg binaries for macOS
- [ ] Test Windows Game DVR fallback
- [ ] Adapt keyboard shortcuts for macOS
- [ ] Update UI for macOS design guidelines

#### Week 4-5: Testing & Validation
- [ ] Test on Intel Mac with Boot Camp
- [ ] Verify League of Legends detection
- [ ] Test recording functionality
- [ ] Test auto-edit workflow
- [ ] Test YouTube upload

#### Week 6: Documentation & Release
- [ ] Create Boot Camp setup guide
- [ ] Update documentation for macOS
- [ ] Create macOS installer (.dmg)
- [ ] Publish v1.3.0 release

### User Documentation
```markdown
# macOS Installation Guide

## Requirements
- Intel Mac (Boot Camp compatible)
- macOS 10.15 or later
- Windows 10/11 license
- 100GB+ free space for Boot Camp partition

## Setup Steps
1. Install Boot Camp using Boot Camp Assistant
2. Install Windows 10/11 on Boot Camp partition
3. Boot into Windows via Boot Camp
4. Install League of Legends (Windows version)
5. Download and install LoLShorts.app
6. Run LoLShorts in Windows environment

## Important Notes
- M1/M2 Macs are NOT supported (no Boot Camp)
- League of Legends must run in Windows environment
- All recording and editing done in Windows
```

---

## 📊 Development Metrics

### Code Complexity Estimates

| Feature | Frontend (TS) | Backend (Rust) | Tests | Total LOC |
|---------|---------------|----------------|-------|-----------|
| Auto-Edit Core | 2,000 | 3,500 | 1,000 | 6,500 |
| Canvas Editor | 1,500 | 800 | 500 | 2,800 |
| Audio Mixing | 500 | 1,200 | 300 | 2,000 |
| YouTube OAuth | 400 | 1,000 | 200 | 1,600 |
| YouTube Upload | 800 | 2,000 | 600 | 3,400 |
| Metadata Gen | 600 | 1,500 | 400 | 2,500 |
| macOS Support | 300 | 800 | 200 | 1,300 |
| **TOTAL** | **6,100** | **10,800** | **3,200** | **20,100** |

### Performance Targets

| Feature | Target | Critical Path |
|---------|--------|---------------|
| Auto-Edit (60s video) | <2 minutes | FFmpeg rendering |
| Canvas Render | <10 seconds | Overlay application |
| Audio Mix | <5 seconds | FFmpeg audio filter |
| YouTube Upload (50MB) | <60 seconds | Network speed |
| OAuth Flow | <10 seconds | Google redirect |
| Metadata Generation | <1 second | Algorithm speed |

---

## 🔧 Technical Stack

### Core Technologies
```yaml
Frontend:
  - React 18
  - TypeScript 5
  - Tailwind CSS
  - shadcn/ui
  - Zustand (state)
  - React Query

Backend:
  - Rust 1.75+
  - Tauri 2.0
  - FFmpeg 6.0
  - Windows Game DVR API
  - SQLite (local DB)

APIs:
  - League Client API (LCU)
  - Live Client Data API
  - YouTube Data API v3
  - Google OAuth2

Build:
  - Cargo (Rust)
  - pnpm (Node)
  - FFmpeg (bundled)
```

### External Dependencies

#### Rust Crates
```toml
[dependencies]
# Core
tauri = "2.0"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }

# Video Processing
ffmpeg-next = "6.0"

# Recording
windows = { version = "0.51", features = ["Graphics_Capture", "Media"] }

# YouTube Integration
google-youtube3 = "5.0"
oauth2 = "4.4"
reqwest = { version = "0.11", features = ["multipart", "stream"] }

# Database
sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }
```

#### Frontend Packages
```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tauri-apps/api": "^2.0.0",
    "zustand": "^4.5.0",
    "@tanstack/react-query": "^5.0.0",
    "tailwindcss": "^3.4.0"
  }
}
```

---

## 🧪 Testing Strategy

### Test Coverage Goals
- **Unit Tests**: >80% coverage (backend)
- **Integration Tests**: Critical paths covered
- **E2E Tests**: Major user workflows

### Test Automation
```bash
# Backend tests
cargo test --all-features

# Frontend tests
pnpm test

# E2E tests (Playwright)
pnpm test:e2e

# Coverage report
cargo tarpaulin --out Html
```

### Key Test Scenarios

#### Auto-Edit
- [ ] Single game, 5 clips, 60s target → Completes in <2min
- [ ] Multiple games, 10 clips, 120s target → Completes in <3min
- [ ] Canvas overlay with text → Renders correctly
- [ ] Background music at 80% → Mixes properly
- [ ] Cancellation mid-process → Cleans up resources

#### YouTube Upload
- [ ] OAuth flow → Tokens stored securely
- [ ] 50MB video upload → Completes in <60s
- [ ] Metadata generation → Title/description accurate
- [ ] Upload failure → Retry works
- [ ] Quota exceeded → Proper error message

---

## 📈 Success Metrics

### Technical KPIs
- **Auto-Edit Success Rate**: >95%
- **Upload Success Rate**: >95%
- **Average Auto-Edit Time**: <2 minutes
- **Average Upload Time**: <60 seconds
- **App Crash Rate**: <0.1%
- **Memory Usage (Peak)**: <2GB

### User Experience KPIs
- **Time from Game End to Published Short**: <5 minutes
- **User Satisfaction (NPS)**: >50
- **Feature Adoption**: >70% use auto-edit within first week
- **Retention (7-day)**: >60%
- **Retention (30-day)**: >40%

---

## 🚀 Release Checklist

### Pre-Release (v1.2.0)
- [ ] All features implemented and tested
- [ ] Performance targets met
- [ ] Documentation complete
- [ ] User guide published
- [ ] Demo video created
- [ ] Marketing materials ready

### Release Day
- [ ] Build production binaries (Windows)
- [ ] Upload to GitHub Releases
- [ ] Update website with download links
- [ ] Announce on social media
- [ ] Monitor error reports

### Post-Release
- [ ] Collect user feedback
- [ ] Monitor performance metrics
- [ ] Fix critical bugs within 24h
- [ ] Plan v1.2.1 patch if needed
- [ ] Begin v1.3.0 development

---

## 📞 Support & Community

### User Support Channels
- **GitHub Issues**: Bug reports and feature requests
- **Discord**: Community chat and support
- **Email**: support@lolshorts.app
- **Twitter**: @lolshorts

### Developer Resources
- **Documentation**: `CLAUDE.md` (development guidelines)
- **API Specs**: `AUTO_EDIT_SPEC.md`, `YOUTUBE_UPLOAD_SPEC.md`
- **Architecture**: `src-tauri/src/` (Rust backend)
- **Components**: `src/components/` (React frontend)

---

## 🎓 Learning Resources

### For New Contributors
1. Read `CLAUDE.md` (coding standards)
2. Review `AUTO_EDIT_SPEC.md` and `YOUTUBE_UPLOAD_SPEC.md`
3. Setup development environment (see README.md)
4. Pick a "good first issue" from GitHub
5. Submit PR following contribution guidelines

### Key Technologies to Learn
- **Rust**: The Rust Book (https://doc.rust-lang.org/book/)
- **Tauri**: Official Docs (https://tauri.app/v2/guides/)
- **React**: React Docs (https://react.dev/)
- **FFmpeg**: FFmpeg Documentation (https://ffmpeg.org/documentation.html)
- **YouTube API**: Google Developers (https://developers.google.com/youtube/v3)

---

**Last Updated**: 2025-11-06
**Next Review**: After v1.2.0 release
