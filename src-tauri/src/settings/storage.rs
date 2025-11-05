use super::models::RecordingSettings;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Failed to get config directory")]
    ConfigDirNotFound,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SettingsError>;

impl RecordingSettings {
    /// Load settings from file
    ///
    /// If the settings file doesn't exist, returns default settings.
    /// Location: %APPDATA%/Roaming/LoLShorts/settings.json (Windows)
    pub fn load() -> Result<Self> {
        let settings_path = Self::get_settings_path()?;

        if settings_path.exists() {
            let json = fs::read_to_string(&settings_path)?;
            let settings = serde_json::from_str(&json)?;
            tracing::info!("Loaded settings from: {:?}", settings_path);
            Ok(settings)
        } else {
            tracing::info!("Settings file not found, using defaults");
            Ok(Self::default())
        }
    }

    /// Save settings to file
    ///
    /// Creates the config directory if it doesn't exist.
    pub fn save(&self) -> Result<()> {
        let settings_path = Self::get_settings_path()?;

        // Ensure parent directory exists
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&settings_path, json)?;

        tracing::info!("Saved settings to: {:?}", settings_path);
        Ok(())
    }

    /// Get the path to the settings file
    ///
    /// Platform-specific:
    /// - Windows: %APPDATA%/Roaming/LoLShorts/settings.json
    /// - macOS: ~/Library/Application Support/LoLShorts/settings.json
    /// - Linux: ~/.config/LoLShorts/settings.json
    fn get_settings_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or(SettingsError::ConfigDirNotFound)?;

        let lolshorts_dir = config_dir.join("LoLShorts");
        Ok(lolshorts_dir.join("settings.json"))
    }

    /// Reset settings to default and save
    pub fn reset_to_default() -> Result<Self> {
        let settings = Self::default();
        settings.save()?;
        tracing::info!("Settings reset to default");
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_settings_path() {
        let path = RecordingSettings::get_settings_path().unwrap();
        assert!(path.to_string_lossy().contains("LoLShorts"));
        assert!(path.to_string_lossy().ends_with("settings.json"));
    }

    #[test]
    fn test_save_and_load() {
        let mut settings = RecordingSettings::default();
        settings.event_filter.min_priority = 4;
        settings.audio.microphone_volume = 150;

        // Save
        settings.save().unwrap();

        // Load
        let loaded = RecordingSettings::load().unwrap();
        assert_eq!(loaded.event_filter.min_priority, 4);
        assert_eq!(loaded.audio.microphone_volume, 150);

        // Cleanup
        let path = RecordingSettings::get_settings_path().unwrap();
        if path.exists() {
            fs::remove_file(path).ok();
        }
    }

    #[test]
    fn test_reset_to_default() {
        // Create modified settings
        let mut settings = RecordingSettings::default();
        settings.event_filter.min_priority = 5;
        settings.save().unwrap();

        // Reset
        let reset_settings = RecordingSettings::reset_to_default().unwrap();
        assert_eq!(reset_settings.event_filter.min_priority, 2); // default value

        // Verify persisted
        let loaded = RecordingSettings::load().unwrap();
        assert_eq!(loaded.event_filter.min_priority, 2);

        // Cleanup
        let path = RecordingSettings::get_settings_path().unwrap();
        if path.exists() {
            fs::remove_file(path).ok();
        }
    }
}
