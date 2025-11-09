#![allow(clippy::upper_case_acronyms)]
use super::audio::AudioConfig;
use super::{GameEvent, RecordingStats, RecordingStatus};
use crate::storage::GameMetadata;
use crate::utils::circuit_breaker::{
    CircuitBreaker as ProductionCircuitBreaker, CircuitBreakerConfig,
};
use crate::utils::retry::{retry_with_backoff, RetryConfig};
use anyhow::{Context as AnyhowContext, Result};
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock as TokioRwLock;

// Configuration constants
const SEGMENT_DURATION_SECS: u64 = 10;
const BUFFER_SEGMENTS: usize = 6; // 60 seconds total (6 × 10s)
const MAX_CLIP_DURATION_SECS: f64 = 60.0;
const DEFAULT_BITRATE: u32 = 20_000_000; // 20 Mbps for 1080p60
const DEFAULT_FPS: u32 = 60;

// Error recovery configuration
const FFMPEG_RETRY_CONFIG: RetryConfig = RetryConfig {
    max_attempts: 3,
    initial_delay: Duration::from_millis(500),
    max_delay: Duration::from_secs(5),
    backoff_multiplier: 2.0,
    jitter_factor: 0.1,
};

/// Quality information for UI display
pub struct QualityInfo {
    pub encoder: String,
    pub codec: String,
    pub resolution: String,
    pub fps: u32,
    pub bitrate_mbps: f64,
    pub audio_enabled: bool,
}

/// Windows-specific recording implementation using windows-capture
/// with H.265 hardware-accelerated encoding
///
/// Architecture:
/// 1. Circular buffer with 6 × 10-second segments (60s total)
/// 2. Hardware H.265 encoding (NVENC/QSV/AMF/Software)
/// 3. Automatic segment rotation and cleanup
/// 4. FFmpeg-based clip concatenation for final output
/// 5. Error recovery with circuit breaker pattern
/// 6. Graceful degradation on failures
pub struct WindowsRecorder {
    status: Arc<TokioRwLock<RecordingStatus>>,
    stats: Arc<RwLock<RecordingStats>>,
    output_dir: PathBuf,
    current_game: Arc<TokioRwLock<Option<GameMetadata>>>,
    segment_buffer: Arc<TokioRwLock<SegmentBuffer>>,
    config: RecordingConfig,
    circuit_breaker: Arc<ProductionCircuitBreaker>,
}

#[derive(Clone)]
struct RecordingConfig {
    resolution: (u32, u32),
    fps: u32,
    bitrate: u32,
    codec: VideoCodec,
    audio: AudioConfig,
    hardware_encoder: HardwareEncoder,
}

impl Default for RecordingConfig {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            fps: DEFAULT_FPS,
            bitrate: DEFAULT_BITRATE,
            codec: VideoCodec::HEVC,
            audio: AudioConfig::default(),
            hardware_encoder: HardwareEncoder::detect(),
        }
    }
}

impl RecordingConfig {
    /// Calculate optimal bitrate for given resolution and FPS
    /// Based on YouTube recommendations and H.265 efficiency
    fn calculate_optimal_bitrate(resolution: (u32, u32), fps: u32, codec: VideoCodec) -> u32 {
        let (width, height) = resolution;
        let pixels = width * height;

        // Base bitrate calculation (bits per pixel)
        let base_bpp = match codec {
            VideoCodec::HEVC => 0.10, // H.265 is ~50% more efficient
            VideoCodec::H264 => 0.15, // H.264 baseline
        };

        // FPS scaling factor
        let fps_factor = match fps {
            0..=30 => 1.0,
            31..=60 => 1.3,
            _ => 1.5,
        };

        // Calculate bitrate in bps
        let bitrate = (pixels as f64 * base_bpp * fps as f64 * fps_factor) as u32;

        // Round to nearest 1 Mbps
        (bitrate / 1_000_000) * 1_000_000
    }

    /// Get the appropriate encoder name for the current config
    fn get_encoder_name(&self) -> &'static str {
        match self.codec {
            VideoCodec::HEVC => self.hardware_encoder.hevc_encoder(),
            VideoCodec::H264 => self.hardware_encoder.h264_encoder(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VideoCodec {
    HEVC, // H.265 (preferred for quality/size)
    H264, // H.264 (fallback for compatibility)
}

/// Hardware encoder types
#[derive(Clone, Copy, Debug, PartialEq)]
enum HardwareEncoder {
    NVENC,    // NVIDIA GPU
    QSV,      // Intel Quick Sync
    AMF,      // AMD GPU
    Software, // CPU fallback
}

impl HardwareEncoder {
    /// Get FFmpeg encoder name for H.265/HEVC
    fn hevc_encoder(&self) -> &'static str {
        match self {
            Self::NVENC => "hevc_nvenc",
            Self::QSV => "hevc_qsv",
            Self::AMF => "hevc_amf",
            Self::Software => "libx265",
        }
    }

    /// Get FFmpeg encoder name for H.264
    fn h264_encoder(&self) -> &'static str {
        match self {
            Self::NVENC => "h264_nvenc",
            Self::QSV => "h264_qsv",
            Self::AMF => "h264_amf",
            Self::Software => "libx264",
        }
    }

    /// Get optimal encoding preset for this encoder
    /// Presets vary by encoder type
    fn get_preset(&self) -> &'static str {
        match self {
            Self::NVENC => "p4",     // NVENC: p1 (fastest) to p7 (slowest), p4 is balanced
            Self::QSV => "balanced", // QSV: balanced quality/speed
            Self::AMF => "balanced", // AMF: balanced quality/speed
            Self::Software => "medium", // x265: medium quality/speed
        }
    }

    /// Get additional encoder-specific options
    fn get_encoder_options(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            Self::NVENC => vec![
                ("-rc", "vbr"),          // Variable bitrate for better quality
                ("-rc-lookahead", "20"), // Lookahead for better quality
                ("-spatial-aq", "1"),    // Spatial AQ for better quality
                ("-temporal-aq", "1"),   // Temporal AQ for motion
            ],
            Self::QSV => vec![
                ("-look_ahead", "1"),        // Lookahead enabled
                ("-look_ahead_depth", "40"), // Lookahead depth
            ],
            Self::AMF => vec![
                ("-rc", "vbr_latency"),   // VBR for quality with low latency
                ("-quality", "balanced"), // Balanced quality preset
            ],
            Self::Software => vec![
                ("-x265-params", "aq-mode=3"), // Best adaptive quantization
            ],
        }
    }

    /// Detect available hardware encoder
    /// Tests encoders in priority order and returns first working one
    fn detect() -> Self {
        tracing::info!("Detecting available hardware encoder...");

        // Test in priority order: NVENC > QSV > AMF > Software
        for encoder in [Self::NVENC, Self::QSV, Self::AMF] {
            if Self::test_encoder(encoder.hevc_encoder()) {
                tracing::info!("Hardware encoder detected: {:?}", encoder);
                return encoder;
            }
        }

        tracing::warn!("No hardware encoder available, falling back to software encoding");
        Self::Software
    }

    /// Test if an encoder is available by running a quick FFmpeg test
    fn test_encoder(encoder_name: &str) -> bool {
        let result = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "nullsrc=s=256x256:d=0.1",
                "-c:v",
                encoder_name,
                "-f",
                "null",
                "-",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        result.is_ok() && result.unwrap().success()
    }
}

/// Manages circular buffer of video segments
struct SegmentBuffer {
    segments: VecDeque<PathBuf>,
    max_segments: usize,
    current_segment: usize,
    temp_dir: PathBuf,
}

impl SegmentBuffer {
    fn new(temp_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            segments: VecDeque::with_capacity(BUFFER_SEGMENTS),
            max_segments: BUFFER_SEGMENTS,
            current_segment: 0,
            temp_dir,
        })
    }

    /// Add a new segment to the circular buffer
    fn add_segment(&mut self, segment_path: PathBuf) -> Result<()> {
        // Remove oldest segment if at capacity
        if self.segments.len() >= self.max_segments {
            if let Some(old_path) = self.segments.pop_front() {
                if old_path.exists() {
                    std::fs::remove_file(&old_path)
                        .with_context(|| format!("Failed to remove old segment: {:?}", old_path))?;
                    tracing::debug!("Removed old segment: {:?}", old_path);
                }
            }
        }

        self.segments.push_back(segment_path.clone());
        self.current_segment += 1;

        tracing::debug!(
            "Added segment: {:?} (total: {}/{})",
            segment_path,
            self.segments.len(),
            self.max_segments
        );

        Ok(())
    }

    /// Get all segments in chronological order
    fn get_all_segments(&self) -> Vec<PathBuf> {
        self.segments.iter().cloned().collect()
    }

    /// Generate path for next segment
    fn next_segment_path(&self) -> PathBuf {
        self.temp_dir
            .join(format!("segment_{:04}.mp4", self.current_segment))
    }

    /// Clear all segments
    fn clear(&mut self) -> Result<()> {
        for segment in self.segments.drain(..) {
            if segment.exists() {
                std::fs::remove_file(&segment)?;
            }
        }
        self.current_segment = 0;
        Ok(())
    }
}

/// FFmpeg-based recording handler for segment capture
#[cfg(target_os = "windows")]
struct SegmentRecorder {
    segment_buffer: Arc<TokioRwLock<SegmentBuffer>>,
    status: Arc<TokioRwLock<RecordingStatus>>,
    config: RecordingConfig,
    ffmpeg_process: Option<Child>,
    current_segment_start: Instant,
    current_segment_path: PathBuf,
    is_recording: Arc<parking_lot::Mutex<bool>>,
    circuit_breaker: Arc<ProductionCircuitBreaker>,
}

#[cfg(target_os = "windows")]
impl SegmentRecorder {
    fn new(
        segment_buffer: Arc<TokioRwLock<SegmentBuffer>>,
        status: Arc<TokioRwLock<RecordingStatus>>,
        config: RecordingConfig,
        circuit_breaker: Arc<ProductionCircuitBreaker>,
    ) -> Self {
        Self {
            segment_buffer,
            status,
            config,
            ffmpeg_process: None,
            current_segment_start: Instant::now(),
            current_segment_path: PathBuf::new(),
            is_recording: Arc::new(parking_lot::Mutex::new(false)),
            circuit_breaker,
        }
    }

    /// Start FFmpeg recording for a new segment
    async fn start_segment_recording(&mut self) -> Result<()> {
        // Get next segment path
        let buffer = self.segment_buffer.read().await;
        self.current_segment_path = buffer.next_segment_path();
        drop(buffer);

        // Get encoder name based on detected hardware
        let video_encoder = self.config.get_encoder_name();

        // Use configured bitrate (calculated based on resolution in settings)
        let bitrate = format!("{}k", self.config.bitrate / 1000);

        tracing::info!(
            "Starting FFmpeg segment recording: {:?} (duration: {}s, bitrate: {}, encoder: {})",
            self.current_segment_path,
            SEGMENT_DURATION_SECS,
            bitrate,
            video_encoder
        );

        // Build audio arguments from AudioConfig
        let (audio_inputs, audio_filters, audio_maps, audio_codec) =
            self.config.audio.build_ffmpeg_args();

        // Build complete FFmpeg command
        let mut ffmpeg_args = vec![
            "-f".to_string(),
            "gdigrab".to_string(), // Windows GDI screen capture
            "-framerate".to_string(),
            self.config.fps.to_string(),
            "-i".to_string(),
            "desktop".to_string(), // Capture entire desktop
        ];

        // Add audio inputs (microphone and/or system audio)
        ffmpeg_args.extend(audio_inputs);

        // Video encoding args
        ffmpeg_args.extend(vec![
            "-c:v".to_string(),
            video_encoder.to_string(), // Hardware encoder
            "-preset".to_string(),
            self.config.hardware_encoder.get_preset().to_string(), // Encoder-specific preset
            "-b:v".to_string(),
            bitrate.clone(), // Bitrate
            "-maxrate".to_string(),
            bitrate.clone(), // Max bitrate
            "-bufsize".to_string(),
            format!("{}k", self.config.bitrate * 2 / 1000), // Buffer size
            "-pix_fmt".to_string(),
            "yuv420p".to_string(), // Pixel format
        ]);

        // Add encoder-specific optimization options
        for (key, value) in self.config.hardware_encoder.get_encoder_options() {
            ffmpeg_args.extend(vec![key.to_string(), value.to_string()]);
        }

        // Add audio filter_complex if audio is enabled
        if !audio_filters.is_empty() {
            ffmpeg_args.extend(audio_filters);
        }

        // Add stream mapping
        if !audio_maps.is_empty() {
            // Audio enabled: map both video and audio
            ffmpeg_args.extend(audio_maps);
        } else {
            // No audio: map video only
            ffmpeg_args.extend(vec!["-map".to_string(), "0:v".to_string()]);
        }

        // Add audio codec args if audio is enabled
        if !audio_codec.is_empty() {
            ffmpeg_args.extend(audio_codec);
        }

        // Duration and output
        ffmpeg_args.extend(vec![
            "-t".to_string(),
            SEGMENT_DURATION_SECS.to_string(), // Duration
            "-y".to_string(),                  // Overwrite output file
            self.current_segment_path.to_str().unwrap().to_string(),
        ]);

        // Start FFmpeg process with retry logic and circuit breaker protection
        // Clone necessary data for closure
        let ffmpeg_args_clone = ffmpeg_args.clone();
        let circuit_breaker = Arc::clone(&self.circuit_breaker);

        let child = circuit_breaker
            .call(|| async {
                retry_with_backoff(FFMPEG_RETRY_CONFIG, "FFmpeg process startup", || async {
                    // Spawn FFmpeg process (sync operation wrapped in async)
                    Command::new("ffmpeg")
                        .args(&ffmpeg_args_clone)
                        .stdout(Stdio::null())
                        .stderr(Stdio::piped())
                        .spawn()
                        .context("Failed to start FFmpeg process")
                })
                .await
            })
            .await?;

        self.ffmpeg_process = Some(child);
        self.current_segment_start = Instant::now();
        *self.is_recording.lock() = true;

        tracing::info!(
            "FFmpeg segment recording started successfully: {:?}",
            self.current_segment_path
        );

        Ok(())
    }

    /// Stop current segment recording and add to buffer
    async fn stop_segment_recording(&mut self) -> Result<()> {
        if let Some(mut process) = self.ffmpeg_process.take() {
            tracing::debug!("Stopping FFmpeg segment: {:?}", self.current_segment_path);

            // Try graceful termination first
            match process.try_wait() {
                Ok(Some(status)) => {
                    tracing::debug!("FFmpeg process already exited with status: {}", status);
                }
                Ok(None) => {
                    // Process still running, kill it
                    tracing::debug!("Terminating FFmpeg process");
                    if let Err(e) = process.kill() {
                        tracing::warn!("Failed to kill FFmpeg process: {}", e);
                    }
                    // Wait for process to terminate
                    if let Err(e) = process.wait() {
                        tracing::warn!("Failed to wait for FFmpeg process: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Error checking FFmpeg process status: {}", e);
                }
            }

            *self.is_recording.lock() = false;

            // Verify segment file was created and has content
            if self.current_segment_path.exists() {
                let file_size = std::fs::metadata(&self.current_segment_path)
                    .context("Failed to get segment file metadata")?
                    .len();

                if file_size > 0 {
                    // Add completed segment to buffer
                    let segment_path = self.current_segment_path.clone();

                    let mut buffer = self.segment_buffer.write().await;
                    if let Err(e) = buffer.add_segment(segment_path.clone()) {
                        tracing::error!("Failed to add segment to buffer: {}", e);
                    } else {
                        tracing::info!(
                            "Segment added to buffer: {:?} (size: {} bytes)",
                            segment_path,
                            file_size
                        );
                    }
                } else {
                    tracing::warn!(
                        "Segment file is empty, not adding to buffer: {:?}",
                        self.current_segment_path
                    );
                }
            } else {
                tracing::warn!("Segment file not found: {:?}", self.current_segment_path);
            }
        }

        Ok(())
    }

    /// Rotate to a new segment
    async fn rotate_segment(&mut self) -> Result<()> {
        // Stop current recording
        self.stop_segment_recording().await?;

        // Start new segment
        self.start_segment_recording().await?;

        Ok(())
    }

    /// Check if recording should rotate based on duration
    fn should_rotate(&self) -> bool {
        self.current_segment_start.elapsed() >= Duration::from_secs(SEGMENT_DURATION_SECS)
    }
}

impl WindowsRecorder {
    pub fn new(output_dir: PathBuf) -> Result<Self> {
        let temp_dir = output_dir.join("temp_segments");
        std::fs::create_dir_all(&temp_dir)?;

        // Initialize production circuit breaker for critical FFmpeg operations
        let circuit_breaker = Arc::new(ProductionCircuitBreaker::new(
            "FFmpeg Recording",
            CircuitBreakerConfig::aggressive(), // Critical service requires aggressive failure detection
        ));

        Ok(Self {
            status: Arc::new(TokioRwLock::new(RecordingStatus::Idle)),
            stats: Arc::new(RwLock::new(RecordingStats::default())),
            output_dir,
            current_game: Arc::new(TokioRwLock::new(None)),
            segment_buffer: Arc::new(TokioRwLock::new(SegmentBuffer::new(temp_dir)?)),
            config: RecordingConfig::default(),
            circuit_breaker,
        })
    }

    // Note: Circuit breaker state management is now handled automatically
    // via the ProductionCircuitBreaker::call() method in critical operations.
    // Manual success/failure tracking and state checks are no longer needed.

    /// Update audio configuration from settings
    /// Note: Changes will take effect on next segment recording (after rotation)
    pub fn update_audio_config(&mut self, audio_settings: &crate::settings::models::AudioSettings) {
        use crate::settings::models::{AudioBitrate, SampleRate};

        // Convert AudioSettings to AudioConfig
        let sample_rate = match audio_settings.sample_rate {
            SampleRate::Hz44100 => 44100,
            SampleRate::Hz48000 => 48000,
        };

        let bitrate = match audio_settings.bitrate {
            AudioBitrate::Kbps128 => 128,
            AudioBitrate::Kbps192 => 192,
            AudioBitrate::Kbps256 => 256,
            AudioBitrate::Kbps320 => 320,
        };

        self.config.audio = AudioConfig {
            record_microphone: audio_settings.record_microphone,
            microphone_device: audio_settings.microphone_device.clone(),
            microphone_volume: audio_settings.microphone_volume,
            record_system_audio: audio_settings.record_system_audio,
            system_audio_device: audio_settings.system_audio_device.clone(),
            system_audio_volume: audio_settings.system_audio_volume,
            sample_rate,
            bitrate,
        };

        tracing::info!(
            "Audio config updated: mic={}, sys_audio={}, sample_rate={}Hz, bitrate={}kbps",
            self.config.audio.record_microphone,
            self.config.audio.record_system_audio,
            sample_rate,
            bitrate
        );
    }

    /// Start the replay buffer (continuous recording with FFmpeg)
    /// Circuit breaker protection is applied at FFmpeg spawn level
    #[cfg(target_os = "windows")]
    pub async fn start_replay_buffer(&self) -> Result<()> {
        let mut status = self.status.write().await;

        if *status != RecordingStatus::Idle {
            anyhow::bail!("Replay buffer already running");
        }

        *status = RecordingStatus::Buffering;
        drop(status);

        tracing::info!(
            "Starting FFmpeg-based replay buffer with {}s window",
            SEGMENT_DURATION_SECS * BUFFER_SEGMENTS as u64
        );

        // Create segment recorder with circuit breaker
        let mut recorder = SegmentRecorder::new(
            Arc::clone(&self.segment_buffer),
            Arc::clone(&self.status),
            self.config.clone(),
            Arc::clone(&self.circuit_breaker),
        );

        // Start initial segment (circuit breaker protection applied at FFmpeg spawn)
        let result = recorder.start_segment_recording().await;

        match &result {
            Ok(_) => {
                tracing::info!("Initial segment recording started successfully");
            }
            Err(e) => {
                tracing::error!("Failed to start initial segment: {}", e);

                // Update status to error
                let mut status = self.status.write().await;
                *status = RecordingStatus::Error;

                return Err(anyhow::anyhow!("Failed to start recording: {}", e));
            }
        }

        // Spawn background task to handle segment rotation
        let is_recording = Arc::clone(&recorder.is_recording);
        let status_clone = Arc::clone(&self.status);

        tokio::spawn(async move {
            tracing::info!("Segment rotation task started");

            loop {
                // Sleep for 1 second between checks
                tokio::time::sleep(Duration::from_secs(1)).await;

                // Check if recording should stop (status changed to Idle or Error)
                let current_status = {
                    let status = status_clone.read().await;
                    *status
                };

                if current_status == RecordingStatus::Idle
                    || current_status == RecordingStatus::Error
                {
                    tracing::info!("Recording stopped (status: {:?}), stopping segments and exiting rotation task", current_status);

                    // Stop the current recording segment
                    if let Err(e) = recorder.stop_segment_recording().await {
                        tracing::error!("Failed to stop segment recording: {}", e);
                    }

                    *is_recording.lock() = false;
                    break;
                }

                // Check if segment should rotate
                if recorder.should_rotate() {
                    tracing::info!("Rotating segment");

                    match recorder.rotate_segment().await {
                        Ok(_) => {
                            tracing::debug!("Segment rotation successful");
                        }
                        Err(e) => {
                            tracing::error!("Segment rotation failed: {}", e);

                            // Update status to error
                            let mut status = status_clone.write().await;
                            *status = RecordingStatus::Error;

                            // Stop recording
                            *is_recording.lock() = false;
                            break;
                        }
                    }
                }
            }

            tracing::info!("Segment rotation task ended");
        });

        Ok(())
    }

    /// Start the replay buffer (stub for non-Windows platforms)
    #[cfg(not(target_os = "windows"))]
    pub async fn start_replay_buffer(&self) -> Result<()> {
        anyhow::bail!("Recording is only supported on Windows")
    }

    /// Stop the replay buffer
    pub async fn stop_replay_buffer(&self) -> Result<()> {
        let mut status = self.status.write().await;

        if *status == RecordingStatus::Idle {
            return Ok(());
        }

        *status = RecordingStatus::Idle;
        drop(status);

        // Clear segment buffer
        let mut buffer = self.segment_buffer.write().await;
        buffer.clear()?;

        tracing::info!("Replay buffer stopped and cleared");

        Ok(())
    }

    /// Save a clip from the replay buffer
    ///
    /// This concatenates the available segments into a single output file
    pub async fn save_clip(
        &self,
        _event: &GameEvent,
        clip_id: String,
        priority: u8,
        duration_secs: f64,
    ) -> Result<PathBuf> {
        let duration = duration_secs.min(MAX_CLIP_DURATION_SECS);

        // Ensure we're buffering or recording
        let status = self.status.read().await;
        match *status {
            RecordingStatus::Idle => {
                anyhow::bail!("Cannot save clip: replay buffer not active");
            }
            RecordingStatus::Error => {
                anyhow::bail!("Cannot save clip: recording in error state");
            }
            _ => {} // Buffering, Recording, or Processing is OK
        }
        drop(status);

        // Generate output filename
        let game = self.current_game.read().await;
        let game_id = game
            .as_ref()
            .map(|g| g.game_id.clone())
            .unwrap_or_else(|| "unknown".to_string());
        drop(game);

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let clip_filename = format!("{}_{}_p{}_{}.mp4", game_id, timestamp, priority, clip_id);
        let output_path = self.output_dir.join(&clip_filename);

        tracing::info!(
            "Saving clip: {} (duration: {:.1}s, priority: {})",
            clip_filename,
            duration,
            priority
        );

        // Set status to processing
        *self.status.write().await = RecordingStatus::Processing;

        // Get all available segments
        let buffer = self.segment_buffer.read().await;
        let segments = buffer.get_all_segments();
        drop(buffer);

        if segments.is_empty() {
            anyhow::bail!("No segments available to save");
        }

        // Concatenate segments using FFmpeg
        self.concat_segments(&segments, &output_path, duration)
            .await?;

        // Update stats
        {
            let mut stats = self.stats.write();
            stats.clips_created += 1;
        }

        // Restore status
        *self.status.write().await = RecordingStatus::Buffering;

        tracing::info!("Clip saved successfully: {:?}", output_path);

        Ok(output_path)
    }

    /// Concatenate video segments using FFmpeg
    ///
    /// Uses FFmpeg's concat demuxer for fast, lossless concatenation
    async fn concat_segments(
        &self,
        segments: &[PathBuf],
        output_path: &PathBuf,
        duration_secs: f64,
    ) -> Result<()> {
        use std::process::Command;

        // Create concat file for FFmpeg
        let concat_file = self.output_dir.join("concat_list.txt");
        let mut content = String::new();

        for segment in segments {
            if segment.exists() {
                content.push_str(&format!("file '{}'\n", segment.display()));
            }
        }

        std::fs::write(&concat_file, content).context("Failed to write concat list")?;

        tracing::debug!("Concatenating {} segments", segments.len());

        // Run FFmpeg concat with retry logic for transient failures
        let concat_file_clone = concat_file.clone();
        let output_path_clone = output_path.clone();
        let duration_str = duration_secs.to_string();

        let status = retry_with_backoff(FFMPEG_RETRY_CONFIG, "FFmpeg concatenation", || async {
            Command::new("ffmpeg")
                .args([
                    "-f",
                    "concat",
                    "-safe",
                    "0",
                    "-i",
                    concat_file_clone.to_str().unwrap(),
                    "-t",
                    &duration_str, // Limit duration
                    "-c",
                    "copy", // Copy without re-encoding
                    "-y",   // Overwrite output
                    output_path_clone.to_str().unwrap(),
                ])
                .status()
                .context("Failed to execute FFmpeg")
        })
        .await?;

        // Cleanup concat file
        let _ = std::fs::remove_file(&concat_file);

        if !status.success() {
            anyhow::bail!("FFmpeg concatenation failed with status: {}", status);
        }

        tracing::info!(
            "Successfully concatenated {} segments to {:?}",
            segments.len(),
            output_path
        );

        Ok(())
    }

    pub async fn get_state(&self) -> RecordingStatus {
        *self.status.read().await
    }

    pub async fn get_stats(&self) -> RecordingStats {
        self.stats.read().clone()
    }

    pub async fn set_current_game(&self, game: Option<GameMetadata>) {
        let mut current = self.current_game.write().await;
        *current = game;
    }

    /// Get quality information for UI display
    pub fn get_quality_info(&self) -> QualityInfo {
        let encoder_name = format!("{:?}", self.config.hardware_encoder);
        let codec_name = match self.config.codec {
            VideoCodec::HEVC => "H.265/HEVC",
            VideoCodec::H264 => "H.264/AVC",
        };

        QualityInfo {
            encoder: encoder_name,
            codec: codec_name.to_string(),
            resolution: format!("{}x{}", self.config.resolution.0, self.config.resolution.1),
            fps: self.config.fps,
            bitrate_mbps: self.config.bitrate as f64 / 1_000_000.0,
            audio_enabled: self.config.audio.is_enabled(),
        }
    }
}

// Implement Clone manually (Arc types are Clone)
impl Clone for WindowsRecorder {
    fn clone(&self) -> Self {
        Self {
            status: Arc::clone(&self.status),
            stats: Arc::clone(&self.stats),
            output_dir: self.output_dir.clone(),
            current_game: Arc::clone(&self.current_game),
            segment_buffer: Arc::clone(&self.segment_buffer),
            config: self.config.clone(),
            circuit_breaker: Arc::clone(&self.circuit_breaker),
        }
    }
}

// Thread safety markers
unsafe impl Send for WindowsRecorder {}
unsafe impl Sync for WindowsRecorder {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_recorder_creation() {
        let temp_dir = TempDir::new().unwrap();
        let recorder = WindowsRecorder::new(temp_dir.path().to_path_buf()).unwrap();

        let status = recorder.get_state().await;
        assert_eq!(status, RecordingStatus::Idle);
    }

    #[tokio::test]
    #[ignore] // Requires FFmpeg to be installed
    async fn test_state_transitions() {
        let temp_dir = TempDir::new().unwrap();
        let recorder = WindowsRecorder::new(temp_dir.path().to_path_buf()).unwrap();

        assert_eq!(recorder.get_state().await, RecordingStatus::Idle);

        recorder.start_replay_buffer().await.unwrap();
        assert_eq!(recorder.get_state().await, RecordingStatus::Buffering);

        recorder.stop_replay_buffer().await.unwrap();
        assert_eq!(recorder.get_state().await, RecordingStatus::Idle);
    }

    #[tokio::test]
    async fn test_segment_buffer() {
        let temp_dir = TempDir::new().unwrap();
        let segment_dir = temp_dir.path().join("segments");

        let mut buffer = SegmentBuffer::new(segment_dir.clone()).unwrap();

        // Add segments up to capacity
        for _ in 0..BUFFER_SEGMENTS {
            let path = buffer.next_segment_path();
            std::fs::File::create(&path).unwrap();
            buffer.add_segment(path).unwrap();
        }

        assert_eq!(buffer.segments.len(), BUFFER_SEGMENTS);

        // Add one more - should remove oldest
        let path = buffer.next_segment_path();
        std::fs::File::create(&path).unwrap();
        buffer.add_segment(path).unwrap();

        assert_eq!(buffer.segments.len(), BUFFER_SEGMENTS);

        // Clear all
        buffer.clear().unwrap();
        assert_eq!(buffer.segments.len(), 0);
    }

    #[tokio::test]
    async fn test_save_clip_requires_active_buffer() {
        let temp_dir = TempDir::new().unwrap();
        let recorder = WindowsRecorder::new(temp_dir.path().to_path_buf()).unwrap();

        let event = GameEvent {
            event_id: 1,
            event_name: "TestEvent".to_string(),
            event_time: 0.0,
            killer_name: None,
            victim_name: None,
            assisters: vec![],
            priority: 3,
            timestamp: Instant::now(),
        };

        // Should fail - buffer not active
        let result = recorder
            .save_clip(&event, "test".to_string(), 3, 30.0)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not active"));
    }
}
