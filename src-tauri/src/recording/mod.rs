#![allow(dead_code)]
// Platform-specific recording implementations
#[cfg(target_os = "windows")]
mod windows_backend;

#[cfg(target_os = "macos")]
mod macos_backend; // Will be implemented in Wave 5

// Common types and interfaces
pub mod audio;
pub mod auto_clip_manager;
pub mod commands;
pub mod live_client;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

// Re-export the platform-specific recorder as RecordingManager
#[cfg(target_os = "windows")]
pub use windows_backend::WindowsRecorder as RecordingManager;

#[cfg(target_os = "macos")]
pub use macos_backend::MacOSRecorder as RecordingManager;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
compile_error!("LoLShorts only supports Windows and macOS");

/// Recording status states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    /// Not recording, replay buffer disabled
    Idle,
    /// Replay buffer active, not in-game
    Buffering,
    /// In-game, actively recording
    Recording,
    /// Temporarily paused
    Paused,
    /// Processing video (encoding/concatenating)
    Processing,
    /// Error state
    Error,
}

impl Default for RecordingStatus {
    fn default() -> Self {
        Self::Idle
    }
}

/// Recording statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecordingStats {
    /// Total frames captured
    pub frames_captured: u64,
    /// Total clips created
    pub clips_created: u64,
    /// Current buffer size in MB
    pub buffer_size_mb: f64,
    /// Average FPS
    pub average_fps: f64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
}

/// Game event types for clip creation
/// Note: Serialize only - Instant cannot be deserialized
#[derive(Debug, Clone, Serialize)]
pub struct GameEvent {
    pub event_id: u64,
    pub event_name: String,
    pub event_time: f64,
    pub killer_name: Option<String>,
    pub victim_name: Option<String>,
    pub assisters: Vec<String>,
    pub priority: u8, // 1-5 (pentakill = 5)
    #[serde(skip)]
    pub timestamp: Instant,
}

/// Metadata for saved clips
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMetadata {
    pub clip_id: String,
    pub game_id: String,
    pub event_type: String,
    pub priority: u8,
    pub duration_secs: f64,
    pub file_path: PathBuf,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Platform detection utilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows,
    MacOS,
    Unsupported,
}

impl Platform {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return Platform::Windows;

        #[cfg(target_os = "macos")]
        return Platform::MacOS;

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        return Platform::Unsupported;
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::Windows)
    }

    pub fn is_macos(&self) -> bool {
        matches!(self, Platform::MacOS)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
            Platform::Unsupported => "Unsupported",
        }
    }
}

// Platform-specific initialization
pub fn initialize_recording_backend(output_dir: PathBuf) -> Result<RecordingManager> {
    let platform = Platform::current();

    tracing::info!("Initializing recording backend for {}", platform.name());

    #[cfg(target_os = "windows")]
    {
        tracing::info!("Using windows-capture with H.265 hardware acceleration");
        RecordingManager::new(output_dir)
    }

    #[cfg(target_os = "macos")]
    {
        tracing::info!("Using XCap + ffmpeg-sidecar (Wave 5 implementation)");
        RecordingManager::new(output_dir)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        compile_error!("Unsupported platform");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::current();

        #[cfg(target_os = "windows")]
        assert_eq!(platform, Platform::Windows);

        #[cfg(target_os = "macos")]
        assert_eq!(platform, Platform::MacOS);

        assert!(platform.is_windows() || platform.is_macos());
    }

    #[test]
    fn test_platform_names() {
        assert_eq!(Platform::Windows.name(), "Windows");
        assert_eq!(Platform::MacOS.name(), "macOS");
        assert_eq!(Platform::Unsupported.name(), "Unsupported");
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
