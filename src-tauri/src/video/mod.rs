pub mod auto_composer;
pub mod commands;
pub mod processor;
pub mod thumbnail;

pub use auto_composer::{
    AudioLevels, AutoComposer, AutoEditConfig, AutoEditProgress, AutoEditResult, AutoEditStatus,
    BackgroundLayer, BackgroundMusic, CanvasElement, CanvasTemplate, Position,
};
pub use processor::VideoProcessor;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VideoError {
    #[error("FFmpeg error: {0}")]
    FfmpegError(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("Processing error: {0}")]
    ProcessingError(String),
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, VideoError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipInfo {
    pub id: i64,
    pub event_type: String,
    pub event_time: f64,
    pub priority: i32,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub duration: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_processor_creation() {
        let _processor = VideoProcessor::new();
    }
}
