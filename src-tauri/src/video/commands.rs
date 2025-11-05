use crate::auth::middleware::{require_auth, require_tier};
use crate::auth::SubscriptionTier;
use crate::storage::models::ClipMetadata;
use crate::video::VideoProcessor;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_clips(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<Vec<ClipMetadata>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state
        .storage
        .load_clip_metadata(&game_id)
        .map_err(|e| e.to_string())
}

/// Extract a clip from a video file (PRO feature)
#[tauri::command]
pub async fn extract_clip(
    state: State<'_, AppState>,
    input_path: String,
    output_path: String,
    start_time: f64,
    duration: f64,
) -> Result<String, String> {
    // Require PRO tier for manual clip extraction
    require_tier(&state.auth, SubscriptionTier::Pro).map_err(|e| e.to_string())?;
    let processor = VideoProcessor::new();

    let result_path = processor
        .extract_clip(
            PathBuf::from(&input_path),
            PathBuf::from(&output_path),
            start_time,
            duration,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Compose multiple clips into a YouTube Short (9:16 aspect ratio) (PRO feature)
#[tauri::command]
pub async fn compose_shorts(
    state: State<'_, AppState>,
    clip_paths: Vec<String>,
    output_path: String,
) -> Result<String, String> {
    // Require PRO tier for YouTube Shorts composition
    require_tier(&state.auth, SubscriptionTier::Pro).map_err(|e| e.to_string())?;
    let processor = VideoProcessor::new();

    // Convert String paths to PathBuf
    let paths: Vec<PathBuf> = clip_paths.iter().map(PathBuf::from).collect();

    // Standard YouTube Shorts resolution: 1080x1920 (9:16)
    let result_path = processor
        .compose_shorts(&paths, PathBuf::from(&output_path), 1080, 1920)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Generate a thumbnail from a video file (PRO feature)
#[tauri::command]
pub async fn generate_thumbnail(
    state: State<'_, AppState>,
    input_path: String,
    output_path: String,
    time_offset: f64,
) -> Result<String, String> {
    // Require PRO tier for thumbnail generation
    require_tier(&state.auth, SubscriptionTier::Pro).map_err(|e| e.to_string())?;
    let processor = VideoProcessor::new();

    let result_path = processor
        .generate_thumbnail(
            PathBuf::from(&input_path),
            PathBuf::from(&output_path),
            time_offset,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Get video duration in seconds
#[tauri::command]
pub async fn get_video_duration(
    state: State<'_, AppState>,
    input_path: String,
) -> Result<f64, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;
    let processor = VideoProcessor::new();

    let duration = processor
        .get_duration(PathBuf::from(&input_path))
        .await
        .map_err(|e| e.to_string())?;

    Ok(duration)
}

/// Delete a clip from storage
#[tauri::command]
pub async fn delete_clip(
    state: State<'_, AppState>,
    clip_file_path: String,
    game_id: String,
) -> Result<(), String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    let path = PathBuf::from(&clip_file_path);

    // Delete the video file
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        tracing::info!("Deleted clip file: {:?}", path);
    }

    // Delete from JSON storage
    state.storage.delete_clip_metadata(&game_id, &clip_file_path)
        .map_err(|e| format!("Failed to delete clip metadata: {}", e))?;

    tracing::info!("Successfully deleted clip and metadata: {:?}", path);
    Ok(())
}
