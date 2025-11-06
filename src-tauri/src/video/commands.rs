use crate::auth::middleware::{require_auth, require_tier};
use crate::auth::SubscriptionTier;
use crate::storage::models::ClipMetadata;
use crate::video::{AutoEditConfig, AutoEditProgress, AutoEditResult, VideoProcessor};
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

/// Start auto-edit composition for YouTube Shorts (PRO feature)
///
/// This is the main entry point for automated Shorts generation.
/// It will intelligently select clips, apply canvas overlays, mix audio,
/// and produce a final 60/120/180 second video ready for upload.
#[tauri::command]
pub async fn start_auto_edit(
    state: State<'_, AppState>,
    config: AutoEditConfig,
) -> Result<AutoEditResult, String> {
    // Require PRO tier for auto-edit feature
    require_tier(&state.auth, SubscriptionTier::Pro).map_err(|e| e.to_string())?;

    // Generate unique job ID
    let job_id = format!("auto_edit_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));

    tracing::info!("Starting auto-edit job: {} with target duration: {}s", job_id, config.target_duration);

    // Start auto-composition
    let result = state.auto_composer
        .compose(config, job_id.clone())
        .await
        .map_err(|e| {
            tracing::error!("Auto-edit failed for job {}: {}", job_id, e);
            format!("Auto-edit failed: {}", e)
        })?;

    tracing::info!("Auto-edit completed successfully: {:?}", result.output_path);
    Ok(result)
}

/// Get progress of an auto-edit job
///
/// Returns current status, progress percentage, and estimated completion time.
/// Frontend should poll this endpoint every 1-2 seconds to update UI.
#[tauri::command]
pub async fn get_auto_edit_progress(
    state: State<'_, AppState>,
) -> Result<Option<AutoEditProgress>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    let progress = state.auto_composer.get_progress().await;
    Ok(progress)
}

// ========================================================================
// Canvas Template Management
// ========================================================================

/// Save a canvas template to the library for reuse
#[tauri::command]
pub async fn save_canvas_template(
    state: State<'_, AppState>,
    template: crate::video::CanvasTemplate,
) -> Result<(), String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state
        .storage
        .save_canvas_template(&template)
        .map_err(|e| format!("Failed to save canvas template: {}", e))?;

    Ok(())
}

/// Load a canvas template by ID
#[tauri::command]
pub async fn load_canvas_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<crate::video::CanvasTemplate, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    let template = state
        .storage
        .load_canvas_template(&template_id)
        .map_err(|e| format!("Failed to load canvas template: {}", e))?;

    Ok(template)
}

/// List all available canvas templates
#[tauri::command]
pub async fn list_canvas_templates(
    state: State<'_, AppState>,
) -> Result<Vec<crate::storage::CanvasTemplateInfo>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    let templates = state
        .storage
        .list_canvas_templates()
        .map_err(|e| format!("Failed to list canvas templates: {}", e))?;

    Ok(templates)
}

/// Delete a canvas template
#[tauri::command]
pub async fn delete_canvas_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<(), String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state
        .storage
        .delete_canvas_template(&template_id)
        .map_err(|e| format!("Failed to delete canvas template: {}", e))?;

    Ok(())
}
