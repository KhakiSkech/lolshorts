use super::{GameInfo, LcuClient};
use crate::auth::middleware::require_auth;
use crate::AppState;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

// Note: connect_lcu and check_lcu_status do NOT require authentication
// They are system checks to see if League client is running
// Only game data access commands (get_current_game, is_in_game) require auth

// Global LCU client (initialized lazily)
static LCU_CLIENT: Lazy<Arc<Mutex<LcuClient>>> = Lazy::new(|| {
    Arc::new(Mutex::new(LcuClient::new()))
});

#[tauri::command]
pub async fn connect_lcu() -> Result<bool, String> {
    // No authentication required - this is a system check
    let mut client = LCU_CLIENT.lock().await;

    match client.connect().await {
        Ok(()) => Ok(true),
        Err(e) => {
            tracing::debug!("LCU connection attempt failed (League client may not be running): {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn check_lcu_status() -> Result<bool, String> {
    // No authentication required - this is a system check
    let client = LCU_CLIENT.lock().await;
    Ok(client.is_connected())
}

#[tauri::command]
pub async fn get_current_game(state: State<'_, AppState>) -> Result<Option<GameInfo>, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;
    let client = LCU_CLIENT.lock().await;

    if !client.is_connected() {
        return Err("LCU not connected. Call connect_lcu first.".to_string());
    }

    client
        .get_current_game()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn is_in_game(state: State<'_, AppState>) -> Result<bool, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    let client = LCU_CLIENT.lock().await;

    if !client.is_connected() {
        return Ok(false);
    }

    client
        .is_in_game()
        .await
        .map_err(|e| e.to_string())
}
