use crate::auth::middleware::{require_auth, require_tier};
use crate::auth::SubscriptionTier;
use crate::storage::models::ClipMetadata;
use crate::utils::security;
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

    // Validate game_id (prevent SQL injection)
    let validated_game_id = security::validate_game_id(&game_id).map_err(|e| e.to_string())?;

    state
        .storage
        .load_clip_metadata(&validated_game_id)
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

    // Security validation
    let validated_input =
        security::validate_video_input_path(&input_path).map_err(|e| e.to_string())?;
    let validated_output =
        security::validate_video_output_path(&output_path).map_err(|e| e.to_string())?;
    let validated_start_time =
        security::validate_time_offset(start_time).map_err(|e| e.to_string())?;
    let validated_duration = security::validate_duration(duration).map_err(|e| e.to_string())?;

    let processor = VideoProcessor::new();

    let result_path = processor
        .extract_clip(
            validated_input,
            validated_output,
            validated_start_time,
            validated_duration,
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

    // Security validation
    let validated_clips: Result<Vec<PathBuf>, String> = clip_paths
        .iter()
        .map(|p| security::validate_video_input_path(p).map_err(|e| e.to_string()))
        .collect();
    let validated_clips = validated_clips?;

    let validated_output =
        security::validate_video_output_path(&output_path).map_err(|e| e.to_string())?;

    let processor = VideoProcessor::new();

    // Standard YouTube Shorts resolution: 1080x1920 (9:16)
    let result_path = processor
        .compose_shorts(&validated_clips, validated_output, 1080, 1920)
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

    // Security validation
    let validated_input =
        security::validate_video_input_path(&input_path).map_err(|e| e.to_string())?;
    let validated_output =
        security::validate_thumbnail_path(&output_path).map_err(|e| e.to_string())?;
    let validated_time_offset =
        security::validate_time_offset(time_offset).map_err(|e| e.to_string())?;

    let processor = VideoProcessor::new();

    let result_path = processor
        .generate_thumbnail(validated_input, validated_output, validated_time_offset)
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

    // Security validation
    let validated_input =
        security::validate_video_input_path(&input_path).map_err(|e| e.to_string())?;

    let processor = VideoProcessor::new();

    let duration = processor
        .get_duration(validated_input)
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

    // Security validation
    let validated_path =
        security::validate_video_input_path(&clip_file_path).map_err(|e| e.to_string())?;
    let validated_game_id = security::validate_game_id(&game_id).map_err(|e| e.to_string())?;

    // Delete the video file
    if validated_path.exists() {
        std::fs::remove_file(&validated_path).map_err(|e| e.to_string())?;
        tracing::info!("Deleted clip file: {:?}", validated_path);
    }

    // Delete from JSON storage
    state
        .storage
        .delete_clip_metadata(&validated_game_id, &clip_file_path)
        .map_err(|e| format!("Failed to delete clip metadata: {}", e))?;

    tracing::info!(
        "Successfully deleted clip and metadata: {:?}",
        validated_path
    );
    Ok(())
}

/// Start auto-edit composition for YouTube Shorts
///
/// This is the main entry point for automated Shorts generation.
/// It will intelligently select clips, apply canvas overlays, mix audio,
/// and produce a final 60/120/180 second video ready for upload.
///
/// Quota limits:
/// - FREE tier: 5 auto-edits per month
/// - PRO tier: Unlimited
#[tauri::command]
pub async fn start_auto_edit(
    state: State<'_, AppState>,
    config: AutoEditConfig,
) -> Result<AutoEditResult, String> {
    // Require authentication (both FREE and PRO can use auto-edit)
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Check tier and quota
    let tier = state.auth.get_tier().map_err(|e| e.to_string())?;
    let is_pro = matches!(tier, SubscriptionTier::Pro);

    // Check quota before starting
    let remaining = state
        .storage
        .check_auto_edit_quota(is_pro)
        .map_err(|e| format!("Quota check failed: {}", e))?;

    tracing::info!(
        "Auto-edit quota check passed: tier={:?}, remaining={}",
        tier,
        if is_pro { "unlimited".to_string() } else { remaining.to_string() }
    );

    // Generate unique job ID
    let job_id = format!("auto_edit_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));

    tracing::info!(
        "Starting auto-edit job: {} with target duration: {}s",
        job_id,
        config.target_duration
    );

    // Start auto-composition
    let result = state
        .auto_composer
        .compose(config, job_id.clone())
        .await
        .map_err(|e| {
            tracing::error!("Auto-edit failed for job {}: {}", job_id, e);
            format!("Auto-edit failed: {}", e)
        })?;

    // Increment usage counter on success (only for FREE tier, PRO is unlimited)
    if !is_pro {
        state
            .storage
            .increment_auto_edit_usage()
            .map_err(|e| {
                tracing::error!("Failed to increment usage: {}", e);
                // Don't fail the whole operation if usage increment fails
                format!("Warning: Usage tracking failed: {}", e)
            })
            .ok();
    }

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

    // Security validation
    let validated_template_id =
        security::validate_template_id(&template_id).map_err(|e| e.to_string())?;

    let template = state
        .storage
        .load_canvas_template(&validated_template_id)
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

    // Security validation
    let validated_template_id =
        security::validate_template_id(&template_id).map_err(|e| e.to_string())?;

    state
        .storage
        .delete_canvas_template(&validated_template_id)
        .map_err(|e| format!("Failed to delete canvas template: {}", e))?;

    Ok(())
}
