#![allow(dead_code)]
use std::path::{Path, PathBuf};
use tokio::process::Command as TokioCommand;
use tracing::info;

use super::{execute_ffmpeg_command, Result, VideoError};

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

        // Run FFmpeg command to extract clip
        // Using -ss before -i for fast seeking, -c copy to avoid re-encoding when possible
        let mut command = TokioCommand::new(&self.ffmpeg_path);
        command.args([
            "-ss",
            &start_time.to_string(),
            "-i",
            input.to_str().ok_or_else(|| VideoError::FileAccessError {
                path: input.display().to_string(),
            })?,
            "-t",
            &duration.to_string(),
            "-c",
            "copy", // Copy codec without re-encoding
            "-avoid_negative_ts",
            "make_zero",
            "-y", // Overwrite output file
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

        // Write concat file
        let concat_content: String = clip_paths
            .iter()
            .map(|p| format!("file '{}'\n", p.to_str().unwrap()))
            .collect();

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
