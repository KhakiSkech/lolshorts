use crate::auth::middleware::require_auth;
use crate::auth::SubscriptionTier;
use crate::storage::{AutoEditUsage, ClipMetadata, EventData, GameMetadata, StorageStats};
use crate::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// List all games (sorted by most recent)
#[tauri::command]
pub async fn list_games(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    // FREE tier feature - no authentication required
    state.storage.list_games().map_err(|e| e.to_string())
}

/// Get metadata for a specific game
#[tauri::command]
pub async fn get_game_metadata(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<GameMetadata, String> {
    // FREE tier feature - no authentication required
    state
        .storage
        .load_game_metadata(&game_id)
        .map_err(|e| e.to_string())
}

/// Save game metadata
#[tauri::command]
pub async fn save_game_metadata(
    state: State<'_, AppState>,
    game_id: String,
    metadata: GameMetadata,
) -> Result<(), String> {
    // FREE tier feature - no authentication required
    state
        .storage
        .save_game_metadata(&game_id, &metadata)
        .map_err(|e| e.to_string())
}

/// Load events for a game
#[tauri::command]
pub async fn get_game_events(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<Vec<EventData>, String> {
    // FREE tier feature - no authentication required
    state
        .storage
        .load_events(&game_id)
        .map_err(|e| e.to_string())
}

/// Save events for a game
#[tauri::command]
pub async fn save_game_events(
    state: State<'_, AppState>,
    game_id: String,
    events: Vec<EventData>,
) -> Result<(), String> {
    // FREE tier feature - no authentication required
    state
        .storage
        .save_events(&game_id, &events)
        .map_err(|e| e.to_string())
}

/// Save clip metadata
#[tauri::command]
pub async fn save_clip_metadata(
    state: State<'_, AppState>,
    game_id: String,
    clip: ClipMetadata,
) -> Result<(), String> {
    // FREE tier feature - no authentication required
    state
        .storage
        .save_clip_metadata(&game_id, &clip)
        .map_err(|e| e.to_string())
}

/// Delete a game and all its data
#[tauri::command]
pub async fn delete_game(state: State<'_, AppState>, game_id: String) -> Result<(), String> {
    // FREE tier feature - no authentication required
    state
        .storage
        .delete_game(&game_id)
        .map_err(|e| e.to_string())
}

/// Get storage statistics
#[tauri::command]
pub async fn get_storage_stats(state: State<'_, AppState>) -> Result<StorageStats, String> {
    // FREE tier feature - no authentication required
    state.storage.get_stats().map_err(|e| e.to_string())
}

/// List all clips for a specific game
#[tauri::command]
pub async fn list_clips(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<Vec<ClipMetadata>, String> {
    // FREE tier feature - no authentication required
    state
        .storage
        .load_clip_metadata(&game_id)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Auto-Edit Quota Commands
// ============================================================================

/// Get auto-edit usage and quota information
///
/// Returns current month's usage and remaining quota based on user tier.
#[tauri::command]
pub async fn get_auto_edit_quota(state: State<'_, AppState>) -> Result<AutoEditQuotaInfo, String> {
    // Require authentication to check tier
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    let tier = state.auth.get_tier().map_err(|e| e.to_string())?;
    let is_pro = matches!(tier, SubscriptionTier::Pro);

    // Load current usage
    let usage = state
        .storage
        .load_auto_edit_usage()
        .map_err(|e| e.to_string())?;

    // Calculate remaining quota
    let limit = if is_pro { u32::MAX } else { 5 };
    let remaining = if is_pro {
        u32::MAX
    } else {
        limit.saturating_sub(usage.usage_count)
    };

    Ok(AutoEditQuotaInfo {
        tier: format!("{:?}", tier),
        is_pro,
        usage: usage.usage_count,
        limit,
        remaining,
        month: usage.month,
    })
}

/// Auto-edit quota information for frontend display
#[derive(Debug, Serialize, Deserialize)]
pub struct AutoEditQuotaInfo {
    /// User's subscription tier (FREE or PRO)
    pub tier: String,

    /// Whether user is PRO tier
    pub is_pro: bool,

    /// Number of auto-edits used this month
    pub usage: u32,

    /// Monthly limit (5 for FREE, u32::MAX for PRO)
    pub limit: u32,

    /// Remaining auto-edits this month
    pub remaining: u32,

    /// Current month (YYYY-MM)
    pub month: String,
}

// ============================================================================
// Auto-Edit Results Commands
// ============================================================================

/// Get all auto-edit results
#[tauri::command]
pub async fn get_auto_edit_results(
    state: State<'_, AppState>,
) -> Result<Vec<crate::storage::AutoEditResultMetadata>, String> {
    state
        .storage
        .load_auto_edit_results()
        .map_err(|e| e.to_string())
}

/// Get a specific auto-edit result by ID
#[tauri::command]
pub async fn get_auto_edit_result(
    state: State<'_, AppState>,
    result_id: String,
) -> Result<crate::storage::AutoEditResultMetadata, String> {
    state
        .storage
        .load_auto_edit_result(&result_id)
        .map_err(|e| e.to_string())
}

/// Delete an auto-edit result
///
/// If delete_file is true, also deletes the video file and thumbnail.
#[tauri::command]
pub async fn delete_auto_edit_result(
    state: State<'_, AppState>,
    result_id: String,
    delete_file: bool,
) -> Result<(), String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state
        .storage
        .delete_auto_edit_result(&result_id, delete_file)
        .map_err(|e| e.to_string())
}

/// Update YouTube upload status for an auto-edit result
#[tauri::command]
pub async fn update_auto_edit_youtube_status(
    state: State<'_, AppState>,
    result_id: String,
    status: crate::storage::YouTubeUploadStatus,
) -> Result<(), String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state
        .storage
        .update_auto_edit_youtube_status(&result_id, status)
        .map_err(|e| e.to_string())
}

/// Get dashboard statistics (total games, clips, storage used)
#[tauri::command]
pub async fn get_dashboard_stats(state: State<'_, AppState>) -> Result<StorageStats, String> {
    // FREE tier feature - no authentication required
    state.storage.get_stats().map_err(|e| e.to_string())
}
