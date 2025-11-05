# LoLShorts Development Commands

## Essential Commands

### Development
```bash
# Start development server (frontend + backend hot reload)
cargo tauri dev
# or
npm run tauri:dev

# Start frontend only
npm run dev

# Start backend only
cargo run --manifest-path src-tauri/Cargo.toml
```

### Building
```bash
# Development build
cargo tauri build --debug

# Production build (optimized, signed)
cargo tauri build

# Build frontend only
npm run build

# Build backend only
cargo build --manifest-path src-tauri/Cargo.toml --release
```

### Testing
```bash
# Run all Rust tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run specific test
cargo test --manifest-path src-tauri/Cargo.toml <test_name>

# Run tests with output
cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture

# Run frontend tests
npm test

# Watch mode (frontend)
npm run test:watch

# Integration tests
cargo test --manifest-path src-tauri/Cargo.toml --test recording_integration
```

### Code Quality
```bash
# Format Rust code
cargo fmt --manifest-path src-tauri/Cargo.toml

# Check Rust formatting
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Lint Rust code (clippy)
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Format TypeScript/React code
npm run format

# Lint TypeScript/React code
npm run lint
```

### Dependency Management
```bash
# Update Rust dependencies
cargo update --manifest-path src-tauri/Cargo.toml

# Add Rust dependency
cd src-tauri && cargo add <package>

# Add Rust dev dependency
cd src-tauri && cargo add --dev <package>

# Install npm dependencies
npm install

# Add npm dependency
npm install <package>

# Add npm dev dependency
npm install --save-dev <package>
```

### Database Management
```bash
# Run SQLite migrations (when implemented)
cargo run --manifest-path src-tauri/Cargo.toml -- migrate

# Reset database (development)
rm -f %APPDATA%\lolshorts\lolshorts.db
```

### Debugging
```bash
# Run with debug logging
RUST_LOG=debug cargo tauri dev

# Run with trace logging
RUST_LOG=trace cargo tauri dev

# Run frontend with dev tools
npm run dev
```

### Git Workflow
```bash
# Check status
git status

# Create feature branch
git checkout -b feature/<name>

# Stage changes
git add .

# Commit with message
git commit -m "feat(recording): add pentakill detection"

# Push to remote
git push origin feature/<name>
```

## Windows-Specific Commands

### File Operations
```powershell
# List directory contents
Get-ChildItem
# or
dir

# List recursively
Get-ChildItem -Recurse

# Find files
Get-ChildItem -Recurse -Filter "*.rs"

# Search content
Select-String -Path "*.rs" -Pattern "keyword"
```

### Process Management
```powershell
# Find FFmpeg processes
Get-Process ffmpeg

# Kill FFmpeg processes
Stop-Process -Name ffmpeg -Force

# Monitor resource usage
Get-Process lolshorts | Select-Object CPU,WS
```

### Environment Variables
```powershell
# Set environment variable (session)
$env:RUST_LOG = "debug"

# Set environment variable (permanent)
[System.Environment]::SetEnvironmentVariable("FFMPEG_PATH", "C:\ffmpeg\bin", "User")

# Check PATH
echo $env:PATH
```

## Platform Detection
Current system: Windows

## Performance Profiling
```bash
# Flamegraph profiling (requires cargo-flamegraph)
cargo install flamegraph
cargo flamegraph --manifest-path src-tauri/Cargo.toml

# Benchmark tests
cargo bench --manifest-path src-tauri/Cargo.toml
```

## Troubleshooting Commands
```bash
# Clear build cache
cargo clean --manifest-path src-tauri/Cargo.toml
rm -rf node_modules .vite

# Reinstall dependencies
npm install

# Verify FFmpeg installation
ffmpeg -version

# Check Tauri dependencies
cargo tauri info
```

## Pre-Commit Checklist Commands
```bash
# Run before committing
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run lint
npm run format
npm test
```