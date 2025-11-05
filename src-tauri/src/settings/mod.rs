pub mod commands;
pub mod models;
pub mod storage;

// Re-export public types
pub use models::{
    AudioBitrate, AudioSettings, BitratePreset, ClipTimingSettings, EncoderPreference,
    EventFilterSettings, EventTiming, FrameRate, GameModeSettings, HotkeySettings,
    RecordingSettings, Resolution, SampleRate, VideoCodec, VideoSettings,
};

// Re-export commands for easy registration
pub use commands::{get_recording_settings, reset_settings_to_default, save_recording_settings};

// Re-export errors
pub use storage::{Result, SettingsError};
