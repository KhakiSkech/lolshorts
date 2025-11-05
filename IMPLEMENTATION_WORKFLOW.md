# LoLShorts Implementation Workflow

자동 하이라이트 클립 녹화 시스템 구현을 위한 단계별 워크플로우

---

## 🎯 Overall Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     User Configuration                      │
│  (Settings UI: Event Filters, Game Modes, Video/Audio)     │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                   LCU Client Monitor                        │
│  • Auto-connect every 3s                                    │
│  • Detect game start/end                                    │
│  • Get game mode and champion info                          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Live Client Event Monitor                      │
│  • Poll Live Client API (http://127.0.0.1:2999)            │
│  • Detect events: kills, dragons, barons, etc.             │
│  • Calculate event priority (1-5)                           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                 Event Window Manager                        │
│  • Merge consecutive events (15s threshold)                 │
│  • Track event sequences (pentakill detection)              │
│  • Determine clip boundaries (pre/post duration)            │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                  Auto Clip Manager                          │
│  • Check if event should be recorded (settings filter)      │
│  • Trigger clip save from circular buffer                   │
│  • Generate V2 metadata with rich context                   │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│           Recording System (Circular Buffer)                │
│  • 60-second rolling buffer (6 × 10s segments)             │
│  • FFmpeg H.265 hardware encoding                           │
│  • Multi-track audio (mic + system)                         │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              Storage Layer (V2 Metadata)                    │
│  • Save video: clip_xxx.mp4                                 │
│  • Save metadata: clip_xxx.json (rich context)              │
│  • Save thumbnail: clip_xxx.jpg                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 Implementation Phases

### Phase 0: Foundation Setup ✅ (Current Phase)

**Objective**: Prepare infrastructure for implementation

**Tasks**:
- [x] Design clip metadata V2 schema
- [x] Design recording settings structure
- [x] Design editor integration workflow
- [x] Create implementation roadmap
- [ ] Review existing codebase architecture
- [ ] Identify integration points

**Deliverables**:
- ✅ CLIP_METADATA_DESIGN.md
- ✅ RECORDING_SETTINGS_DESIGN.md
- ✅ EDITOR_INTEGRATION_WORKFLOW.md
- ✅ IMPLEMENTATION_WORKFLOW.md (this file)

**Status**: 90% Complete (this document completes Phase 0)

---

### Phase 1: Settings System Backend

**Objective**: Implement settings storage and management in Rust

**Duration**: 2-3 days

**Dependencies**: Phase 0 complete

#### Tasks

##### 1.1: Create Settings Module Structure
```
src-tauri/src/settings/
├── mod.rs                    # Public API
├── models.rs                 # Settings structures
├── commands.rs               # Tauri commands
└── storage.rs                # File I/O
```

**Files to Create**:
- `src-tauri/src/settings/mod.rs`
- `src-tauri/src/settings/models.rs`
- `src-tauri/src/settings/commands.rs`
- `src-tauri/src/settings/storage.rs`

**Implementation Details**:
```rust
// src-tauri/src/settings/models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub event_filter: EventFilterSettings,
    pub game_mode: GameModeSettings,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub clip_timing: ClipTimingSettings,
    pub hotkeys: HotkeySettings,
    pub auto_start_with_league: bool,
    pub minimize_to_tray: bool,
    pub show_notifications: bool,
}

// Implement Default, load(), save(), should_record_event(), etc.
```

##### 1.2: Implement Settings Commands
**File**: `src-tauri/src/settings/commands.rs`

```rust
#[tauri::command]
pub async fn get_recording_settings() -> Result<RecordingSettings, String> { }

#[tauri::command]
pub async fn save_recording_settings(settings: RecordingSettings) -> Result<(), String> { }

#[tauri::command]
pub async fn reset_settings_to_default() -> Result<RecordingSettings, String> { }

#[tauri::command]
pub async fn get_audio_devices() -> Result<AudioDevices, String> { }
```

##### 1.3: Register Commands in main.rs
**File**: `src-tauri/src/main.rs`

```rust
mod settings;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands
            settings::commands::get_recording_settings,
            settings::commands::save_recording_settings,
            settings::commands::reset_settings_to_default,
            settings::commands::get_audio_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Testing**:
```bash
# Unit tests
cargo test --package lolshorts-tauri --lib settings

# Integration test
cargo tauri dev
# Call get_recording_settings from frontend
```

**Success Criteria**:
- ✅ Settings can be loaded/saved to JSON file
- ✅ Default settings work correctly
- ✅ All Tauri commands respond properly
- ✅ Settings persist across app restarts

---

### Phase 2: Settings UI

**Objective**: Build React UI for configuring recording settings

**Duration**: 3-4 days

**Dependencies**: Phase 1 complete

#### Tasks

##### 2.1: Create Settings Page Structure
**Files to Create**:
- `src/pages/Settings.tsx` (main page)
- `src/components/settings/EventFilterSettings.tsx`
- `src/components/settings/GameModeSettings.tsx`
- `src/components/settings/VideoSettings.tsx`
- `src/components/settings/AudioSettings.tsx`
- `src/components/settings/ClipTimingSettings.tsx`

**Implementation**:
```tsx
// src/pages/Settings.tsx
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { EventFilterSettings } from "@/components/settings/EventFilterSettings";
import { GameModeSettings } from "@/components/settings/GameModeSettings";
// ... other imports

export function Settings() {
  const [settings, setSettings] = useState<RecordingSettings | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    const settings = await invoke<RecordingSettings>("get_recording_settings");
    setSettings(settings);
  };

  const saveSettings = async () => {
    await invoke("save_recording_settings", { settings });
    // Show success toast
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-3xl font-bold">Settings</h1>
        <Button onClick={saveSettings}>Save Settings</Button>
      </div>

      <Tabs defaultValue="events">
        <TabsList>
          <TabsTrigger value="events">이벤트</TabsTrigger>
          <TabsTrigger value="modes">게임 모드</TabsTrigger>
          <TabsTrigger value="video">영상</TabsTrigger>
          <TabsTrigger value="audio">오디오</TabsTrigger>
          <TabsTrigger value="timing">타이밍</TabsTrigger>
        </TabsList>

        <TabsContent value="events">
          <EventFilterSettings
            settings={settings.event_filter}
            onChange={(eventFilter) =>
              setSettings({ ...settings, event_filter: eventFilter })
            }
          />
        </TabsContent>
        {/* ... other tabs */}
      </Tabs>
    </div>
  );
}
```

##### 2.2: Implement Event Filter Component
**File**: `src/components/settings/EventFilterSettings.tsx`

**Features**:
- Toggle switches for each event type
- Grouped by category (킬, 데스, 오브젝트, etc.)
- Priority slider (1-5)
- Visual feedback on selection

##### 2.3: Add Settings Route
**File**: `src/App.tsx`

```tsx
import { Settings } from "@/pages/Settings";

// Add route
<Route path="/settings" element={<Settings />} />

// Add navigation link in sidebar/header
<Link to="/settings">Settings</Link>
```

**Testing**:
- [ ] All toggles work correctly
- [ ] Settings save and load properly
- [ ] UI reflects current settings state
- [ ] Changes persist across app restarts
- [ ] Responsive design works on different screen sizes

**Success Criteria**:
- ✅ User can configure all event filters
- ✅ User can configure game modes
- ✅ User can configure video/audio settings
- ✅ Settings save successfully
- ✅ Settings load on app startup

---

### Phase 3: Event Detection Integration

**Objective**: Connect Live Client API monitoring to recording system

**Duration**: 3-4 days

**Dependencies**: Phase 1 complete

#### Tasks

##### 3.1: Enhance Live Client Monitor
**File**: `src-tauri/src/recording/live_client.rs`

**Current State**: Event detection code exists but not connected

**Changes Needed**:
```rust
pub struct LiveClientMonitor {
    client: reqwest::Client,
    base_url: String,
    event_sender: mpsc::Sender<GameEvent>,  // NEW: Send events to manager
    last_event_id: Arc<RwLock<u64>>,
    settings: Arc<RwLock<RecordingSettings>>, // NEW: Settings integration
}

impl LiveClientMonitor {
    /// Start monitoring game events
    pub async fn start_monitoring(&self) -> mpsc::Receiver<GameEvent> {
        let (tx, rx) = mpsc::channel(100);

        let monitor = self.clone();
        tokio::spawn(async move {
            loop {
                // Poll events every 1 second
                match monitor.fetch_events().await {
                    Ok(events) => {
                        for event in events {
                            // Check if event should be recorded
                            let settings = monitor.settings.read().await;
                            if settings.should_record_event(&event.event_type) {
                                tx.send(event).await.ok();
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to fetch events: {}", e);
                    }
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        rx
    }

    /// Fetch events from Live Client API
    async fn fetch_events(&self) -> Result<Vec<GameEvent>> {
        let url = format!("{}/eventdata", self.base_url);
        let response = self.client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;

        // Parse events and filter new ones
        let events = self.parse_events(&data).await?;

        Ok(events)
    }
}
```

##### 3.2: Implement Event Priority Calculation
**File**: `src-tauri/src/recording/event_priority.rs` (new file)

```rust
use super::live_client::{GameEvent, EventType};

pub struct EventPriorityCalculator {
    kill_sequence: Vec<(String, f64)>, // (killer, timestamp)
}

impl EventPriorityCalculator {
    /// Calculate priority for an event
    pub fn calculate_priority(&mut self, event: &GameEvent) -> u8 {
        match &event.event_type {
            EventType::ChampionKill => self.calculate_kill_priority(event),
            EventType::DragonKill => 3,
            EventType::BaronKill => 4,
            EventType::Ace => 4,
            EventType::FirstBlood => 3,
            _ => 1,
        }
    }

    /// Check for multikills based on recent kill sequence
    fn calculate_kill_priority(&mut self, event: &GameEvent) -> u8 {
        // Detect double, triple, quadra, penta kills
        // based on 10-second time window

        let now = event.timestamp;
        let killer = event.killer_name.as_ref().unwrap();

        // Find recent kills by same player
        let recent_kills: Vec<_> = self.kill_sequence
            .iter()
            .filter(|(k, t)| k == killer && (now - t) <= 10.0)
            .collect();

        let kill_count = recent_kills.len() + 1;

        // Add this kill to sequence
        self.kill_sequence.push((killer.clone(), now));

        // Calculate priority based on multikill
        match kill_count {
            1 => 1,  // Single kill
            2 => 2,  // Double kill
            3 => 3,  // Triple kill
            4 => 4,  // Quadra kill
            5.. => 5, // Penta kill
            _ => 1,
        }
    }
}
```

##### 3.3: Test Event Detection
**Testing Scenarios**:
1. **Kill Detection**: Enter practice mode, kill a bot, verify event detected
2. **Dragon Detection**: Kill dragon, verify event with correct priority
3. **Multikill Detection**: Get double/triple kill, verify priority calculation
4. **Settings Filtering**: Disable kills in settings, verify no events sent

**Success Criteria**:
- ✅ Events detected from Live Client API
- ✅ Event priority calculated correctly
- ✅ Multikill detection works (pentakill = priority 5)
- ✅ Settings filter applied correctly
- ✅ Events sent to next stage (clip manager)

---

### Phase 4: Auto Clip Manager

**Objective**: Implement event window merging and automatic clip saving

**Duration**: 4-5 days

**Dependencies**: Phase 3 complete

#### Tasks

##### 4.1: Create Event Window Manager
**File**: `src-tauri/src/recording/event_window.rs` (new file)

```rust
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// Manages event windows and merging logic
pub struct EventWindowManager {
    pending_events: VecDeque<GameEvent>,
    merge_threshold: f64, // 15 seconds default
    settings: Arc<RwLock<ClipTimingSettings>>,
}

impl EventWindowManager {
    /// Add an event and check if window should be closed
    pub fn add_event(&mut self, event: GameEvent) -> Option<EventWindow> {
        self.pending_events.push_back(event.clone());

        // Check if we should close the window
        if self.should_close_window(&event) {
            return Some(self.create_window());
        }

        None
    }

    /// Check if enough time has passed to close window
    fn should_close_window(&self, new_event: &GameEvent) -> bool {
        if let Some(first) = self.pending_events.front() {
            let time_diff = new_event.timestamp - first.timestamp;
            time_diff > self.merge_threshold
        } else {
            false
        }
    }

    /// Create an event window from pending events
    fn create_window(&mut self) -> EventWindow {
        let events: Vec<_> = self.pending_events.drain(..).collect();

        let primary_event = events.iter()
            .max_by_key(|e| e.priority)
            .unwrap()
            .clone();

        let merged_events: Vec<_> = events.into_iter()
            .filter(|e| e.event_id != primary_event.event_id)
            .collect();

        let settings = self.settings.read().await;
        let timing = settings.get_timing_for_event(&primary_event.event_type);

        EventWindow {
            primary_event,
            merged_events,
            merge_strategy: MergeStrategy::ConsecutiveEvents,
            time_threshold_secs: self.merge_threshold,
            events_merged: merged_events.len() + 1,
            pre_duration: timing.pre_duration as f64,
            post_duration: timing.post_duration as f64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EventWindow {
    pub primary_event: GameEvent,
    pub merged_events: Vec<GameEvent>,
    pub merge_strategy: MergeStrategy,
    pub time_threshold_secs: f64,
    pub events_merged: usize,
    pub pre_duration: f64,
    pub post_duration: f64,
}
```

##### 4.2: Create Auto Clip Manager
**File**: `src-tauri/src/recording/auto_clip_manager.rs` (new file)

```rust
use super::event_window::{EventWindow, EventWindowManager};
use super::windows_backend::WindowsRecorder;
use crate::storage::{ClipMetadataV2, Storage};

pub struct AutoClipManager {
    event_window_manager: EventWindowManager,
    recorder: Arc<WindowsRecorder>,
    storage: Arc<Storage>,
    settings: Arc<RwLock<RecordingSettings>>,
}

impl AutoClipManager {
    /// Process incoming game events
    pub async fn process_event(&mut self, event: GameEvent) -> Result<()> {
        // Add to event window
        if let Some(window) = self.event_window_manager.add_event(event) {
            // Window closed, save clip
            self.save_clip_from_window(window).await?;
        }

        Ok(())
    }

    /// Save a clip based on event window
    async fn save_clip_from_window(&self, window: EventWindow) -> Result<PathBuf> {
        let game = self.recorder.current_game.read().await;
        let game_id = game.as_ref()
            .map(|g| g.game_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        drop(game);

        // Calculate clip duration
        let clip_duration = window.pre_duration + window.post_duration;

        // Generate clip ID
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let event_name = format!("{:?}", window.primary_event.event_type)
            .to_lowercase();
        let clip_id = format!("clip_{}_{}", timestamp, event_name);

        // Save video clip using existing recorder
        let clip_path = self.recorder.save_clip(
            &window.primary_event,
            clip_id.clone(),
            window.primary_event.priority,
            clip_duration,
        ).await?;

        // Create V2 metadata
        let metadata = self.create_clip_metadata_v2(
            &clip_id,
            &game_id,
            &clip_path,
            &window,
        ).await?;

        // Save V2 metadata
        self.storage.save_clip_metadata_v2(&game_id, &metadata)?;

        tracing::info!("Auto-saved clip: {} (priority {})", clip_id, metadata.priority);

        Ok(clip_path)
    }

    /// Create comprehensive V2 metadata for clip
    async fn create_clip_metadata_v2(
        &self,
        clip_id: &str,
        game_id: &str,
        clip_path: &Path,
        window: &EventWindow,
    ) -> Result<ClipMetadataV2> {
        let settings = self.settings.read().await;

        let metadata = ClipMetadataV2 {
            clip_id: clip_id.to_string(),
            game_id: game_id.to_string(),
            file_path: clip_path.to_string_lossy().to_string(),
            thumbnail_path: None, // TODO: Generate thumbnail

            created_at: Utc::now(),
            game_time_start: window.primary_event.timestamp - window.pre_duration,
            game_time_end: window.primary_event.timestamp + window.post_duration,
            clip_duration: window.pre_duration + window.post_duration,

            primary_event: self.convert_to_event_info(&window.primary_event, window.pre_duration),
            merged_events: window.merged_events.iter()
                .map(|e| self.convert_to_event_info(e, window.pre_duration))
                .collect(),
            event_window: window.clone().into(),

            priority: window.primary_event.priority,
            tags: self.generate_tags(&window),

            video_info: self.get_video_info(&settings.video),
            audio_info: self.get_audio_info(&settings.audio),
            timeline: self.generate_timeline(&window),
            game_context: self.get_game_context().await?,
            annotations: None,
        };

        Ok(metadata)
    }

    /// Generate timeline markers for editor
    fn generate_timeline(&self, window: &EventWindow) -> ClipTimeline {
        let mut markers = Vec::new();

        // Add marker for primary event
        markers.push(TimelineMarker {
            timestamp: window.pre_duration, // Event at center
            marker_type: MarkerType::EventPeak,
            label: format!("{:?}", window.primary_event.event_type),
            color: Some("#FFD700".to_string()),
        });

        // Add markers for merged events
        for event in &window.merged_events {
            let relative_time = event.timestamp - window.primary_event.timestamp + window.pre_duration;
            markers.push(TimelineMarker {
                timestamp: relative_time,
                marker_type: MarkerType::EventStart,
                label: format!("{:?}", event.event_type),
                color: Some("#00FF00".to_string()),
            });
        }

        ClipTimeline {
            markers,
            chapters: vec![],
        }
    }
}
```

##### 4.3: Integrate with Recording Manager
**File**: `src-tauri/src/recording/mod.rs`

```rust
pub mod auto_clip_manager;
pub mod event_window;

use auto_clip_manager::AutoClipManager;

pub struct RecordingManager {
    // ... existing fields
    auto_clip_manager: Arc<Mutex<AutoClipManager>>,
}

impl RecordingManager {
    pub async fn start(&self) -> Result<()> {
        // ... existing recording start logic

        // Start event monitoring
        let mut event_rx = self.live_client_monitor.start_monitoring().await;

        // Process events
        let clip_manager = self.auto_clip_manager.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                if let Err(e) = clip_manager.lock().await.process_event(event).await {
                    tracing::error!("Failed to process event: {}", e);
                }
            }
        });

        Ok(())
    }
}
```

**Testing**:
1. **Single Event**: Kill a champion, verify clip saved with correct duration
2. **Merged Events**: Get kills within 15s, verify merged into single clip
3. **Priority-Based**: Disable low-priority events, verify not saved
4. **Metadata**: Verify V2 JSON created with all fields

**Success Criteria**:
- ✅ Events merged correctly (15s threshold)
- ✅ Clips saved automatically when events occur
- ✅ V2 metadata generated with rich context
- ✅ Timeline markers created for editor
- ✅ Settings filtering applied

---

### Phase 5: V2 Metadata Integration

**Objective**: Ensure V2 metadata flows through entire system

**Duration**: 2-3 days

**Dependencies**: Phase 4 complete

#### Tasks

##### 5.1: Update Recording Commands
**File**: `src-tauri/src/recording/commands.rs`

Add V2 metadata commands:
```rust
#[tauri::command]
pub async fn get_clips_v2(game_id: String) -> Result<Vec<ClipMetadataV2>, String> {
    let storage = get_storage()?;
    storage.load_all_clips_v2(&game_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_clip_metadata_v2(clip_path: String) -> Result<ClipMetadataV2, String> {
    let storage = get_storage()?;
    storage.load_clip_metadata_v2(&clip_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_clip_rating(
    clip_path: String,
    rating: u8,
) -> Result<(), String> {
    let storage = get_storage()?;
    let mut clip = storage.load_clip_metadata_v2(&clip_path)
        .map_err(|e| e.to_string())?;

    clip.set_rating(rating);

    let game_id = ClipMetadataV2::extract_game_id(&clip_path);
    storage.save_clip_metadata_v2(&game_id, &clip)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_clip_favorite(clip_path: String) -> Result<(), String> {
    let storage = get_storage()?;
    let mut clip = storage.load_clip_metadata_v2(&clip_path)
        .map_err(|e| e.to_string())?;

    clip.toggle_favorite();

    let game_id = ClipMetadataV2::extract_game_id(&clip_path);
    storage.save_clip_metadata_v2(&game_id, &clip)
        .map_err(|e| e.to_string())
}
```

##### 5.2: Create TypeScript Types
**File**: `src/lib/types/metadata.ts`

```typescript
export interface ClipMetadataV2 {
  clip_id: string;
  game_id: string;
  file_path: string;
  thumbnail_path?: string;

  created_at: string;
  game_time_start: number;
  game_time_end: number;
  clip_duration: number;

  primary_event: EventInfo;
  merged_events: EventInfo[];
  event_window: EventWindow;

  priority: number;
  tags: string[];

  video_info: VideoInfo;
  audio_info: AudioInfo;
  timeline: ClipTimeline;
  game_context: GameContext;
  annotations?: UserAnnotations;
}

// ... other interfaces
```

##### 5.3: Update Clips Display UI
**File**: `src/pages/Clips.tsx`

```tsx
export function Clips() {
  const [clips, setClips] = useState<ClipMetadataV2[]>([]);

  const loadClips = async () => {
    const gameId = getCurrentGameId();
    const clips = await invoke<ClipMetadataV2[]>('get_clips_v2', { gameId });
    setClips(clips);
  };

  return (
    <div className="grid grid-cols-3 gap-4">
      {clips.map(clip => (
        <ClipCard key={clip.clip_id} clip={clip} />
      ))}
    </div>
  );
}

function ClipCard({ clip }: { clip: ClipMetadataV2 }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center justify-between">
          <span>{clip.primary_event.event_type}</span>
          <Badge>Priority {clip.priority}</Badge>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <video src={clip.file_path} controls className="w-full rounded" />
        <div className="mt-2 flex gap-2">
          {clip.tags.map(tag => (
            <Badge key={tag} variant="secondary">{tag}</Badge>
          ))}
        </div>
        <p className="text-sm text-muted-foreground mt-2">
          {clip.merged_events.length} events merged
        </p>
      </CardContent>
    </Card>
  );
}
```

**Testing**:
- [ ] V2 metadata displayed in UI
- [ ] All metadata fields accessible
- [ ] Rating and favorite updates work
- [ ] Timeline markers visible

**Success Criteria**:
- ✅ V2 metadata fully integrated
- ✅ UI displays rich metadata
- ✅ User can rate and favorite clips
- ✅ All CRUD operations work

---

### Phase 6: Hotkey System

**Objective**: Global hotkey capture for manual clip saving

**Duration**: 2-3 days

**Dependencies**: Phase 4 complete

#### Tasks

##### 6.1: Add Hotkey Dependencies
**File**: `src-tauri/Cargo.toml`

```toml
[dependencies]
global-hotkey = "0.5"
```

##### 6.2: Implement Hotkey Manager
**File**: `src-tauri/src/hotkeys/mod.rs` (new)

```rust
use global_hotkey::{GlobalHotKeyManager, GlobalHotKeyEvent, hotkey::{Code, Modifiers, HotKey}};
use tokio::sync::mpsc;

pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkeys: HashMap<String, HotKey>,
    event_sender: mpsc::Sender<HotkeyEvent>,
}

#[derive(Debug, Clone)]
pub enum HotkeyEvent {
    ManualSaveClip,
    ToggleRecording,
    DeleteLastClip,
}

impl HotkeyManager {
    pub fn new() -> Result<(Self, mpsc::Receiver<HotkeyEvent>)> {
        let manager = GlobalHotKeyManager::new()?;
        let (tx, rx) = mpsc::channel(10);

        // Register default hotkeys
        let f8 = HotKey::new(None, Code::F8);
        let f9 = HotKey::new(None, Code::F9);
        let f10 = HotKey::new(None, Code::F10);

        manager.register(f8)?;
        manager.register(f9)?;
        manager.register(f10)?;

        let mut hotkeys = HashMap::new();
        hotkeys.insert("manual_save".to_string(), f8);
        hotkeys.insert("toggle_recording".to_string(), f9);
        hotkeys.insert("delete_last".to_string(), f10);

        Ok((
            Self {
                manager,
                hotkeys,
                event_sender: tx,
            },
            rx,
        ))
    }

    pub async fn start_listening(&self) {
        let tx = self.event_sender.clone();

        tokio::spawn(async move {
            loop {
                if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
                    match event.id {
                        id if id == self.hotkeys.get("manual_save").unwrap().id() => {
                            tx.send(HotkeyEvent::ManualSaveClip).await.ok();
                        }
                        id if id == self.hotkeys.get("toggle_recording").unwrap().id() => {
                            tx.send(HotkeyEvent::ToggleRecording).await.ok();
                        }
                        id if id == self.hotkeys.get("delete_last").unwrap().id() => {
                            tx.send(HotkeyEvent::DeleteLastClip).await.ok();
                        }
                        _ => {}
                    }
                }

                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }
}
```

##### 6.3: Integrate with Recording Manager
**File**: `src-tauri/src/recording/mod.rs`

```rust
use crate::hotkeys::{HotkeyManager, HotkeyEvent};

impl RecordingManager {
    pub async fn handle_hotkey_event(&self, event: HotkeyEvent) -> Result<()> {
        match event {
            HotkeyEvent::ManualSaveClip => {
                // Save last 30 seconds as manual clip
                self.save_manual_clip(30.0).await?;
                tracing::info!("Manual clip saved via F8");
            }
            HotkeyEvent::ToggleRecording => {
                // Toggle recording on/off
                if self.is_recording().await {
                    self.stop().await?;
                } else {
                    self.start().await?;
                }
            }
            HotkeyEvent::DeleteLastClip => {
                // Delete most recent clip
                self.delete_last_clip().await?;
                tracing::info!("Last clip deleted via F10");
            }
        }

        Ok(())
    }

    async fn save_manual_clip(&self, duration: f64) -> Result<PathBuf> {
        let game = self.current_game.read().await;
        let game_id = game.as_ref()
            .map(|g| g.game_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        drop(game);

        // Create manual event
        let manual_event = GameEvent {
            event_id: 0,
            event_name: "ManualSave".to_string(),
            event_time: chrono::Utc::now().timestamp() as f64,
            // ... other fields
        };

        let clip_id = format!("clip_manual_{}", Utc::now().format("%Y%m%d_%H%M%S"));

        // Save clip
        let clip_path = self.recorder.save_clip(
            &manual_event,
            clip_id.clone(),
            3, // Manual saves get priority 3
            duration,
        ).await?;

        // Create V2 metadata with manual save strategy
        let metadata = ClipMetadataV2 {
            // ... fields
            event_window: EventWindow {
                merge_strategy: MergeStrategy::ManualSave,
                // ... other fields
            },
            // ...
        };

        self.storage.save_clip_metadata_v2(&game_id, &metadata)?;

        Ok(clip_path)
    }
}
```

**Testing**:
1. Press F8 → Verify clip saved
2. Press F9 → Verify recording toggles
3. Press F10 → Verify last clip deleted
4. Configure custom hotkeys → Verify they work

**Success Criteria**:
- ✅ F8 saves manual clip (30s default)
- ✅ F9 toggles recording
- ✅ F10 deletes last clip
- ✅ Hotkeys work even when app minimized
- ✅ Custom hotkeys configurable in settings

---

### Phase 7: Audio Recording

**Objective**: Multi-track audio capture (microphone + system audio)

**Duration**: 3-4 days

**Dependencies**: Phase 1 complete

#### Tasks

##### 7.1: Research Audio Capture Libraries
**Options**:
- `cpal` (Rust audio library)
- `windows-rs` (Windows Audio API)
- FFmpeg audio capture (integrate with existing FFmpeg)

**Recommendation**: Use FFmpeg audio filters for simplicity

##### 7.2: Implement Audio Device Detection
**File**: `src-tauri/src/recording/audio.rs` (new)

```rust
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub device_type: AudioDeviceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioDeviceType {
    Input,   // Microphone
    Output,  // System audio
}

pub fn list_audio_devices() -> Result<Vec<AudioDevice>> {
    // Use FFmpeg to list devices
    let output = Command::new("ffmpeg")
        .args(&["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse FFmpeg output to extract device names
    let mut devices = Vec::new();

    // ... parsing logic

    Ok(devices)
}
```

##### 7.3: Update FFmpeg Recording Command
**File**: `src-tauri/src/recording/windows_backend.rs`

```rust
impl WindowsRecorder {
    fn build_ffmpeg_command(&self, segment_path: &Path) -> Result<Command> {
        let settings = self.settings.read().await;

        let mut cmd = Command::new("ffmpeg");

        // Video input (Game DVR)
        cmd.args(&[
            "-f", "gdigrab",
            "-i", "desktop",
        ]);

        // Microphone input (if enabled)
        if settings.audio.record_microphone {
            if let Some(mic_device) = &settings.audio.microphone_device {
                cmd.args(&[
                    "-f", "dshow",
                    "-i", &format!("audio={}", mic_device),
                ]);
            }
        }

        // System audio input (if enabled)
        if settings.audio.record_system_audio {
            if let Some(system_device) = &settings.audio.system_audio_device {
                cmd.args(&[
                    "-f", "dshow",
                    "-i", &format!("audio={}", system_device),
                ]);
            }
        }

        // Audio mixing with volume control
        let mic_volume = settings.audio.microphone_volume as f32 / 100.0;
        let system_volume = settings.audio.system_audio_volume as f32 / 100.0;

        cmd.args(&[
            "-filter_complex",
            &format!(
                "[1:a]volume={}[mic];[2:a]volume={}[sys];[mic][sys]amerge=inputs=2[aout]",
                mic_volume, system_volume
            ),
            "-map", "0:v",
            "-map", "[aout]",
        ]);

        // ... rest of encoding settings

        Ok(cmd)
    }
}
```

##### 7.4: Add Audio Settings UI
**File**: `src/components/settings/AudioSettings.tsx`

```tsx
export function AudioSettings({ settings, onChange }) {
  const [devices, setDevices] = useState<AudioDevice[]>([]);

  useEffect(() => {
    loadAudioDevices();
  }, []);

  const loadAudioDevices = async () => {
    const devices = await invoke<AudioDevice[]>("get_audio_devices");
    setDevices(devices);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Audio Recording Settings</CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Microphone */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <Label>Microphone Recording</Label>
            <Switch
              checked={settings.record_microphone}
              onCheckedChange={(checked) =>
                onChange({ ...settings, record_microphone: checked })
              }
            />
          </div>

          {settings.record_microphone && (
            <>
              <Select
                value={settings.microphone_device || "default"}
                onValueChange={(value) =>
                  onChange({ ...settings, microphone_device: value })
                }
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select microphone" />
                </SelectTrigger>
                <SelectContent>
                  {devices
                    .filter((d) => d.device_type === "Input")
                    .map((device) => (
                      <SelectItem key={device.id} value={device.id}>
                        {device.name}
                      </SelectItem>
                    ))}
                </SelectContent>
              </Select>

              <div className="space-y-2">
                <Label>Microphone Volume: {settings.microphone_volume}%</Label>
                <Slider
                  min={0}
                  max={200}
                  step={10}
                  value={[settings.microphone_volume]}
                  onValueChange={(value) =>
                    onChange({ ...settings, microphone_volume: value[0] })
                  }
                />
              </div>
            </>
          )}
        </div>

        {/* System Audio */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <Label>System Audio Recording</Label>
            <Switch
              checked={settings.record_system_audio}
              onCheckedChange={(checked) =>
                onChange({ ...settings, record_system_audio: checked })
              }
            />
          </div>

          {settings.record_system_audio && (
            <>
              <Select
                value={settings.system_audio_device || "default"}
                onValueChange={(value) =>
                  onChange({ ...settings, system_audio_device: value })
                }
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select output device" />
                </SelectTrigger>
                <SelectContent>
                  {devices
                    .filter((d) => d.device_type === "Output")
                    .map((device) => (
                      <SelectItem key={device.id} value={device.id}>
                        {device.name}
                      </SelectItem>
                    ))}
                </SelectContent>
              </Select>

              <div className="space-y-2">
                <Label>System Volume: {settings.system_audio_volume}%</Label>
                <Slider
                  min={0}
                  max={200}
                  step={10}
                  value={[settings.system_audio_volume]}
                  onValueChange={(value) =>
                    onChange({ ...settings, system_audio_volume: value[0] })
                  }
                />
              </div>
            </>
          )}
        </div>

        {/* Microphone Test */}
        <div>
          <Button onClick={testMicrophone}>Test Microphone</Button>
        </div>
      </CardContent>
    </Card>
  );
}
```

**Testing**:
1. List audio devices → Verify microphones and speakers shown
2. Select microphone → Verify captured in recording
3. Adjust volume → Verify volume changes applied
4. Test microphone → Verify real-time audio level display

**Success Criteria**:
- ✅ Audio devices detected and listed
- ✅ Microphone and system audio can be recorded
- ✅ Volume controls work (0-200%)
- ✅ Audio tracks properly mixed
- ✅ Microphone test feature works

---

### Phase 8: Testing & Validation

**Objective**: End-to-end testing and quality assurance

**Duration**: 3-5 days

**Dependencies**: All previous phases complete

#### Tasks

##### 8.1: Unit Testing
**Files to Test**:
- `src-tauri/src/settings/*.rs`
- `src-tauri/src/recording/event_window.rs`
- `src-tauri/src/recording/auto_clip_manager.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_window_merging() {
        let mut manager = EventWindowManager::new();

        // Add kill at t=100
        let event1 = create_test_event(100.0, EventType::ChampionKill);
        assert!(manager.add_event(event1).is_none());

        // Add kill at t=110 (within threshold)
        let event2 = create_test_event(110.0, EventType::ChampionKill);
        assert!(manager.add_event(event2).is_none());

        // Add kill at t=130 (exceeds threshold)
        let event3 = create_test_event(130.0, EventType::ChampionKill);
        let window = manager.add_event(event3);

        assert!(window.is_some());
        let window = window.unwrap();
        assert_eq!(window.events_merged, 2);
    }

    #[tokio::test]
    async fn test_settings_persistence() {
        let settings = RecordingSettings::default();
        settings.save().unwrap();

        let loaded = RecordingSettings::load().unwrap();
        assert_eq!(loaded.event_filter.record_kills, settings.event_filter.record_kills);
    }
}
```

##### 8.2: Integration Testing
**Test Scenarios**:

1. **Full Recording Flow**:
   ```
   Start app → Connect to League → Start game →
   Kill champion → Verify clip saved → Stop game →
   Verify metadata correct
   ```

2. **Settings Flow**:
   ```
   Open settings → Change event filters → Save →
   Restart app → Verify settings persisted →
   Play game → Verify filters applied
   ```

3. **Hotkey Flow**:
   ```
   Start recording → Press F8 →
   Verify manual clip saved →
   Press F10 → Verify clip deleted
   ```

##### 8.3: Performance Testing
**Metrics to Measure**:
- CPU usage during recording (<30% on modern CPU)
- Memory usage (<500MB idle, <2GB recording)
- Disk I/O rate (sustainable for long sessions)
- Event detection latency (<500ms)
- Clip save time (<5s for 30s clip)

##### 8.4: User Acceptance Testing
**Test with Real Users**:
- Can they configure settings easily?
- Do they understand event filtering?
- Are hotkeys intuitive?
- Is the UI responsive and clear?

**Success Criteria**:
- ✅ All unit tests pass
- ✅ Integration tests pass
- ✅ Performance within targets
- ✅ No memory leaks
- ✅ No crashes during 1-hour session
- ✅ User feedback positive (>80% satisfaction)

---

## 🎯 Success Metrics

### Technical Metrics
- [ ] **Code Coverage**: >80% for critical paths
- [ ] **Performance**: CPU <30%, Memory <2GB during recording
- [ ] **Reliability**: No crashes during 10-game test session
- [ ] **Latency**: Event detection <500ms, Clip save <5s

### User Experience Metrics
- [ ] **Setup Time**: User can configure settings in <5 minutes
- [ ] **Auto-Recording**: 95%+ of expected events captured
- [ ] **Clip Quality**: User satisfaction >80% with clip quality
- [ ] **Usability**: Settings UI intuitive (no confusion)

### Quality Metrics
- [ ] **Test Coverage**: All phases have unit + integration tests
- [ ] **Documentation**: All public APIs documented
- [ ] **Error Handling**: All errors logged and user-friendly
- [ ] **Edge Cases**: Tested with practice mode, ARAM, ranked

---

## 📅 Timeline Estimate

| Phase | Duration | Dependencies | Status |
|-------|----------|--------------|--------|
| Phase 0: Foundation | 1 day | None | ✅ 90% Complete |
| Phase 1: Settings Backend | 2-3 days | Phase 0 | ⏳ Pending |
| Phase 2: Settings UI | 3-4 days | Phase 1 | ⏳ Pending |
| Phase 3: Event Detection | 3-4 days | Phase 1 | ⏳ Pending |
| Phase 4: Auto Clip Manager | 4-5 days | Phase 3 | ⏳ Pending |
| Phase 5: V2 Metadata | 2-3 days | Phase 4 | ⏳ Pending |
| Phase 6: Hotkey System | 2-3 days | Phase 4 | ⏳ Pending |
| Phase 7: Audio Recording | 3-4 days | Phase 1 | ⏳ Pending |
| Phase 8: Testing | 3-5 days | All phases | ⏳ Pending |

**Total Estimated Time**: 22-35 days (1-1.5 months)

**Parallel Work Opportunities**:
- Phase 2 (UI) can start when Phase 1 backend is stable
- Phase 6 (Hotkeys) can start independently after Phase 1
- Phase 7 (Audio) can start independently after Phase 1

**Realistic Timeline**: With parallel work, can be completed in **3-4 weeks**

---

## 🚀 Next Immediate Steps

1. **Complete Phase 0**: Review this workflow document ✅
2. **Start Phase 1**: Create settings module structure
   - Create `src-tauri/src/settings/` directory
   - Implement `models.rs` with all settings structs
   - Implement `storage.rs` for JSON persistence
3. **Parallel**: Start Phase 2 Settings UI mockup
   - Design UI layouts for each settings tab
   - Create React component structure

**First Task**: Implement `src-tauri/src/settings/models.rs` with complete settings structures from RECORDING_SETTINGS_DESIGN.md
