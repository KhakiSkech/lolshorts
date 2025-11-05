/// Audio capture utilities for Windows using DirectShow
///
/// This module provides:
/// - Audio device enumeration via FFmpeg/DirectShow
/// - Audio input configuration for microphone and system audio
/// - Volume control and mixing parameters
/// - FFmpeg command builder for audio capture

use anyhow::{Result, Context as AnyhowContext};
use std::process::Command;
use serde::{Deserialize, Serialize};

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
    pub fn is_enabled(&self) -> bool {
        self.record_microphone || self.record_system_audio
    }

    /// Build FFmpeg audio input arguments
    ///
    /// Returns (input_args, filter_args, map_args, codec_args)
    /// where each component is a Vec of FFmpeg argument strings
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

            let mic_device = self.microphone_device.as_ref()
                .map(|d| format!("audio={}", d))
                .unwrap_or_else(|| "audio=@device_cm_{33D9A762-90C8-11D0-BD43-00A0C911CE86}\\wave_in".to_string());
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

            let sys_device = self.system_audio_device.as_ref()
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
            filter_parts.push(format!("{}amix=inputs={}[aout]", mix_inputs.join(""), mix_inputs.len()));
            vec![
                "-filter_complex".to_string(),
                filter_parts.join(";"),
            ]
        } else if mix_inputs.len() == 1 {
            // Single audio source, just apply volume
            vec![
                "-filter_complex".to_string(),
                filter_parts.join(";"),
                "-map".to_string(),
                "0:v".to_string(),
                "-map".to_string(),
                if self.record_microphone { "[mic]" } else { "[sys]" }.to_string(),
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

/// List available audio devices on Windows
///
/// Uses FFmpeg's DirectShow to enumerate devices
pub fn list_audio_devices() -> Result<Vec<AudioDevice>> {
    tracing::debug!("Listing DirectShow audio devices...");

    let output = Command::new("ffmpeg")
        .args(&[
            "-list_devices", "true",
            "-f", "dshow",
            "-i", "dummy"
        ])
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
            in_audio_section = false;
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
                        || name.to_lowercase().contains("input") {
                        AudioDeviceType::Microphone
                    } else {
                        AudioDeviceType::SystemAudio
                    };

                    devices.push(AudioDevice {
                        name,
                        device_type,
                    });
                }
            }
        }
    }

    tracing::info!("Found {} audio devices", devices.len());
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
