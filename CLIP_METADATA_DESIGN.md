# Clip Metadata Architecture for Editor Integration

## Overview

This document defines the comprehensive clip metadata structure that enables advanced video editing features. Each clip has a corresponding JSON file with rich metadata that the editor consumes.

---

## File Organization

```
clips/
  └── <game_id>/
      ├── metadata.json                    # Game-level metadata
      ├── events.json                      # All raw events from game
      ├── clips.json                       # Lightweight clip index
      └── clips/
          ├── clip_<timestamp>_<id>.mp4   # Video file
          ├── clip_<timestamp>_<id>.json  # Rich metadata (THIS FILE)
          └── clip_<timestamp>_<id>.jpg   # Thumbnail
```

---

## Enhanced ClipMetadata Structure (Rust)

### Core Structure

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Comprehensive clip metadata for editor integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMetadataV2 {
    // === Identification ===
    pub clip_id: String,
    pub game_id: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,

    // === Temporal Information ===
    pub created_at: DateTime<Utc>,
    pub game_time_start: f64,        // When in game (seconds)
    pub game_time_end: f64,          // When in game (seconds)
    pub clip_duration: f64,          // Total clip duration (seconds)

    // === Event Information ===
    pub primary_event: EventInfo,    // Main event that triggered clip
    pub merged_events: Vec<EventInfo>, // Additional events merged into this clip
    pub event_window: EventWindow,   // How events were merged

    // === Priority & Filtering ===
    pub priority: u8,                // 1-5 (5=pentakill)
    pub tags: Vec<String>,           // ["pentakill", "yasuo", "ranked"]

    // === Video Technical Details ===
    pub video_info: VideoInfo,

    // === Audio Information ===
    pub audio_info: AudioInfo,

    // === Timeline Markers (for Editor) ===
    pub timeline: ClipTimeline,

    // === Game Context ===
    pub game_context: GameContext,

    // === User Annotations (Optional) ===
    pub annotations: Option<UserAnnotations>,
}
```

### EventInfo Structure

```rust
/// Information about a single event in the clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub event_id: u64,
    pub event_type: EventType,
    pub timestamp: f64,              // Game time (seconds)
    pub clip_timestamp: f64,         // Time within clip (seconds)
    pub priority: u8,

    // Participants
    pub killer: Option<String>,
    pub victim: Option<String>,
    pub assisters: Vec<String>,

    // Additional context
    pub gold_earned: Option<u32>,
    pub shutdown_bounty: Option<u32>,
    pub details: Option<serde_json::Value>,
}
```

### EventWindow Structure

```rust
/// How multiple events were merged into this clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWindow {
    pub merge_strategy: MergeStrategy,
    pub time_threshold_secs: f64,    // 15 seconds default
    pub events_merged: usize,        // How many events combined
    pub pre_duration: f64,           // Seconds before first event
    pub post_duration: f64,          // Seconds after last event
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    SingleEvent,                     // Just one event
    ConsecutiveEvents,               // Multiple events within threshold
    ManualSave,                      // User pressed hotkey (F8)
}
```

### VideoInfo Structure

```rust
/// Technical video information for editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub resolution: Resolution,
    pub frame_rate: FrameRate,
    pub bitrate_kbps: u32,
    pub codec: VideoCodec,
    pub encoder: String,             // "nvenc_h265", "x264", etc.
    pub file_size_bytes: u64,
    pub total_frames: u64,

    // Color information
    pub color_space: String,         // "bt709", "bt2020"
    pub pixel_format: String,        // "yuv420p", "yuv444p"

    // Encoding parameters
    pub crf: Option<u8>,             // Quality (0-51, lower=better)
    pub preset: Option<String>,      // "fast", "medium", "slow"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    R1920x1080,   // 1080p
    R2560x1440,   // 1440p
    R3840x2160,   // 4K
    Custom { width: u32, height: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameRate {
    Fps30,
    Fps60,
    Fps120,
    Fps144,
    Custom(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264,
    H265,
    Av1,
}
```

### AudioInfo Structure

```rust
/// Audio track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub tracks: Vec<AudioTrack>,
    pub sample_rate: u32,            // 48000 Hz typical
    pub channels: u8,                // 2 for stereo
    pub bitrate_kbps: u32,
    pub codec: String,               // "aac", "opus"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub track_id: u8,                // 0, 1, 2...
    pub track_type: AudioTrackType,
    pub volume_percent: u8,          // 0-200%
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioTrackType {
    SystemAudio,                     // Game + Discord + Music
    Microphone,                      // User voice
    Mixed,                           // Pre-mixed
}
```

### ClipTimeline Structure

```rust
/// Timeline markers for editor scrubbing and navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipTimeline {
    pub markers: Vec<TimelineMarker>,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMarker {
    pub timestamp: f64,              // Time in clip (seconds)
    pub marker_type: MarkerType,
    pub label: String,
    pub color: Option<String>,       // Hex color for UI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerType {
    EventStart,                      // Event begins
    EventPeak,                       // Climax of event (e.g., pentakill moment)
    EventEnd,                        // Event ends
    KillMoment,                      // Individual kill
    ObjectiveTaken,                  // Dragon/Baron secured
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub start: f64,
    pub end: f64,
    pub title: String,
    pub description: Option<String>,
}
```

### GameContext Structure

```rust
/// Game state context when clip was created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameContext {
    pub champion: String,
    pub game_mode: GameMode,
    pub queue_type: QueueType,
    pub map_id: u32,

    // Team state
    pub team: Team,
    pub team_score: TeamScore,

    // Player state at event time
    pub player_state: PlayerState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    Classic,      // Summoner's Rift
    Aram,         // Howling Abyss
    Arena,        // 2v2v2v2
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueType {
    RankedSolo,
    RankedFlex,
    Normal,
    QuickPlay,
    Aram,
    Arena,
    Custom,
    Practice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Team {
    Blue,
    Red,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamScore {
    pub kills: u32,
    pub towers: u32,
    pub dragons: u32,
    pub barons: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub level: u8,
    pub gold: u32,
    pub items: Vec<u32>,             // Item IDs
    pub kda: (u32, u32, u32),       // (kills, deaths, assists)
    pub cs: u32,                     // Creep score
}
```

### UserAnnotations Structure

```rust
/// User-added annotations for editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnnotations {
    pub title: Option<String>,
    pub description: Option<String>,
    pub rating: Option<u8>,          // 1-5 stars
    pub favorite: bool,
    pub notes: Vec<Note>,
    pub custom_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub timestamp: f64,
    pub text: String,
    pub created_at: DateTime<Utc>,
}
```

---

## Example JSON Output

```json
{
  "clip_id": "clip_20250110_143052_pentakill",
  "game_id": "1234567890",
  "file_path": "C:/LoLShorts/clips/1234567890/clips/clip_20250110_143052_pentakill.mp4",
  "thumbnail_path": "C:/LoLShorts/clips/1234567890/clips/clip_20250110_143052_pentakill.jpg",

  "created_at": "2025-01-10T14:30:52Z",
  "game_time_start": 1245.0,
  "game_time_end": 1275.0,
  "clip_duration": 30.0,

  "primary_event": {
    "event_id": 89,
    "event_type": { "multikill": 5 },
    "timestamp": 1260.0,
    "clip_timestamp": 15.0,
    "priority": 5,
    "killer": "YasuoMain123",
    "victim": null,
    "assisters": ["Ally1", "Ally2"],
    "gold_earned": 1500,
    "shutdown_bounty": 450,
    "details": {
      "multikill_type": "pentakill",
      "kill_sequence": [
        {"victim": "Enemy1", "timestamp": 1256.2},
        {"victim": "Enemy2", "timestamp": 1257.5},
        {"victim": "Enemy3", "timestamp": 1258.8},
        {"victim": "Enemy4", "timestamp": 1260.1},
        {"victim": "Enemy5", "timestamp": 1261.3}
      ]
    }
  },

  "merged_events": [
    {
      "event_id": 87,
      "event_type": "dragon_kill",
      "timestamp": 1250.0,
      "clip_timestamp": 5.0,
      "priority": 3,
      "details": { "dragon_type": "infernal" }
    }
  ],

  "event_window": {
    "merge_strategy": "consecutive_events",
    "time_threshold_secs": 15.0,
    "events_merged": 2,
    "pre_duration": 15.0,
    "post_duration": 5.0
  },

  "priority": 5,
  "tags": ["pentakill", "yasuo", "ranked", "infernal_dragon"],

  "video_info": {
    "resolution": "r1920x1080",
    "frame_rate": "fps60",
    "bitrate_kbps": 20000,
    "codec": "h265",
    "encoder": "nvenc_h265",
    "file_size_bytes": 75000000,
    "total_frames": 1800,
    "color_space": "bt709",
    "pixel_format": "yuv420p",
    "crf": 23,
    "preset": "medium"
  },

  "audio_info": {
    "tracks": [
      {
        "track_id": 0,
        "track_type": "system_audio",
        "volume_percent": 100,
        "device_name": "Speakers (Realtek)"
      },
      {
        "track_id": 1,
        "track_type": "microphone",
        "volume_percent": 120,
        "device_name": "Microphone (Blue Yeti)"
      }
    ],
    "sample_rate": 48000,
    "channels": 2,
    "bitrate_kbps": 192,
    "codec": "aac"
  },

  "timeline": {
    "markers": [
      {
        "timestamp": 5.0,
        "marker_type": "objective_taken",
        "label": "Infernal Dragon Secured",
        "color": "#FF4500"
      },
      {
        "timestamp": 11.2,
        "marker_type": "kill_moment",
        "label": "First Kill (Enemy1)",
        "color": "#00FF00"
      },
      {
        "timestamp": 12.5,
        "marker_type": "kill_moment",
        "label": "Second Kill (Enemy2)",
        "color": "#00FF00"
      },
      {
        "timestamp": 13.8,
        "marker_type": "kill_moment",
        "label": "Third Kill (Enemy3)",
        "color": "#00FF00"
      },
      {
        "timestamp": 15.1,
        "marker_type": "kill_moment",
        "label": "Fourth Kill (Enemy4)",
        "color": "#FFD700"
      },
      {
        "timestamp": 16.3,
        "marker_type": "event_peak",
        "label": "PENTAKILL!",
        "color": "#FFD700"
      }
    ],
    "chapters": [
      {
        "start": 0.0,
        "end": 10.0,
        "title": "Setup",
        "description": "Team secures dragon, positioning for teamfight"
      },
      {
        "start": 10.0,
        "end": 20.0,
        "title": "Pentakill Sequence",
        "description": "Yasuo eliminates entire enemy team"
      },
      {
        "start": 20.0,
        "end": 30.0,
        "title": "Aftermath",
        "description": "Team pushes mid lane advantage"
      }
    ]
  },

  "game_context": {
    "champion": "Yasuo",
    "game_mode": "classic",
    "queue_type": "ranked_solo",
    "map_id": 11,
    "team": "blue",
    "team_score": {
      "kills": 25,
      "towers": 5,
      "dragons": 2,
      "barons": 0
    },
    "player_state": {
      "level": 16,
      "gold": 12500,
      "items": [3031, 3046, 3087, 3006, 3072, 3363],
      "kda": [12, 3, 8],
      "cs": 215
    }
  },

  "annotations": {
    "title": "Epic Pentakill After Dragon Fight",
    "description": "Secured infernal drake then wiped enemy team for pentakill. Used ult perfectly after Malphite engage.",
    "rating": 5,
    "favorite": true,
    "notes": [
      {
        "timestamp": 15.1,
        "text": "Remember to use this ult timing in future montages",
        "created_at": "2025-01-10T14:35:00Z"
      }
    ],
    "custom_tags": ["montage_worthy", "upload_to_youtube"]
  }
}
```

---

## Editor Integration Benefits

### Timeline Navigation
- **Markers**: Editor can show markers on timeline for quick scrubbing
- **Chapters**: Divide long clips into logical sections
- **Event Peaks**: Automatically highlight the most important moments

### Audio Editing
- **Separate Tracks**: Edit microphone and system audio independently
- **Volume Levels**: Preserve original volume settings
- **Device Info**: Show what devices were used for recording

### Metadata-Driven Editing
- **Auto-Captions**: Use event types to generate captions ("PENTAKILL!")
- **Transition Effects**: Apply effects based on event priority
- **Smart Cropping**: Focus camera on killer/victim using participant info

### Search & Filtering
- **By Event Type**: Find all pentakills, baron steals, etc.
- **By Champion**: Filter clips by champion played
- **By Priority**: Show only high-priority clips (4-5 stars)
- **By Tags**: Custom user tags for organization

### Quality Preservation
- **Original Settings**: Know exact encoding settings for re-encoding
- **File Size**: Track storage usage
- **Technical Specs**: Ensure compatibility when exporting

---

## Storage Layer Implementation

### Save Clip with Metadata

```rust
impl RecordingManager {
    pub async fn save_clip_with_metadata(
        &self,
        clip: ClipMetadataV2,
    ) -> Result<PathBuf> {
        // 1. Save video file (already implemented in windows_backend.rs)
        let video_path = self.save_clip_video(&clip).await?;

        // 2. Generate thumbnail
        let thumbnail_path = self.generate_thumbnail(&video_path).await?;

        // 3. Create JSON metadata file
        let json_path = video_path.with_extension("json");
        let json = serde_json::to_string_pretty(&clip)?;
        fs::write(&json_path, json).await?;

        // 4. Update clips.json index (lightweight references)
        self.update_clips_index(&clip).await?;

        tracing::info!("Saved clip with metadata: {}", clip.clip_id);

        Ok(video_path)
    }
}
```

### Load Clip for Editor

```rust
impl RecordingManager {
    pub async fn load_clip_for_editor(
        &self,
        clip_id: &str,
    ) -> Result<ClipMetadataV2> {
        // Find JSON file
        let json_path = self.find_clip_json(clip_id).await?;

        // Load and parse
        let json = fs::read_to_string(&json_path).await?;
        let metadata: ClipMetadataV2 = serde_json::from_str(&json)?;

        Ok(metadata)
    }
}
```

---

## Migration Path

### From Current ClipMetadata to V2

```rust
impl From<ClipMetadata> for ClipMetadataV2 {
    fn from(old: ClipMetadata) -> Self {
        ClipMetadataV2 {
            clip_id: generate_id_from_path(&old.file_path),
            game_id: extract_game_id(&old.file_path),
            file_path: old.file_path,
            thumbnail_path: old.thumbnail_path,

            created_at: old.created_at,
            game_time_start: old.event_time,
            game_time_end: old.event_time + old.duration,
            clip_duration: old.duration,

            primary_event: EventInfo {
                event_id: 0, // Unknown in old format
                event_type: old.event_type,
                timestamp: old.event_time,
                clip_timestamp: 10.0, // Assume 10s pre-roll
                priority: old.priority,
                killer: None,
                victim: None,
                assisters: vec![],
                gold_earned: None,
                shutdown_bounty: None,
                details: None,
            },

            merged_events: vec![],
            event_window: EventWindow {
                merge_strategy: MergeStrategy::SingleEvent,
                time_threshold_secs: 0.0,
                events_merged: 1,
                pre_duration: 10.0,
                post_duration: 3.0,
            },

            priority: old.priority,
            tags: vec![],

            // Fill with defaults for missing data
            video_info: VideoInfo::default(),
            audio_info: AudioInfo::default(),
            timeline: ClipTimeline::default(),
            game_context: GameContext::default(),
            annotations: None,
        }
    }
}
```

---

## Summary

**Yes, clip metadata WILL be stored as comprehensive JSON files** that enable advanced editing features:

✅ **Individual JSON per clip** - Each video has its own metadata file
✅ **Rich event information** - Full details about what happened and when
✅ **Timeline markers** - Frame-accurate navigation for editor
✅ **Video/audio technical specs** - All encoding details preserved
✅ **Game context** - Champion, items, KDA, team state
✅ **User annotations** - Notes, ratings, custom tags
✅ **Editor-friendly structure** - Designed for video editing workflows

This architecture ensures the editor can:
- Navigate clips with precision
- Auto-generate effects and captions
- Edit audio tracks independently
- Search and filter clips intelligently
- Preserve quality during re-encoding
