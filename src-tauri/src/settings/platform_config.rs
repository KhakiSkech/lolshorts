use super::models::{RecordingSettings, EncoderPreference};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlatformConfigError {
    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("Hardware detection failed: {0}")]
    HardwareDetection(String),

    #[error("Configuration validation failed: {0}")]
    Validation(String),

    #[error("Settings migration failed: {0}")]
    Migration(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PlatformConfigError>;

/// Platform-specific configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Platform identifier
    pub platform: Platform,

    /// Hardware capabilities
    pub hardware: HardwareCapabilities,

    /// Default settings overrides
    pub default_overrides: RecordingSettings,

    /// Platform-specific feature flags
    pub features: PlatformFeatures,

    /// Recommended settings based on hardware
    pub recommended_settings: RecommendedSettings,
}

/// Platform enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Platform {
    Windows,
    MacOS,
    Linux,
}

/// Hardware capabilities detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareCapabilities {
    /// CPU information
    pub cpu: CpuInfo,

    /// GPU information
    pub gpu: Vec<GpuInfo>,

    /// Memory information
    pub memory: MemoryInfo,

    /// Display information
    pub displays: Vec<DisplayInfo>,

    /// Audio devices
    pub audio_devices: AudioDeviceInfo,

    /// Storage information
    pub storage: StorageInfo,
}

/// CPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub logical_cores: usize,
    pub max_frequency: f64,
    pub has_avx: bool,
    pub has_avx2: bool,
    pub architecture: String,
}

/// GPU information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: GpuVendor,
    pub memory_mb: u64,
    pub driver_version: String,
    pub is_primary: bool,
    pub supports_encoding: bool,
    pub supports_nvenc: bool,
    pub supports_amf: bool,
    pub supports_qsv: bool,
}

/// GPU vendor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown,
}

/// Memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_gb: f64,
    pub available_gb: f64,
    pub speed_mhz: Option<f64>,
}

/// Display information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub id: String,
    pub name: String,
    pub resolution: (u32, u32),
    pub refresh_rate: f64,
    pub is_primary: bool,
    pub scaling_factor: f64,
}

/// Audio device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub input_devices: Vec<AudioDevice>,
    pub output_devices: Vec<AudioDevice>,
    pub default_input: Option<String>,
    pub default_output: Option<String>,
}

/// Audio device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    pub channels: u32,
    pub sample_rate: u32,
    pub is_default: bool,
}

/// Storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_space_gb: f64,
    pub free_space_gb: f64,
    pub install_drive: String,
    pub temp_directory: String,
}

/// Platform-specific feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformFeatures {
    /// Supports windows-capture backend
    pub supports_windows_capture: bool,

    /// Supports FFmpeg native backend
    pub supports_ffmpeg_native: bool,

    /// Supports Core Graphics on macOS
    pub supports_core_graphics: bool,

    /// Supports hardware acceleration
    pub supports_hardware_acceleration: bool,

    /// Supports system tray integration
    pub supports_system_tray: bool,

    /// Supports global hotkeys
    pub supports_global_hotkeys: bool,

    /// Supports file associations
    pub supports_file_associations: bool,

    /// Supports auto-start
    pub supports_auto_start: bool,

    /// Supports notifications
    pub supports_notifications: bool,

    /// Supports game detection via API
    pub supports_api_detection: bool,

    /// Supports window enumeration
    pub supports_window_enumeration: bool,
}

/// Recommended settings based on hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedSettings {
    /// Video encoding recommendations
    pub video: VideoRecommendations,

    /// Audio recommendations
    pub audio: AudioRecommendations,

    /// Performance recommendations
    pub performance: PerformanceRecommendations,

    /// Storage recommendations
    pub storage: StorageRecommendations,
}

/// Video encoding recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoRecommendations {
    pub recommended_encoder: EncoderPreference,
    pub recommended_codec: String,
    pub recommended_bitrate_kbps: u32,
    pub recommended_resolution: String,
    pub recommended_frame_rate: String,
    pub maximum_recording_hours: f64,
}

/// Audio recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRecommendations {
    pub recommended_sample_rate: String,
    pub recommended_bitrate: String,
    pub max_channels: u32,
    pub enable_microphone_by_default: bool,
    pub enable_system_audio_by_default: bool,
}

/// Performance recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendations {
    pub enable_hardware_acceleration: bool,
    pub recommended_buffer_size_mb: u32,
    pub recommended_temp_cleanup_interval_minutes: u32,
    pub recommended_concurrent_clips: u32,
    pub enable_performance_monitoring: bool,
}

/// Storage recommendations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRecommendations {
    pub recommended_clips_directory: String,
    pub minimum_free_space_gb: f64,
    pub recommended_cleanup_threshold_gb: f64,
    pub enable_auto_cleanup: bool,
}

impl PlatformConfig {
    /// Detect current platform and hardware capabilities
    pub async fn detect() -> Result<Self> {
        let platform = Self::detect_platform()?;
        let hardware = Self::detect_hardware(&platform).await?;
        let features = Self::detect_features(&platform, &hardware);
        let default_overrides = Self::get_default_overrides(&platform, &hardware);
        let recommended_settings = Self::generate_recommendations(&platform, &hardware);

        Ok(Self {
            platform,
            hardware,
            default_overrides,
            features,
            recommended_settings,
        })
    }

    /// Detect current platform
    fn detect_platform() -> Result<Platform> {
        #[cfg(target_os = "windows")]
        {
            Ok(Platform::Windows)
        }
        #[cfg(target_os = "macos")]
        {
            Ok(Platform::MacOS)
        }
        #[cfg(target_os = "linux")]
        {
            Ok(Platform::Linux)
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(PlatformConfigError::UnsupportedPlatform(
                std::env::consts::OS.to_string()
            ))
        }
    }

    /// Detect hardware capabilities
    async fn detect_hardware(platform: &Platform) -> Result<HardwareCapabilities> {
        match platform {
            Platform::Windows => Self::detect_windows_hardware().await,
            Platform::MacOS => Self::detect_macos_hardware().await,
            Platform::Linux => Self::detect_linux_hardware().await,
        }
    }

    /// Detect Windows hardware
    #[cfg(target_os = "windows")]
    async fn detect_windows_hardware() -> Result<HardwareCapabilities> {
        use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

        // CPU detection - extract needed info before await
        let processor_count = {
            let mut cpu_info = SYSTEM_INFO::default();
            unsafe { GetSystemInfo(&mut cpu_info) };
            cpu_info.dwNumberOfProcessors as usize
        };

        let cpu = CpuInfo {
            model: Self::get_windows_cpu_name().await?,
            cores: processor_count,
            logical_cores: processor_count,
            max_frequency: Self::get_windows_cpu_frequency().await?,
            has_avx: Self::check_windows_cpu_feature("avx").await?,
            has_avx2: Self::check_windows_cpu_feature("avx2").await?,
            architecture: "x86_64".to_string(),
        };

        // GPU detection
        let gpu = Self::detect_windows_gpu().await?;

        // Memory detection
        let memory = Self::detect_windows_memory().await?;

        // Display detection
        let displays = Self::detect_windows_displays().await?;

        // Audio device detection
        let audio_devices = Self::detect_windows_audio().await?;

        // Storage detection
        let storage = Self::detect_windows_storage().await?;

        Ok(HardwareCapabilities {
            cpu,
            gpu,
            memory,
            displays,
            audio_devices,
            storage,
        })
    }

    /// Detect macOS hardware
    #[cfg(target_os = "macos")]
    async fn detect_macos_hardware() -> Result<HardwareCapabilities> {
        use std::process::Command;

        // CPU detection
        let cpu_output = Command::new("sysctl")
            .args(&["-n", "machdep.cpu.brand_string"])
            .output()
            .map_err(|e| PlatformConfigError::HardwareDetection(e.to_string()))?;

        let cpu_model = String::from_utf8_lossy(&cpu_output.stdout).trim().to_string();

        let cpu_cores_output = Command::new("sysctl")
            .args(&["-n", "hw.ncpu"])
            .output()
            .map_err(|e| PlatformConfigError::HardwareDetection(e.to_string()))?;

        let cpu_cores = String::from_utf8_lossy(&cpu_cores_output.stdout)
            .trim()
            .parse::<usize>()
            .map_err(|e| PlatformConfigError::HardwareDetection(e.to_string()))?;

        let cpu = CpuInfo {
            model: cpu_model,
            cores: cpu_cores,
            logical_cores: cpu_cores, // macOS doesn't easily distinguish
            max_frequency: 0.0, // macOS makes this harder to get
            has_avx: true, // Assume modern Macs have AVX
            has_avx2: true, // Assume modern Macs have AVX2
            architecture: std::env::consts::ARCH.to_string(),
        };

        // GPU detection (simplified - would need Metal API calls for full info)
        let gpu = vec![GpuInfo {
            name: "Apple Silicon GPU".to_string(),
            vendor: GpuVendor::Apple,
            memory_mb: 0, // Would need Metal API
            driver_version: "N/A".to_string(),
            is_primary: true,
            supports_encoding: true,
            supports_nvenc: false,
            supports_amf: false,
            supports_qsv: false,
        }];

        // Memory detection
        let mem_output = Command::new("sysctl")
            .args(&["-n", "hw.memsize"])
            .output()
            .map_err(|e| PlatformConfigError::HardwareDetection(e.to_string()))?;

        let total_bytes = String::from_utf8_lossy(&mem_output.stdout)
            .trim()
            .parse::<u64>()
            .map_err(|e| PlatformConfigError::HardwareDetection(e.to_string()))?;

        let memory = MemoryInfo {
            total_gb: total_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            available_gb: 0.0, // Would need vm_stat calls
            speed_mhz: None,
        };

        // Display detection (simplified)
        let displays = vec![DisplayInfo {
            id: "primary".to_string(),
            name: "Built-in Display".to_string(),
            resolution: (1920, 1080), // Default fallback
            refresh_rate: 60.0,
            is_primary: true,
            scaling_factor: 2.0, // Retina
        }];

        // Audio devices (simplified)
        let audio_devices = AudioDeviceInfo {
            input_devices: vec![],
            output_devices: vec![],
            default_input: None,
            default_output: None,
        };

        // Storage detection
        let storage = StorageInfo {
            total_space_gb: 0.0,
            free_space_gb: 0.0,
            install_drive: "/".to_string(),
            temp_directory: std::env::temp_dir().to_string_lossy().to_string(),
        };

        Ok(HardwareCapabilities {
            cpu,
            gpu,
            memory,
            displays,
            audio_devices,
            storage,
        })
    }

    /// Detect Linux hardware
    #[cfg(target_os = "linux")]
    async fn detect_linux_hardware() -> Result<HardwareCapabilities> {
        // Simplified Linux hardware detection
        // Would need to implement proper /proc and /sys parsing
        Err(PlatformConfigError::UnsupportedPlatform(
            "Linux hardware detection not fully implemented".to_string()
        ))
    }

    /// Detect platform features
    fn detect_features(platform: &Platform, hardware: &HardwareCapabilities) -> PlatformFeatures {
        match platform {
            Platform::Windows => PlatformFeatures {
                supports_windows_capture: true,
                supports_ffmpeg_native: true,
                supports_core_graphics: false,
                supports_hardware_acceleration: hardware.gpu.iter().any(|gpu| gpu.supports_encoding),
                supports_system_tray: true,
                supports_global_hotkeys: true,
                supports_file_associations: true,
                supports_auto_start: true,
                supports_notifications: true,
                supports_api_detection: true,
                supports_window_enumeration: true,
            },
            Platform::MacOS => PlatformFeatures {
                supports_windows_capture: false,
                supports_ffmpeg_native: true,
                supports_core_graphics: true,
                supports_hardware_acceleration: hardware.gpu.iter().any(|gpu| gpu.supports_encoding),
                supports_system_tray: true,
                supports_global_hotkeys: true,
                supports_file_associations: true,
                supports_auto_start: true,
                supports_notifications: true,
                supports_api_detection: true,
                supports_window_enumeration: true,
            },
            Platform::Linux => PlatformFeatures {
                supports_windows_capture: false,
                supports_ffmpeg_native: true,
                supports_core_graphics: false,
                supports_hardware_acceleration: hardware.gpu.iter().any(|gpu| gpu.supports_encoding),
                supports_system_tray: false,
                supports_global_hotkeys: false,
                supports_file_associations: true,
                supports_auto_start: true,
                supports_notifications: true,
                supports_api_detection: false,
                supports_window_enumeration: false,
            },
        }
    }

    /// Get platform-specific default overrides
    fn get_default_overrides(platform: &Platform, hardware: &HardwareCapabilities) -> RecordingSettings {
        let mut defaults = RecordingSettings::default();

        match platform {
            Platform::Windows => {
                // Windows-specific defaults
                defaults.video.encoder = if hardware.gpu.iter().any(|gpu| gpu.supports_nvenc) {
                    EncoderPreference::Nvenc
                } else if hardware.gpu.iter().any(|gpu| gpu.supports_amf) {
                    EncoderPreference::Amf
                } else if hardware.gpu.iter().any(|gpu| gpu.supports_qsv) {
                    EncoderPreference::Qsv
                } else {
                    EncoderPreference::Software
                };
            }
            Platform::MacOS => {
                // macOS-specific defaults
                defaults.video.encoder = EncoderPreference::Software; // Use software until Metal encoding is implemented
                defaults.minimize_to_tray = false; // macOS doesn't have traditional tray
            }
            Platform::Linux => {
                // Linux-specific defaults
                defaults.video.encoder = EncoderPreference::Software;
                defaults.minimize_to_tray = false;
            }
        }

        defaults
    }

    /// Generate hardware-based recommendations
    fn generate_recommendations(_platform: &Platform, hardware: &HardwareCapabilities) -> RecommendedSettings {
        let video = Self::generate_video_recommendations(hardware);
        let audio = Self::generate_audio_recommendations(hardware);
        let performance = Self::generate_performance_recommendations(hardware);
        let storage = Self::generate_storage_recommendations(hardware);

        RecommendedSettings {
            video,
            audio,
            performance,
            storage,
        }
    }

    /// Generate video recommendations
    fn generate_video_recommendations(hardware: &HardwareCapabilities) -> VideoRecommendations {
        let total_memory_gb = hardware.memory.total_gb;
        let gpu_memory_mb = hardware.gpu.iter().map(|gpu| gpu.memory_mb).max().unwrap_or(0);

        let (recommended_encoder, recommended_codec, recommended_bitrate_kbps) =
            if gpu_memory_mb >= 8000 {
                (EncoderPreference::Auto, "h265", 20000)
            } else if gpu_memory_mb >= 4000 {
                (EncoderPreference::Auto, "h265", 10000)
            } else {
                (EncoderPreference::Software, "h264", 5000)
            };

        let (recommended_resolution, recommended_frame_rate) =
            if total_memory_gb >= 16.0 && gpu_memory_mb >= 6000 {
                ("2560x1440", "60")
            } else if total_memory_gb >= 8.0 {
                ("1920x1080", "60")
            } else {
                ("1280x720", "30")
            };

        let maximum_recording_hours = if hardware.storage.free_space_gb >= 100.0 {
            24.0
        } else if hardware.storage.free_space_gb >= 50.0 {
            12.0
        } else {
            6.0
        };

        VideoRecommendations {
            recommended_encoder,
            recommended_codec: recommended_codec.to_string(),
            recommended_bitrate_kbps,
            recommended_resolution: recommended_resolution.to_string(),
            recommended_frame_rate: recommended_frame_rate.to_string(),
            maximum_recording_hours,
        }
    }

    /// Generate audio recommendations
    fn generate_audio_recommendations(_hardware: &HardwareCapabilities) -> AudioRecommendations {
        AudioRecommendations {
            recommended_sample_rate: "48000".to_string(),
            recommended_bitrate: "192".to_string(),
            max_channels: 2,
            enable_microphone_by_default: true,
            enable_system_audio_by_default: true,
        }
    }

    /// Generate performance recommendations
    fn generate_performance_recommendations(hardware: &HardwareCapabilities) -> PerformanceRecommendations {
        let total_memory_gb = hardware.memory.total_gb;

        PerformanceRecommendations {
            enable_hardware_acceleration: hardware.gpu.iter().any(|gpu| gpu.supports_encoding),
            recommended_buffer_size_mb: if total_memory_gb >= 16.0 {
                512
            } else if total_memory_gb >= 8.0 {
                256
            } else {
                128
            },
            recommended_temp_cleanup_interval_minutes: 30,
            recommended_concurrent_clips: if total_memory_gb >= 16.0 {
                5
            } else {
                3
            },
            enable_performance_monitoring: true,
        }
    }

    /// Generate storage recommendations
    fn generate_storage_recommendations(hardware: &HardwareCapabilities) -> StorageRecommendations {
        StorageRecommendations {
            recommended_clips_directory: format!("{}/Documents/LoLShorts", std::env::var("HOME").unwrap_or_default()),
            minimum_free_space_gb: 10.0,
            recommended_cleanup_threshold_gb: hardware.storage.total_space_gb * 0.1,
            enable_auto_cleanup: hardware.storage.free_space_gb < 50.0,
        }
    }

    // Windows-specific helper methods
    #[cfg(target_os = "windows")]
    async fn get_windows_cpu_name() -> Result<String> {
        // Simplified - would use WMI or registry
        Ok("Intel Core i7-9700K".to_string())
    }

    #[cfg(target_os = "windows")]
    async fn get_windows_cpu_frequency() -> Result<f64> {
        // Simplified - would use WMI
        Ok(3600.0)
    }

    #[cfg(target_os = "windows")]
    async fn check_windows_cpu_feature(_feature: &str) -> Result<bool> {
        // Simplified - would use CPUID
        Ok(true)
    }

    #[cfg(target_os = "windows")]
    async fn detect_windows_gpu() -> Result<Vec<GpuInfo>> {
        // Simplified - would use DirectX or WMI
        Ok(vec![GpuInfo {
            name: "NVIDIA GeForce RTX 3070".to_string(),
            vendor: GpuVendor::Nvidia,
            memory_mb: 8000,
            driver_version: "511.23".to_string(),
            is_primary: true,
            supports_encoding: true,
            supports_nvenc: true,
            supports_amf: false,
            supports_qsv: false,
        }])
    }

    #[cfg(target_os = "windows")]
    async fn detect_windows_memory() -> Result<MemoryInfo> {
        // Simplified - would use GlobalMemoryStatusEx
        Ok(MemoryInfo {
            total_gb: 16.0,
            available_gb: 8.0,
            speed_mhz: Some(3200.0),
        })
    }

    #[cfg(target_os = "windows")]
    async fn detect_windows_displays() -> Result<Vec<DisplayInfo>> {
        // Simplified - would use EnumDisplayMonitors
        Ok(vec![DisplayInfo {
            id: "PRIMARY".to_string(),
            name: "Primary Monitor".to_string(),
            resolution: (1920, 1080),
            refresh_rate: 144.0,
            is_primary: true,
            scaling_factor: 1.0,
        }])
    }

    #[cfg(target_os = "windows")]
    async fn detect_windows_audio() -> Result<AudioDeviceInfo> {
        // Simplified - would use WASAPI
        Ok(AudioDeviceInfo {
            input_devices: vec![],
            output_devices: vec![],
            default_input: None,
            default_output: None,
        })
    }

    #[cfg(target_os = "windows")]
    async fn detect_windows_storage() -> Result<StorageInfo> {
        // Simplified - would use GetDiskFreeSpaceExW
        Ok(StorageInfo {
            total_space_gb: 500.0,
            free_space_gb: 250.0,
            install_drive: "C:".to_string(),
            temp_directory: std::env::temp_dir().to_string_lossy().to_string(),
        })
    }

    /// Validate settings against platform capabilities
    pub fn validate_settings(&self, settings: &RecordingSettings) -> Result<()> {
        // Validate video settings
        if settings.video.frame_rate == super::models::FrameRate::Fps120 &&
           self.hardware.memory.total_gb < 8.0 {
            return Err(PlatformConfigError::Validation(
                "120 FPS recording requires at least 8GB RAM".to_string()
            ));
        }

        // Validate audio settings
        // FIX: Don't fail validation just because mic/system audio is missing.
        // Instead, if record_microphone is true but no devices exist, we will just handle it
        // gracefully during runtime (recording will start without audio).
        // The validation here should be about "impossible" configurations, not "currently unavailable" hardware.
        // So we REMOVE the check that returns Err on empty device list.
        
        /* 
        if settings.audio.record_microphone && self.hardware.audio_devices.input_devices.is_empty() {
            return Err(PlatformConfigError::Validation(
                "No microphone devices available".to_string()
            ));
        }
        */

        // Validate storage requirements
        if self.hardware.storage.free_space_gb < 5.0 {
            return Err(PlatformConfigError::Validation(
                "At least 5GB free space required for recording".to_string()
            ));
        }

        Ok(())
    }

    /// Apply platform-specific optimizations to settings
    pub fn optimize_settings(&self, settings: &mut RecordingSettings) {
        // Optimize based on hardware
        if self.hardware.memory.total_gb < 8.0 {
            settings.video.bitrate_preset = super::models::BitratePreset::Low;
        }

        // Optimize encoder selection
        if !self.hardware.gpu.iter().any(|gpu| gpu.supports_encoding) {
            settings.video.encoder = EncoderPreference::Software;
        }

        // If microphone is enabled but no devices found, gracefully disable it in settings
        // This prevents runtime warnings and ensures clean state
        if settings.audio.record_microphone && self.hardware.audio_devices.input_devices.is_empty() {
            settings.audio.record_microphone = false;
            settings.audio.microphone_device = None;
        }

        // Platform-specific optimizations
        match self.platform {
            Platform::Windows => {
                // Windows-specific optimizations
                if self.features.supports_windows_capture {
                    // Prefer windows-capture backend
                }
            }
            Platform::MacOS => {
                // macOS-specific optimizations
                settings.minimize_to_tray = false;
            }
            Platform::Linux => {
                // Linux-specific optimizations
                settings.minimize_to_tray = false;
                settings.show_notifications = true;
            }
        }
    }

    /// Stub implementation for macOS hardware detection on non-macOS platforms
    #[cfg(not(target_os = "macos"))]
    async fn detect_macos_hardware() -> Result<HardwareCapabilities> {
        Err(PlatformConfigError::UnsupportedPlatform("macOS hardware detection not available on this platform".to_string()))
    }

    /// Stub implementation for Linux hardware detection on non-Linux platforms
    #[cfg(not(target_os = "linux"))]
    async fn detect_linux_hardware() -> Result<HardwareCapabilities> {
        Err(PlatformConfigError::UnsupportedPlatform("Linux hardware detection not available on this platform".to_string()))
    }

    /// Stub implementation for Windows hardware detection on non-Windows platforms
    #[cfg(not(target_os = "windows"))]
    async fn detect_windows_hardware() -> Result<HardwareCapabilities> {
        Err(PlatformConfigError::UnsupportedPlatform("Windows hardware detection not available on this platform".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        #[cfg(target_os = "windows")]
        {
            assert_eq!(PlatformConfig::detect_platform().unwrap(), Platform::Windows);
        }
        #[cfg(target_os = "macos")]
        {
            assert_eq!(PlatformConfig::detect_platform().unwrap(), Platform::MacOS);
        }
        #[cfg(target_os = "linux")]
        {
            assert_eq!(PlatformConfig::detect_platform().unwrap(), Platform::Linux);
        }
    }

    #[test]
    fn test_gpu_vendor_serialization() {
        let vendor = GpuVendor::Nvidia;
        let serialized = serde_json::to_string(&vendor).unwrap();
        let deserialized: GpuVendor = serde_json::from_str(&serialized).unwrap();
        assert_eq!(vendor, deserialized);
    }

    #[tokio::test]
    async fn test_platform_config_creation() {
        // This test would need mocking for hardware detection
        // For now, just test the structure
        let config = PlatformConfig {
            platform: Platform::Windows,
            hardware: HardwareCapabilities {
                cpu: CpuInfo {
                    model: "Test CPU".to_string(),
                    cores: 8,
                    logical_cores: 8,
                    max_frequency: 3600.0,
                    has_avx: true,
                    has_avx2: true,
                    architecture: "x86_64".to_string(),
                },
                gpu: vec![],
                memory: MemoryInfo {
                    total_gb: 16.0,
                    available_gb: 8.0,
                    speed_mhz: Some(3200.0),
                },
                displays: vec![],
                audio_devices: AudioDeviceInfo {
                    input_devices: vec![],
                    output_devices: vec![],
                    default_input: None,
                    default_output: None,
                },
                storage: StorageInfo {
                    total_space_gb: 500.0,
                    free_space_gb: 250.0,
                    install_drive: "C:".to_string(),
                    temp_directory: "/tmp".to_string(),
                },
            },
            default_overrides: RecordingSettings::default(),
            features: PlatformFeatures {
                supports_windows_capture: true,
                supports_ffmpeg_native: true,
                supports_core_graphics: false,
                supports_hardware_acceleration: false,
                supports_system_tray: true,
                supports_global_hotkeys: true,
                supports_file_associations: true,
                supports_auto_start: true,
                supports_notifications: true,
                supports_api_detection: true,
                supports_window_enumeration: true,
            },
            recommended_settings: RecommendedSettings {
                video: VideoRecommendations {
                    recommended_encoder: EncoderPreference::Auto,
                    recommended_codec: "h265".to_string(),
                    recommended_bitrate_kbps: 10000,
                    recommended_resolution: "1920x1080".to_string(),
                    recommended_frame_rate: "60".to_string(),
                    maximum_recording_hours: 12.0,
                },
                audio: AudioRecommendations {
                    recommended_sample_rate: "48000".to_string(),
                    recommended_bitrate: "192".to_string(),
                    max_channels: 2,
                    enable_microphone_by_default: true,
                    enable_system_audio_by_default: true,
                },
                performance: PerformanceRecommendations {
                    enable_hardware_acceleration: true,
                    recommended_buffer_size_mb: 256,
                    recommended_temp_cleanup_interval_minutes: 30,
                    recommended_concurrent_clips: 3,
                    enable_performance_monitoring: true,
                },
                storage: StorageRecommendations {
                    recommended_clips_directory: "/test".to_string(),
                    minimum_free_space_gb: 10.0,
                    recommended_cleanup_threshold_gb: 50.0,
                    enable_auto_cleanup: false,
                },
            },
        };

        assert_eq!(config.platform, Platform::Windows);
        assert!(config.features.supports_windows_capture);
    }
}