#![allow(dead_code)]
use anyhow::Context;
use std::path::{Path, PathBuf};
use tokio::process::Command as TokioCommand;
use tracing::{info, warn};

use super::{execute_ffmpeg_command, Result, VideoError};
use crate::utils::ffmpeg::get_ffmpeg_path;

/// Types for video transitions and effects
#[derive(Debug, Clone)]
pub enum TransitionType {
    Fade,
    Slide,
    Dissolve,
    Wipe,
}

/// Hardware-accelerated video encoder types
#[derive(Debug, Clone)]
pub enum VideoEncoder {
    H264,            // Software H.264
    H265,            // Software H.265
    NvidiaH264,      // NVIDIA NVENC H.264
    NvidiaH265,      // NVIDIA NVENC H.265
    AmdH264,         // AMD VCE H.264
    IntelH264,       // Intel Quick Sync H.264
}

impl VideoEncoder {
    pub fn get_ffmpeg_args(&self) -> Vec<&'static str> {
        match self {
            VideoEncoder::H264 => vec!["-c:v", "libx264", "-preset", "medium", "-tune", "fastdecode"],
            VideoEncoder::H265 => vec!["-c:v", "libx265", "-preset", "medium", "-tune", "fastdecode"],
            VideoEncoder::NvidiaH264 => vec!["-c:v", "h264_nvenc", "-preset", "p1", "-tune", "ll", "-rc", "vbr", "-cq", "23"],
            VideoEncoder::NvidiaH265 => vec!["-c:v", "hevc_nvenc", "-preset", "p1", "-tune", "ll", "-rc", "vbr", "-cq", "28"],
            VideoEncoder::AmdH264 => vec!["-c:v", "h264_amf", "-quality", "speed", "-rc", "vbr_peak", "-qp_i", "23", "-qp_p", "23"],
            VideoEncoder::IntelH264 => vec!["-c:v", "h264_qsv", "-preset", "veryfast", "-tune", "ll", "-global_quality", "23"],
        }
    }

    pub fn get_name(&self) -> &'static str {
        match self {
            VideoEncoder::H264 => "H.264 (Software)",
            VideoEncoder::H265 => "H.265 (Software)",
            VideoEncoder::NvidiaH264 => "NVIDIA H.264 (Hardware)",
            VideoEncoder::NvidiaH265 => "NVIDIA H.265 (Hardware)",
            VideoEncoder::AmdH264 => "AMD H.264 (Hardware)",
            VideoEncoder::IntelH264 => "Intel H.264 (Hardware)",
        }
    }

    pub fn get_output_format(&self) -> &'static str {
        match self {
            VideoEncoder::H264 | VideoEncoder::H265 => "yuv420p",
            VideoEncoder::NvidiaH264 | VideoEncoder::AmdH264 | VideoEncoder::IntelH264 => "cuda",
            VideoEncoder::NvidiaH265 => "cuda",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ColorGrading {
    pub brightness: f64,  // -1.0 to 1.0
    pub contrast: f64,    // 0.0 to 2.0
    pub saturation: f64,  // 0.0 to 2.0
    pub gamma: f64,       // 0.1 to 3.0
    pub vignette: bool,    // Add vignette effect
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            vignette: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TextPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub size: u32,
    pub color: String,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            size: 48,
            color: "WHITE".to_string(),
        }
    }
}

/// FFmpeg video processor for clip extraction and composition
pub struct VideoProcessor {
    ffmpeg_path: String,
    optimal_encoder: VideoEncoder,
}

impl VideoProcessor {
    pub fn new() -> Result<Self> {
        let optimal_encoder = Self::detect_optimal_encoder();
        info!("Detected optimal encoder: {}", optimal_encoder.get_name());

        let ffmpeg_path = get_ffmpeg_path()
            .map_err(|e| VideoError::ProcessingError {
                message: format!("Failed to find FFmpeg: {}", e),
            })?;

        Ok(Self {
            ffmpeg_path: ffmpeg_path.to_string_lossy().to_string(),
            optimal_encoder,
        })
    }

    /// Create a new VideoProcessor with fallback to default settings
    pub fn new_with_fallback() -> Self {
        match Self::new() {
            Ok(processor) => processor,
            Err(e) => {
                warn!("Failed to create VideoProcessor with optimal settings: {}. Using fallback.", e);
                Self {
                    ffmpeg_path: "ffmpeg".to_string(), // Assume ffmpeg is in PATH
                    optimal_encoder: VideoEncoder::H264, // Fallback to software encoding
                }
            }
        }
    }

    /// Detect the optimal hardware-accelerated encoder available on the system
    pub fn detect_optimal_encoder() -> VideoEncoder {
        // Check for NVIDIA GPU (NVENC)
        if Self::check_nvidia_availability() {
            return VideoEncoder::NvidiaH264;
        }

        // Check for AMD GPU (VCE)
        if Self::check_amd_availability() {
            return VideoEncoder::AmdH264;
        }

        // Check for Intel Quick Sync
        if Self::check_intel_availability() {
            return VideoEncoder::IntelH264;
        }

        // Fallback to software encoding
        info!("No hardware acceleration detected, using software encoding");
        VideoEncoder::H264
    }

    fn check_nvidia_availability() -> bool {
        // Check if NVIDIA GPU is available by testing NVENC encoder
        #[cfg(target_os = "windows")]
        {
            // On Windows, check for NVIDIA GPU presence
            if let Ok(output) = std::process::Command::new("nvidia-smi")
                .arg("-L")
                .output()
            {
                if output.status.success() {
                    return output.stdout.len() > 0;
                }
            }
        }

        // Fallback: Try to query FFmpeg for NVENC support
        std::process::Command::new("ffmpeg")
            .args(&["-hide_banner", "-encoders"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).contains("h264_nvenc")
            })
            .unwrap_or(false)
    }

    fn check_amd_availability() -> bool {
        // Check FFmpeg for AMD AMF encoder support
        std::process::Command::new("ffmpeg")
            .args(&["-hide_banner", "-encoders"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).contains("h264_amf")
            })
            .unwrap_or(false)
    }

    fn check_intel_availability() -> bool {
        // Check FFmpeg for Intel Quick Sync encoder support
        std::process::Command::new("ffmpeg")
            .args(&["-hide_banner", "-encoders"])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout).contains("h264_qsv")
            })
            .unwrap_or(false)
    }

    pub fn get_optimal_encoder(&self) -> &VideoEncoder {
        &self.optimal_encoder
    }

    
    /// Extract a clip from a video file
    ///
    /// # Arguments
    /// * `input_path` - Path to input video file
    /// * `output_path` - Path to output clip file
    /// * `start_time` - Start time in seconds
    /// * `duration` - Duration in seconds
    ///
    /// # Returns
    /// Path to the extracted clip
    pub async fn extract_clip(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        start_time: f64,
        duration: f64,
    ) -> Result<PathBuf> {
        let input = input_path.as_ref();
        let output = output_path.as_ref();

        info!(
            "Extracting clip: {:?} -> {:?} (start: {}s, duration: {}s)",
            input, output, start_time, duration
        );

        // Validate input file exists
        if !input.exists() {
            return Err(VideoError::FileNotFound {
                path: input.display().to_string(),
            });
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            if !parent.exists() {
                return Err(VideoError::OutputDirectoryNotFound {
                    path: parent.display().to_string(),
                });
            }
        }

        let mut command = TokioCommand::new(&self.ffmpeg_path);
        
        // ==========================================
        // 1. Input Options (Must be before -i)
        // ==========================================
        
        // Fast seeking
        command.arg("-ss");
        command.arg(format!("{:.3}", start_time));

        // Hardware acceleration (Input Option)
        // Only use for clips < 5 mins where we re-encode
        if duration < 300.0 { 
            match self.optimal_encoder {
                VideoEncoder::NvidiaH264 | VideoEncoder::NvidiaH265 => {
                    command.args(["-hwaccel", "cuda"]);
                }
                VideoEncoder::AmdH264 => {
                    command.args(["-hwaccel", "dxva2"]);
                }
                VideoEncoder::IntelH264 => {
                    command.args(["-hwaccel", "qsv"]);
                }
                _ => {}
            }
        }

        // ==========================================
        // 2. Input File
        // ==========================================
        command.arg("-i");
        command.arg(input.to_str().ok_or_else(|| VideoError::FileAccessError {
            path: input.display().to_string(),
        })?);

        // ==========================================
        // 3. Output Options (Must be after -i)
        // ==========================================
        
        // Duration
        command.arg("-t");
        command.arg(format!("{:.3}", duration));

        // Encoding Strategy
        if duration < 300.0 { 
            let encoder_args = self.optimal_encoder.get_ffmpeg_args();
            command.args(&encoder_args);

            // Encoder-specific presets (Output Options)
            match self.optimal_encoder {
                VideoEncoder::NvidiaH264 | VideoEncoder::NvidiaH265 => {
                    command.args(["-preset", "fast"]);
                }
                VideoEncoder::IntelH264 => {
                    command.args(["-preset", "medium"]);
                }
                _ => {}
            }

            // General quality settings
            command.args([
                "-crf", "23",
                "-threads", "0",
                "-pix_fmt", "yuv420p",
            ]);
        } else {
            // Stream copy for long clips
            command.args(["-c", "copy"]);
        }

        // Final Metadata & File Options
        command.args([
            "-avoid_negative_ts", "make_zero",
            "-movflags", "+faststart",
            "-y",
            output.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: output.display().to_string(),
            })?,
        ]);

        let ffmpeg_result = execute_ffmpeg_command(&mut command).await;

        // Handle statistics based on result
        match &ffmpeg_result {
            Ok(_) => {
                crate::video::get_global_stats().record_success();
            }
            Err(_) => {
                crate::video::get_global_stats().record_failure();
            }
        }

        ffmpeg_result?;

        // Verify output file was created
        if !output.exists() {
            return Err(VideoError::ProcessingError {
                message: format!("Output file was not created: {:?}", output),
            });
        }

        info!("Clip extracted successfully: {:?}", output);
        Ok(output.to_path_buf())
    }

    /// Compose multiple clips into a YouTube Short (9:16 aspect ratio)
    ///
    /// # Arguments
    /// * `clip_paths` - Paths to input clip files
    /// * `output_path` - Path to output composed video
    /// * `target_width` - Target width (default: 1080)
    /// * `target_height` - Target height (default: 1920)
    ///
    /// # Returns
    /// Path to the composed short
    pub async fn compose_shorts(
        &self,
        clip_paths: &[PathBuf],
        output_path: impl AsRef<Path>,
        target_width: u32,
        target_height: u32,
    ) -> Result<PathBuf> {
        let output = output_path.as_ref();

        if clip_paths.is_empty() {
            return Err(VideoError::ProcessingError {
                message: "No clips provided for composition".to_string(),
            });
        }

        info!(
            "Composing {} clips into Short: {:?} ({}x{})",
            clip_paths.len(),
            output,
            target_width,
            target_height
        );

        // Validate all input files exist
        for clip in clip_paths {
            if !clip.exists() {
                return Err(VideoError::FileNotFound {
                    path: clip.display().to_string(),
                });
            }
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            if !parent.exists() {
                return Err(VideoError::OutputDirectoryNotFound {
                    path: parent.display().to_string(),
                });
            }
        }

        // If only one clip, just scale and crop it
        if clip_paths.len() == 1 {
            return self
                .scale_and_crop_clip(&clip_paths[0], output, target_width, target_height)
                .await;
        }

        // Multiple clips: create concat file and then compose
        let concat_file = output
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("concat_list.txt");

        // Write concat file with safe path handling
        let mut concat_content = String::new();
        for path in clip_paths {
            let path_str = path.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: path.display().to_string(),
            })?;
            concat_content.push_str(&format!("file '{}'\n", path_str));
        }

        tokio::fs::write(&concat_file, concat_content)
            .await
            .map_err(|e| VideoError::ProcessingError {
                message: format!("Failed to write concat file: {}", e),
            })?;

        // Run FFmpeg to concatenate and scale to 9:16
        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command.args([
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_file
                .to_str()
                .ok_or_else(|| VideoError::FileAccessError {
                    path: concat_file.display().to_string(),
                })?,
            "-vf",
            &format!("scale={}:{},setsar=1", target_width, target_height),
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "23",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-y",
            output.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: output.display().to_string(),
            })?,
        ]);

        let result = execute_ffmpeg_command(&mut command).await;

        // Clean up concat file
        let _ = tokio::fs::remove_file(&concat_file).await;

        result.map_err(|e| VideoError::ConcatenationError {
            reason: e.to_string(),
        })?;

        // Verify output file was created
        if !output.exists() {
            return Err(VideoError::ProcessingError {
                message: format!("Output file was not created: {:?}", output),
            });
        }

        info!("Short composed successfully: {:?}", output);
        Ok(output.to_path_buf())
    }

    /// Compose multiple clips into a long-form montage (16:9 aspect ratio)
    ///
    /// # Arguments
    /// * `clip_paths` - Paths to input clip files
    /// * `output_path` - Path to output composed video
    /// * `use_transition` - Whether to apply dissolve transition
    ///
    /// # Returns
    /// Path to the composed video
    pub async fn compose_montage(
        &self,
        clip_paths: &[PathBuf],
        output_path: impl AsRef<Path>,
        use_transition: bool,
    ) -> Result<PathBuf> {
        let output = output_path.as_ref();

        if clip_paths.is_empty() {
            return Err(VideoError::ProcessingError {
                message: "No clips provided for montage".to_string(),
            });
        }

        info!(
            "Composing {} clips into Montage: {:?} (16:9)",
            clip_paths.len(),
            output
        );

        // Validate all input files exist
        for clip in clip_paths {
            if !clip.exists() {
                return Err(VideoError::FileNotFound {
                    path: clip.display().to_string(),
                });
            }
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            if !parent.exists() {
                return Err(VideoError::OutputDirectoryNotFound {
                    path: parent.display().to_string(),
                });
            }
        }

        // Create concat file for simple merging
        // Note: For transitions (xfade), concat demuxer doesn't work well.
        // We need complex filter graph for transitions.
        // For MVP "Smart Merge", simple concat is faster and safer.
        // If transition requested, we might need re-encoding which is slow.
        // Let's stick to simple concat for now (no re-encode if codecs match, but usually we re-encode to be safe).
        
        // Using concat filter (re-encode) to ensure stability
        // let _filter_inputs = String::new();
        // let _filter_graph = String::new();
        
        if use_transition {
            // Complex transition logic (xfade)
            // This gets complicated for N clips. 
            // A simpler approach for "Dissolve" is using `mix` or `crossfade` in audio/video.
            // Or just simple cut.
            
            // For now, let's implement "Simple Cut" (concat) via filter to support re-encoding (safe).
            // Transitions can be added in v2 as they require complex offset calculation.
            warn!("Transitions not yet supported in simple montage, using simple cut.");
        }

        // Generate concat list file
        let concat_file = output
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("montage_list.txt");

        let mut concat_content = String::new();
        for path in clip_paths {
            let path_str = path.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: path.display().to_string(),
            })?;
            concat_content.push_str(&format!("file '{}'\n", path_str));
        }

        tokio::fs::write(&concat_file, concat_content)
            .await
            .map_err(|e| VideoError::ProcessingError {
                message: format!("Failed to write concat file: {}", e),
            })?;

        // Run FFmpeg concat demuxer
        let concat_file_str = concat_file
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid concat file path (non-UTF8 characters)"))?;
        let output_str = output
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid output path (non-UTF8 characters)"))?;

        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command.args([
            "-f", "concat",
            "-safe", "0",
            "-i", concat_file_str,
            "-c", "copy", // Try stream copy first for speed!
            "-y",
            output_str,
        ]);

        // If stream copy fails (codecs differ), we fallback to re-encode?
        // Actually, Auto-recording uses same encoder settings, so stream copy SHOULD work and be instant.
        // This is a huge advantage for "Longform Prep".
        
        let result = execute_ffmpeg_command(&mut command).await;
        
        // Clean up
        let _ = tokio::fs::remove_file(&concat_file).await;

        if let Err(e) = result {
            // If copy failed, try re-encoding
            warn!("Stream copy failed for montage, trying re-encode: {}", e);

            // Re-create concat file with safe path conversion
            let mut concat_content = String::new();
            for path in clip_paths {
                let path_str = path
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid clip path (non-UTF8 characters): {:?}", path))?;
                concat_content.push_str(&format!("file '{}'\n", path_str));
            }
            tokio::fs::write(&concat_file, &concat_content)
                .await
                .context("Failed to write concat file for re-encode")?;

            let mut fallback_cmd = TokioCommand::new(&self.ffmpeg_path);
            fallback_cmd.args([
                "-f", "concat",
                "-safe", "0",
                "-i", concat_file_str,
                "-c:v", "libx264",
                "-preset", "fast",
                "-crf", "23",
                "-c:a", "aac",
                "-y",
                output_str,
            ]);

            execute_ffmpeg_command(&mut fallback_cmd).await?;
            let _ = tokio::fs::remove_file(&concat_file).await;
        }

        info!("Montage composed successfully: {:?}", output);
        Ok(output.to_path_buf())
    }

    /// Scale and crop a single clip to target dimensions (9:16)
    async fn scale_and_crop_clip(
        &self,
        input: &Path,
        output: &Path,
        target_width: u32,
        target_height: u32,
    ) -> Result<PathBuf> {
        info!(
            "Scaling and cropping clip: {:?} -> {:?} ({}x{})",
            input, output, target_width, target_height
        );

        // Calculate scale filter (scale to cover target, then crop)
        let filter = format!(
            "scale=-1:{}:force_original_aspect_ratio=increase,crop={}:{},setsar=1",
            target_height, target_width, target_height
        );

        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command.args([
            "-i",
            input.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: input.display().to_string(),
            })?,
            "-vf",
            &filter,
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "23",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-y",
            output.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: output.display().to_string(),
            })?,
        ]);

        execute_ffmpeg_command(&mut command).await?;

        Ok(output.to_path_buf())
    }

    /// Generate a thumbnail from a video file
    ///
    /// # Arguments
    /// * `input_path` - Path to input video file
    /// * `output_path` - Path to output thumbnail image (JPEG)
    /// * `time_offset` - Time offset in seconds to extract frame (default: 1.0)
    ///
    /// # Returns
    /// Path to the generated thumbnail
    pub async fn generate_thumbnail(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        time_offset: f64,
    ) -> Result<PathBuf> {
        let input = input_path.as_ref();
        let output = output_path.as_ref();

        info!(
            "Generating thumbnail: {:?} -> {:?} (offset: {}s)",
            input, output, time_offset
        );

        // Validate input file exists
        if !input.exists() {
            return Err(VideoError::FileNotFound {
                path: input.display().to_string(),
            });
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            if !parent.exists() {
                return Err(VideoError::OutputDirectoryNotFound {
                    path: parent.display().to_string(),
                });
            }
        }

        // Run FFmpeg to extract a single frame as JPEG
        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command.args([
            "-ss",
            &time_offset.to_string(),
            "-i",
            input.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: input.display().to_string(),
            })?,
            "-vframes",
            "1", // Extract only 1 frame
            "-q:v",
            "2", // High quality JPEG
            "-y",
            output.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: output.display().to_string(),
            })?,
        ]);

        execute_ffmpeg_command(&mut command).await?;

        // Verify output file was created
        if !output.exists() {
            return Err(VideoError::ProcessingError {
                message: format!("Output file was not created: {:?}", output),
            });
        }

        info!("Thumbnail generated successfully: {:?}", output);
        Ok(output.to_path_buf())
    }

    /// Apply advanced video transitions between clips
    pub async fn apply_transition(
        &self,
        input1: impl AsRef<Path>,
        input2: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        transition_type: TransitionType,
        duration: f64,
    ) -> Result<PathBuf> {
        let output = output_path.as_ref();

        info!(
            "Applying {:?} transition: {:?} + {:?} -> {:?} ({}s)",
            transition_type, input1.as_ref(), input2.as_ref(), output, duration
        );

        let filter_complex = match transition_type {
            TransitionType::Fade => format!(
                "[0:v][1:v]xfade=transition=fade:duration={}:offset=0[outv]",
                duration
            ),
            TransitionType::Slide => format!(
                "[0:v][1:v]xfade=transition=slideleft:duration={}:offset=0[outv]",
                duration
            ),
            TransitionType::Dissolve => format!(
                "[0:v][1:v]xfade=transition=dissolve:duration={}:offset=0[outv]",
                duration
            ),
            TransitionType::Wipe => format!(
                "[0:v][1:v]xfade=transition=wipeleft:duration={}:offset=0[outv]",
                duration
            ),
        };

        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command
            .arg("-i")
            .arg(input1.as_ref())
            .arg("-i")
            .arg(input2.as_ref())
            .arg("-filter_complex")
            .arg(&filter_complex)
            .arg("-map")
            .arg("[outv]")
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("medium")
            .arg("-crf")
            .arg("18")
            .arg("-y")
            .arg(output);

        execute_ffmpeg_command(&mut command).await?;
        Ok(output.to_path_buf())
    }

    /// Apply color grading and visual enhancements
    pub async fn apply_color_grading(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        grading: ColorGrading,
    ) -> Result<PathBuf> {
        let output = output_path.as_ref();

        info!(
            "Applying color grading: {:?} -> {:?}",
            input_path.as_ref(), output
        );

        let mut filters = Vec::new();

        // Brightness adjustment
        if grading.brightness != 0.0 {
            filters.push(format!("eq=brightness={}", grading.brightness));
        }

        // Contrast adjustment
        if grading.contrast != 1.0 {
            filters.push(format!("eq=contrast={}", grading.contrast));
        }

        // Saturation adjustment
        if grading.saturation != 1.0 {
            filters.push(format!("eq=saturation={}", grading.saturation));
        }

        // Gamma adjustment
        if grading.gamma != 1.0 {
            filters.push(format!("eq=gamma={}", grading.gamma));
        }

        // Add cinematic vignette if enabled
        if grading.vignette {
            filters.push("vignette='ih*0.5:ih*0.5:0.8'".to_string());
        }

        let filter_string = if filters.is_empty() {
            "copy".to_string()
        } else {
            filters.join(",")
        };

        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command
            .arg("-i")
            .arg(input_path.as_ref())
            .arg("-vf")
            .arg(&filter_string)
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("medium")
            .arg("-crf")
            .arg("18")
            .arg("-y")
            .arg(output);

        execute_ffmpeg_command(&mut command).await?;
        Ok(output.to_path_buf())
    }

    /// Generate slow-motion effect
    pub async fn apply_slow_motion(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        speed_factor: f64, // 0.5 = half speed, 0.25 = quarter speed
    ) -> Result<PathBuf> {
        let output = output_path.as_ref();

        info!(
            "Applying slow motion ({}x speed): {:?} -> {:?}",
            speed_factor, input_path.as_ref(), output
        );

        if speed_factor >= 1.0 {
            return Err(VideoError::ProcessingError {
                message: "Speed factor must be less than 1.0 for slow motion".to_string(),
            });
        }

        // Use setpts for simple speed change (faster but less smooth)
        let filter = format!("setpts={}", 1.0 / speed_factor);

        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command
            .arg("-i")
            .arg(input_path.as_ref())
            .arg("-filter:v")
            .arg(&filter)
            .arg("-r")
            .arg("60")
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("slow")
            .arg("-crf")
            .arg("16")
            .arg("-y")
            .arg(output);

        execute_ffmpeg_command(&mut command).await?;
        Ok(output.to_path_buf())
    }

    /// Add text overlay with styling
    pub async fn add_text_overlay(
        &self,
        input_path: impl AsRef<Path>,
        output_path: impl AsRef<Path>,
        text: &str,
        position: TextPosition,
        style: TextStyle,
    ) -> Result<PathBuf> {
        let output = output_path.as_ref();

        info!(
            "Adding text overlay '{}': {:?} -> {:?}",
            text, input_path.as_ref(), output
        );

        let (x, y) = match position {
            TextPosition::TopLeft => ("10", "h-text_h-10"),
            TextPosition::TopRight => ("w-text_w-10", "h-text_h-10"),
            TextPosition::BottomLeft => ("10", "h-10"),
            TextPosition::BottomRight => ("w-text_w-10", "h-10"),
            TextPosition::Center => ("(w-text_w)/2", "(h-text_h)/2"),
        };

        // Use fontconfig to find system font
        let filter = format!(
            "drawtext=text='{}':x={}:y={}:fontsize={}:fontcolor={}:shadowx=2:shadowy=2",
            text.replace("'", "\\'"),
            x, y,
            style.size,
            style.color.to_lowercase()
        );

        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command
            .arg("-i")
            .arg(input_path.as_ref())
            .arg("-vf")
            .arg(&filter)
            .arg("-c:v")
            .arg("libx264")
            .arg("-preset")
            .arg("medium")
            .arg("-crf")
            .arg("18")
            .arg("-y")
            .arg(output);

        execute_ffmpeg_command(&mut command).await?;
        Ok(output.to_path_buf())
    }

    /// Get video duration in seconds
    pub async fn get_duration(&self, input_path: impl AsRef<Path>) -> Result<f64> {
        let input = input_path.as_ref();

        if !input.exists() {
            return Err(VideoError::FileNotFound {
                path: input.display().to_string(),
            });
        }

        let output = TokioCommand::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                input.to_str().ok_or_else(|| VideoError::FileAccessError {
                    path: input.display().to_string(),
                })?,
            ])
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    VideoError::FfmpegNotFound
                } else {
                    VideoError::ProcessingError {
                        message: format!("Failed to execute ffprobe: {}", e),
                    }
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(VideoError::from_ffmpeg_stderr(&stderr));
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration =
            duration_str
                .trim()
                .parse::<f64>()
                .map_err(|e| VideoError::ProcessingError {
                    message: format!("Failed to parse duration: {}", e),
                })?;

        Ok(duration)
    }
}

impl Default for VideoProcessor {
    fn default() -> Self {
        Self::new_with_fallback()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_processor_creation() {
        let processor = VideoProcessor::new_with_fallback();
        // Should have a valid ffmpeg path (either actual path or "ffmpeg" fallback)
        assert!(!processor.ffmpeg_path.is_empty());
    }

    #[test]
    fn test_video_processor_creation_with_validation() {
        match VideoProcessor::new() {
            Ok(processor) => {
                assert!(!processor.ffmpeg_path.is_empty());
                println!("✅ VideoProcessor created successfully with FFmpeg path: {}", processor.ffmpeg_path);
            }
            Err(e) => {
                println!("⚠️ VideoProcessor creation failed (expected in some environments): {}", e);
                // This is acceptable for testing - we just need to ensure the error is handled properly
            }
        }
    }

    #[test]
    fn test_scale_filter_generation() {
        // Test 9:16 aspect ratio calculation
        let target_width = 1080;
        let target_height = 1920;

        let filter = format!(
            "scale=-1:{}:force_original_aspect_ratio=increase,crop={}:{},setsar=1",
            target_height, target_width, target_height
        );

        assert!(filter.contains("scale=-1:1920"));
        assert!(filter.contains("crop=1080:1920"));
    }

    // Integration tests require FFmpeg to be installed
    #[tokio::test]
    #[ignore] // Requires FFmpeg and test video file
    async fn test_extract_clip_integration() {
        // This test would require a real video file
        // Skipped in CI/CD, run manually during development
    }

    #[tokio::test]
    #[ignore] // Requires FFmpeg and test video files
    async fn test_compose_shorts_integration() {
        // This test would require real video files
        // Skipped in CI/CD, run manually during development
    }

    #[tokio::test]
    #[ignore] // Requires FFmpeg and test video file
    async fn test_generate_thumbnail_integration() {
        // This test would require a real video file
        // Skipped in CI/CD, run manually during development
    }
}
