use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use super::{ClipInfo, VideoProcessor};
use crate::storage::Storage;

/// Configuration for auto-edit composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEditConfig {
    /// Target duration in seconds (60, 120, or 180)
    pub target_duration: u32,

    /// Selected game IDs to include clips from
    pub game_ids: Vec<String>,

    /// Manually selected clip IDs (overrides auto-selection)
    pub selected_clip_ids: Option<Vec<i64>>,

    /// Canvas template configuration
    pub canvas_template: Option<CanvasTemplate>,

    /// Background music configuration
    pub background_music: Option<BackgroundMusic>,

    /// Audio mixing levels
    pub audio_levels: AudioLevels,
}

/// Canvas template for overlays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasTemplate {
    pub id: String,
    pub name: String,
    pub background: BackgroundLayer,
    pub elements: Vec<CanvasElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackgroundLayer {
    Color { value: String },
    Gradient { value: String },
    Image { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum CanvasElement {
    Text {
        id: String,
        content: String,
        font: String,
        size: u32,
        color: String,
        outline: Option<String>,
        position: Position,
    },
    Image {
        id: String,
        path: String,
        width: u32,
        height: u32,
        position: Position,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// X position as percentage (0-100)
    pub x: f32,
    /// Y position as percentage (0-100)
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundMusic {
    /// Path to MP3 file
    pub file_path: String,
    /// Whether to loop music if shorter than video
    pub loop_music: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLevels {
    /// Game audio volume (0-100)
    pub game_audio: u32,
    /// Background music volume (0-100)
    pub background_music: u32,
}

impl Default for AudioLevels {
    fn default() -> Self {
        Self {
            game_audio: 60,
            background_music: 80,
        }
    }
}

/// Result of auto-composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEditResult {
    /// Path to the final composed video
    pub output_path: String,

    /// Selected clips that were used
    pub selected_clips: Vec<ClipInfo>,

    /// Total duration of final video
    pub total_duration: f64,

    /// Number of clips used
    pub clip_count: usize,
}

/// Progress tracking for auto-edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoEditProgress {
    /// Unique job ID
    pub job_id: String,

    /// Current status
    pub status: AutoEditStatus,

    /// Progress percentage (0-100)
    pub progress: f64,

    /// Current step description
    pub current_step: String,

    /// Elapsed time in seconds
    pub elapsed_seconds: f64,

    /// Estimated total time in seconds
    pub estimated_seconds: f64,

    /// Output path (available when completed)
    pub output_path: Option<String>,

    /// Error message (available when failed)
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AutoEditStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

/// Auto-composer for creating YouTube Shorts
pub struct AutoComposer {
    video_processor: Arc<VideoProcessor>,
    storage: Arc<Storage>,
    progress: Arc<RwLock<Option<AutoEditProgress>>>,
}

impl AutoComposer {
    /// Create a new AutoComposer
    pub fn new(video_processor: Arc<VideoProcessor>, storage: Arc<Storage>) -> Self {
        Self {
            video_processor,
            storage,
            progress: Arc::new(RwLock::new(None)),
        }
    }

    /// Main composition workflow
    ///
    /// This is the entry point for auto-edit functionality.
    /// It orchestrates all steps: clip selection, processing, overlay, audio mixing.
    pub async fn compose(&self, config: AutoEditConfig, job_id: String) -> Result<AutoEditResult> {
        info!("Starting auto-composition for job: {}", job_id);

        // Initialize progress tracking
        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            0.0,
            "Initializing auto-edit...".to_string(),
        )
        .await;

        let start_time = std::time::Instant::now();

        // Step 1: Load clips from database (10% progress)
        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            10.0,
            "Loading clips from database...".to_string(),
        )
        .await;

        let all_clips = self.load_clips_from_games(&config.game_ids).await?;

        if all_clips.is_empty() {
            anyhow::bail!("No clips found for selected games");
        }

        // Step 2: Select clips based on priority and duration (20% progress)
        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            20.0,
            format!("Selecting clips from {} available...", all_clips.len()),
        )
        .await;

        let selected_clips = self.select_clips(&all_clips, &config).await?;

        if selected_clips.is_empty() {
            anyhow::bail!("No clips selected after filtering");
        }

        info!(
            "Selected {} clips for composition (target: {}s)",
            selected_clips.len(),
            config.target_duration
        );

        // Step 3: Trim and prepare clips (40% progress)
        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            40.0,
            "Trimming and preparing clips...".to_string(),
        )
        .await;

        let prepared_clips = self.prepare_clips(&selected_clips, config.target_duration).await?;

        // Step 4: Concatenate clips (60% progress)
        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            60.0,
            "Concatenating clips...".to_string(),
        )
        .await;

        let concatenated_path = self.concatenate_clips(&prepared_clips).await?;

        // Step 5: Apply canvas overlay (75% progress)
        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            75.0,
            "Applying canvas overlay...".to_string(),
        )
        .await;

        let with_overlay = if let Some(canvas) = &config.canvas_template {
            self.apply_canvas_overlay(&concatenated_path, canvas)
                .await?
        } else {
            concatenated_path
        };

        // Step 6: Mix audio with background music (90% progress)
        self.update_progress(
            &job_id,
            AutoEditStatus::Processing,
            90.0,
            "Mixing audio...".to_string(),
        )
        .await;

        let final_path = if let Some(music) = &config.background_music {
            self.mix_audio(&with_overlay, music, &config.audio_levels)
                .await?
        } else {
            with_overlay
        };

        // Step 7: Get final duration
        let total_duration = self.video_processor.get_duration(&final_path).await?;

        // Step 8: Complete (100% progress)
        let elapsed = start_time.elapsed().as_secs_f64();
        self.update_progress_complete(
            &job_id,
            final_path.to_string_lossy().to_string(),
            elapsed,
        )
        .await;

        let result = AutoEditResult {
            output_path: final_path.to_string_lossy().to_string(),
            selected_clips,
            total_duration,
            clip_count: prepared_clips.len(),
        };

        info!(
            "Auto-composition completed in {:.2}s: {:?}",
            elapsed, result.output_path
        );

        Ok(result)
    }

    /// Select clips based on priority and target duration
    ///
    /// Algorithm:
    /// 1. If manual selection provided, use those clips
    /// 2. Otherwise, sort clips by priority (5 → 1)
    /// 3. Select clips until target duration is reached
    /// 4. Apply intelligent trimming if needed
    async fn select_clips(
        &self,
        all_clips: &[ClipInfo],
        config: &AutoEditConfig,
    ) -> Result<Vec<ClipInfo>> {
        // If manual selection provided, use it
        if let Some(selected_ids) = &config.selected_clip_ids {
            let selected: Vec<ClipInfo> = all_clips
                .iter()
                .filter(|c| selected_ids.contains(&c.id))
                .cloned()
                .collect();

            if selected.is_empty() {
                anyhow::bail!("None of the manually selected clip IDs were found");
            }

            return Ok(selected);
        }

        // Auto-selection based on priority
        let mut sorted_clips = all_clips.to_vec();
        sorted_clips.sort_by(|a, b| b.priority.cmp(&a.priority)); // Descending priority

        let target_duration = config.target_duration as f64;
        let buffer_duration = target_duration * 0.9; // Reserve 10% for transitions/padding

        let mut selected = Vec::new();
        let mut total_duration = 0.0;

        for clip in &sorted_clips {
            // Get clip duration (use stored or default to 10s)
            let clip_duration = clip.duration.unwrap_or(10.0);

            // Check if adding this clip would exceed target
            if total_duration + clip_duration <= buffer_duration {
                total_duration += clip_duration;
                selected.push(clip.clone());
            }

            // Stop if we've reached target duration
            if total_duration >= buffer_duration {
                break;
            }
        }

        if selected.is_empty() {
            // If no clips fit, take the highest priority clip and trim it
            if let Some(best_clip) = sorted_clips.first() {
                selected.push(best_clip.clone());
            }
        }

        Ok(selected)
    }

    /// Prepare clips by trimming to fit target duration
    async fn prepare_clips(
        &self,
        clips: &[ClipInfo],
        target_duration: u32,
    ) -> Result<Vec<PathBuf>> {
        // For now, just return the clip paths as-is
        // In the future, implement intelligent trimming
        let paths: Vec<PathBuf> = clips.iter().map(|c| PathBuf::from(&c.file_path)).collect();

        // Validate all files exist
        for path in &paths {
            if !path.exists() {
                anyhow::bail!("Clip file not found: {:?}", path);
            }
        }

        Ok(paths)
    }

    /// Concatenate multiple clips
    async fn concatenate_clips(&self, clip_paths: &[PathBuf]) -> Result<PathBuf> {
        let output_dir = std::env::temp_dir().join("lolshorts_auto_edit");
        tokio::fs::create_dir_all(&output_dir)
            .await
            .context("Failed to create temp directory")?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let output_path = output_dir.join(format!("concatenated_{}.mp4", timestamp));

        // Use VideoProcessor to compose clips into 9:16 format
        self.video_processor
            .compose_shorts(clip_paths, &output_path, 1080, 1920)
            .await
    }

    /// Apply canvas overlay (background + text + images)
    async fn apply_canvas_overlay(
        &self,
        video_path: &Path,
        _canvas: &CanvasTemplate,
    ) -> Result<PathBuf> {
        // TODO: Implement canvas overlay using FFmpeg filters
        // For now, return the video as-is
        warn!("Canvas overlay not yet implemented, skipping...");
        Ok(video_path.to_path_buf())
    }

    /// Mix game audio with background music
    async fn mix_audio(
        &self,
        video_path: &Path,
        _music: &BackgroundMusic,
        _levels: &AudioLevels,
    ) -> Result<PathBuf> {
        // TODO: Implement audio mixing using FFmpeg amix filter
        // For now, return the video as-is
        warn!("Audio mixing not yet implemented, skipping...");
        Ok(video_path.to_path_buf())
    }

    /// Load clips from database for given game IDs
    async fn load_clips_from_games(&self, game_ids: &[String]) -> Result<Vec<ClipInfo>> {
        let mut all_clips = Vec::new();
        let mut clip_id_counter = 0i64;

        for game_id in game_ids {
            // Load clips for this game
            let storage_clips = self
                .storage
                .load_clip_metadata(game_id)
                .context(format!("Failed to load clips for game {}", game_id))?;

            info!(
                "Loaded {} clips from game {}",
                storage_clips.len(),
                game_id
            );

            // Convert ClipMetadata to ClipInfo
            for clip in storage_clips {
                // Convert EventType to string
                let event_type = match &clip.event_type {
                    crate::storage::models::EventType::ChampionKill => "ChampionKill".to_string(),
                    crate::storage::models::EventType::Multikill(2) => "DoubleKill".to_string(),
                    crate::storage::models::EventType::Multikill(3) => "TripleKill".to_string(),
                    crate::storage::models::EventType::Multikill(4) => "QuadraKill".to_string(),
                    crate::storage::models::EventType::Multikill(5) => "PentaKill".to_string(),
                    crate::storage::models::EventType::Multikill(n) => {
                        format!("Multikill({})", n)
                    }
                    crate::storage::models::EventType::TurretKill => "TurretKill".to_string(),
                    crate::storage::models::EventType::InhibitorKill => {
                        "InhibitorKill".to_string()
                    }
                    crate::storage::models::EventType::DragonKill => "DragonKill".to_string(),
                    crate::storage::models::EventType::BaronKill => "BaronKill".to_string(),
                    crate::storage::models::EventType::Ace => "Ace".to_string(),
                    crate::storage::models::EventType::FirstBlood => "FirstBlood".to_string(),
                    crate::storage::models::EventType::Custom(s) => s.clone(),
                };

                all_clips.push(ClipInfo {
                    id: clip_id_counter,
                    event_type,
                    event_time: clip.event_time,
                    priority: clip.priority as i32,
                    file_path: clip.file_path,
                    thumbnail_path: clip.thumbnail_path,
                    duration: Some(clip.duration),
                });

                clip_id_counter += 1;
            }
        }

        info!(
            "Total clips loaded from {} games: {}",
            game_ids.len(),
            all_clips.len()
        );

        Ok(all_clips)
    }

    /// Update progress
    async fn update_progress(
        &self,
        job_id: &str,
        status: AutoEditStatus,
        progress: f64,
        current_step: String,
    ) {
        let mut progress_guard = self.progress.write().await;
        *progress_guard = Some(AutoEditProgress {
            job_id: job_id.to_string(),
            status,
            progress,
            current_step,
            elapsed_seconds: 0.0,
            estimated_seconds: 120.0, // Default estimate: 2 minutes
            output_path: None,
            error: None,
        });
    }

    /// Update progress to completed
    async fn update_progress_complete(&self, job_id: &str, output_path: String, elapsed: f64) {
        let mut progress_guard = self.progress.write().await;
        *progress_guard = Some(AutoEditProgress {
            job_id: job_id.to_string(),
            status: AutoEditStatus::Completed,
            progress: 100.0,
            current_step: "Auto-edit completed!".to_string(),
            elapsed_seconds: elapsed,
            estimated_seconds: elapsed,
            output_path: Some(output_path),
            error: None,
        });
    }

    /// Update progress to failed
    async fn update_progress_failed(&self, job_id: &str, error: String, elapsed: f64) {
        let mut progress_guard = self.progress.write().await;
        *progress_guard = Some(AutoEditProgress {
            job_id: job_id.to_string(),
            status: AutoEditStatus::Failed,
            progress: 0.0,
            current_step: "Auto-edit failed".to_string(),
            elapsed_seconds: elapsed,
            estimated_seconds: elapsed,
            output_path: None,
            error: Some(error),
        });
    }

    /// Get current progress
    pub async fn get_progress(&self) -> Option<AutoEditProgress> {
        self.progress.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_storage() -> Arc<Storage> {
        let temp_dir = std::env::temp_dir().join(format!("lolshorts_test_{}", std::process::id()));
        Arc::new(Storage::new(&temp_dir).expect("Failed to create test storage"))
    }

    fn create_test_clip(id: i64, priority: i32, duration: f64, event_type: &str) -> ClipInfo {
        ClipInfo {
            id,
            event_type: event_type.to_string(),
            event_time: 100.0,
            priority,
            file_path: format!("/tmp/clip_{}.mp4", id),
            thumbnail_path: None,
            duration: Some(duration),
        }
    }

    #[tokio::test]
    async fn test_clip_selection_by_priority() {
        let processor = Arc::new(VideoProcessor::new());
        let storage = create_test_storage();
        let composer = AutoComposer::new(processor, storage);

        let clips = vec![
            create_test_clip(1, 1, 10.0, "Kill"),         // Priority 1
            create_test_clip(2, 3, 15.0, "Triple Kill"),  // Priority 3
            create_test_clip(3, 5, 12.0, "Pentakill"),    // Priority 5
            create_test_clip(4, 2, 8.0, "Double Kill"),   // Priority 2
            create_test_clip(5, 4, 10.0, "Quadrakill"),   // Priority 4
        ];

        let config = AutoEditConfig {
            target_duration: 60,
            game_ids: vec!["game1".to_string()],
            selected_clip_ids: None,
            canvas_template: None,
            background_music: None,
            audio_levels: AudioLevels::default(),
        };

        let selected = composer.select_clips(&clips, &config).await.unwrap();

        // Should select highest priority clips first
        assert!(!selected.is_empty());
        assert_eq!(selected[0].priority, 5); // Pentakill first
        assert!(selected.iter().all(|c| c.priority >= 2)); // Should skip low priority clips

        // Total duration should be <= 54s (90% of 60s)
        let total_duration: f64 = selected.iter().map(|c| c.duration.unwrap()).sum();
        assert!(total_duration <= 54.0);
    }

    #[tokio::test]
    async fn test_clip_selection_fits_duration() {
        let processor = Arc::new(VideoProcessor::new());
        let storage = create_test_storage();
        let composer = AutoComposer::new(processor, storage);

        let clips = vec![
            create_test_clip(1, 5, 20.0, "Pentakill"),
            create_test_clip(2, 4, 25.0, "Quadrakill"),
            create_test_clip(3, 3, 30.0, "Triple Kill"),
        ];

        let config = AutoEditConfig {
            target_duration: 60,
            game_ids: vec!["game1".to_string()],
            selected_clip_ids: None,
            canvas_template: None,
            background_music: None,
            audio_levels: AudioLevels::default(),
        };

        let selected = composer.select_clips(&clips, &config).await.unwrap();

        // Should fit within buffer (54s)
        let total_duration: f64 = selected.iter().map(|c| c.duration.unwrap()).sum();
        assert!(total_duration <= 54.0);
        assert_eq!(selected.len(), 2); // Should select 2 clips (20 + 25 = 45s)
    }

    #[tokio::test]
    async fn test_manual_clip_selection() {
        let processor = Arc::new(VideoProcessor::new());
        let storage = create_test_storage();
        let composer = AutoComposer::new(processor, storage);

        let clips = vec![
            create_test_clip(1, 1, 10.0, "Kill"),
            create_test_clip(2, 3, 15.0, "Triple Kill"),
            create_test_clip(3, 5, 12.0, "Pentakill"),
        ];

        let config = AutoEditConfig {
            target_duration: 60,
            game_ids: vec!["game1".to_string()],
            selected_clip_ids: Some(vec![1, 3]), // Manually select clips 1 and 3
            canvas_template: None,
            background_music: None,
            audio_levels: AudioLevels::default(),
        };

        let selected = composer.select_clips(&clips, &config).await.unwrap();

        // Should return exactly the manually selected clips
        assert_eq!(selected.len(), 2);
        assert!(selected.iter().any(|c| c.id == 1));
        assert!(selected.iter().any(|c| c.id == 3));
    }

    #[test]
    fn test_audio_levels_default() {
        let levels = AudioLevels::default();
        assert_eq!(levels.game_audio, 60);
        assert_eq!(levels.background_music, 80);
    }

    #[test]
    fn test_canvas_element_serialization() {
        let text_element = CanvasElement::Text {
            id: "title".to_string(),
            content: "PENTAKILL!".to_string(),
            font: "Bebas Neue".to_string(),
            size: 48,
            color: "#FFD700".to_string(),
            outline: Some("#000000".to_string()),
            position: Position { x: 50.0, y: 10.0 },
        };

        let json = serde_json::to_string(&text_element).unwrap();
        assert!(json.contains("\"type\":\"text\""));
        assert!(json.contains("PENTAKILL"));
    }
}
