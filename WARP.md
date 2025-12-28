# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project overview
LoLShorts is a Windows-first desktop app built with:
- Frontend: Vite + React + TypeScript (in `src/`)
- Backend: Tauri 2 + Rust (in `src-tauri/`)
- Auth/licensing (cloud): Supabase (frontend uses Supabase JS; backend uses its own `SupabaseClient` for license checks)
- Storage (local-first): JSON + files on disk (clips, events, templates, auto-edit results)

The app’s core runtime is the Rust backend: it manages recording (FFmpeg + platform backends), talks to League Client (LCU) + Live Client APIs, stores metadata locally, and exposes commands to the UI via Tauri `invoke`.

## Common commands

### Install
```powershell
npm ci
```

### Run in dev
- Full app (recommended):
```powershell
npm run tauri:dev
```

- Frontend only (Vite):
```powershell
npm run dev
```

### Build
- Frontend build:
```powershell
npm run build
```

- Full desktop build (bundles):
```powershell
npm run tauri:build
# (equivalent to `cargo tauri build`)
```

### Lint / format / typecheck
```powershell
npm run lint
npm run format
npm run typecheck
```

Rust (run from `src-tauri/`):
```powershell
cd src-tauri
cargo fmt
cargo clippy -- -D warnings
```

### Tests
Frontend unit tests (Jest):
```powershell
npm test
```

Run a single Jest test file:
```powershell
npm test -- path\to\test-file.test.ts
```

Run a single Jest test by name:
```powershell
npm test -- -t "test name substring"
```

E2E tests (Playwright):
```powershell
npm run test:e2e
```

Run a single Playwright spec:
```powershell
npm run test:e2e -- tests/e2e/example.spec.ts
```

Rust tests (run from `src-tauri/`):
```powershell
cd src-tauri
cargo test
```

Run a single Rust test by name:
```powershell
cd src-tauri
cargo test test_name_substring
```

Run a single Rust integration test target:
```powershell
cd src-tauri
cargo test --test integration_test_name
```

### Environment/setup scripts
Windows dev machine bootstrap (installs Rust/Node/FFmpeg/etc):
```powershell
# Run PowerShell as Administrator
.\scripts\setup-dev-windows.ps1
```

Cross-platform build helper (Windows PowerShell):
```powershell
# Example: release build for Windows, run tests, and prepare bundled FFmpeg
.\scripts\build-cross-platform.ps1 -BuildType release -Platform windows

# Clean + skip tests
.\scripts\build-cross-platform.ps1 -Clean -SkipTests
```

### FFmpeg bundling for production
The Windows build expects FFmpeg binaries to exist under `src-tauri/binaries/` (see `src-tauri/tauri.conf.json` `bundle.externalBin`).

Prepare/download FFmpeg:
```powershell
cd src-tauri\build_scripts
.\prepare_ffmpeg.ps1
```

### Local Supabase / DB (optional)
This repo includes two ways to run local auth/license DB:

- With Supabase CLI (see `supabase/README.md`):
```bash
supabase start
supabase db push
```

- With Docker Compose (runs Postgres + PostgREST + GoTrue + Kong):
```powershell
docker compose up -d
```

## High-level architecture

### Frontend (React/TS)
Entry points:
- `src/main.tsx`: React bootstrap
- `src/App.tsx`: Router + page shell

Command boundary:
- `src/api/client.ts` defines `cmd()` which wraps `@tauri-apps/api/core` `invoke()` and normalizes Rust errors.
- `src/api/*.ts` are thin command wrappers (e.g. `recordingApi`, `storageApi`, `lcuApi`, `youtubeApi`).
- UI state uses Zustand stores (e.g. `src/stores/recordingStore.ts`) that call these APIs and poll for status.

Auth boundary:
- Frontend uses Supabase JS (`src/lib/supabase.ts`).
- After login/signup/session refresh, the frontend syncs tokens to the Rust backend via `authApi.setSession(...)` (see `src/lib/auth.ts`).

### Backend (Rust/Tauri)
Entry points:
- `src-tauri/src/main.rs`: sets up logging, initializes state, starts background tasks (hotkeys + game monitoring), and registers Tauri commands via `invoke_handler`.
- `src-tauri/src/lib.rs`: defines modules + the shared `AppState` injected into commands.

Command registration:
- Most Tauri commands are listed in a single place in `src-tauri/src/main.rs` under `tauri::generate_handler![ ... ]`.
- Feature gating is implemented in Rust command handlers via auth middleware (`require_auth`, `require_tier`) and license checks.

Key subsystems:

1) Recording pipeline (`src-tauri/src/recording/*`)
- `RecordingManager` (re-exported from `recording/integration_backend.rs`) is the main runtime interface.
- Handles start/stop, replay buffer behavior, encoder detection, and “save last N seconds”.
- `GameStateMonitor` + `AutoClipManager` orchestrate automatic capture around game lifecycle and events.
- Platform backends exist (Windows + macOS modules) but the integration backend is the abstraction the rest of the app uses.

2) League integration (`src-tauri/src/lcu/*` and `src-tauri/src/recording/live_client.rs`)
- LCU commands support match history + replay download/launch.
- Used by the UI “Replays” flow and by capture logic to understand what’s happening in-game.

3) Video processing / auto-edit (`src-tauri/src/video/*`)
- `VideoProcessor` runs FFmpeg-based operations (clip extraction, shorts composition, thumbnails).
- `AutoComposer` builds longer compositions/auto-edit results from stored clips.
- Many operations are PRO-gated in `video/commands.rs`.

4) Storage (local-first) (`src-tauri/src/storage/*`)
- JSON-on-disk storage for:
  - per-game metadata (`metadata.json`), events (`events.json`), and clip index (`clips.json`)
  - generic key/value settings (`settings.json`) used for things like YouTube credentials
  - canvas templates (`templates/*.json`)
  - auto-edit quota + results

5) Auth/licensing (`src-tauri/src/auth/*` + `src-tauri/src/supabase/*`)
- The backend keeps a current session/user in `AuthManager`.
- The backend Supabase client reads `SUPABASE_URL` / `SUPABASE_ANON_KEY` (see `src-tauri/src/supabase/client.rs`).

6) YouTube integration (`src-tauri/src/youtube/*`)
- OAuth flow + (optional) local callback server.
- Upload client persists credentials in local storage.

### Where app data lives
The backend derives an app data directory via `dirs::data_dir()` and uses a `lolshorts/` subfolder (see `src-tauri/src/main.rs`). Expect local assets like logs/recordings/clips/settings to be created under that directory.

## Versioning / release notes
Version is duplicated across multiple places; keep these in sync when bumping:
- `package.json` (`version`)
- `src-tauri/Cargo.toml` (`package.version`)
- `src-tauri/tauri.conf.json` (`version`)

(See `README.md` for the tag-based GitHub Actions release flow.)

## Environment variables (.env)
Examples:
- `.env.example` (dev)
- `.env.production.example` (prod-oriented)

Commonly used keys:
- Frontend (Vite): `VITE_SUPABASE_URL`, `VITE_SUPABASE_ANON_KEY`
- Backend (Rust): `SUPABASE_URL`, `SUPABASE_ANON_KEY`
- YouTube uploads: `YOUTUBE_CLIENT_ID`, `YOUTUBE_CLIENT_SECRET`, `YOUTUBE_REDIRECT_URI`

Note: `src-tauri/src/main.rs` calls `dotenvy::dotenv().ok();` so a root `.env` is loaded at runtime in dev.