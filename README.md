# LoLShorts

<div align="center">

![LoLShorts Banner](docs/images/banner.png)

**Automatically record and edit your League of Legends gameplay into YouTube Shorts format**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows)
[![Version](https://img.shields.io/badge/version-1.0.0-green.svg)](https://github.com/KhakiSkech/lolshorts/releases)

[Features](#-features) • [Download](#-download) • [Usage](#-usage) • [Development](#-development) • [License](#-license)

</div>

---

## 📖 Overview

LoLShorts is a desktop application that automatically records your League of Legends gameplay and creates highlight clips optimized for YouTube Shorts. Never miss a pentakill, baron steal, or epic teamfight again!

### Key Features

- ✨ **Automatic Event Detection**: Detects pentakills, multi-kills, baron/dragon steals, and more
- 🎥 **Replay Buffer**: Always recording the last 2 minutes, save any moment instantly
- 🚀 **Hardware Acceleration**: GPU-accelerated video encoding (NVENC/QSV/AMF)
- 🎬 **Built-in Editor**: Timeline editing with transitions, effects, and text overlays
- 📊 **Smart Prioritization**: Automatically ranks clips by importance
- 🎨 **Customizable**: Full control over quality, format, and recording settings
- 🔒 **Privacy First**: All processing happens locally, no data leaves your computer

---

## 🎯 Features

### Automatic Recording
- Real-time monitoring of League of Legends game state via LCU API
- Intelligent event detection system with priority scoring
- Hardware-accelerated H.265 encoding for efficient storage
- Configurable replay buffer (30s - 5min)

### Event Detection
| Event Type | Priority | Description |
|------------|----------|-------------|
| Pentakill | ⭐⭐⭐⭐⭐ | 5 consecutive kills |
| Quadrakill | ⭐⭐⭐⭐ | 4 consecutive kills |
| Baron Steal | ⭐⭐⭐⭐ | Steal Baron from enemy team |
| Dragon Steal | ⭐⭐⭐ | Steal Dragon from enemy team |
| Triple Kill | ⭐⭐⭐ | 3 consecutive kills |
| Multi-Kill | ⭐⭐ | 2+ kills in sequence |

### Video Editor
- Drag-and-drop timeline interface
- Trim, cut, and merge clips
- Add transitions and effects
- Text overlays and annotations
- Export directly to YouTube Shorts format (9:16, max 60s)

### Recording Settings
- **Quality Presets**: Low, Medium, High, Ultra (720p-1440p)
- **Frame Rates**: 30 FPS or 60 FPS
- **Bitrate Control**: 1-20 Mbps customizable
- **Audio Settings**: Game audio + microphone support
- **Storage Management**: Automatic cleanup, disk usage monitoring

---

## 📥 Download

### System Requirements

| Component | Requirement |
|-----------|-------------|
| **OS** | Windows 10 (64-bit) or later |
| **CPU** | Intel Core i5 / AMD Ryzen 5 or better |
| **RAM** | 8 GB minimum, 16 GB recommended |
| **GPU** | NVIDIA GTX 1050 / AMD RX 560 or better (for hardware encoding) |
| **Storage** | 10 GB free space (more for recordings) |
| **Game** | League of Legends installed and up-to-date |

### Installation

1. Download the latest installer:
   - **NSIS Installer (Recommended)**: [`LoLShorts_1.0.0_x64-setup.exe`](https://github.com/KhakiSkech/lolshorts/releases/download/v1.0.0/LoLShorts_1.0.0_x64-setup.exe)
   - **MSI Installer (Enterprise)**: [`LoLShorts_1.0.0_x64_en-US.msi`](https://github.com/KhakiSkech/lolshorts/releases/download/v1.0.0/LoLShorts_1.0.0_x64_en-US.msi)

2. Run the installer with administrator privileges

3. Follow the installation wizard

4. Launch LoLShorts from the Start Menu or Desktop shortcut

---

## 🚀 Usage

### Quick Start Guide

1. **Launch LoLShorts**
   - The app will automatically detect if League of Legends is running
   - LCU connection status will be displayed in the dashboard

2. **Configure Settings** (Optional)
   - Open Settings page
   - Choose quality preset (High recommended for 1080p 60fps)
   - Configure audio settings
   - Set replay buffer duration

3. **Start Recording**
   - Click "Start Recording" button
   - Launch League of Legends and enter a game
   - LoLShorts will automatically monitor for highlight events

4. **Save Highlights**
   - When a highlight event occurs, it's automatically detected
   - Click "Save Replay" to manually save the last 2 minutes
   - All clips are saved to the Clip Library

5. **Edit and Export**
   - Open Clip Library to view saved clips
   - Select clips to edit in the Video Editor
   - Add transitions, effects, and text
   - Export to YouTube Shorts format

### Hotkeys

| Key | Action |
|-----|--------|
| `Ctrl + Shift + R` | Start/Stop Recording |
| `Ctrl + Shift + S` | Save Replay Buffer |
| `Ctrl + Shift + E` | Open Video Editor |

---

## 🛠️ Development

### Tech Stack

**Frontend**:
- React 18 + TypeScript
- Zustand (State Management)
- shadcn/ui + Tailwind CSS
- Vite 6

**Backend**:
- Rust + Tokio (Async Runtime)
- Tauri 2.0 (Desktop Framework)
- FFmpeg (Video Processing)
- Windows Media Foundation

**APIs**:
- LCU API (League Client)
- Live Client Data API (In-Game Events)

### Prerequisites

- **Rust**: Latest stable version (install via [rustup](https://rustup.rs/))
- **Node.js**: v18 or later
- **npm** or **pnpm**: Package manager
- **FFmpeg**: Required for video processing

### Building from Source

```bash
# Clone the repository
git clone https://github.com/KhakiSkech/lolshorts.git
cd lolshorts

# Install frontend dependencies
npm install

# Install Rust dependencies (automatically done by cargo)
cd src-tauri
cargo build

# Run development build
cd ..
npm run tauri:dev

# Build production release
npm run tauri:build
```

### Project Structure

```
lolshorts/
├── src/                      # Frontend React app
│   ├── components/          # UI components
│   ├── stores/              # Zustand stores
│   ├── lib/                 # Utilities
│   └── assets/              # Static assets
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── auth/           # License and authentication
│   │   ├── recording/      # Video recording logic
│   │   ├── storage/        # Clip storage and metadata
│   │   ├── video/          # Video processing (FFmpeg)
│   │   └── main.rs         # Application entry point
│   └── Cargo.toml          # Rust dependencies
├── docs/                    # Documentation
├── tests/                   # Test suites
└── README.md               # This file
```

### Testing

```bash
# Run Rust tests
cd src-tauri
cargo test

# Run frontend tests
cd ..
npm test

# Run E2E tests
npm run test:e2e
```

### Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📋 Roadmap

### Version 1.1.0 (Planned)
- [ ] macOS support
- [ ] Linux support
- [ ] Cloud storage integration
- [ ] Direct upload to YouTube/Twitch
- [ ] Multi-language support (Korean, Chinese, Japanese)

### Version 1.2.0 (Planned)
- [ ] AI-powered highlight detection
- [ ] Automatic commentary generation
- [ ] Advanced editing features (slow motion, zoom, filters)
- [ ] Team collaboration features

### Version 2.0.0 (Future)
- [ ] Support for other games (Valorant, CS2, Dota 2)
- [ ] Live streaming integration
- [ ] Mobile companion app

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🤝 Support

- **Issues**: [GitHub Issues](https://github.com/KhakiSkech/lolshorts/issues)
- **Discussions**: [GitHub Discussions](https://github.com/KhakiSkech/lolshorts/discussions)
- **Email**: support@lolshorts.com

---

## 📚 Documentation

- [Installation Guide](docs/INSTALLATION.md)
- [User Manual](docs/USER_MANUAL.md)
- [Developer Guide](docs/DEVELOPER_GUIDE.md)
- [API Documentation](docs/API.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)

---

## 🙏 Acknowledgments

- **Riot Games** - For League of Legends and the LCU API
- **FFmpeg** - Video processing library
- **Tauri** - Desktop application framework
- **shadcn/ui** - Beautiful UI components

---

## ⚖️ Legal Notice

LoLShorts is not endorsed by Riot Games and does not reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games and all associated properties are trademarks or registered trademarks of Riot Games, Inc.

This application complies with Riot Games' [Third Party Application Policy](https://developer.riotgames.com/policies/general).

---

<div align="center">

Made with ❤️ by the LoLShorts Team

[Website](https://lolshorts.com) • [Twitter](https://twitter.com/lolshorts) • [Discord](https://discord.gg/lolshorts)

</div>
