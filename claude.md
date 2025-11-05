# LoLShorts: Claude Code Development Guidelines

**Project**: LoLShorts - League of Legends Auto-Recording & Editing Desktop App
**Last Updated**: 2025-10-13
**Purpose**: Coding standards, best practices, and AI-assisted development guidelines

---

## 🎯 Development Philosophy

### Core Principles
1. **Test-Driven Development (TDD)**: Write tests before implementation
2. **Incremental Progress**: Small, verifiable steps with continuous validation
3. **Evidence-Based Decisions**: Measure performance, validate assumptions
4. **Security First**: Never compromise on security fundamentals
5. **User-Centric Design**: Performance and UX are features, not afterthoughts

### SuperClaude Integration
This project leverages SuperClaude framework capabilities:
- **Sequential MCP**: Complex multi-step reasoning and analysis
- **Context7 MCP**: Framework patterns and library documentation
- **Task Management**: Hierarchical task tracking with TodoWrite
- **Quality Gates**: 8-step validation cycle for all major features

---

## 🦀 Rust Backend Guidelines

### Code Style

#### Naming Conventions
```rust
// ✅ Good
pub struct GameDvrController { }
pub enum LicenseTier { FREE, PRO }
pub async fn start_recording() -> Result<()> { }
const MAX_CLIPS_PER_GAME: usize = 20;

// ❌ Bad
pub struct game_dvr_controller { }
pub enum licenseTier { free, pro }
pub async fn StartRecording() -> Result<()> { }
const maxClipsPerGame: usize = 20;
```

**Rules**:
- `StructsAndEnums`: PascalCase
- `functions_and_variables`: snake_case
- `CONSTANTS`: SCREAMING_SNAKE_CASE
- `'lifetimes`: lowercase single letter or descriptive

#### Module Organization
```rust
// src-tauri/src/recording/mod.rs
pub mod lcu_client;
pub mod live_monitor;
pub mod game_dvr;
pub mod event_detector;
pub mod clip_manager;

// Re-export commonly used types
pub use lcu_client::LcuClient;
pub use live_monitor::LiveClientMonitor;
pub use game_dvr::GameDvrController;
```

### Error Handling

#### Define Custom Errors
```rust
// src-tauri/src/utils/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Recording error: {0}")]
    Recording(String),

    #[error("Video processing error: {0}")]
    VideoProcessing(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Feature not available in FREE tier")]
    FeatureGated,

    #[error("License expired")]
    LicenseExpired,
}

pub type Result<T> = std::result::Result<T, AppError>;
```

#### Error Propagation
```rust
// ✅ Good: Use ? operator
pub async fn save_clip(&self, event: &GameEvent) -> Result<()> {
    let metadata = self.extract_metadata(event)?;
    self.db.insert_clip(&metadata).await?;
    Ok(())
}

// ❌ Bad: Unwrap or panic
pub async fn save_clip(&self, event: &GameEvent) {
    let metadata = self.extract_metadata(event).unwrap(); // ❌
    self.db.insert_clip(&metadata).await.expect("DB failed"); // ❌
}
```

#### Tauri Command Error Handling
```rust
// ✅ Good: Return Result<T, String>
#[tauri::command]
async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    state.recorder.start()
        .await
        .map_err(|e| e.to_string())
}

// ✅ Good: Use anyhow for complex error chains
#[tauri::command]
async fn generate_video(clips: Vec<String>) -> Result<String, String> {
    generate_video_internal(clips)
        .await
        .map_err(|e| format!("Video generation failed: {}", e))
}
```

### Async Programming

#### Tokio Best Practices
```rust
// ✅ Good: Use tokio::spawn for concurrent tasks
pub async fn monitor_events(&self) -> mpsc::Receiver<GameEvent> {
    let (tx, rx) = mpsc::channel(100);
    let monitor = self.clone();

    tokio::spawn(async move {
        loop {
            if let Ok(event) = monitor.fetch_event().await {
                tx.send(event).await.ok();
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    rx
}

// ❌ Bad: Blocking in async context
pub async fn process_video(&self) -> Result<()> {
    std::thread::sleep(Duration::from_secs(10)); // ❌ Blocks executor
    Ok(())
}

// ✅ Good: Use tokio::time::sleep
pub async fn process_video(&self) -> Result<()> {
    tokio::time::sleep(Duration::from_secs(10)).await; // ✅
    Ok(())
}
```

#### Avoiding Deadlocks
```rust
// ✅ Good: Lock scopes are minimal
pub async fn update_state(&self, value: i32) -> Result<()> {
    {
        let mut state = self.state.lock().await;
        *state = value;
    } // Lock dropped here

    self.notify_observers().await?; // No lock held
    Ok(())
}

// ❌ Bad: Lock held across await
pub async fn update_state(&self, value: i32) -> Result<()> {
    let mut state = self.state.lock().await;
    *state = value;
    self.notify_observers().await?; // ❌ Lock still held
    Ok(())
}
```

### Testing

#### Unit Test Structure
```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions at top
    fn create_test_event() -> GameEvent {
        GameEvent {
            event_id: 1,
            event_name: "ChampionKill".to_string(),
            event_time: 100.0,
            killer_name: Some("Player1".to_string()),
            victim_name: Some("Enemy1".to_string()),
            assisters: vec![],
        }
    }

    // Tests grouped by functionality
    mod priority_calculation {
        use super::*;

        #[test]
        fn test_pentakill_priority() {
            let detector = EventDetector::new();
            let events = create_pentakill_sequence();

            let priority = detector.calculate_priority(&events[4]);

            assert_eq!(priority, 5);
        }

        #[test]
        fn test_single_kill_priority() {
            let detector = EventDetector::new();
            let event = create_test_event();

            let priority = detector.calculate_priority(&event);

            assert_eq!(priority, 1);
        }
    }

    mod event_filtering {
        use super::*;

        #[test]
        fn test_filter_by_threshold() {
            let detector = EventDetector::new();
            let events = vec![
                create_event_with_priority(1),
                create_event_with_priority(3),
                create_event_with_priority(5),
            ];

            let filtered = detector.filter_by_threshold(&events, 3);

            assert_eq!(filtered.len(), 2);
            assert!(filtered.iter().all(|e| e.priority >= 3));
        }
    }
}
```

#### Async Test Patterns
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lcu_connection() {
        let client = LcuClient::new();

        let result = client.connect().await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_stream() {
        let monitor = LiveClientMonitor::new();
        let mut rx = monitor.monitor_events().await;

        // Test with timeout
        let event = tokio::time::timeout(
            Duration::from_secs(5),
            rx.recv()
        ).await;

        assert!(event.is_ok());
    }
}
```

#### Mock and Test Fixtures
```rust
#[cfg(test)]
mod tests {
    use mockall::predicate::*;
    use mockall::mock;

    // Define mock
    mock! {
        pub Database {
            async fn insert_clip(&self, clip: &ClipMetadata) -> Result<i64>;
            async fn get_clips(&self, game_id: i64) -> Result<Vec<Clip>>;
        }
    }

    #[tokio::test]
    async fn test_clip_manager_save() {
        let mut mock_db = MockDatabase::new();
        mock_db.expect_insert_clip()
            .with(predicate::always())
            .times(1)
            .returning(|_| Ok(1));

        let manager = ClipManager::new(mock_db);
        let result = manager.save_clip(&create_test_clip()).await;

        assert!(result.is_ok());
    }
}
```

### Performance Optimization

#### Avoid Unnecessary Clones
```rust
// ✅ Good: Borrow when possible
pub fn calculate_score(&self, events: &[GameEvent]) -> f64 {
    events.iter()
        .map(|e| self.score_event(e))
        .sum()
}

// ❌ Bad: Unnecessary clone
pub fn calculate_score(&self, events: &[GameEvent]) -> f64 {
    events.clone() // ❌ Expensive
        .iter()
        .map(|e| self.score_event(e))
        .sum()
}
```

#### Use Iterators Instead of Loops
```rust
// ✅ Good: Iterator chain
pub fn get_high_priority_clips(&self, clips: &[Clip]) -> Vec<Clip> {
    clips.iter()
        .filter(|c| c.priority >= 3)
        .cloned()
        .collect()
}

// ❌ Acceptable but less idiomatic
pub fn get_high_priority_clips(&self, clips: &[Clip]) -> Vec<Clip> {
    let mut result = Vec::new();
    for clip in clips {
        if clip.priority >= 3 {
            result.push(clip.clone());
        }
    }
    result
}
```

#### Parallel Processing
```rust
use rayon::prelude::*;

// ✅ Good: Parallel processing for CPU-bound tasks
pub fn analyze_clips(&self, clips: Vec<ClipPath>) -> Vec<ClipAnalysis> {
    clips.par_iter()
        .map(|clip| self.analyze_single_clip(clip))
        .collect()
}
```

---

## ⚛️ React/TypeScript Frontend Guidelines

### Code Style

#### Component Structure
```tsx
// ✅ Good: Consistent component structure
interface DashboardProps {
  user: User;
  onGameSelect: (gameId: string) => void;
}

export function Dashboard({ user, onGameSelect }: DashboardProps) {
  // Hooks at top
  const [games, setGames] = useState<Game[]>([]);
  const { lcuStatus } = useRecordingStore();

  // Effects
  useEffect(() => {
    loadRecentGames();
  }, []);

  // Event handlers
  const handleGameClick = useCallback((gameId: string) => {
    onGameSelect(gameId);
  }, [onGameSelect]);

  // Helper functions
  const loadRecentGames = async () => {
    const result = await invoke<Game[]>('get_recent_games');
    setGames(result);
  };

  // Render
  return (
    <div className="p-6 space-y-6">
      <StatusCard status={lcuStatus} />
      <GameList games={games} onGameClick={handleGameClick} />
    </div>
  );
}
```

#### Naming Conventions
```typescript
// ✅ Good
interface User { }              // PascalCase for types/interfaces
type UserId = string;            // PascalCase for type aliases
const MAX_CLIPS = 20;            // SCREAMING_SNAKE_CASE for constants
function calculateScore() { }    // camelCase for functions
const userName = "John";         // camelCase for variables

// Component names match filename
// Dashboard.tsx exports Dashboard component
```

### TypeScript Best Practices

#### Type Safety
```typescript
// ✅ Good: Explicit types
interface GameEvent {
  eventId: number;
  eventName: string;
  eventTime: number;
  killerName?: string;
  victimName?: string;
}

async function getGameEvents(gameId: string): Promise<GameEvent[]> {
  return invoke<GameEvent[]>('get_game_events', { gameId });
}

// ❌ Bad: any types
async function getGameEvents(gameId: any): Promise<any> {
  return invoke('get_game_events', { gameId });
}
```

#### Union Types and Type Guards
```typescript
// ✅ Good: Discriminated unions
type LicenseTier = 'FREE' | 'PRO';

interface License {
  tier: LicenseTier;
  expiresAt?: Date;
}

function isProUser(license: License): boolean {
  return license.tier === 'PRO' &&
         (!license.expiresAt || license.expiresAt > new Date());
}

// ✅ Good: Type guards
function isGameEvent(obj: unknown): obj is GameEvent {
  return typeof obj === 'object' &&
         obj !== null &&
         'eventId' in obj &&
         'eventName' in obj;
}
```

#### Avoid Type Assertions
```typescript
// ✅ Good: Validate and type guard
async function loadGame(gameId: string): Promise<Game> {
  const data = await invoke('get_game', { gameId });

  if (!isGame(data)) {
    throw new Error('Invalid game data');
  }

  return data;
}

// ❌ Bad: Type assertion without validation
async function loadGame(gameId: string): Promise<Game> {
  const data = await invoke('get_game', { gameId });
  return data as Game; // ❌ Unsafe
}
```

### React Hooks

#### Custom Hooks
```typescript
// ✅ Good: Custom hook encapsulates logic
function useRecordingStatus() {
  const [status, setStatus] = useState<RecordingStatus>('idle');
  const [currentGame, setCurrentGame] = useState<Game | null>(null);

  useEffect(() => {
    const unlisten = listen<RecordingStatus>('recording-status', (event) => {
      setStatus(event.payload);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  const startRecording = useCallback(async () => {
    await invoke('start_recording');
    setStatus('recording');
  }, []);

  return { status, currentGame, startRecording };
}

// Usage
function RecordingMonitor() {
  const { status, currentGame, startRecording } = useRecordingStatus();

  return (
    <div>
      <Badge>{status}</Badge>
      {currentGame && <GameInfo game={currentGame} />}
      <Button onClick={startRecording}>Start</Button>
    </div>
  );
}
```

#### useEffect Best Practices
```typescript
// ✅ Good: Cleanup function
useEffect(() => {
  const interval = setInterval(() => {
    checkRecordingStatus();
  }, 1000);

  return () => clearInterval(interval);
}, []);

// ✅ Good: Proper dependencies
useEffect(() => {
  if (gameId) {
    loadGameData(gameId);
  }
}, [gameId]); // Include all used variables

// ❌ Bad: Missing dependencies
useEffect(() => {
  if (gameId) {
    loadGameData(gameId);
  }
}, []); // ❌ gameId should be in deps
```

#### useMemo and useCallback
```typescript
// ✅ Good: Memoize expensive calculations
const sortedClips = useMemo(() => {
  return clips
    .slice()
    .sort((a, b) => b.priority - a.priority);
}, [clips]);

// ✅ Good: Memoize callbacks passed to children
const handleClipClick = useCallback((clipId: string) => {
  navigate(`/clips/${clipId}`);
}, [navigate]);

// ❌ Bad: Unnecessary memoization
const trivialValue = useMemo(() => clips.length, [clips]); // ❌ Too simple
```

### State Management (Zustand)

#### Store Structure
```typescript
// src/stores/recordingStore.ts
interface RecordingStore {
  // State
  status: RecordingStatus;
  currentGame: Game | null;
  clips: Clip[];

  // Actions
  setStatus: (status: RecordingStatus) => void;
  setCurrentGame: (game: Game | null) => void;
  addClip: (clip: Clip) => void;
  clearClips: () => void;

  // Async actions
  loadClips: (gameId: string) => Promise<void>;
}

export const useRecordingStore = create<RecordingStore>((set, get) => ({
  // Initial state
  status: 'idle',
  currentGame: null,
  clips: [],

  // Sync actions
  setStatus: (status) => set({ status }),
  setCurrentGame: (game) => set({ currentGame: game }),
  addClip: (clip) => set((state) => ({ clips: [...state.clips, clip] })),
  clearClips: () => set({ clips: [] }),

  // Async actions
  loadClips: async (gameId) => {
    try {
      const clips = await invoke<Clip[]>('get_clips', { gameId });
      set({ clips });
    } catch (error) {
      console.error('Failed to load clips:', error);
    }
  },
}));
```

#### Derived State with Selectors
```typescript
// ✅ Good: Use selectors for derived state
const highPriorityClips = useRecordingStore(
  (state) => state.clips.filter(c => c.priority >= 3)
);

// ✅ Good: Shallow equality for multiple values
const { status, currentGame } = useRecordingStore(
  (state) => ({ status: state.status, currentGame: state.currentGame }),
  shallow
);
```

### Tauri Integration

#### Invoke Commands
```typescript
// ✅ Good: Type-safe invocations
async function startRecording(): Promise<void> {
  try {
    await invoke<void>('start_recording');
  } catch (error) {
    console.error('Failed to start recording:', error);
    throw error;
  }
}

async function getClips(gameId: string): Promise<Clip[]> {
  return invoke<Clip[]>('get_clips', { gameId });
}

// ✅ Good: Wrap in custom hooks
function useRecordingCommand() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const start = async () => {
    setIsLoading(true);
    setError(null);

    try {
      await invoke('start_recording');
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  return { start, isLoading, error };
}
```

#### Event Listeners
```typescript
// ✅ Good: Setup listener with cleanup
useEffect(() => {
  let unlisten: (() => void) | undefined;

  listen<GameEvent>('game-event', (event) => {
    console.log('Received event:', event.payload);
    addEventToFeed(event.payload);
  }).then(fn => {
    unlisten = fn;
  });

  return () => {
    unlisten?.();
  };
}, []);
```

### UI Components

#### shadcn/ui Usage
```tsx
// ✅ Good: Compose shadcn components
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';

export function ClipCard({ clip }: { clip: Clip }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <span>{clip.eventType}</span>
          <Badge variant={getPriorityVariant(clip.priority)}>
            {clip.priority} ⭐
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <video src={clip.filePath} className="w-full rounded" />
        <div className="flex gap-2 mt-4">
          <Button variant="outline" size="sm">Play</Button>
          <Button variant="outline" size="sm">Edit</Button>
        </div>
      </CardContent>
    </Card>
  );
}
```

#### Conditional Rendering
```tsx
// ✅ Good: Early return for loading/error states
if (isLoading) {
  return <LoadingSpinner />;
}

if (error) {
  return <ErrorAlert message={error} />;
}

return <ActualContent data={data} />;

// ✅ Good: Ternary for simple conditions
{isRecording ? (
  <Badge variant="destructive">Recording</Badge>
) : (
  <Badge variant="secondary">Idle</Badge>
)}

// ✅ Good: Logical AND for optional rendering
{currentGame && <GameInfo game={currentGame} />}
```

---

## 🎨 Styling Guidelines

### Tailwind CSS

#### Class Organization
```tsx
// ✅ Good: Logical grouping with line breaks
<div className="
  flex items-center justify-between
  p-6 space-y-4
  bg-background border rounded-lg
  hover:bg-accent transition-colors
">
```

#### Use CSS Variables
```css
/* globals.css */
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  --primary: 47 70% 64%; /* LoL Gold */
}

.dark {
  --background: 222.2 84% 4.9%;
  --foreground: 210 40% 98%;
}
```

#### Component-Specific Styles
```tsx
// ✅ Good: Use cn() for conditional classes
import { cn } from '@/lib/utils';

<Button
  className={cn(
    "w-full",
    isPro && "bg-gradient-to-r from-yellow-400 to-yellow-600",
    disabled && "opacity-50 cursor-not-allowed"
  )}
>
```

---

## 🔒 Security Guidelines

### Input Validation

#### Backend Validation
```rust
// ✅ Good: Validate all inputs
#[tauri::command]
async fn save_clip(game_id: i64, file_path: String) -> Result<(), String> {
    // Validate game_id
    if game_id <= 0 {
        return Err("Invalid game ID".to_string());
    }

    // Validate file path
    let path = PathBuf::from(&file_path);
    if !path.exists() || !path.is_file() {
        return Err("Invalid file path".to_string());
    }

    // Prevent path traversal
    if file_path.contains("..") {
        return Err("Invalid file path".to_string());
    }

    // Proceed with save
    save_clip_internal(game_id, &path).await
        .map_err(|e| e.to_string())
}
```

#### Frontend Validation
```typescript
// ✅ Good: Validate before invoking
async function saveTemplate(name: string, data: TemplateData) {
  // Validate name
  if (!name || name.length > 100) {
    throw new Error('Invalid template name');
  }

  // Validate data
  if (!isValidTemplateData(data)) {
    throw new Error('Invalid template data');
  }

  await invoke('save_template', { name, data });
}
```

### Sensitive Data

#### Never Log Secrets
```rust
// ✅ Good: Sanitize logs
tracing::info!("Connecting to LCU at port {}", self.port);
// Password is NOT logged

// ❌ Bad: Logging sensitive data
tracing::info!("LCU credentials: {}:{}", self.username, self.password); // ❌
```

#### Secure Storage
```rust
// ✅ Good: Use Windows Credential Manager for tokens
use windows::Security::Credentials::PasswordVault;

pub fn store_token(service: &str, token: &str) -> Result<()> {
    let vault = PasswordVault::new()?;
    let credential = PasswordCredential::new(service, "user", token)?;
    vault.Add(&credential)?;
    Ok(())
}
```

---

## 📊 Performance Guidelines

### Profiling

#### Rust Profiling
```bash
# CPU profiling
cargo install flamegraph
cargo flamegraph --bin lolshorts-tauri

# Memory profiling
cargo install valgrind
valgrind --tool=massif target/release/lolshorts-tauri
```

#### Frontend Profiling
```typescript
// Use React DevTools Profiler
import { Profiler } from 'react';

<Profiler id="VideoEditor" onRender={onRenderCallback}>
  <VideoEditor />
</Profiler>

function onRenderCallback(
  id: string,
  phase: "mount" | "update",
  actualDuration: number
) {
  console.log(`${id} ${phase} took ${actualDuration}ms`);
}
```

### Performance Targets
- **App Startup**: <3s cold start
- **LCU Connection**: <2s
- **Event Detection**: <500ms latency
- **Video Processing**: <30s per minute of footage
- **Memory Usage**: <500MB idle, <2GB during processing

---

## 📝 Documentation

### Code Comments

#### When to Comment
```rust
// ✅ Good: Explain WHY, not WHAT
// Throttle clip saves to prevent excessive disk I/O
// and avoid triggering Windows Game DVR rate limits
if now.duration_since(last_save) < MIN_INTERVAL {
    return Err(AppError::TooFrequent);
}

// ❌ Bad: Obvious comment
// Check if duration is less than minimum interval
if now.duration_since(last_save) < MIN_INTERVAL {
```

#### Function Documentation
```rust
/// Calculates the priority score for a game event.
///
/// Priority is calculated based on event type and context:
/// - Pentakill: 5
/// - Quadrakill: 4
/// - Baron/Dragon: 3-4
/// - Regular kills: 1-2
///
/// # Arguments
/// * `event` - The game event to score
///
/// # Returns
/// Priority score from 1 (lowest) to 5 (highest)
///
/// # Examples
/// ```
/// let event = GameEvent { event_name: "ChampionKill", ... };
/// let priority = calculate_priority(&event);
/// assert!(priority >= 1 && priority <= 5);
/// ```
pub fn calculate_priority(event: &GameEvent) -> u8 {
    // Implementation
}
```

### Commit Messages

#### Format
```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Types
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `test`: Add/update tests
- `docs`: Documentation changes
- `perf`: Performance improvements
- `chore`: Maintenance tasks

#### Examples
```bash
# Good
feat(recording): add pentakill detection with 10s time window

Implements kill sequence tracking to detect multi-kills.
Uses sliding time window of 10 seconds to group consecutive
kills by the same player.

Closes #42

# Good
fix(video): prevent FFmpeg zombie processes

Ensures FFmpeg child processes are properly terminated
when video generation is cancelled.

# Bad
fix: stuff
```

---

## 🧪 Testing Strategy

### Test Coverage Goals
- **Unit Tests**: >80% coverage (backend)
- **Integration Tests**: Critical paths covered
- **E2E Tests**: Major user workflows

### Test Pyramid
```
        E2E Tests (10%)
      /              \
     /                \
    /                  \
   / Integration (30%)  \
  /______________________\
 /                        \
/   Unit Tests (60%)       \
```

### TDD Workflow
1. **Write failing test** - Define expected behavior
2. **Write minimal code** - Make test pass
3. **Refactor** - Improve code quality
4. **Commit** - Save working state
5. **Repeat** - Next feature

### Example TDD Session
```rust
// Step 1: Write failing test
#[test]
fn test_filter_clips_by_priority() {
    let clips = vec![
        create_clip(1), // priority 1
        create_clip(3), // priority 3
        create_clip(5), // priority 5
    ];

    let filtered = filter_by_priority(&clips, 3);

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|c| c.priority >= 3));
}

// Step 2: Run test (fails)
// $ cargo test
// test test_filter_clips_by_priority ... FAILED

// Step 3: Implement minimal code
pub fn filter_by_priority(clips: &[Clip], threshold: u8) -> Vec<Clip> {
    clips.iter()
        .filter(|c| c.priority >= threshold)
        .cloned()
        .collect()
}

// Step 4: Run test (passes)
// $ cargo test
// test test_filter_clips_by_priority ... ok

// Step 5: Refactor if needed

// Step 6: Commit
// $ git add . && git commit -m "feat(video): add clip priority filtering"
```

---

## 🚀 Deployment

### Build Process
```bash
# Development build
cargo tauri dev

# Production build
cargo tauri build

# Build for specific target
cargo tauri build --target x86_64-pc-windows-msvc
```

### Pre-Release Checklist
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] No ESLint errors
- [ ] Performance benchmarks met
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped (Cargo.toml, package.json, tauri.conf.json)

### Versioning
Follow Semantic Versioning (SemVer):
- **Major** (1.0.0): Breaking changes
- **Minor** (0.1.0): New features (backward compatible)
- **Patch** (0.0.1): Bug fixes

---

## 🆘 Troubleshooting

### Common Issues

#### FFmpeg Not Found
```bash
# Windows: Add to PATH
setx PATH "%PATH%;C:\ffmpeg\bin"

# Or set in tauri.conf.json
{
  "tauri": {
    "bundle": {
      "externalBin": ["ffmpeg.exe"]
    }
  }
}
```

#### Rust Compilation Errors
```bash
# Clear cache and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

#### Frontend Hot Reload Not Working
```bash
# Clear Vite cache
rm -rf node_modules/.vite
pnpm dev
```

---

## 📚 Resources

### Rust
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tauri Docs](https://tauri.app/v2/guides/)

### React/TypeScript
- [React Docs](https://react.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [shadcn/ui](https://ui.shadcn.com/)

### FFmpeg
- [FFmpeg Documentation](https://ffmpeg.org/documentation.html)
- [FFmpeg Filters](https://ffmpeg.org/ffmpeg-filters.html)

### League of Legends APIs
- [LCU API](https://hextechdocs.dev/lol/lcu/)
- [Live Client Data API](https://developer.riotgames.com/docs/lol#game-client-api)

---

**Remember**: Code quality is not optional. Every feature must be:
- ✅ Tested (TDD)
- ✅ Documented
- ✅ Performant
- ✅ Secure
- ✅ Maintainable

**When in doubt**: Write a test first, make it pass, refactor, commit.
