// Library exports for integration testing
// This file allows integration tests to access the application modules

pub mod auth;
pub mod feature_gate;
pub mod hotkey;
pub mod lcu;
pub mod payments;
pub mod recording;
pub mod settings;
pub mod storage;
pub mod supabase;
pub mod utils;
pub mod video;
pub mod youtube;

use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state shared across all Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<storage::Storage>,
    pub auth: Arc<auth::AuthManager>,
    pub feature_gate: Arc<feature_gate::FeatureGate>,
    pub recording_manager: Arc<RwLock<recording::RecordingManager>>,
    pub auto_clip_manager: Arc<recording::auto_clip_manager::AutoClipManager>,
    pub recording_settings: Arc<RwLock<settings::models::RecordingSettings>>,
    pub hotkey_manager: Arc<hotkey::HotkeyManager>,
    pub metrics_collector: Arc<utils::metrics::MetricsCollector>,
    pub cleanup_manager: Arc<utils::cleanup::CleanupManager>,
    pub auto_composer: Arc<video::AutoComposer>,
    pub youtube_manager: Arc<youtube::YouTubeManager>,
}
