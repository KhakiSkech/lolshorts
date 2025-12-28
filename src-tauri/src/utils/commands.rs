use crate::utils::metrics::{HealthStatus, RecordingMetrics, SystemMetrics};
/// Tauri commands for production utilities
///
/// Exposes metrics, health status, and system info to frontend
use crate::AppState;
use tauri::State;
use sysinfo::Disks;

/// Get current recording performance metrics
#[tauri::command]
pub async fn get_recording_metrics(state: State<'_, AppState>) -> Result<RecordingMetrics, String> {
    Ok(state.metrics_collector.get_recording_metrics().await)
}

/// Get current system resource metrics
#[tauri::command]
pub async fn get_system_metrics(state: State<'_, AppState>) -> Result<SystemMetrics, String> {
    Ok(state.metrics_collector.get_system_metrics().await)
}

/// Get current system health status
#[tauri::command]
pub async fn get_health_status(state: State<'_, AppState>) -> Result<HealthStatus, String> {
    Ok(state.metrics_collector.check_health().await)
}

/// Get application version info
#[tauri::command]
pub fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Force cleanup of temporary files
#[tauri::command]
pub async fn force_cleanup(state: State<'_, AppState>) -> Result<u64, String> {
    state
        .cleanup_manager
        .cleanup_on_startup()
        .await
        .map(|_| 0) // Return 0 as it's async, actual cleanup happens in background
        .map_err(|e| e.to_string())
}

/// Get disk space info for recordings directory using sysinfo for real data
#[tauri::command]
pub async fn get_disk_space_info(state: State<'_, AppState>) -> Result<DiskSpaceInfo, String> {
    // Get disk usage from sysinfo
    let disks = Disks::new_with_refreshed_list();
    let recordings_path = state.storage.base_path().join("recordings");
    
    // Find the disk where recordings are stored
    let mut total_space = 0;
    let mut available_space = 0;
    
    // Default fallback (if no disk found)
    let mut found_disk = false;

    for disk in &disks {
        if recordings_path.starts_with(disk.mount_point()) {
            total_space = disk.total_space();
            available_space = disk.available_space();
            found_disk = true;
            break;
        }
    }

    // If not found by mount point, try the first disk as fallback or use system root
    if !found_disk && !disks.is_empty() {
        // Try C: on Windows or / on Linux/Mac
        #[cfg(target_os = "windows")]
        let root = std::path::Path::new("C:\\");
        #[cfg(not(target_os = "windows"))]
        let root = std::path::Path::new("/");

        for disk in &disks {
            if disk.mount_point() == root {
                total_space = disk.total_space();
                available_space = disk.available_space();
                found_disk = true;
                break;
            }
        }
        
        // If still not found, just use the first one
        if !found_disk {
             if let Some(disk) = disks.first() {
                total_space = disk.total_space();
                available_space = disk.available_space();
             }
        }
    }

    // Convert bytes to GB
    let total_gb = total_space as f64 / (1024.0 * 1024.0 * 1024.0);
    let available_gb = available_space as f64 / (1024.0 * 1024.0 * 1024.0);
    let used_gb = total_gb - available_gb;

    Ok(DiskSpaceInfo {
        available_gb,
        total_gb,
        used_gb,
    })
}

#[derive(serde::Serialize)]
pub struct DiskSpaceInfo {
    pub available_gb: f64,
    pub total_gb: f64,
    pub used_gb: f64,
}

/// Show file/folder in system file explorer
#[tauri::command]
pub async fn show_in_folder(file_path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &file_path])
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &file_path])
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| format!("Failed to open file manager: {}", e))?;
        }
    }

    Ok(())
}

/// Open file with default system application
#[tauri::command]
pub async fn open_file_with_default_app(file_path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &file_path])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }

    Ok(())
}

/// Check if file exists at given path
#[tauri::command]
pub async fn check_file_exists(file_path: String) -> Result<bool, String> {
    Ok(std::path::Path::new(&file_path).exists())
}

/// Get comprehensive FFmpeg information
#[tauri::command]
pub async fn get_ffmpeg_info() -> Result<crate::utils::ffmpeg::FFmpegInfo, String> {
    crate::utils::ffmpeg::get_ffmpeg_info()
        .map_err(|e| e.to_string())
}

/// Get hardware-accelerated encoders
#[tauri::command]
pub async fn get_hardware_encoders() -> Result<Vec<crate::utils::ffmpeg::EncoderInfo>, String> {
    crate::utils::ffmpeg::get_hardware_encoders()
        .map_err(|e| e.to_string())
}

/// Get available video encoders
#[tauri::command]
pub async fn get_video_encoders() -> Result<Vec<crate::utils::ffmpeg::EncoderInfo>, String> {
    crate::utils::ffmpeg::get_video_encoders()
        .map_err(|e| e.to_string())
}
