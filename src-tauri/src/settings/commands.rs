use super::models::RecordingSettings;
use crate::AppState;
use tauri::State;

/// Get current recording settings
#[tauri::command]
pub async fn get_recording_settings(state: State<'_, AppState>) -> Result<RecordingSettings, String> {
    // Read from shared in-memory settings
    let settings = state.recording_settings.read().await;
    Ok(settings.clone())
}

/// Save recording settings
#[tauri::command]
pub async fn save_recording_settings(
    state: State<'_, AppState>,
    settings: RecordingSettings,
) -> Result<(), String> {
    // Save to disk first
    settings.save().map_err(|e| e.to_string())?;

    // Update recording manager audio config
    // Note: Changes take effect on next segment recording
    state.recording_manager.write().await
        .update_audio_config(&settings.audio);

    // Update shared in-memory settings
    let mut current_settings = state.recording_settings.write().await;
    *current_settings = settings;

    Ok(())
}

/// Reset settings to default values
#[tauri::command]
pub async fn reset_settings_to_default(state: State<'_, AppState>) -> Result<RecordingSettings, String> {
    // Reset to defaults and save
    let defaults = RecordingSettings::reset_to_default().map_err(|e| e.to_string())?;

    // Update recording manager audio config with defaults
    state.recording_manager.write().await
        .update_audio_config(&defaults.audio);

    // Update shared in-memory settings
    let mut current_settings = state.recording_settings.write().await;
    *current_settings = defaults.clone();

    Ok(defaults)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_settings_command() {
        let result = get_recording_settings().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_save_settings_command() {
        let settings = RecordingSettings::default();
        let result = save_recording_settings(settings).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_reset_command() {
        let result = reset_settings_to_default().await;
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.event_filter.min_priority, 2);
    }
}
