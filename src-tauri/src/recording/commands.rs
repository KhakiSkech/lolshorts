use super::{GameEvent, RecordingStatus};
use crate::auth::middleware::require_auth;
use crate::AppState;
use std::path::PathBuf;
use std::time::Instant;
use tauri::State;

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    // FREE tier feature - no authentication required
    state
        .recording_manager
        .write()
        .await
        .start_replay_buffer()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<(), String> {
    // FREE tier feature - no authentication required
    state
        .recording_manager
        .write()
        .await
        .stop_replay_buffer()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recording_status(state: State<'_, AppState>) -> Result<String, String> {
    // FREE tier feature - no authentication required
    let status = state.recording_manager.read().await.get_state().await;

    // Convert RecordingStatus to string for frontend
    let status_str = match status {
        RecordingStatus::Idle => "idle",
        RecordingStatus::Buffering => "buffering",
        RecordingStatus::Recording => "recording",
        RecordingStatus::Paused => "paused",
        RecordingStatus::Processing => "processing",
        RecordingStatus::Error => "error",
    };

    Ok(status_str.to_string())
}

#[tauri::command]
pub async fn start_auto_capture(state: State<'_, AppState>) -> Result<(), String> {
    // FREE tier feature - no authentication required

    // Start the replay buffer
    state
        .recording_manager
        .write()
        .await
        .start_replay_buffer()
        .await
        .map_err(|e| e.to_string())?;

    // Start event monitoring to automatically capture highlights
    state
        .auto_clip_manager
        .start_event_monitoring()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn stop_auto_capture(state: State<'_, AppState>) -> Result<(), String> {
    // FREE tier feature - no authentication required

    // Stop event monitoring first
    state
        .auto_clip_manager
        .stop_event_monitoring()
        .await
        .map_err(|e| e.to_string())?;

    // Stop the replay buffer
    state
        .recording_manager
        .write()
        .await
        .stop_replay_buffer()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn save_replay(state: State<'_, AppState>, seconds: u32) -> Result<PathBuf, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;
    // Create a dummy GameEvent for manual save
    let manual_event = GameEvent {
        event_id: 0,
        event_name: "ManualSave".to_string(),
        event_time: 0.0,
        killer_name: None,
        victim_name: None,
        assisters: vec![],
        priority: 3,
        timestamp: Instant::now(),
    };

    // Save clip with new API (returns PathBuf directly)
    let clip_path = state
        .recording_manager
        .read()
        .await
        .save_clip(
            &manual_event,
            format!("manual_{}", Instant::now().elapsed().as_secs()),
            3, // priority = 3 (medium priority)
            seconds as f64,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(clip_path)
}

#[tauri::command]
pub async fn get_saved_clips(
    state: State<'_, AppState>,
) -> Result<Vec<crate::storage::models::ClipMetadata>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Get all games
    let games = state.storage.list_games().map_err(|e| e.to_string())?;

    // Collect all clips from all games
    let mut all_clips = Vec::new();
    for game_id in games {
        let clips = state
            .storage
            .load_clip_metadata(&game_id)
            .map_err(|e| e.to_string())?;
        all_clips.extend(clips);
    }

    Ok(all_clips)
}

#[tauri::command]
pub async fn clear_saved_clips(state: State<'_, AppState>) -> Result<(), String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Get all games and delete them
    let games = state.storage.list_games().map_err(|e| e.to_string())?;

    for game_id in games {
        state
            .storage
            .delete_game(&game_id)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// List available audio devices (Windows DirectShow)
#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<crate::recording::audio::AudioDevice>, String> {
    crate::recording::audio::list_audio_devices().map_err(|e| e.to_string())
}

/// Get recording quality info (encoder, bitrate, resolution)
#[tauri::command]
pub async fn get_recording_quality_info(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    use serde_json::json;

    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    let manager = state.recording_manager.read().await;
    let quality_info = manager.get_quality_info();

    Ok(json!({
        "encoder": quality_info.encoder,
        "codec": quality_info.codec,
        "resolution": quality_info.resolution,
        "fps": quality_info.fps,
        "bitrate_mbps": quality_info.bitrate_mbps,
        "audio_enabled": quality_info.audio_enabled,
    }))
}

// Screenshot capture moved to screenshot::commands module
