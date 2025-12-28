# Development Guide

This guide covers setting up the development environment and contributing to LoLShorts.

## Quick Start

### Prerequisites
- **Rust**: Latest stable version
- **Node.js**: Version 18 or later
- **FFmpeg**: Version 4.4 or later

### Setup (Automated)

Run the appropriate setup script for your platform:

**Windows (PowerShell as Administrator):**
```powershell
.\scripts\setup-dev-windows.ps1
```

**macOS:**
```bash
source scripts/setup-dev-macos.sh
```

**Linux:**
```bash
source scripts/setup-dev-linux.sh
```

### Setup (Manual)

1. **Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Install dependencies:**

**Windows:**
```powershell
choco install ffmpeg nodejs visualstudio2022buildtools
```

**macOS:**
```bash
brew install ffmpeg node pkg-config
xcode-select --install
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install ffmpeg nodejs npm pkg-config libssl-dev libgtk-3-dev
```

3. **Clone and setup:**
```bash
git clone https://github.com/your-repo/lolshorts.git
cd lolshorts
npm ci
cargo install tauri-cli --locked
```

4. **Run development server:**
```bash
npm run tauri dev
```

## Development Workflow

### Development Commands

Use the development helper script:

```bash
# Source the helper
source scripts/dev-helper.sh  # macOS/Linux
# or
.\scripts\dev-helper.ps1      # Windows

# Run Rust development
dev_rust dev
dev_rust build
dev_rust test
dev_rust lint

# Run frontend development
dev_frontend dev
dev_frontend build
dev_frontend type-check

# Run full development (both frontend and backend)
dev_full
```

### Standard npm scripts:

```bash
# Development
npm run tauri dev              # Start development server
npm run tauri build           # Production build

# Frontend only
npm run dev                   # Vite dev server
npm run build                 # Frontend build
npm run preview               # Preview production build

# Code quality
npm run format                # Format code
npm run format:check          # Check formatting
npm run lint                  # ESLint
npm run type-check            # TypeScript check
npm run test                  # Frontend tests
```

### Rust commands (from src-tauri/):

```bash
cargo run --bin lolshorts-tauri    # Run Rust backend
cargo build                        # Build Rust code
cargo test                         # Run tests
cargo fmt                          # Format code
cargo clippy -- -D warnings        # Lint code
cargo run --bin test_platform_detection  # Test platform detection
```

## Testing

### Run all tests:
```bash
npm run test          # Frontend tests
cargo test            # Rust tests
```

### Cross-platform testing:
```bash
# Test platform detection
cargo run --bin test_platform_detection

# Run cross-platform compilation tests
cargo test cross_platform_compilation

# Test all available backends
cargo test --test '*' --verbose
```

### Coverage:
```bash
# Rust coverage (requires cargo-llvm-cov)
cargo install cargo-llvm-cov
cargo llvm-cov --html --lcov

# Frontend coverage (configured in package.json)
npm run test:coverage
```

## Code Quality

### Pre-commit Hooks
Pre-commit hooks automatically run before each commit:

- ✅ Rust formatting (`cargo fmt`)
- ✅ Rust linting (`cargo clippy`)
- ✅ TypeScript formatting (`prettier`)
- ✅ TypeScript linting (`eslint`)
- ✅ Security audits (`cargo audit`, `npm audit`)

### Manual Quality Checks
```bash
# Rust
cargo fmt --check          # Check formatting
cargo clippy -- -D warnings  # Lint with warnings as errors
cargo audit               # Security audit

# Frontend
npm run format:check      # Check formatting
npm run lint              # ESLint
npm run type-check        # TypeScript check
npm audit                 # Security audit
```

### Configuration Files
- **Rust**: `.clippy.toml` (linting rules)
- **Dependencies**: `deny.toml` (security and licensing)
- **TypeScript**: `tsconfig.json`, `.eslintrc.js`, `.prettierrc`

## Architecture

### Project Structure
```
LoLShorts/
├── src/                          # Frontend (React/TypeScript)
├── src-tauri/                    # Backend (Rust)
│   ├── src/
│   │   ├── lib.rs               # Main application
│   │   ├── recording/           # Recording system
│   │   │   ├── platform/        # Platform abstraction
│   │   │   ├── audio.rs         # Audio processing
│   │   │   └── video.rs         # Video processing
│   │   ├── youtube/             # YouTube integration
│   │   ├── lcu/                 # League client API
│   │   └── auth/                # Authentication
│   ├── Cargo.toml               # Rust dependencies
│   └── tests/                   # Rust tests
├── scripts/                     # Development scripts
├── .github/workflows/           # CI/CD pipelines
└── docs/                       # Documentation
```

### Cross-Platform Recording
The recording system uses a platform abstraction layer:

- **Windows**: Windows Graphics Capture API / Direct3D
- **macOS**: ScreenCaptureKit / AVFoundation
- **Linux**: Pipewire / X11

See [CROSS_PLATFORM_DEVELOPMENT.md](./CROSS_PLATFORM_DEVELOPMENT.md) for details.

## Platform-Specific Development

### Windows Development
- Requires Visual Studio Build Tools 2022
- Windows 10 version 1903+ for Graphics Capture API
- Run PowerShell as Administrator for setup

### macOS Development
- Requires Xcode Command Line Tools
- macOS 12.3+ for ScreenCaptureKit
- Grant screen recording permissions in System Preferences

### Linux Development
- Install distribution-specific development libraries
- Wayland support requires additional setup
- Audio capture may need user in audio group

## Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug npm run tauri dev
```

### Platform-Specific Debugging
```bash
# Test platform detection
cargo run --bin test_platform_detection

# Debug backend issues
RUST_LOG=debug cargo test --test cross_platform_compilation
```

### Common Issues
- **FFmpeg not found**: Install and add to PATH
- **Build errors**: Check platform-specific prerequisites
- **Permission denied**: Grant screen/audio recording permissions
- **Missing dependencies**: Install development libraries

## Contributing

### Pull Request Process
1. Fork repository and create feature branch
2. Implement changes with tests
3. Ensure all quality checks pass
4. Update documentation
5. Submit pull request with clear description

### Code Style
- Follow Rust guidelines (`cargo fmt`)
- Follow TypeScript guidelines (ESLint/Prettier)
- Write descriptive commit messages
- Include tests for new features
- Update documentation

### Testing Requirements
- All tests must pass on current platform
- Cross-platform features tested on multiple platforms
- Integration tests for major functionality
- Performance tests for video processing

## Security

### Security Checks
```bash
# Rust dependencies
cargo audit

# Node.js dependencies
npm audit

# License compliance
cargo deny check
```

### Security Guidelines
- Validate all external inputs
- Handle sensitive data securely
- Use secure defaults
- Regularly update dependencies
- Follow platform security best practices

## Performance

### Profiling
```bash
# CPU profiling (requires flamegraph)
cargo install flamegraph
cargo flamegraph --bin lolshorts-tauri

# Memory profiling
valgrind --tool=massif target/release/lolshorts-tauri
```

### Benchmarks
```bash
# Run benchmarks
cargo bench

# Generate HTML reports
cargo bench -- --output-format html
```

### Optimization Targets
- App startup: <3s cold start
- Video processing: <30s per minute
- Memory usage: <500MB idle
- Frame rate: 60fps stable capture

## Release Process

### Version Management
- Follow Semantic Versioning (semver)
- Update version in `Cargo.toml`, `package.json`, `tauri.conf.json`
- Update `CHANGELOG.md` with changes

### Build Release
```bash
# Build for current platform
npm run tauri build

# Build for specific targets (requires appropriate environments)
npm run tauri build --target x86_64-pc-windows-msvc
npm run tauri build --target x86_64-apple-darwin
npm run tauri build --target x86_64-unknown-linux-gnu
```

### Release Checklist
- [ ] All tests pass
- [ ] No security vulnerabilities
- [ ] Performance benchmarks met
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version numbers consistent
- [ ] Code signed (production)

## Continuous Integration

### GitHub Actions
- **Cross-platform builds**: Windows, macOS, Linux
- **Quality checks**: Formatting, linting, security
- **Testing**: Unit, integration, cross-platform
- **Performance**: Benchmarking and regression detection

### Local Testing Before CI
```bash
# Run full CI-like check locally
cargo test
cargo fmt --check
cargo clippy -- -D warnings
cargo audit
npm run test
npm run lint
npm run type-check
npm audit
```

## Getting Help

### Documentation
- [Cross-Platform Development](./CROSS_PLATFORM_DEVELOPMENT.md)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Tauri Documentation](https://tauri.app/v1/guides/)
- [React Documentation](https://react.dev/)

### Troubleshooting
- Check platform-specific setup instructions
- Review GitHub Actions failures for CI issues
- Search existing issues for common problems
- Create detailed bug reports with platform information

### Community
- GitHub Issues: Report bugs and request features
- GitHub Discussions: General questions and discussions
- Check README for additional community resources

## Development Tools

### Recommended VS Code Extensions
- **Rust Analyzer**: Rust language support
- **Tauri**: Tauri framework integration
- **ESLint**: TypeScript linting
- **Prettier**: Code formatting
- **GitLens**: Git integration

### Useful Cargo Commands
```bash
# Update dependencies
cargo update

# Check for outdated packages
cargo install cargo-outdated
cargo outdated

# Find unused dependencies
cargo install cargo-udeps
cargo +nightly udeps

# Generate documentation
cargo doc --open
```

### Development Environment
- Use the development helper scripts for convenience
- Configure your IDE for Rust and TypeScript
- Set up appropriate linters and formatters
- Use Git hooks for quality enforcement