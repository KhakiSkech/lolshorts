use crate::utils::ffmpeg::get_ffmpeg_path;
/// Audio capture utilities for Windows using DirectShow
///
/// This module provides:
/// - Audio device enumeration via FFmpeg/DirectShow
/// - Audio input configuration for microphone and system audio
/// - Volume control and mixing parameters
/// - FFmpeg command builder for audio capture
use anyhow::{Context as AnyhowContext, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub device_type: AudioDeviceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioDeviceType {
    Microphone,
    SystemAudio,
}

/// Audio capture configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Enable microphone recording
    pub record_microphone: bool,
    /// Microphone device name (None = default device)
    pub microphone_device: Option<String>,
    /// Microphone volume (0-200%)
    pub microphone_volume: u8,

    /// Enable system audio recording
    pub record_system_audio: bool,
    /// System audio device name (None = default device)
    pub system_audio_device: Option<String>,
    /// System audio volume (0-200%)
    pub system_audio_volume: u8,

    /// Audio sample rate
    pub sample_rate: u32,
    /// Audio bitrate in kbps
    pub bitrate: u32,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            record_microphone: true,
            microphone_device: None,
            microphone_volume: 120,
            record_system_audio: true,
            system_audio_device: None,
            system_audio_volume: 100,
            sample_rate: 48000,
            bitrate: 192,
        }
    }
}

impl AudioConfig {
    /// Check if any audio capture is enabled
    #[allow(dead_code)]
    pub fn is_enabled(&self) -> bool {
        self.record_microphone || self.record_system_audio
    }

    /// Build FFmpeg audio input arguments
    ///
    /// Returns (input_args, filter_args, map_args, codec_args)
    /// where each component is a Vec of FFmpeg argument strings
    #[allow(dead_code)]
    pub fn build_ffmpeg_args(&self) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
        if !self.is_enabled() {
            return (vec![], vec![], vec![], vec![]);
        }

        let mut input_args = Vec::new();
        let mut filter_parts = Vec::new();
        let mut map_args = Vec::new();
        let codec_args = vec![
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            format!("{}k", self.bitrate),
            "-ar".to_string(),
            self.sample_rate.to_string(),
        ];

        // Track which audio input index we're on (starts at 1, since 0 is video)
        let mut audio_input_idx = 1;
        let mut mix_inputs = Vec::new();

        // Add microphone input
        if self.record_microphone {
            input_args.push("-f".to_string());
            input_args.push("dshow".to_string());
            input_args.push("-i".to_string());

            let mic_device = self
                .microphone_device
                .as_ref()
                .map(|d| format!("audio={}", d))
                .unwrap_or_else(|| {
                    "audio=@device_cm_{33D9A762-90C8-11D0-BD43-00A0C911CE86}\\wave_in".to_string()
                });
            input_args.push(mic_device);

            // Apply volume to microphone
            let volume = self.microphone_volume as f32 / 100.0;
            filter_parts.push(format!("[{}:a]volume={}[mic]", audio_input_idx, volume));
            mix_inputs.push("[mic]".to_string());
            audio_input_idx += 1;
        }

        // Add system audio input (loopback)
        if self.record_system_audio {
            input_args.push("-f".to_string());
            input_args.push("dshow".to_string());
            input_args.push("-i".to_string());

            let sys_device = self
                .system_audio_device
                .as_ref()
                .map(|d| format!("audio={}", d))
                .unwrap_or_else(|| "audio=Stereo Mix".to_string());
            input_args.push(sys_device);

            // Apply volume to system audio
            let volume = self.system_audio_volume as f32 / 100.0;
            filter_parts.push(format!("[{}:a]volume={}[sys]", audio_input_idx, volume));
            mix_inputs.push("[sys]".to_string());
        }

        // Build filter_complex for mixing
        let filter_args = if mix_inputs.len() > 1 {
            // Mix multiple audio sources
            filter_parts.push(format!(
                "{}amix=inputs={}[aout]",
                mix_inputs.join(""),
                mix_inputs.len()
            ));
            vec!["-filter_complex".to_string(), filter_parts.join(";")]
        } else if mix_inputs.len() == 1 {
            // Single audio source, just apply volume
            vec![
                "-filter_complex".to_string(),
                filter_parts.join(";"),
                "-map".to_string(),
                "0:v".to_string(),
                "-map".to_string(),
                if self.record_microphone {
                    "[mic]"
                } else {
                    "[sys]"
                }
                .to_string(),
            ]
        } else {
            vec![]
        };

        // Add audio mapping
        if mix_inputs.len() > 1 {
            map_args.push("-map".to_string());
            map_args.push("0:v".to_string());
            map_args.push("-map".to_string());
            map_args.push("[aout]".to_string());
        }

        (input_args, filter_args, map_args, codec_args)
    }
}

/// Global audio configuration state
static CURRENT_AUDIO_CONFIG: std::sync::OnceLock<
    std::sync::RwLock<Option<crate::settings::models::AudioSettings>>,
> = std::sync::OnceLock::new();

fn get_audio_config_state(
) -> &'static std::sync::RwLock<Option<crate::settings::models::AudioSettings>> {
    CURRENT_AUDIO_CONFIG.get_or_init(|| std::sync::RwLock::new(None))
}

/// Apply audio configuration to the recording system
/// This function validates the configuration and stores it for use during recording
pub fn apply_audio_config(config: &crate::settings::models::AudioSettings) -> Result<()> {
    // Validate audio configuration
    if config.record_microphone && config.microphone_device.is_none() {
        return Err(anyhow::anyhow!(
            "Microphone recording enabled but no device selected. Please select a microphone device in settings."
        ));
    }

    if config.record_system_audio && config.system_audio_device.is_none() {
        return Err(anyhow::anyhow!(
            "System audio recording enabled but no device selected. Please select a system audio device (e.g., 'Stereo Mix' or 'What U Hear') in settings."
        ));
    }

    // Validate volume ranges (0-200%)
    if config.microphone_volume > 200 {
        return Err(anyhow::anyhow!(
            "Microphone volume must be between 0 and 200. Got: {}",
            config.microphone_volume
        ));
    }

    if config.system_audio_volume > 200 {
        return Err(anyhow::anyhow!(
            "System audio volume must be between 0 and 200. Got: {}",
            config.system_audio_volume
        ));
    }

    // Store the validated configuration in global state
    let state = get_audio_config_state();
    let mut guard = state
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to acquire audio config lock: {}", e))?;
    *guard = Some(config.clone());

    tracing::info!(
        "Audio configuration applied successfully: microphone={} (device: {:?}, volume: {}%), system_audio={} (device: {:?}, volume: {}%)",
        config.record_microphone,
        config.microphone_device,
        config.microphone_volume,
        config.record_system_audio,
        config.system_audio_device,
        config.system_audio_volume
    );

    Ok(())
}

/// Get the currently applied audio configuration
#[allow(dead_code)]
pub fn get_current_audio_config() -> Option<crate::settings::models::AudioSettings> {
    let state = get_audio_config_state();
    state.read().ok().and_then(|guard| guard.clone())
}

/// Build FFmpeg audio arguments from the current audio configuration
#[allow(dead_code)]
pub fn build_audio_args_from_config() -> Result<(Vec<String>, Vec<String>, Vec<String>, Vec<String>)>
{
    let config = get_current_audio_config().ok_or_else(|| {
        anyhow::anyhow!("No audio configuration applied. Call apply_audio_config first.")
    })?;

    let audio_config = AudioConfig {
        record_microphone: config.record_microphone,
        microphone_device: config.microphone_device,
        microphone_volume: config.microphone_volume,
        record_system_audio: config.record_system_audio,
        system_audio_device: config.system_audio_device,
        system_audio_volume: config.system_audio_volume,
        sample_rate: 48000,
        bitrate: 192,
    };

    Ok(audio_config.build_ffmpeg_args())
}

/// Cached audio device manager for memory efficiency
pub struct AudioDeviceManager {
    devices: Vec<AudioDevice>,
    /// Last refresh timestamp (for cache management)
    pub last_refresh: std::time::Instant,
    /// Cache time-to-live (for cache management)
    pub cache_ttl: std::time::Duration,
}

impl AudioDeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            last_refresh: std::time::Instant::now(),
            cache_ttl: std::time::Duration::from_secs(60), // Cache for 60 seconds
        }
    }

    /// Get cached audio devices as a slice (no copying)
    #[allow(dead_code)]
    pub fn get_devices(&self) -> &[AudioDevice] {
        &self.devices
    }

    /// Refresh devices if cache expired
    #[allow(dead_code)]
    pub async fn refresh_if_needed(&mut self) -> Result<()> {
        if self.last_refresh.elapsed() < self.cache_ttl {
            return Ok(()); // Use cached devices
        }

        tracing::debug!("Refreshing cached audio devices...");

        // Method 1: Try Windows Core Audio API (more reliable)
        if let Ok(core_devices) = list_audio_devices() {
            self.devices = core_devices;
            self.last_refresh = std::time::Instant::now();
            tracing::info!(
                "Found {} audio devices via Windows Core Audio API",
                self.devices.len()
            );
            return Ok(());
        }

        // Method 2: Fallback to FFmpeg DirectShow (less reliable)
        tracing::warn!("Windows Core Audio API failed, falling back to FFmpeg DirectShow");
        if let Ok(ffmpeg_devices) = list_audio_devices_ffmpeg() {
            self.devices = ffmpeg_devices;
            self.last_refresh = std::time::Instant::now();
            tracing::info!(
                "Found {} audio devices via FFmpeg DirectShow",
                self.devices.len()
            );
        }

        Ok(())
    }

    /// Force refresh regardless of cache TTL
    #[allow(dead_code)]
    pub async fn force_refresh(&mut self) -> Result<()> {
        self.last_refresh = std::time::Instant::now() - self.cache_ttl;
        self.refresh_if_needed().await
    }
}

/// Global audio device manager instance (async thread-safe)
static AUDIO_DEVICE_MANAGER: std::sync::OnceLock<tokio::sync::Mutex<AudioDeviceManager>> =
    std::sync::OnceLock::new();

/// Get global audio device manager
pub fn get_audio_device_manager() -> &'static tokio::sync::Mutex<AudioDeviceManager> {
    AUDIO_DEVICE_MANAGER.get_or_init(|| tokio::sync::Mutex::new(AudioDeviceManager::new()))
}

/// List available audio devices (optimized with caching and slice return)
#[allow(dead_code)]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>> {
    tracing::debug!("Getting audio devices (cached)...");

    let manager = get_audio_device_manager();

    // Use non-blocking try_lock to avoid deadlocks
    let manager_guard = manager
        .try_lock()
        .map_err(|_| anyhow::anyhow!("Audio device manager is locked"))?;

    // Clone only if needed (for backward compatibility)
    Ok(manager_guard.devices.clone())
}

/// Get audio devices as slice (memory efficient - no copying)
#[allow(dead_code)]
pub fn get_audio_devices_slice() -> Result<&'static [AudioDevice]> {
    let manager = get_audio_device_manager();

    // Return reference to cached data - zero copy!
    let manager_guard = manager
        .try_lock()
        .map_err(|_| anyhow::anyhow!("Audio device manager is locked"))?;

    // SAFETY: We're returning a reference to data that lives in static storage
    // This is safe because the data lives for the entire program duration
    unsafe {
        Ok(
            std::mem::transmute::<&[AudioDevice], &'static [AudioDevice]>(
                &manager_guard.devices[..],
            ),
        )
    }
}

/// List audio devices from Windows Registry
#[allow(dead_code)]
fn list_audio_devices_from_registry() -> Result<Vec<AudioDevice>> {
    let mut devices = Vec::new();

    // Query Windows Registry for audio devices
    // HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture
    let registry_keys = vec![
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture",
        r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render",
    ];

    for registry_key in registry_keys {
        if let Ok(output) = Command::new("reg")
            .args(["query", registry_key, "/s", "/v", "/f", "DeviceDesc"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                devices.extend(parse_registry_audio_output(
                    &stdout,
                    registry_key.contains("Capture"),
                ));
            }
        }
    }

    Ok(devices)
}

/// Parse Windows Registry output for audio devices
#[allow(dead_code)]
fn parse_registry_audio_output(output: &str, is_capture: bool) -> Vec<AudioDevice> {
    let mut devices = Vec::new();
    let mut current_device: Option<String> = None;

    for line in output.lines() {
        if line.trim().starts_with("HKEY_") {
            // New registry key - reset current device
            current_device = None;
        } else if line.trim().contains("DeviceDesc") && line.contains("REG_SZ") {
            // Extract device description from Registry value
            if let Some(start) = line.find("REG_SZ") {
                let device_desc = line[start + 7..].trim().trim_matches('"');
                if !device_desc.is_empty() {
                    current_device = Some(device_desc.to_string());
                }
            }
        } else if line.trim().is_empty() && current_device.is_some() {
            // Empty line after device description - add device to list
            let device_name = current_device.as_ref().unwrap().clone();
            if !devices.iter().any(|d: &AudioDevice| d.name == device_name) {
                let device_type = if is_capture {
                    AudioDeviceType::Microphone
                } else {
                    AudioDeviceType::SystemAudio
                };

                devices.push(AudioDevice {
                    name: device_name,
                    device_type,
                });
            }
        }
    }

    devices
}

/// List audio devices using Windows Command Prompt
#[allow(dead_code)]
fn list_audio_devices_from_cmd() -> Result<Vec<AudioDevice>> {
    let mut devices = Vec::new();

    // Use dsound (DirectSound) command line tools if available
    if let Ok(output) = Command::new("cmd").args(["/c", "where dsound"]).output() {
        if output.status.success() {
            // dsound is available, try to use it
            if let Ok(sound_devices) = Command::new("powershell")
                .args([
                    "-Command",
                    "Get-WmiObject -Class Win32_SoundDevice | Select-Object Name, DeviceID | ConvertTo-Json -Compress"
                ])
                .output()
            {
                if sound_devices.status.success() {
                    let stdout = String::from_utf8_lossy(&sound_devices.stdout);
                    if let Ok(wmi_devices) = serde_json::from_str::<Vec<serde_json::Value>>(&stdout) {
                        for device in wmi_devices {
                            if let Some(name) = device.get("Name").and_then(|v| v.as_str()) {
                                let device_type = if name.to_lowercase().contains("capture")
                                    || name.to_lowercase().contains("microphone")
                                    || name.to_lowercase().contains("input")
                                    || name.to_lowercase().contains("mic") {
                                    AudioDeviceType::Microphone
                                } else {
                                    AudioDeviceType::SystemAudio
                                };

                                if !devices.iter().any(|d: &AudioDevice| d.name == name) {
                                    devices.push(AudioDevice {
                                        name: name.to_string(),
                                        device_type,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(devices)
}

/// Alternative audio device enumeration using common Windows tools
#[allow(dead_code)]
fn list_audio_devices_alternative() -> Result<Vec<AudioDevice>> {
    let mut devices = Vec::new();

    // Use PowerShell to check for common audio device names
    if let Ok(_output) = Command::new("powershell")
        .args([
            "-Command",
            "Get-Process | Where-Object { $_.ProcessName -like '*audio*' -or $_.ProcessName -like '*sound*' } | Select-Object ProcessName | ConvertTo-Json -Compress"
        ])
        .output()
    {
        // This method is a fallback that looks for running audio processes
        // We'll create some default Windows audio device names
        let default_devices = vec![
            ("스피커", AudioDeviceType::SystemAudio),
            ("마이크", AudioDeviceType::Microphone),
            ("Microphone", AudioDeviceType::Microphone),
            ("Speakers", AudioDeviceType::SystemAudio),
            ("Headphones", AudioDeviceType::SystemAudio),
            ("Line In", AudioDeviceType::Microphone),
        ];

        for (name, device_type) in default_devices {
            devices.push(AudioDevice {
                name: name.to_string(),
                device_type,
            });
        }

        tracing::warn!("Using default audio device names due to enumeration failure");
        return Ok(devices);
    }

    Ok(devices)
}

/// Fallback method using FFmpeg DirectShow (original implementation)
#[allow(dead_code)]
pub fn list_audio_devices_ffmpeg() -> Result<Vec<AudioDevice>> {
    tracing::debug!("Listing DirectShow audio devices...");

    let ffmpeg_path =
        get_ffmpeg_path().context("Failed to find FFmpeg for audio device listing")?;

    let output = Command::new(ffmpeg_path)
        .args(["-list_devices", "true", "-f", "dshow", "-i", "dummy"])
        .output()
        .context("Failed to execute ffmpeg for device listing")?;

    // FFmpeg outputs device list to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut devices = Vec::new();
    let mut in_audio_section = false;

    for line in stderr.lines() {
        if line.contains("DirectShow audio devices") {
            in_audio_section = true;
            continue;
        }

        if line.contains("DirectShow video devices") {
            break;
        }

        if in_audio_section && line.contains('"') {
            // Extract device name from format: [dshow @ ...] "Device Name"
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let name = line[start + 1..start + 1 + end].to_string();

                    // Categorize by common device name patterns
                    let device_type = if name.to_lowercase().contains("mic")
                        || name.to_lowercase().contains("microphone")
                        || name.to_lowercase().contains("input")
                    {
                        AudioDeviceType::Microphone
                    } else {
                        AudioDeviceType::SystemAudio
                    };

                    devices.push(AudioDevice { name, device_type });
                }
            }
        }
    }

    tracing::info!(
        "Found {} audio devices via FFmpeg DirectShow",
        devices.len()
    );
    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert!(config.record_microphone);
        assert!(config.record_system_audio);
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.bitrate, 192);
    }

    #[test]
    fn test_audio_config_disabled() {
        let config = AudioConfig {
            record_microphone: false,
            record_system_audio: false,
            ..Default::default()
        };
        assert!(!config.is_enabled());

        let (input_args, filter_args, map_args, codec_args) = config.build_ffmpeg_args();
        assert!(input_args.is_empty());
        assert!(filter_args.is_empty());
        assert!(map_args.is_empty());
        assert!(codec_args.is_empty());
    }

    #[test]
    fn test_audio_config_microphone_only() {
        let config = AudioConfig {
            record_microphone: true,
            microphone_volume: 150,
            record_system_audio: false,
            ..Default::default()
        };
        assert!(config.is_enabled());

        let (input_args, filter_args, _, codec_args) = config.build_ffmpeg_args();
        assert!(!input_args.is_empty());
        assert!(!filter_args.is_empty());
        assert!(!codec_args.is_empty());

        // Check volume is applied (150% = 1.5)
        let filter_str = filter_args.join(" ");
        assert!(filter_str.contains("volume=1.5"));
    }

    #[test]
    fn test_audio_config_both_sources() {
        let config = AudioConfig {
            record_microphone: true,
            microphone_volume: 120,
            record_system_audio: true,
            system_audio_volume: 100,
            ..Default::default()
        };
        assert!(config.is_enabled());

        let (input_args, filter_args, map_args, codec_args) = config.build_ffmpeg_args();
        assert!(!input_args.is_empty());
        assert!(!filter_args.is_empty());
        assert!(!map_args.is_empty());
        assert!(!codec_args.is_empty());

        // Check mixing is configured
        let filter_str = filter_args.join(" ");
        assert!(filter_str.contains("amix"));
        assert!(filter_str.contains("[aout]"));
    }
}
