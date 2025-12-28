use crate::auth::middleware::{require_auth, require_tier};
use crate::auth::SubscriptionTier;
use crate::error::{AppError, AppResult};
use crate::storage::models::ClipMetadata;
use crate::storage::CanvasTemplateInfo;
use crate::utils::security;
use crate::video::{AutoEditConfig, AutoEditProgress, AutoEditResult, VideoProcessor};
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_clips(
    state: State<'_, AppState>,
    game_id: String,
) -> AppResult<Vec<ClipMetadata>> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    // Validate game_id (prevent SQL injection)
    let validated_game_id = security::validate_game_id(&game_id).map_err(|e| AppError::Validation(e.to_string()))?;

    state
        .storage
        .load_clip_metadata(&validated_game_id)
        .map_err(|e| AppError::Database(e.to_string()))
}

/// Extract a clip from a video file (PRO feature)
#[tauri::command]
pub async fn extract_clip(
    state: State<'_, AppState>,
    input_path: String,
    output_path: String,
    start_time: f64,
    duration: f64,
) -> AppResult<String> {
    // Require PRO tier for manual clip extraction
    require_tier(&state.auth, SubscriptionTier::Pro).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_input =
        security::validate_video_input_path(&input_path).map_err(|e| AppError::Validation(e.to_string()))?;
    let validated_output =
        security::validate_video_output_path(&output_path).map_err(|e| AppError::Validation(e.to_string()))?;
    let validated_start_time =
        security::validate_time_offset(start_time).map_err(|e| AppError::Validation(e.to_string()))?;
    let validated_duration = security::validate_duration(duration).map_err(|e| AppError::Validation(e.to_string()))?;

    let processor = VideoProcessor::new_with_fallback();

    let result_path = processor
        .extract_clip(
            validated_input,
            validated_output,
            validated_start_time,
            validated_duration,
        )
        .await
        .map_err(|e| AppError::Video(e.to_string()))?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Compose multiple clips into a YouTube Short (9:16 aspect ratio) (PRO feature)
#[tauri::command]
pub async fn compose_shorts(
    state: State<'_, AppState>,
    clip_paths: Vec<String>,
    output_path: String,
) -> AppResult<String> {
    // Require PRO tier for YouTube Shorts composition
    require_tier(&state.auth, SubscriptionTier::Pro).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_clips: Result<Vec<PathBuf>, AppError> = clip_paths
        .iter()
        .map(|p| security::validate_video_input_path(p).map_err(|e| AppError::Validation(e.to_string())))
        .collect();
    let validated_clips = validated_clips?;

    let validated_output =
        security::validate_video_output_path(&output_path).map_err(|e| AppError::Validation(e.to_string()))?;

    let processor = VideoProcessor::new_with_fallback();

    // Standard YouTube Shorts resolution: 1080x1920 (9:16)
    let result_path = processor
        .compose_shorts(&validated_clips, validated_output, 1080, 1920)
        .await
        .map_err(|e| AppError::Video(e.to_string()))?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Generate a thumbnail from a video file (PRO feature)
#[tauri::command]
pub async fn generate_thumbnail(
    state: State<'_, AppState>,
    input_path: String,
    output_path: String,
    time_offset: f64,
) -> AppResult<String> {
    // Require PRO tier for custom thumbnail generation
    require_tier(&state.auth, SubscriptionTier::Pro).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_input =
        security::validate_video_input_path(&input_path).map_err(|e| AppError::Validation(e.to_string()))?;
    let validated_output =
        security::validate_thumbnail_path(&output_path).map_err(|e| AppError::Validation(e.to_string()))?;
    let validated_time_offset =
        security::validate_time_offset(time_offset).map_err(|e| AppError::Validation(e.to_string()))?;

    let processor = VideoProcessor::new_with_fallback();

    let result_path = processor
        .generate_thumbnail(validated_input, validated_output, validated_time_offset)
        .await
        .map_err(|e| AppError::Video(e.to_string()))?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Generate a basic thumbnail for clip preview (available to all authenticated users)
#[tauri::command]
pub async fn generate_clip_thumbnail(
    state: State<'_, AppState>,
    clip_file_path: String,
) -> AppResult<String> {
    // Require authentication (available to both FREE and PRO)
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_input =
        security::validate_video_input_path(&clip_file_path).map_err(|e| AppError::Validation(e.to_string()))?;

    // Generate thumbnail path in same directory as clip
    let clip_path = std::path::Path::new(&clip_file_path);
    let clip_name = clip_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("clip");

    let thumbnail_name = format!("{}_preview.jpg", clip_name);
    let thumbnail_path = clip_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join(thumbnail_name);

    // Validate thumbnail path
    let _validated_output =
        security::validate_thumbnail_path(&thumbnail_path.to_string_lossy()).map_err(|e| AppError::Validation(e.to_string()))?;

    // Use thumbnail helper function to generate at midpoint
    let result_path = crate::video::thumbnail::auto_generate_thumbnail(
        validated_input,
        thumbnail_path.parent().unwrap_or_else(|| std::path::Path::new("."))
    )
    .await
    .map_err(|e| AppError::Video(e.to_string()))?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Get video duration in seconds
#[tauri::command]
pub async fn get_video_duration(
    state: State<'_, AppState>,
    input_path: String,
) -> AppResult<f64> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_input =
        security::validate_video_input_path(&input_path).map_err(|e| AppError::Validation(e.to_string()))?;

    let processor = VideoProcessor::new_with_fallback();

    let duration = processor
        .get_duration(validated_input)
        .await
        .map_err(|e| AppError::Video(e.to_string()))?;

    Ok(duration)
}

/// Delete a clip from storage
#[tauri::command]
pub async fn delete_clip(
    state: State<'_, AppState>,
    clip_file_path: String,
    game_id: String,
) -> AppResult<()> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_path =
        security::validate_video_input_path(&clip_file_path).map_err(|e| AppError::Validation(e.to_string()))?;
    let validated_game_id = security::validate_game_id(&game_id).map_err(|e| AppError::Validation(e.to_string()))?;

    // Delete the video file
    if validated_path.exists() {
        std::fs::remove_file(&validated_path).map_err(|e| AppError::Io(e.to_string()))?;
        tracing::info!("Deleted clip file: {:?}", validated_path);
    }

    // Delete from JSON storage
    state
        .storage
        .delete_clip_metadata(&validated_game_id, &clip_file_path)
        .map_err(|e| AppError::Database(format!("Failed to delete clip metadata: {}", e)))?;

    tracing::info!(
        "Successfully deleted clip and metadata: {:?}",
        validated_path
    );
    Ok(())
}

/// Create a long-form montage video (16:9) from selected clips
#[tauri::command]
pub async fn create_longform_video(
    state: State<'_, AppState>,
    clip_paths: Vec<String>,
    output_path: String,
) -> AppResult<String> {
    // Require PRO tier? Maybe allow for free users with watermark later.
    // For now, basic merge is a core utility. Let's allow it for authenticated users.
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    let validated_clips: Result<Vec<PathBuf>, AppError> = clip_paths
        .iter()
        .map(|p| security::validate_video_input_path(p).map_err(|e| AppError::Validation(e.to_string())))
        .collect();
    let validated_clips = validated_clips?;

    let validated_output =
        security::validate_video_output_path(&output_path).map_err(|e| AppError::Validation(e.to_string()))?;

    // Use the optimized VideoProcessor
    // Note: compose_montage is newly added to Processor
    let processor = VideoProcessor::new_with_fallback();
    
    let result_path = processor
        .compose_montage(&validated_clips, validated_output, false)
        .await
        .map_err(|e| AppError::Video(e.to_string()))?;

    Ok(result_path.to_string_lossy().to_string())
}

/// Start auto-edit composition for YouTube Shorts
#[tauri::command]
pub async fn start_auto_edit(
    state: State<'_, AppState>,
    config: AutoEditConfig,
) -> AppResult<AutoEditResult> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    // Check tier and quota
    let tier = state.auth.get_tier().map_err(|e| AppError::Auth(e.to_string()))?;
    let is_pro = matches!(tier, SubscriptionTier::Pro);

    // Check quota before starting
    let remaining = state
        .storage
        .check_auto_edit_quota(is_pro)
        .map_err(|e| AppError::Validation(format!("Quota check failed: {}", e)))?;

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
        .compose(config, job_id.clone(), is_pro)
        .await
        .map_err(|e| {
            tracing::error!("Auto-edit failed for job {}: {}", job_id, e);
            AppError::Video(format!("Auto-edit failed: {}", e))
        })?;

    // Increment usage counter on success
    if !is_pro {
        state
            .storage
            .increment_auto_edit_usage()
            .map_err(|e| {
                tracing::error!("Failed to increment usage: {}", e);
                // Just log warning, don't fail operation
            })
            .ok();
    }

    tracing::info!("Auto-edit completed successfully: {:?}", result.output_path);
    Ok(result)
}

/// Get progress of an auto-edit job
#[tauri::command]
pub async fn get_auto_edit_progress(
    state: State<'_, AppState>,
) -> AppResult<Option<AutoEditProgress>> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

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
) -> AppResult<()> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    state
        .storage
        .save_canvas_template(&template)
        .map_err(|e| AppError::Database(format!("Failed to save canvas template: {}", e)))?;

    Ok(())
}

/// Load a canvas template by ID
#[tauri::command]
pub async fn load_canvas_template(
    state: State<'_, AppState>,
    template_id: String,
) -> AppResult<crate::video::CanvasTemplate> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_template_id =
        security::validate_template_id(&template_id).map_err(|e| AppError::Validation(e.to_string()))?;

    let template = state
        .storage
        .load_canvas_template(&validated_template_id)
        .map_err(|e| AppError::Database(format!("Failed to load canvas template: {}", e)))?;

    Ok(template)
}

/// List all available canvas templates
#[tauri::command]
pub async fn list_canvas_templates(
    state: State<'_, AppState>,
) -> AppResult<Vec<CanvasTemplateInfo>> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    let templates = state
        .storage
        .list_canvas_templates()
        .map_err(|e| AppError::Database(format!("Failed to list canvas templates: {}", e)))?;

    Ok(templates)
}

/// Delete a canvas template
#[tauri::command]
pub async fn delete_canvas_template(
    state: State<'_, AppState>,
    template_id: String,
) -> AppResult<()> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    // Security validation
    let validated_template_id =
        security::validate_template_id(&template_id).map_err(|e| AppError::Validation(e.to_string()))?;

    state
        .storage
        .delete_canvas_template(&validated_template_id)
        .map_err(|e| AppError::Database(format!("Failed to delete canvas template: {}", e)))?;

    Ok(())
}

// ============================================================================
// Statistics Commands
// ============================================================================

/// Get simple clip statistics
#[tauri::command]
pub async fn get_clip_statistics(
    state: State<'_, AppState>,
) -> AppResult<(u64, u64, u64, f64)> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    let stats = crate::video::get_global_stats();
    let (total, successful, failed) = stats.get_counts();
    let success_rate = stats.success_rate();

    Ok((total, successful, failed, success_rate))
}

/// Reset all statistics
#[tauri::command]
pub async fn reset_clip_statistics(
    state: State<'_, AppState>,
) -> AppResult<()> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| AppError::Auth(e.to_string()))?;

    crate::video::get_global_stats().reset();
    Ok(())
}
