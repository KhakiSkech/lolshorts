// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod auth;
mod storage;
mod feature_gate;
mod lcu;
mod recording;
mod video;
mod supabase;
mod payments;
mod settings;
mod hotkey;
mod utils;

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber;

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
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file (development)
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    tracing::info!("Starting LoLShorts application...");

    // Get application data directory
    let app_data_dir = dirs::data_dir()
        .expect("Failed to get data directory")
        .join("lolshorts");

    // Initialize storage
    let storage = Arc::new(
        storage::Storage::new(&app_data_dir)
            .expect("Failed to initialize storage"),
    );

    // Initialize auth manager
    let auth = Arc::new(auth::AuthManager::new());

    // Initialize feature gate
    let feature_gate = Arc::new(feature_gate::FeatureGate::new(auth.clone()));

    // Initialize recording manager (platform-specific backend)
    let recordings_dir = app_data_dir.join("recordings");
    std::fs::create_dir_all(&recordings_dir).expect("Failed to create recordings directory");

    let recording_manager = Arc::new(RwLock::new(
        recording::initialize_recording_backend(recordings_dir)
            .expect("Failed to initialize recording backend")
    ));

    tracing::info!("Recording backend initialized for {}", recording::Platform::current().name());

    // Load recording settings
    let recording_settings = Arc::new(RwLock::new(
        settings::models::RecordingSettings::load()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to load recording settings, using defaults: {}", e);
                settings::models::RecordingSettings::default()
            })
    ));

    tracing::info!("Recording settings loaded");

    // Initialize Auto Clip Manager
    let auto_clip_manager = Arc::new(
        recording::auto_clip_manager::AutoClipManager::new(
            Arc::clone(&recording_manager),
            Arc::clone(&storage),
            Arc::clone(&recording_settings),
        )
    );

    tracing::info!("Auto Clip Manager initialized");

    // Initialize Hotkey Manager
    let hotkey_manager = Arc::new(hotkey::HotkeyManager::new());

    tracing::info!("Hotkey Manager initialized");

    // Initialize Metrics Collector
    let metrics_collector = Arc::new(utils::metrics::MetricsCollector::new(
        utils::metrics::HealthThresholds::default()
    ));

    tracing::info!("Metrics Collector initialized");

    // Initialize Cleanup Manager
    let cleanup_config = utils::cleanup::CleanupConfig::default();
    let cleanup_manager = Arc::new(utils::cleanup::CleanupManager::new(
        app_data_dir.clone(),
        cleanup_config
    ));

    // Run startup cleanup
    if let Err(e) = cleanup_manager.cleanup_on_startup().await {
        tracing::error!("Startup cleanup failed: {}", e);
    }

    tracing::info!("Cleanup Manager initialized");

    let app_state = AppState {
        storage,
        auth,
        feature_gate,
        recording_manager: Arc::clone(&recording_manager),
        auto_clip_manager: Arc::clone(&auto_clip_manager),
        recording_settings,
        hotkey_manager: Arc::clone(&hotkey_manager),
        metrics_collector: Arc::clone(&metrics_collector),
        cleanup_manager: Arc::clone(&cleanup_manager),
    };

    // Start hotkey system with callbacks
    let recording_manager_hotkey = Arc::clone(&recording_manager);
    let auto_clip_manager_hotkey = Arc::clone(&auto_clip_manager);

    tokio::spawn(async move {
        hotkey_manager.start(move |event| {
            let rm = Arc::clone(&recording_manager_hotkey);
            let acm = Arc::clone(&auto_clip_manager_hotkey);

            tokio::spawn(async move {
                use hotkey::HotkeyEvent;

                match event {
                    HotkeyEvent::ToggleAutoCapture => {
                        // Check if auto-capture is running
                        let is_monitoring = acm.is_monitoring().await;

                        if is_monitoring {
                            // Stop auto-capture
                            tracing::info!("Hotkey F8: Stopping auto-capture");
                            if let Err(e) = acm.stop_event_monitoring().await {
                                tracing::error!("Failed to stop auto-capture: {}", e);
                            }
                            if let Err(e) = rm.write().await.stop_replay_buffer().await {
                                tracing::error!("Failed to stop replay buffer: {}", e);
                            }
                        } else {
                            // Start auto-capture
                            tracing::info!("Hotkey F8: Starting auto-capture");
                            if let Err(e) = rm.write().await.start_replay_buffer().await {
                                tracing::error!("Failed to start replay buffer: {}", e);
                            }
                            if let Err(e) = acm.start_event_monitoring().await {
                                tracing::error!("Failed to start event monitoring: {}", e);
                            }
                        }
                    },
                    HotkeyEvent::SaveReplay60 => {
                        // Save last 60 seconds
                        tracing::info!("Hotkey F9: Saving 60s replay");

                        use std::time::Instant;
                        use crate::recording::GameEvent;

                        let manual_event = GameEvent {
                            event_id: 0,
                            event_name: "HotkeyReplay60".to_string(),
                            event_time: 0.0,
                            killer_name: None,
                            victim_name: None,
                            assisters: vec![],
                            priority: 3,
                            timestamp: Instant::now(),
                        };

                        match rm.read().await.save_clip(
                            &manual_event,
                            format!("hotkey_60s_{}", Instant::now().elapsed().as_secs()),
                            3,
                            60.0,
                        ).await {
                            Ok(path) => tracing::info!("Saved 60s replay to: {:?}", path),
                            Err(e) => tracing::error!("Failed to save 60s replay: {}", e),
                        }
                    },
                    HotkeyEvent::SaveReplay30 => {
                        // Save last 30 seconds
                        tracing::info!("Hotkey F10: Saving 30s replay");

                        use std::time::Instant;
                        use crate::recording::GameEvent;

                        let manual_event = GameEvent {
                            event_id: 0,
                            event_name: "HotkeyReplay30".to_string(),
                            event_time: 0.0,
                            killer_name: None,
                            victim_name: None,
                            assisters: vec![],
                            priority: 2,
                            timestamp: Instant::now(),
                        };

                        match rm.read().await.save_clip(
                            &manual_event,
                            format!("hotkey_30s_{}", Instant::now().elapsed().as_secs()),
                            2,
                            30.0,
                        ).await {
                            Ok(path) => tracing::info!("Saved 30s replay to: {:?}", path),
                            Err(e) => tracing::error!("Failed to save 30s replay: {}", e),
                        }
                    },
                }
            });
        }).await
            .unwrap_or_else(|e| tracing::error!("Failed to start hotkey system: {}", e));
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            auth::commands::login,
            auth::commands::signup,
            auth::commands::logout,
            auth::commands::get_user_status,
            auth::commands::get_license_info,
            auth::commands::get_user_license,
            auth::commands::refresh_token,
            // Recording commands
            recording::commands::start_recording,
            recording::commands::stop_recording,
            recording::commands::get_recording_status,
            recording::commands::start_auto_capture,
            recording::commands::stop_auto_capture,
            recording::commands::save_replay,
            recording::commands::get_saved_clips,
            recording::commands::clear_saved_clips,
            recording::commands::list_audio_devices,
            recording::commands::get_recording_quality_info,
            // Video commands
            video::commands::get_clips,
            video::commands::extract_clip,
            video::commands::compose_shorts,
            video::commands::generate_thumbnail,
            video::commands::get_video_duration,
            video::commands::delete_clip,
            // LCU commands
            lcu::commands::connect_lcu,
            lcu::commands::check_lcu_status,
            lcu::commands::get_current_game,
            lcu::commands::is_in_game,
            // Payment commands
            payments::commands::create_subscription,
            payments::commands::confirm_payment,
            payments::commands::get_subscription_status,
            // Subscription management commands
            payments::subscription_commands::get_subscription_details,
            payments::subscription_commands::cancel_subscription,
            // Storage commands
            storage::commands::list_games,
            storage::commands::get_game_metadata,
            storage::commands::save_game_metadata,
            storage::commands::get_game_events,
            storage::commands::save_game_events,
            storage::commands::save_clip_metadata,
            storage::commands::delete_game,
            storage::commands::get_storage_stats,
            storage::commands::list_clips,
            // Settings commands
            settings::commands::get_recording_settings,
            settings::commands::save_recording_settings,
            settings::commands::reset_settings_to_default,
            // Utils commands
            utils::commands::get_recording_metrics,
            utils::commands::get_system_metrics,
            utils::commands::get_health_status,
            utils::commands::get_app_version,
            utils::commands::force_cleanup,
            utils::commands::get_disk_space_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
