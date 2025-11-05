use crate::auth::middleware::require_auth;
use crate::storage::{GameMetadata, ClipMetadata, EventData, StorageStats};
use crate::AppState;
use tauri::State;

/// List all games (sorted by most recent)
#[tauri::command]
pub async fn list_games(
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
        .list_games()
        .map_err(|e| e.to_string())
}

/// Get metadata for a specific game
#[tauri::command]
pub async fn get_game_metadata(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<GameMetadata, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
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
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
        .save_game_metadata(&game_id, &metadata)
        .map_err(|e| e.to_string())
}

/// Load events for a game
#[tauri::command]
pub async fn get_game_events(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<Vec<EventData>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
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
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
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
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
        .save_clip_metadata(&game_id, &clip)
        .map_err(|e| e.to_string())
}

/// Delete a game and all its data
#[tauri::command]
pub async fn delete_game(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<(), String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
        .delete_game(&game_id)
        .map_err(|e| e.to_string())
}

/// Get storage statistics
#[tauri::command]
pub async fn get_storage_stats(
    state: State<'_, AppState>,
) -> Result<StorageStats, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
        .get_stats()
        .map_err(|e| e.to_string())
}

/// List all clips for a specific game
#[tauri::command]
pub async fn list_clips(
    state: State<'_, AppState>,
    game_id: String,
) -> Result<Vec<ClipMetadata>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    state.storage
        .load_clip_metadata(&game_id)
        .map_err(|e| e.to_string())
}
