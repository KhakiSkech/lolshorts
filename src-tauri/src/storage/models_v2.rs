#![allow(dead_code)]
// Enhanced clip metadata structures for video editor integration
// This extends the basic ClipMetadata with rich information for editing workflows

use super::models::EventType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Core ClipMetadataV2 Structure
// ============================================================================

/// Comprehensive clip metadata for editor integration
///
/// Each clip has an accompanying JSON file with this structure, enabling:
/// - Frame-accurate timeline navigation
/// - Event-driven effects and captions
/// - Multi-track audio editing
/// - Metadata-driven search and filtering
/// - Quality-preserving re-encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMetadataV2 {
    // === Identification ===
    pub clip_id: String,
    pub game_id: String,
    pub file_path: String,
    pub thumbnail_path: Option<String>,

    // === Temporal Information ===
    pub created_at: DateTime<Utc>,
    pub game_time_start: f64, // When in game (seconds)
    pub game_time_end: f64,   // When in game (seconds)
    pub clip_duration: f64,   // Total clip duration (seconds)

    // === Event Information ===
    pub primary_event: EventInfo,      // Main event that triggered clip
    pub merged_events: Vec<EventInfo>, // Additional events merged into this clip
    pub event_window: EventWindow,     // How events were merged

    // === Priority & Filtering ===
    pub priority: u8,      // 1-5 (5=pentakill)
    pub tags: Vec<String>, // ["pentakill", "yasuo", "ranked"]

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

// ============================================================================
// Event Information
// ============================================================================

/// Information about a single event in the clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub event_id: u64,
    pub event_type: EventType,
    pub timestamp: f64,      // Game time (seconds)
    pub clip_timestamp: f64, // Time within clip (seconds)
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

/// How multiple events were merged into this clip
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventWindow {
    pub merge_strategy: MergeStrategy,
    pub time_threshold_secs: f64, // 15 seconds default
    pub events_merged: usize,     // How many events combined
    pub pre_duration: f64,        // Seconds before first event
    pub post_duration: f64,       // Seconds after last event
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    SingleEvent,       // Just one event
    ConsecutiveEvents, // Multiple events within threshold
    ManualSave,        // User pressed hotkey (F8)
}

// ============================================================================
// Video Technical Information
// ============================================================================

/// Technical video information for editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub resolution: Resolution,
    pub frame_rate: FrameRate,
    pub bitrate_kbps: u32,
    pub codec: VideoCodec,
    pub encoder: String, // "nvenc_h265", "x264", etc.
    pub file_size_bytes: u64,
    pub total_frames: u64,

    // Color information
    pub color_space: String,  // "bt709", "bt2020"
    pub pixel_format: String, // "yuv420p", "yuv444p"

    // Encoding parameters
    pub crf: Option<u8>,        // Quality (0-51, lower=better)
    pub preset: Option<String>, // "fast", "medium", "slow"
}

impl Default for VideoInfo {
    fn default() -> Self {
        Self {
            resolution: Resolution::R1920x1080,
            frame_rate: FrameRate::Fps60,
            bitrate_kbps: 20000,
            codec: VideoCodec::H265,
            encoder: "unknown".to_string(),
            file_size_bytes: 0,
            total_frames: 0,
            color_space: "bt709".to_string(),
            pixel_format: "yuv420p".to_string(),
            crf: Some(23),
            preset: Some("medium".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    R1920x1080, // 1080p
    R2560x1440, // 1440p
    R3840x2160, // 4K
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

// ============================================================================
// Audio Information
// ============================================================================

/// Audio track information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInfo {
    pub tracks: Vec<AudioTrack>,
    pub sample_rate: u32, // 48000 Hz typical
    pub channels: u8,     // 2 for stereo
    pub bitrate_kbps: u32,
    pub codec: String, // "aac", "opus"
}

impl Default for AudioInfo {
    fn default() -> Self {
        Self {
            tracks: vec![],
            sample_rate: 48000,
            channels: 2,
            bitrate_kbps: 192,
            codec: "aac".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioTrack {
    pub track_id: u8, // 0, 1, 2...
    pub track_type: AudioTrackType,
    pub volume_percent: u8, // 0-200%
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioTrackType {
    SystemAudio, // Game + Discord + Music
    Microphone,  // User voice
    Mixed,       // Pre-mixed
}

// ============================================================================
// Timeline Information
// ============================================================================

/// Timeline markers for editor scrubbing and navigation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClipTimeline {
    pub markers: Vec<TimelineMarker>,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineMarker {
    pub timestamp: f64, // Time in clip (seconds)
    pub marker_type: MarkerType,
    pub label: String,
    pub color: Option<String>, // Hex color for UI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerType {
    EventStart,     // Event begins
    EventPeak,      // Climax of event (e.g., pentakill moment)
    EventEnd,       // Event ends
    KillMoment,     // Individual kill
    ObjectiveTaken, // Dragon/Baron secured
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub start: f64,
    pub end: f64,
    pub title: String,
    pub description: Option<String>,
}

// ============================================================================
// Game Context
// ============================================================================

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

impl Default for GameContext {
    fn default() -> Self {
        Self {
            champion: "Unknown".to_string(),
            game_mode: GameMode::Classic,
            queue_type: QueueType::Normal,
            map_id: 11,
            team: Team::Blue,
            team_score: TeamScore::default(),
            player_state: PlayerState::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    Classic, // Summoner's Rift
    Aram,    // Howling Abyss
    Arena,   // 2v2v2v2
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub items: Vec<u32>,      // Item IDs
    pub kda: (u32, u32, u32), // (kills, deaths, assists)
    pub cs: u32,              // Creep score
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            level: 1,
            gold: 0,
            items: vec![],
            kda: (0, 0, 0),
            cs: 0,
        }
    }
}

// ============================================================================
// User Annotations
// ============================================================================

/// User-added annotations for editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAnnotations {
    pub title: Option<String>,
    pub description: Option<String>,
    pub rating: Option<u8>, // 1-5 stars
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

// ============================================================================
// Migration & Helpers
// ============================================================================

impl ClipMetadataV2 {
    /// Generate a clip ID from file path
    pub fn generate_clip_id(file_path: &str) -> String {
        use std::path::Path;

        let path = Path::new(file_path);
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    }

    /// Extract game ID from file path
    pub fn extract_game_id(file_path: &str) -> String {
        use std::path::Path;

        let path = Path::new(file_path);

        // Try to find game_id in path like: clips/<game_id>/clips/clip_xxx.mp4
        let components: Vec<_> = path.components().collect();

        for (i, comp) in components.iter().enumerate() {
            if let Some(s) = comp.as_os_str().to_str() {
                if s == "clips" && i + 1 < components.len() {
                    if let Some(game_id) = components[i + 1].as_os_str().to_str() {
                        return game_id.to_string();
                    }
                }
            }
        }

        "unknown".to_string()
    }

    /// Add a timeline marker
    pub fn add_marker(&mut self, marker: TimelineMarker) {
        self.timeline.markers.push(marker);
    }

    /// Add a timeline chapter
    pub fn add_chapter(&mut self, chapter: Chapter) {
        self.timeline.chapters.push(chapter);
    }

    /// Add a user note
    pub fn add_note(&mut self, timestamp: f64, text: String) {
        if self.annotations.is_none() {
            self.annotations = Some(UserAnnotations {
                title: None,
                description: None,
                rating: None,
                favorite: false,
                notes: vec![],
                custom_tags: vec![],
            });
        }

        if let Some(annotations) = &mut self.annotations {
            annotations.notes.push(Note {
                timestamp,
                text,
                created_at: Utc::now(),
            });
        }
    }

    /// Set user rating (1-5 stars)
    pub fn set_rating(&mut self, rating: u8) {
        if self.annotations.is_none() {
            self.annotations = Some(UserAnnotations {
                title: None,
                description: None,
                rating: None,
                favorite: false,
                notes: vec![],
                custom_tags: vec![],
            });
        }

        if let Some(annotations) = &mut self.annotations {
            annotations.rating = Some(rating.clamp(1, 5));
        }
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        if self.annotations.is_none() {
            self.annotations = Some(UserAnnotations {
                title: None,
                description: None,
                rating: None,
                favorite: true,
                notes: vec![],
                custom_tags: vec![],
            });
        } else if let Some(annotations) = &mut self.annotations {
            annotations.favorite = !annotations.favorite;
        }
    }

    /// Add a custom tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Get all events (primary + merged) sorted by timestamp
    pub fn get_all_events(&self) -> Vec<&EventInfo> {
        let mut events = vec![&self.primary_event];
        events.extend(self.merged_events.iter());
        events.sort_by(|a, b| a.clip_timestamp.partial_cmp(&b.clip_timestamp).unwrap());
        events
    }

    /// Calculate total priority (sum of all events)
    pub fn total_priority(&self) -> u32 {
        self.primary_event.priority as u32
            + self
                .merged_events
                .iter()
                .map(|e| e.priority as u32)
                .sum::<u32>()
    }
}

// ============================================================================
// Migration from V1 to V2
// ============================================================================

impl From<super::models::ClipMetadata> for ClipMetadataV2 {
    fn from(old: super::models::ClipMetadata) -> Self {
        let clip_id = Self::generate_clip_id(&old.file_path);
        let game_id = Self::extract_game_id(&old.file_path);

        ClipMetadataV2 {
            clip_id,
            game_id,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_id_extraction() {
        let path = "C:/LoLShorts/clips/1234567890/clips/clip_20250110_143052_pentakill.mp4";
        let id = ClipMetadataV2::generate_clip_id(path);
        assert_eq!(id, "clip_20250110_143052_pentakill");
    }

    #[test]
    fn test_game_id_extraction() {
        let path = "C:/LoLShorts/clips/1234567890/clips/clip_20250110_143052_pentakill.mp4";
        let game_id = ClipMetadataV2::extract_game_id(path);
        assert_eq!(game_id, "1234567890");
    }

    #[test]
    fn test_add_marker() {
        let mut clip = create_test_clip();

        clip.add_marker(TimelineMarker {
            timestamp: 15.0,
            marker_type: MarkerType::EventPeak,
            label: "Pentakill".to_string(),
            color: Some("#FFD700".to_string()),
        });

        assert_eq!(clip.timeline.markers.len(), 1);
        assert_eq!(clip.timeline.markers[0].label, "Pentakill");
    }

    #[test]
    fn test_rating() {
        let mut clip = create_test_clip();

        clip.set_rating(5);
        assert_eq!(clip.annotations.as_ref().unwrap().rating, Some(5));

        // Test clamping
        clip.set_rating(10);
        assert_eq!(clip.annotations.as_ref().unwrap().rating, Some(5));
    }

    fn create_test_clip() -> ClipMetadataV2 {
        ClipMetadataV2 {
            clip_id: "test_clip".to_string(),
            game_id: "12345".to_string(),
            file_path: "test.mp4".to_string(),
            thumbnail_path: None,
            created_at: Utc::now(),
            game_time_start: 100.0,
            game_time_end: 130.0,
            clip_duration: 30.0,
            primary_event: EventInfo {
                event_id: 1,
                event_type: EventType::Multikill(5),
                timestamp: 115.0,
                clip_timestamp: 15.0,
                priority: 5,
                killer: Some("Player1".to_string()),
                victim: None,
                assisters: vec![],
                gold_earned: Some(1500),
                shutdown_bounty: None,
                details: None,
            },
            merged_events: vec![],
            event_window: EventWindow {
                merge_strategy: MergeStrategy::SingleEvent,
                time_threshold_secs: 0.0,
                events_merged: 1,
                pre_duration: 15.0,
                post_duration: 5.0,
            },
            priority: 5,
            tags: vec!["pentakill".to_string()],
            video_info: VideoInfo::default(),
            audio_info: AudioInfo::default(),
            timeline: ClipTimeline::default(),
            game_context: GameContext::default(),
            annotations: None,
        }
    }
}
