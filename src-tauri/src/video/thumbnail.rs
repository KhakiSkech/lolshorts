use crate::video::{VideoProcessor, Result};
use std::path::{Path, PathBuf};

/// Auto-generate thumbnail for a clip at the midpoint
pub async fn auto_generate_thumbnail(
    clip_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
) -> Result<PathBuf> {
    let processor = VideoProcessor::new();

    // Get video duration to find midpoint
    let duration = processor.get_duration(&clip_path).await?;
    let midpoint = duration / 2.0;

    // Generate thumbnail filename based on clip filename
    let clip_name = clip_path.as_ref()
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("clip");

    let thumbnail_name = format!("{}_thumbnail.jpg", clip_name);
    let thumbnail_path = output_dir.as_ref().join(thumbnail_name);

    // Generate thumbnail at midpoint
    processor.generate_thumbnail(
        clip_path,
        &thumbnail_path,
        midpoint
    ).await?;

    Ok(thumbnail_path)
}

/// Generate thumbnail at specific event timestamp
pub async fn generate_event_thumbnail(
    clip_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    event_time: f64,
    event_name: &str,
) -> Result<PathBuf> {
    let processor = VideoProcessor::new();

    let thumbnail_name = format!("{}_{:.1}s.jpg", event_name, event_time);
    let thumbnail_path = output_dir.as_ref().join(thumbnail_name);

    // Generate thumbnail at event timestamp
    processor.generate_thumbnail(
        clip_path,
        &thumbnail_path,
        event_time
    ).await?;

    Ok(thumbnail_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thumbnail_generation() {
        // This test requires a valid video file to run
        // In production, use actual clip files
    }
}
