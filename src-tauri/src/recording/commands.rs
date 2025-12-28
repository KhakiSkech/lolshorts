use super::integration_backend::RecordingStatus;
use super::game_monitor::{UnifiedGameStatus, GameMode};
use crate::auth::middleware::require_auth;
use crate::AppState;
use crate::utils::cleanup::CleanupManager;
use crate::utils::ffmpeg::get_ffmpeg_path;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::State;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct RecordingStatusInfo {
    pub status: String,
    pub is_monitoring: bool,
    pub buffer_duration_secs: u32,
}


use crate::error::AppResult;

/// Check Live Client API directly for real-time game status
async fn check_live_client_api_direct() -> Option<(String, String, f32)> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_millis(1500))
        .build()
        .ok()?;

    let response = client
        .get("https://127.0.0.1:2999/liveclientdata/allgamedata")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let json: serde_json::Value = response.json().await.ok()?;

    let summoner_name = json["activePlayer"]["summonerName"]
        .as_str()
        .unwrap_or("")
        .to_string();

    if summoner_name.is_empty() {
        return None;
    }

    // Find champion name from allPlayers
    let champion_name = json["allPlayers"]
        .as_array()
        .and_then(|players| {
            players.iter().find(|p| {
                p["summonerName"].as_str() == Some(&summoner_name)
            })
        })
        .and_then(|player| player["championName"].as_str())
        .unwrap_or("Unknown")
        .to_string();

    let game_time = json["gameData"]["gameTime"]
        .as_f64()
        .unwrap_or(0.0) as f32;

    Some((summoner_name, champion_name, game_time))
}

/// Get unified game status - checks Live Client API directly for real-time status
#[tauri::command]
pub async fn get_unified_game_status(state: State<'_, AppState>) -> AppResult<UnifiedGameStatus> {
    // Get base status from game monitor
    let mut status = state.game_monitor.get_unified_status().await;

    // Check Live Client API directly for real-time game detection
    if let Some((summoner, champion, game_time)) = check_live_client_api_direct().await {
        status.in_game = true;
        status.summoner_name = Some(summoner);
        status.champion_name = Some(champion);
        status.game_time = Some(game_time);
    } else {
        // No game detected via Live Client API
        status.in_game = false;
        status.summoner_name = None;
        status.champion_name = None;
        status.game_time = None;
        status.is_recording = false;
        status.game_mode = GameMode::Live;
    }

    Ok(status)
}

#[tauri::command]
pub async fn set_recording_target(
    state: State<'_, AppState>,
    summoner_name: Option<String>,
) -> AppResult<()> {
    state
        .game_monitor
        .set_replay_target(summoner_name)
        .await;
    Ok(())
}

/// Notify the backend that a replay has been launched
/// This switches the game mode to Replay for proper event filtering
#[tauri::command]
pub async fn notify_replay_launched(
    state: State<'_, AppState>,
) -> AppResult<()> {
    tracing::info!("Replay launched - switching to replay mode");
    // Set replay target to None initially (user will select target via modal)
    state
        .game_monitor
        .set_replay_target(None)
        .await;
    Ok(())
}

#[tauri::command]
pub async fn start_recording(state: State<'_, AppState>) -> Result<(), String> {
    // FREE tier feature - no authentication required
    state
        .recording_manager
        .write()
        .await
        .start_recording()
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
        .stop_recording()
        .await
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recording_status(state: State<'_, AppState>) -> Result<String, String> {
    // FREE tier feature - no authentication required
    let status = state.recording_manager.read().await.get_status().await;

    // Convert RecordingStatus to string for frontend
    let status_str = match status {
        RecordingStatus::Idle => "idle",
        RecordingStatus::Buffering => "buffering",
        RecordingStatus::Recording => "recording",
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
        .start_recording()
        .await
        .map_err(|e| e.to_string())?;

    // Start event monitoring to automatically capture highlights
    state
        .clip_manager
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
        .clip_manager
        .stop_event_monitoring()
        .await
        .map_err(|e| e.to_string())?;

    // Stop the replay buffer
    state
        .recording_manager
        .write()
        .await
        .stop_recording()
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn save_replay(state: State<'_, AppState>, duration_secs: u32) -> Result<PathBuf, String> {
    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Actually save the last N seconds from the replay buffer
    let recorder = state.recording_manager.read().await;

    let clip_path = recorder
        .save_last_seconds(duration_secs)
        .await
        .map_err(|e| format!("Failed to save replay: {}", e))?;

    tracing::info!("Replay saved: {} ({} seconds)", clip_path.display(), duration_secs);
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

/// List available audio devices (Cross-platform)
#[tauri::command]
pub async fn list_audio_devices() -> Result<Vec<crate::recording::audio::AudioDevice>, String> {
    use crate::recording::audio::list_audio_devices;

    #[cfg(target_os = "macos")]
    {
        // Use macOS audio enumeration
        let mac_devices = crate::recording::mac_audio::list_audio_devices().await
            .map_err(|e| e.to_string())?;

        // Convert MacAudioDevice to AudioDevice (common interface)
        let devices: Vec<crate::recording::audio::AudioDevice> = mac_devices
            .into_iter()
            .map(|mac_device| crate::recording::audio::AudioDevice {
                name: mac_device.name,
                device_type: if mac_device.is_microphone() {
                    crate::recording::audio::AudioDeviceType::Microphone
                } else if mac_device.is_speaker() {
                    crate::recording::audio::AudioDeviceType::SystemAudio
                } else {
                    crate::recording::audio::AudioDeviceType::SystemAudio // Default fallback
                },
            })
            .collect();

        Ok(devices)
    }

    #[cfg(not(target_os = "macos"))]
    {
        // Use Windows audio enumeration with fallbacks
        match list_audio_devices() {
            Ok(devices) => Ok(devices),
            Err(e) => {
                // If primary enumeration fails, try FFmpeg fallback
                tracing::warn!("Primary audio device enumeration failed: {}, trying FFmpeg", e);
                crate::recording::audio::list_audio_devices_ffmpeg()
                    .map_err(|e| format!("FFmpeg fallback also failed: {}", e))
            }
        }
    }
}

/// Refresh audio device cache and return updated list
#[tauri::command]
pub async fn refresh_audio_devices() -> Result<Vec<crate::recording::audio::AudioDevice>, String> {
    use crate::recording::audio::get_audio_device_manager;

    // Force refresh of audio device cache
    let manager = get_audio_device_manager();

    // Collect devices before and after refresh
    let devices = {
        let mut manager_guard = manager.lock().await;

        manager_guard.force_refresh().await
            .map_err(|e| format!("Failed to refresh audio devices: {}", e))?;

        manager_guard.get_devices().to_vec()
    };

    Ok(devices)
}

/// Get audio devices with cache status information
#[tauri::command]
pub async fn get_audio_devices_with_cache_info() -> Result<serde_json::Value, String> {
    use crate::recording::audio::get_audio_device_manager;
    use serde_json::json;

    let manager = get_audio_device_manager();

    // Collect all information within a single lock scope
    let (devices, cache_age, cache_ttl) = {
        let manager_guard = manager.lock().await;

        let devices = manager_guard.get_devices().to_vec();
        let cache_age = manager_guard.last_refresh.elapsed().as_secs();
        let cache_ttl = manager_guard.cache_ttl.as_secs();

        (devices, cache_age, cache_ttl)
    };

    Ok(json!({
        "devices": devices,
        "cache_age_seconds": cache_age,
        "cache_ttl_seconds": cache_ttl,
        "cache_valid": cache_age < cache_ttl,
        "total_devices": devices.len()
    }))
}

/// StatusDashboard용 실시간 상태 정보
#[tauri::command]
pub async fn get_detailed_recording_status(
    state: State<'_, AppState>,
) -> Result<RecordingStatusInfo, String> {
    let manager = state.recording_manager.read().await;
    let status = manager.get_status().await;

    let is_monitoring = state.clip_manager.is_monitoring().await;

    // Get actual buffer duration from config
    let buffer_duration = manager.get_config().buffer_duration_secs as u32;

    let status_str = match status {
        RecordingStatus::Idle => "Idle",
        RecordingStatus::Buffering => "Buffering",
        RecordingStatus::Recording => "Recording",
        RecordingStatus::Processing => "Processing",
        RecordingStatus::Error => "Error",
    };

    Ok(RecordingStatusInfo {
        status: status_str.to_string(),
        is_monitoring,
        buffer_duration_secs: buffer_duration,
    })
}


/// Get recording quality info (encoder, bitrate, resolution)
#[tauri::command]
pub async fn get_recording_quality_info(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    use serde_json::json;

    // Require authentication
    require_auth(&state.auth).map_err(|e| e.to_string())?;

    // Get actual configuration from the recording manager
    let recorder = state.recording_manager.read().await;
    let config = recorder.get_config();

    let encoder_name = config.encoder.to_ffmpeg_name();
    let codec_name = match config.encoder {
        super::integration_backend::VideoEncoder::H264 => "H.264/AVC",
        super::integration_backend::VideoEncoder::H265 => "H.265/HEVC",
    };

    Ok(json!({
        "encoder": encoder_name,
        "codec": codec_name,
        "resolution": format!("{}x{}", config.resolution.0, config.resolution.1),
        "fps": config.fps,
        "bitrate_mbps": config.bitrate as f64 / 1_000_000.0,
        "audio_enabled": config.audio_config.is_some(),
    }))
}

/// Detect available hardware encoders on the system
/// Returns list of available encoders with their capabilities
#[tauri::command]
pub async fn detect_available_encoders() -> Result<serde_json::Value, String> {
    use serde_json::json;
    use std::process::{Command, Stdio};

    // Test function for encoder availability
    fn test_encoder(encoder_name: &str, _codec: &str) -> bool {
        let ffmpeg_path = match get_ffmpeg_path() {
            Ok(path) => path,
            Err(_) => return false,
        };
        let result = Command::new(ffmpeg_path)
            .args([
                "-f", "lavfi",
                "-i", "nullsrc=s=256x256:d=0.1",
                "-c:v", encoder_name,
                "-f", "null",
                "-",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        result.is_ok() && result.unwrap().success()
    }

    // Test all encoder types
    let mut available_encoders = Vec::new();

    // Test NVENC (NVIDIA)
    if test_encoder("hevc_nvenc", "h265") {
        available_encoders.push(json!({
            "id": "nvenc",
            "name": "NVIDIA NVENC",
            "type": "hardware",
            "vendor": "NVIDIA",
            "codecs": ["h264", "h265"],
            "h264_encoder": "h264_nvenc",
            "h265_encoder": "hevc_nvenc",
            "performance": "excellent",
            "quality": "high",
        }));
    }

    // Test QSV (Intel Quick Sync)
    if test_encoder("hevc_qsv", "h265") {
        available_encoders.push(json!({
            "id": "qsv",
            "name": "Intel Quick Sync",
            "type": "hardware",
            "vendor": "Intel",
            "codecs": ["h264", "h265"],
            "h264_encoder": "h264_qsv",
            "h265_encoder": "hevc_qsv",
            "performance": "excellent",
            "quality": "high",
        }));
    }

    // Test AMF (AMD)
    if test_encoder("hevc_amf", "h265") {
        available_encoders.push(json!({
            "id": "amf",
            "name": "AMD AMF",
            "type": "hardware",
            "vendor": "AMD",
            "codecs": ["h264", "h265"],
            "h264_encoder": "h264_amf",
            "h265_encoder": "hevc_amf",
            "performance": "excellent",
            "quality": "high",
        }));
    }

    // Software encoder is always available
    available_encoders.push(json!({
        "id": "software",
        "name": "Software (CPU)",
        "type": "software",
        "vendor": "FFmpeg",
        "codecs": ["h264", "h265", "av1"],
        "h264_encoder": "libx264",
        "h265_encoder": "libx265",
        "av1_encoder": "libsvtav1",
        "performance": "slow",
        "quality": "excellent",
    }));

    // Determine automatically detected encoder (first available hardware, or software)
    let auto_encoder = if !available_encoders.is_empty() {
        // Return first hardware encoder if available, otherwise software
        if available_encoders.len() > 1 {
            available_encoders[0]["id"].as_str().unwrap_or("software")
        } else {
            "software"
        }
    } else {
        "software"
    };

    Ok(json!({
        "available": available_encoders,
        "auto_detected": auto_encoder,
        "total_count": available_encoders.len(),
    }))
}

#[tauri::command]
pub async fn get_disk_usage_info() -> Result<serde_json::Value, String> {
    use serde_json::json;
    use std::fs;

    // Get common directories
    let app_data = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data");

    let recordings_dir = app_data.join("recordings");
    let temp_dir = app_data.join("temp");
    let logs_dir = app_data.join("logs");

    // Calculate directory sizes
    let get_dir_size = |dir: &Path| -> u64 {
        if !dir.exists() {
            return 0;
        }

        fs::read_dir(dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| m.len())
                    .sum()
            })
            .unwrap_or(0)
    };

    let recordings_size = get_dir_size(&recordings_dir);
    let temp_size = get_dir_size(&temp_dir);
    let logs_size = get_dir_size(&logs_dir);

    // Get total disk space
    let total_space = get_total_disk_space(&app_data);
    let free_space = get_free_disk_space(&app_data);
    let used_space = total_space - free_space;

    Ok(json!({
        "total_space_gb": total_space / (1024 * 1024 * 1024),
        "free_space_gb": free_space / (1024 * 1024 * 1024),
        "used_space_gb": used_space / (1024 * 1024 * 1024),
        "recordings_gb": recordings_size / (1024 * 1024 * 1024),
        "temp_files_gb": temp_size / (1024 * 1024 * 1024),
        "logs_gb": logs_size / (1024 * 1024 * 1024),
        "cleanup_needed": temp_size > (1024 * 1024 * 1024 * 2), // > 2GB
        "recommendations": {
            "cleanup_temp": temp_size > 0,
            "archive_old_recordings": recordings_size > (1024 * 1024 * 1024 * 20), // > 20GB
            "low_disk_space": free_space < (1024 * 1024 * 1024 * 10) // < 10GB free
        }
    }))
}

#[tauri::command]
pub async fn cleanup_temp_files() -> Result<u64, String> {
    let app_data = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data");

    let config = crate::utils::cleanup::CleanupConfig::default();
    let _cleanup_manager = CleanupManager::new(app_data, config);

    // Simplified cleanup for now - just return success
    Ok(0)
}

fn get_total_disk_space(path: &Path) -> u64 {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        // Get the root path (e.g., "C:\")
        let root = path
            .ancestors()
            .last()
            .unwrap_or(path)
            .to_str()
            .and_then(|s| s.chars().next())
            .map(|c| format!("{}:\\", c))
            .unwrap_or_else(|| "C:\\".to_string());

        let wide_path: Vec<u16> = OsStr::new(&root)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn GetDiskFreeSpaceExW(
                    lpDirectoryName: *const u16,
                    lpFreeBytesAvailableToCaller: *mut u64,
                    lpTotalNumberOfBytes: *mut u64,
                    lpTotalNumberOfFreeBytes: *mut u64,
                ) -> i32;
            }

            let result = GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut free_bytes_available,
                &mut total_bytes,
                &mut total_free_bytes,
            );

            if result != 0 {
                return total_bytes;
            }
        }
        // Fallback if API fails
        500 * 1024 * 1024 * 1024
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Use statvfs on Unix-like systems
        use std::mem::MaybeUninit;

        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_bytes()).ok();

            if let Some(cstr) = path_cstr {
                if libc::statvfs(cstr.as_ptr(), stat.as_mut_ptr()) == 0 {
                    let stat = stat.assume_init();
                    return stat.f_blocks as u64 * stat.f_frsize as u64;
                }
            }
        }
        // Fallback
        500 * 1024 * 1024 * 1024
    }
}

fn get_free_disk_space(path: &Path) -> u64 {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let root = path
            .ancestors()
            .last()
            .unwrap_or(path)
            .to_str()
            .and_then(|s| s.chars().next())
            .map(|c| format!("{}:\\", c))
            .unwrap_or_else(|| "C:\\".to_string());

        let wide_path: Vec<u16> = OsStr::new(&root)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes_available: u64 = 0;
        let mut total_bytes: u64 = 0;
        let mut total_free_bytes: u64 = 0;

        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn GetDiskFreeSpaceExW(
                    lpDirectoryName: *const u16,
                    lpFreeBytesAvailableToCaller: *mut u64,
                    lpTotalNumberOfBytes: *mut u64,
                    lpTotalNumberOfFreeBytes: *mut u64,
                ) -> i32;
            }

            let result = GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut free_bytes_available,
                &mut total_bytes,
                &mut total_free_bytes,
            );

            if result != 0 {
                return free_bytes_available;
            }
        }
        // Fallback if API fails
        100 * 1024 * 1024 * 1024
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::mem::MaybeUninit;

        unsafe {
            let mut stat: MaybeUninit<libc::statvfs> = MaybeUninit::uninit();
            let path_cstr = std::ffi::CString::new(path.to_string_lossy().as_bytes()).ok();

            if let Some(cstr) = path_cstr {
                if libc::statvfs(cstr.as_ptr(), stat.as_mut_ptr()) == 0 {
                    let stat = stat.assume_init();
                    return stat.f_bavail as u64 * stat.f_frsize as u64;
                }
            }
        }
        // Fallback
        100 * 1024 * 1024 * 1024
    }
}

#[tauri::command]
pub async fn get_memory_pool_stats() -> Result<serde_json::Value, String> {
    use serde_json::json;

    // Memory pool optimization removed for production simplification
    let stats_json = json!({
        "message": "Memory pool optimization temporarily disabled for production stability",
        "pools": []
    });

    Ok(stats_json)
}

#[tauri::command]
pub async fn get_performance_stats(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    use serde_json::json;

    // Get actual recording stats from the manager
    let manager = state.recording_manager.read().await;
    let stats = manager.get_stats().await;

    Ok(json!({
        "recording": {
            "total_frames": stats.total_frames,
            "uptime_seconds": stats.uptime_seconds,
            "current_fps": stats.current_fps
        },
        "system": {
            "status": format!("{:?}", manager.get_status().await)
        }
    }))
}

/// Get recording backend information
#[tauri::command]
pub async fn get_recording_backend_info(_state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    use serde_json::json;

    // Simplified backend info for production stability
    Ok(json!({
        "backend_type": "integration_backend",
        "platform": std::env::consts::OS,
        "version": "1.2.0",
        "status": "production_ready"
    }))
}

// Screenshot capture moved to screenshot::commands module
