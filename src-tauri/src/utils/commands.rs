/// Tauri commands for production utilities
///
/// Exposes metrics, health status, and system info to frontend

use crate::AppState;
use crate::utils::metrics::{RecordingMetrics, SystemMetrics, HealthStatus};
use tauri::State;

/// Get current recording performance metrics
#[tauri::command]
pub async fn get_recording_metrics(
    state: State<'_, AppState>,
) -> Result<RecordingMetrics, String> {
    Ok(state.metrics_collector.get_recording_metrics().await)
}

/// Get current system resource metrics
#[tauri::command]
pub async fn get_system_metrics(
    state: State<'_, AppState>,
) -> Result<SystemMetrics, String> {
    Ok(state.metrics_collector.get_system_metrics().await)
}

/// Get current system health status
#[tauri::command]
pub async fn get_health_status(
    state: State<'_, AppState>,
) -> Result<HealthStatus, String> {
    Ok(state.metrics_collector.check_health().await)
}

/// Get application version info
#[tauri::command]
pub fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Force cleanup of temporary files
#[tauri::command]
pub async fn force_cleanup(
    state: State<'_, AppState>,
) -> Result<u64, String> {
    state.cleanup_manager
        .cleanup_on_startup()
        .await
        .map(|_| 0) // Return 0 as it's async, actual cleanup happens in background
        .map_err(|e| e.to_string())
}

/// Get disk space info for recordings directory
#[tauri::command]
pub async fn get_disk_space_info(
    state: State<'_, AppState>,
) -> Result<DiskSpaceInfo, String> {
    let available_gb = state.cleanup_manager
        .check_disk_space()
        .map_err(|e| e.to_string())?;

    Ok(DiskSpaceInfo {
        available_gb,
        total_gb: 500.0, // TODO: Get actual disk size
        used_gb: 500.0 - available_gb,
    })
}

#[derive(serde::Serialize)]
pub struct DiskSpaceInfo {
    pub available_gb: f64,
    pub total_gb: f64,
    pub used_gb: f64,
}
