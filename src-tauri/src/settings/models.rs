use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete recording settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSettings {
    pub event_filter: EventFilterSettings,
    pub game_mode: GameModeSettings,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub clip_timing: ClipTimingSettings,
    pub hotkeys: HotkeySettings,

    // General settings
    pub auto_start_with_league: bool,
    pub minimize_to_tray: bool,
    pub show_notifications: bool,
}

impl Default for RecordingSettings {
    fn default() -> Self {
        Self {
            event_filter: EventFilterSettings::default(),
            game_mode: GameModeSettings::default(),
            video: VideoSettings::default(),
            audio: AudioSettings::default(),
            clip_timing: ClipTimingSettings::default(),
            hotkeys: HotkeySettings::default(),

            auto_start_with_league: true,
            minimize_to_tray: true,
            show_notifications: true,
        }
    }
}

// ============================================================================
// Event Filter Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventFilterSettings {
    // 킬 관련
    pub record_kills: bool,
    pub record_multikills: bool,
    pub record_first_blood: bool,

    // 데스 관련
    pub record_deaths: bool,
    pub record_shutdown: bool,

    // 어시스트 관련
    pub record_assists: bool,

    // 오브젝트
    pub record_dragon: bool,
    pub record_baron: bool,
    pub record_elder: bool,
    pub record_herald: bool,

    // 구조물
    pub record_turret: bool,
    pub record_inhibitor: bool,
    pub record_nexus: bool,

    // 특수 이벤트
    pub record_ace: bool,
    pub record_game_end: bool,
    pub record_steal: bool,

    // 우선순위 필터
    pub min_priority: u8, // 1-5
}

impl Default for EventFilterSettings {
    fn default() -> Self {
        Self {
            // 기본적으로 하이라이트만 녹화
            record_kills: true,
            record_multikills: true,
            record_first_blood: true,

            record_deaths: false,  // 데스는 기본적으로 OFF
            record_shutdown: false,

            record_assists: false, // 어시스트는 기본적으로 OFF

            record_dragon: true,
            record_baron: true,
            record_elder: true,
            record_herald: true,

            record_turret: false, // 타워는 너무 많아서 OFF
            record_inhibitor: true,
            record_nexus: true,

            record_ace: true,
            record_game_end: true,
            record_steal: true,

            min_priority: 2, // 우선순위 2 이상만
        }
    }
}

// ============================================================================
// Game Mode Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameModeSettings {
    pub record_ranked_solo: bool,
    pub record_ranked_flex: bool,
    pub record_normal: bool,
    pub record_quick_play: bool,
    pub record_aram: bool,
    pub record_arena: bool,
    pub record_special: bool,
    pub record_custom: bool,
    pub record_practice: bool,
}

impl Default for GameModeSettings {
    fn default() -> Self {
        Self {
            record_ranked_solo: true,
            record_ranked_flex: true,
            record_normal: true,
            record_quick_play: true,
            record_aram: true,
            record_arena: true,
            record_special: false, // 특별 모드는 기본 OFF
            record_custom: false,  // 커스텀은 기본 OFF
            record_practice: false, // 연습은 기본 OFF
        }
    }
}

// ============================================================================
// Video Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub resolution: Resolution,
    pub frame_rate: FrameRate,
    pub bitrate_preset: BitratePreset,
    pub codec: VideoCodec,
    pub encoder: EncoderPreference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    R1920x1080, // 1080p (추천)
    R2560x1440, // 1440p
    R3840x2160, // 4K
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrameRate {
    Fps30,
    Fps60,  // 추천
    Fps120,
    Fps144,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BitratePreset {
    Low,           // 10 Mbps (720p60)
    Medium,        // 20 Mbps (1080p60) - 추천
    High,          // 40 Mbps (1440p60)
    VeryHigh,      // 80 Mbps (4K60)
    Custom(u32),   // 사용자 지정 (kbps)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VideoCodec {
    H264, // 호환성 최고
    H265, // 효율성 최고 (추천)
    Av1,  // 차세대 (실험적)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncoderPreference {
    Auto,     // 자동 선택 (추천)
    Nvenc,    // NVIDIA GPU
    Qsv,      // Intel GPU
    Amf,      // AMD GPU
    Software, // CPU (느림, 호환성 높음)
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            resolution: Resolution::R1920x1080,
            frame_rate: FrameRate::Fps60,
            bitrate_preset: BitratePreset::Medium,
            codec: VideoCodec::H265,
            encoder: EncoderPreference::Auto,
        }
    }
}

// ============================================================================
// Audio Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    // 마이크 녹음
    pub record_microphone: bool,
    pub microphone_device: Option<String>,
    pub microphone_volume: u8, // 0-200%

    // 시스템 오디오 녹음
    pub record_system_audio: bool,
    pub system_audio_device: Option<String>,
    pub system_audio_volume: u8, // 0-200%

    // 오디오 품질
    pub sample_rate: SampleRate,
    pub bitrate: AudioBitrate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleRate {
    Hz44100,
    Hz48000, // 추천
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioBitrate {
    Kbps128,
    Kbps192, // 추천
    Kbps256,
    Kbps320,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            record_microphone: true,
            microphone_device: None, // 기본 장치
            microphone_volume: 120,  // 120%

            record_system_audio: true,
            system_audio_device: None, // 기본 장치
            system_audio_volume: 100,  // 100%

            sample_rate: SampleRate::Hz48000,
            bitrate: AudioBitrate::Kbps192,
        }
    }
}

// ============================================================================
// Clip Timing Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipTimingSettings {
    // 기본 클립 길이
    pub default_pre_duration: u32,  // 이벤트 이전 (초)
    pub default_post_duration: u32, // 이벤트 이후 (초)

    // 이벤트별 커스텀 타이밍
    pub event_timings: HashMap<String, EventTiming>,

    // 이벤트 병합
    pub merge_consecutive_events: bool,
    pub merge_time_threshold: f64, // 15초 기본
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTiming {
    pub pre_duration: u32,
    pub post_duration: u32,
}

impl Default for ClipTimingSettings {
    fn default() -> Self {
        let mut event_timings = HashMap::new();

        // 멀티킬은 길게
        event_timings.insert(
            "multikill".to_string(),
            EventTiming {
                pre_duration: 15,
                post_duration: 5,
            },
        );

        // 스틸은 더 길게 (빌드업 포함)
        event_timings.insert(
            "steal".to_string(),
            EventTiming {
                pre_duration: 20,
                post_duration: 5,
            },
        );

        // 일반 킬은 짧게
        event_timings.insert(
            "kill".to_string(),
            EventTiming {
                pre_duration: 10,
                post_duration: 3,
            },
        );

        Self {
            default_pre_duration: 10,
            default_post_duration: 3,
            event_timings,
            merge_consecutive_events: true,
            merge_time_threshold: 15.0,
        }
    }
}

impl ClipTimingSettings {
    /// Get timing for a specific event type
    pub fn get_timing_for_event(&self, event_type: &str) -> EventTiming {
        self.event_timings
            .get(event_type)
            .cloned()
            .unwrap_or(EventTiming {
                pre_duration: self.default_pre_duration,
                post_duration: self.default_post_duration,
            })
    }
}

// ============================================================================
// Hotkey Settings
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub manual_save_clip: String,  // "F8" 기본
    pub toggle_recording: String,  // "F9" 기본
    pub delete_last_clip: String,  // "F10" 기본
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            manual_save_clip: "F8".to_string(),
            toggle_recording: "F9".to_string(),
            delete_last_clip: "F10".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = RecordingSettings::default();

        // Event filter defaults
        assert!(settings.event_filter.record_kills);
        assert!(settings.event_filter.record_multikills);
        assert!(!settings.event_filter.record_deaths);
        assert_eq!(settings.event_filter.min_priority, 2);

        // Game mode defaults
        assert!(settings.game_mode.record_ranked_solo);
        assert!(!settings.game_mode.record_practice);

        // Video defaults
        assert!(matches!(settings.video.resolution, Resolution::R1920x1080));
        assert!(matches!(settings.video.frame_rate, FrameRate::Fps60));
        assert!(matches!(settings.video.codec, VideoCodec::H265));

        // Audio defaults
        assert!(settings.audio.record_microphone);
        assert_eq!(settings.audio.microphone_volume, 120);
        assert_eq!(settings.audio.system_audio_volume, 100);

        // Clip timing defaults
        assert_eq!(settings.clip_timing.default_pre_duration, 10);
        assert_eq!(settings.clip_timing.default_post_duration, 3);
        assert!(settings.clip_timing.merge_consecutive_events);
        assert_eq!(settings.clip_timing.merge_time_threshold, 15.0);

        // Hotkey defaults
        assert_eq!(settings.hotkeys.manual_save_clip, "F8");
        assert_eq!(settings.hotkeys.toggle_recording, "F9");
        assert_eq!(settings.hotkeys.delete_last_clip, "F10");
    }

    #[test]
    fn test_event_timing_lookup() {
        let settings = ClipTimingSettings::default();

        let multikill_timing = settings.get_timing_for_event("multikill");
        assert_eq!(multikill_timing.pre_duration, 15);
        assert_eq!(multikill_timing.post_duration, 5);

        let unknown_timing = settings.get_timing_for_event("unknown_event");
        assert_eq!(unknown_timing.pre_duration, 10); // fallback to default
        assert_eq!(unknown_timing.post_duration, 3);
    }

    #[test]
    fn test_serialization() {
        let settings = RecordingSettings::default();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&settings).unwrap();
        assert!(json.contains("event_filter"));
        assert!(json.contains("game_mode"));

        // Deserialize back
        let deserialized: RecordingSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_filter.min_priority, settings.event_filter.min_priority);
    }
}
