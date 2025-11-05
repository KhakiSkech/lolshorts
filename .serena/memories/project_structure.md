# LoLShorts Project Structure

## Root Directory
```
LoLShorts/
├── src/                    # React frontend source
├── src-tauri/              # Rust backend source
├── public/                 # Static assets
├── docs/                   # Documentation
├── CLAUDE.md              # AI development guidelines
├── PRODUCTION_ROADMAP.md  # Production implementation plan
├── package.json           # Frontend dependencies
├── tsconfig.json          # TypeScript configuration
├── tailwind.config.js     # Tailwind CSS configuration
├── vite.config.ts         # Vite build configuration
└── .gitignore
```

## Backend Structure (src-tauri/src/)
```
src-tauri/src/
├── main.rs                 # Application entry point, state setup
├── auth/
│   ├── mod.rs             # Auth manager, session handling
│   └── commands.rs        # Tauri commands: login, logout, get_user_status
├── database/
│   ├── mod.rs             # Database connection, initialization
│   ├── models.rs          # Database models (User, Clip, Game)
│   └── migrations.rs      # SQLite schema migrations
├── feature_gate/
│   └── mod.rs             # License tier feature gating (FREE vs PRO)
├── lcu/
│   ├── mod.rs             # LCU client, connection management
│   └── commands.rs        # Tauri commands: connect_lcu, check_status, get_game
├── recording/
│   ├── mod.rs             # Recording manager, platform detection
│   ├── commands.rs        # Tauri commands: start/stop recording, save replay
│   ├── live_client.rs     # Live Client Data API integration
│   └── windows_backend.rs # Windows-specific recording backend (FFmpeg)
├── storage/
│   ├── mod.rs             # File system storage manager
│   └── models.rs          # Storage models
└── video/
    ├── mod.rs             # Video processing utilities
    └── commands.rs        # Tauri commands: get_clips
```

## Frontend Structure (src/)
```
src/
├── main.tsx               # React entry point
├── App.tsx                # Main application component
├── assets/
│   ├── champions/         # Champion portraits (170+ images)
│   └── icons/            # UI icons
├── components/
│   ├── ui/               # shadcn/ui components (Button, Card, etc.)
│   └── <feature>/        # Feature-specific components
├── hooks/
│   └── <feature>.ts      # Custom React hooks
├── stores/
│   └── <feature>Store.ts # Zustand state stores
├── types/
│   └── <feature>.ts      # TypeScript type definitions
└── lib/
    └── utils.ts          # Utility functions (cn, etc.)
```

## Module Responsibilities

### Backend Modules
- **auth**: User authentication, session management, JWT token handling
- **database**: SQLite local database, Supabase cloud sync, migrations
- **feature_gate**: License tier validation, feature access control
- **lcu**: League Client Update API client, lockfile parsing, WebSocket events
- **recording**: Screen recording with FFmpeg, circular buffer management, event-triggered capture
- **storage**: File system operations, path management, clip organization
- **video**: Video processing, FFmpeg wrapper, clip extraction, composition

### Frontend Components (Planned)
- **Dashboard**: Main view, game list, recording status
- **ClipGallery**: Grid view of saved clips, filtering, sorting
- **TimelineEditor**: Drag-and-drop video editor, transitions, music sync
- **Settings**: User preferences, license management, export settings
- **Auth**: Login, signup, password reset

## Data Flow
```
User Action (Frontend)
  ↓
Tauri Command (IPC)
  ↓
Rust Command Handler (src-tauri/src/<module>/commands.rs)
  ↓
Business Logic (src-tauri/src/<module>/mod.rs)
  ↓
Storage/Database (SQLite, File System)
  ↓
Response (JSON)
  ↓
Frontend State Update (Zustand)
  ↓
UI Re-render (React)
```

## Key Files
- **src-tauri/Cargo.toml**: Rust dependencies, features, platform-specific deps
- **src-tauri/tauri.conf.json**: Tauri configuration (bundle, security, windows)
- **src-tauri/src/main.rs**: Application initialization, state setup, command registration
- **package.json**: Frontend dependencies, build scripts
- **CLAUDE.md**: Development guidelines for AI-assisted coding
- **PRODUCTION_ROADMAP.md**: 5-wave implementation plan (Wave 1-5)