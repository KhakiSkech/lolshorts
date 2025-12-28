// Complete windows-capture v2 + FFmpeg integration
mod integration_backend;

// macOS-specific recording backend
#[cfg(target_os = "macos")]
mod mac_backend;
#[cfg(target_os = "macos")]
mod mac_audio;
#[cfg(target_os = "macos")]
mod mac_audio_core;
#[cfg(target_os = "macos")]
mod mac_screen_capture;
#[cfg(target_os = "macos")]
mod mac_ffmpeg;


// Common types and interfaces
pub mod audio;
pub mod auto_clip_manager;
pub mod commands;
pub mod game_monitor;
pub mod live_client;


// Complete integration exports
pub use integration_backend::RecordingManager;


// Platform-specific exports
#[cfg(target_os = "macos")]
pub use mac_backend::{MacRecordingManager, MacRecordingConfig, MacRecordingStatus};
#[cfg(target_os = "macos")]
pub use mac_audio::{MacAudioManager, MacAudioDevice, MacAudioDeviceType};
#[cfg(target_os = "macos")]
pub use mac_audio_core::{CoreAudioManager, CoreAudioDevice, CoreAudioDeviceType};
#[cfg(target_os = "macos")]
pub use mac_screen_capture::{MacScreenCaptureManager, MacScreenCaptureConfig, MacDisplayInfo};
#[cfg(target_os = "macos")]
pub use mac_ffmpeg::{MacFFmpegConfig, MacFFmpegCommandBuilder, detect_available_macos_encoders};





/// Platform detection utilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    #[allow(dead_code)]
    MacOS,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;
    }

    pub fn name(&self) -> &'static str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
        }
    }
}

// Platform-specific initialization
// Note: Use initialize_recording_backend_full for complete initialization with all options
pub use integration_backend::initialize_recording_backend_full;
pub use integration_backend::VideoSettingsConfig;


#[cfg(test)]
mod tests {
    use super::*;
    use crate::recording::integration_backend::RecordingStatus;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();

        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);

        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);

        // Test platform matching
        match platform {
            Platform::Windows | Platform::MacOS => {},
        }
    }

    #[test]
    fn test_platform_names() {
        assert_eq!(Platform::Windows.name(), "Windows");
        assert_eq!(Platform::MacOS.name(), "macOS");
    }

    #[test]
    fn test_recording_status_default() {
        let status = RecordingStatus::default();
        assert_eq!(status, RecordingStatus::Idle);
    }

    #[test]
    fn test_recording_status_equality() {
        assert_eq!(RecordingStatus::Idle, RecordingStatus::Idle);
        assert_ne!(RecordingStatus::Idle, RecordingStatus::Recording);
    }
}
