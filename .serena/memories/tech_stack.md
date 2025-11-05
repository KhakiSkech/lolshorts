# LoLShorts Technology Stack

## Backend (Rust)
- **Framework**: Tauri 2.0
- **Async Runtime**: Tokio 1.41 (full features)
- **HTTP Client**: reqwest 0.12 (JSON support)
- **WebSocket**: tokio-tungstenite 0.24
- **Error Handling**: thiserror 2.0, anyhow 1.0
- **Logging**: tracing 0.1, tracing-subscriber 0.3
- **Date/Time**: chrono 0.4
- **Utilities**: once_cell, dirs, uuid (v4)

## Frontend (TypeScript/React)
- **Framework**: React 18.3.1
- **Language**: TypeScript 5.7
- **Build Tool**: Vite 6.0
- **State Management**: Zustand 5.0
- **Routing**: TanStack Router 1.94
- **UI Components**: shadcn/ui (Tailwind CSS)
- **Icons**: lucide-react 0.468
- **Styling**: Tailwind CSS 3.4, clsx, tailwind-merge
- **Testing**: Jest 29.7, React Testing Library 16.1
- **Linting**: ESLint 9.18, Prettier 3.4

## Video Processing
- **Engine**: FFmpeg CLI (bundled binary)
- **Strategy**: Process-based approach for stability
- **Screen Capture**: gdigrab (Windows)
- **Encoding**: H.265/HEVC with hardware acceleration (NVENC/QSV/AMF)
- **Segment Management**: Circular buffer (6 × 10s = 60s window)

## Alternative Options (Optional)
- **Native Rust Encoder**: rav1e 0.7, mp4 0.14 (feature flag: native-encoder)

## Input Monitoring
- **Hotkey Detection**: rdev 0.5

## Image Processing
- **Core**: image 0.25, imageproc 0.24

## Performance
- **Parallelization**: rayon 1.10
- **Concurrency**: parking_lot 0.12
- **Buffering**: ringbuf 0.4
- **Memory**: bytes 1

## System Integration
- **System Info**: sysinfo 0.31
- **CPU Detection**: num_cpus 1.16

## Platform-Specific Dependencies
### Windows
- **Windows API**: windows 0.58 (Foundation, Threading, COM, Input, Storage, Graphics, UI, Media)

### macOS
- **Graphics**: core-graphics 0.23, core-foundation 0.9

### Linux
- **X11**: x11rb 0.13
- **Wayland**: wayland-client 0.31 (optional, feature: wayland-support)

## Cloud Services
- **Authentication**: Supabase Auth (JWT tokens)
- **Database**: Supabase PostgreSQL + SQLite (local)
- **Storage**: Supabase Storage (video clips)
- **Error Tracking**: Sentry

## Development Tools
- **Testing**: tempfile 3.14 (dev dependency)
- **Code Formatting**: cargo fmt, prettier
- **Linting**: cargo clippy, eslint