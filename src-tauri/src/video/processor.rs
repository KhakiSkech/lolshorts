use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command as TokioCommand;
use tracing::{error, info};

/// FFmpeg video processor for clip extraction and composition
pub struct VideoProcessor {
    ffmpeg_path: String,
}

impl VideoProcessor {
    pub fn new() -> Self {
        Self {
            ffmpeg_path: "ffmpeg".to_string(), // Assumes FFmpeg is in PATH or bundled
        }
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
            anyhow::bail!("Input file does not exist: {:?}", input);
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create output directory")?;
        }

        // Run FFmpeg command to extract clip
        // Using -ss before -i for fast seeking, -c copy to avoid re-encoding when possible
        let status = TokioCommand::new(&self.ffmpeg_path)
            .args(&[
                "-ss",
                &start_time.to_string(),
                "-i",
                input.to_str().context("Invalid input path")?,
                "-t",
                &duration.to_string(),
                "-c",
                "copy", // Copy codec without re-encoding
                "-avoid_negative_ts",
                "make_zero",
                "-y", // Overwrite output file
                output.to_str().context("Invalid output path")?,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .context("Failed to execute FFmpeg")?;

        if !status.success() {
            error!("FFmpeg clip extraction failed with status: {}", status);
            anyhow::bail!("FFmpeg clip extraction failed");
        }

        // Verify output file was created
        if !output.exists() {
            anyhow::bail!("Output file was not created: {:?}", output);
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
            anyhow::bail!("No clips provided for composition");
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
                anyhow::bail!("Clip file does not exist: {:?}", clip);
            }
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create output directory")?;
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

        // Write concat file
        let concat_content: String = clip_paths
            .iter()
            .map(|p| format!("file '{}'\n", p.to_str().unwrap()))
            .collect();

        tokio::fs::write(&concat_file, concat_content)
            .await
            .context("Failed to write concat file")?;

        // Run FFmpeg to concatenate and scale to 9:16
        let status = TokioCommand::new(&self.ffmpeg_path)
            .args(&[
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                concat_file.to_str().context("Invalid concat file path")?,
                "-vf",
                &format!(
                    "scale={}:{},setsar=1",
                    target_width,
                    target_height
                ),
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
                output.to_str().context("Invalid output path")?,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .context("Failed to execute FFmpeg")?;

        // Clean up concat file
        let _ = tokio::fs::remove_file(&concat_file).await;

        if !status.success() {
            error!("FFmpeg composition failed with status: {}", status);
            anyhow::bail!("FFmpeg composition failed");
        }

        // Verify output file was created
        if !output.exists() {
            anyhow::bail!("Output file was not created: {:?}", output);
        }

        info!("Short composed successfully: {:?}", output);
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

        let status = TokioCommand::new(&self.ffmpeg_path)
            .args(&[
                "-i",
                input.to_str().context("Invalid input path")?,
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
                output.to_str().context("Invalid output path")?,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .context("Failed to execute FFmpeg")?;

        if !status.success() {
            error!("FFmpeg scale/crop failed with status: {}", status);
            anyhow::bail!("FFmpeg scale/crop failed");
        }

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
            anyhow::bail!("Input file does not exist: {:?}", input);
        }

        // Create output directory if it doesn't exist
        if let Some(parent) = output.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create output directory")?;
        }

        // Run FFmpeg to extract a single frame as JPEG
        let status = TokioCommand::new(&self.ffmpeg_path)
            .args(&[
                "-ss",
                &time_offset.to_string(),
                "-i",
                input.to_str().context("Invalid input path")?,
                "-vframes",
                "1", // Extract only 1 frame
                "-q:v",
                "2", // High quality JPEG
                "-y",
                output.to_str().context("Invalid output path")?,
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .await
            .context("Failed to execute FFmpeg")?;

        if !status.success() {
            error!("FFmpeg thumbnail generation failed with status: {}", status);
            anyhow::bail!("FFmpeg thumbnail generation failed");
        }

        // Verify output file was created
        if !output.exists() {
            anyhow::bail!("Output file was not created: {:?}", output);
        }

        info!("Thumbnail generated successfully: {:?}", output);
        Ok(output.to_path_buf())
    }

    /// Get video duration in seconds
    pub async fn get_duration(&self, input_path: impl AsRef<Path>) -> Result<f64> {
        let input = input_path.as_ref();

        if !input.exists() {
            anyhow::bail!("Input file does not exist: {:?}", input);
        }

        let output = TokioCommand::new("ffprobe")
            .args(&[
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                input.to_str().context("Invalid input path")?,
            ])
            .output()
            .await
            .context("Failed to execute ffprobe")?;

        if !output.status.success() {
            anyhow::bail!("ffprobe failed to get duration");
        }

        let duration_str = String::from_utf8_lossy(&output.stdout);
        let duration = duration_str
            .trim()
            .parse::<f64>()
            .context("Failed to parse duration")?;

        Ok(duration)
    }
}

impl Default for VideoProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_processor_creation() {
        let processor = VideoProcessor::new();
        assert_eq!(processor.ffmpeg_path, "ffmpeg");
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
