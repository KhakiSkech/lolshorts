pub mod models;
pub mod models_v2;
pub mod commands;

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Re-export public types
pub use models::{GameMetadata, ClipMetadata, EventData};

// Re-export V2 types for editor integration
pub use models_v2::{
    ClipMetadataV2, EventInfo, EventWindow, MergeStrategy,
    VideoInfo, Resolution, FrameRate, VideoCodec,
    AudioInfo, AudioTrack, AudioTrackType,
    ClipTimeline, TimelineMarker, MarkerType, Chapter,
    GameContext, GameMode, QueueType, Team, TeamScore, PlayerState,
    UserAnnotations, Note,
};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Game not found: {0}")]
    GameNotFound(String),
}

pub type Result<T> = std::result::Result<T, StorageError>;

/// JSON-based file storage for clips and metadata
pub struct Storage {
    base_path: PathBuf,
}

impl Storage {
    /// Create a new storage instance
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();

        // Create directory structure
        fs::create_dir_all(&base_path)?;
        fs::create_dir_all(base_path.join("clips"))?;
        fs::create_dir_all(base_path.join("recordings"))?;
        fs::create_dir_all(base_path.join("replays"))?;

        tracing::info!("Storage initialized at: {}", base_path.display());

        Ok(Self { base_path })
    }

    /// Get the base storage path
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }

    /// Get path for a specific game
    pub fn game_path(&self, game_id: &str) -> PathBuf {
        self.base_path.join("clips").join(game_id)
    }

    /// Create a new game directory
    pub fn create_game(&self, game_id: &str, metadata: &GameMetadata) -> Result<()> {
        let game_path = self.game_path(game_id);
        fs::create_dir_all(&game_path)?;

        // Save metadata
        let metadata_path = game_path.join("metadata.json");
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_path, json)?;

        tracing::info!("Created game directory: {}", game_id);
        Ok(())
    }

    /// Save game metadata
    pub fn save_game_metadata(&self, game_id: &str, metadata: &GameMetadata) -> Result<()> {
        let game_path = self.game_path(game_id);

        if !game_path.exists() {
            fs::create_dir_all(&game_path)?;
        }

        let metadata_path = game_path.join("metadata.json");
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(metadata_path, json)?;

        Ok(())
    }

    /// Load game metadata
    pub fn load_game_metadata(&self, game_id: &str) -> Result<GameMetadata> {
        let metadata_path = self.game_path(game_id).join("metadata.json");

        if !metadata_path.exists() {
            return Err(StorageError::GameNotFound(game_id.to_string()));
        }

        let json = fs::read_to_string(metadata_path)?;
        let metadata = serde_json::from_str(&json)?;

        Ok(metadata)
    }

    /// Save events for a game
    pub fn save_events(&self, game_id: &str, events: &[EventData]) -> Result<()> {
        let game_path = self.game_path(game_id);

        if !game_path.exists() {
            fs::create_dir_all(&game_path)?;
        }

        let events_path = game_path.join("events.json");
        let json = serde_json::to_string_pretty(events)?;
        fs::write(events_path, json)?;

        tracing::debug!("Saved {} events for game {}", events.len(), game_id);
        Ok(())
    }

    /// Load events for a game
    pub fn load_events(&self, game_id: &str) -> Result<Vec<EventData>> {
        let events_path = self.game_path(game_id).join("events.json");

        if !events_path.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(events_path)?;
        let events = serde_json::from_str(&json)?;

        Ok(events)
    }

    /// Save clip metadata
    pub fn save_clip_metadata(&self, game_id: &str, clip: &ClipMetadata) -> Result<()> {
        let game_path = self.game_path(game_id);

        if !game_path.exists() {
            fs::create_dir_all(&game_path)?;
        }

        // Load existing clips
        let mut clips = self.load_clip_metadata(game_id).unwrap_or_default();

        // Add or update clip
        if let Some(pos) = clips.iter().position(|c| c.file_path == clip.file_path) {
            clips[pos] = clip.clone();
        } else {
            clips.push(clip.clone());
        }

        // Save clips
        let clips_path = game_path.join("clips.json");
        let json = serde_json::to_string_pretty(&clips)?;
        fs::write(clips_path, json)?;

        Ok(())
    }

    /// Load all clip metadata for a game
    pub fn load_clip_metadata(&self, game_id: &str) -> Result<Vec<ClipMetadata>> {
        let clips_path = self.game_path(game_id).join("clips.json");

        if !clips_path.exists() {
            return Ok(Vec::new());
        }

        let json = fs::read_to_string(clips_path)?;
        let clips = serde_json::from_str(&json)?;

        Ok(clips)
    }

    /// Get all games (sorted by most recent)
    pub fn list_games(&self) -> Result<Vec<String>> {
        let clips_dir = self.base_path.join("clips");

        if !clips_dir.exists() {
            return Ok(Vec::new());
        }

        let mut games = Vec::new();

        for entry in fs::read_dir(clips_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    games.push(name.to_string());
                }
            }
        }

        // Sort by directory modification time (most recent first)
        games.sort_by(|a, b| {
            let a_time = fs::metadata(self.game_path(a))
                .and_then(|m| m.modified())
                .ok();
            let b_time = fs::metadata(self.game_path(b))
                .and_then(|m| m.modified())
                .ok();
            b_time.cmp(&a_time)
        });

        Ok(games)
    }

    /// Delete a game and all its clips
    pub fn delete_game(&self, game_id: &str) -> Result<()> {
        let game_path = self.game_path(game_id);

        if game_path.exists() {
            fs::remove_dir_all(game_path)?;
            tracing::info!("Deleted game: {}", game_id);
        }

        Ok(())
    }

    /// Delete a specific clip's metadata from storage
    pub fn delete_clip_metadata(&self, game_id: &str, file_path: &str) -> Result<()> {
        let mut clips = self.load_clip_metadata(game_id).unwrap_or_default();

        // Remove the clip with matching file path
        let original_len = clips.len();
        clips.retain(|c| c.file_path != file_path);

        if clips.len() == original_len {
            tracing::warn!("Clip not found in metadata: {}", file_path);
        } else {
            tracing::info!("Removed clip from metadata: {}", file_path);
        }

        // Save updated clips list
        let clips_path = self.game_path(game_id).join("clips.json");
        let json = serde_json::to_string_pretty(&clips)?;
        fs::write(clips_path, json)?;

        Ok(())
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> Result<StorageStats> {
        let mut total_clips = 0;
        let mut total_size = 0u64;

        let games = self.list_games()?;

        for game_id in &games {
            let clips = self.load_clip_metadata(game_id).unwrap_or_default();
            total_clips += clips.len();

            // Calculate size
            for clip in clips {
                if let Ok(metadata) = fs::metadata(&clip.file_path) {
                    total_size += metadata.len();
                }
            }
        }

        Ok(StorageStats {
            total_games: games.len(),
            total_clips,
            total_size_bytes: total_size,
        })
    }

    // ========================================================================
    // V2 Metadata Storage (For Editor Integration)
    // ========================================================================

    /// Save comprehensive clip metadata (V2) alongside video file
    ///
    /// This creates an individual JSON file for each clip with rich metadata
    /// that the video editor can consume.
    ///
    /// File structure:
    /// ```
    /// clips/<game_id>/clips/
    ///   ├── clip_xxx.mp4
    ///   └── clip_xxx.json  ← Rich metadata
    /// ```
    pub fn save_clip_metadata_v2(&self, game_id: &str, clip: &ClipMetadataV2) -> Result<()> {
        let game_clips_dir = self.game_path(game_id).join("clips");

        if !game_clips_dir.exists() {
            fs::create_dir_all(&game_clips_dir)?;
        }

        // Generate JSON path from video path
        let video_path = Path::new(&clip.file_path);
        let json_path = video_path.with_extension("json");

        // Save individual clip JSON
        let json = serde_json::to_string_pretty(clip)?;
        fs::write(&json_path, json)?;

        tracing::debug!("Saved V2 metadata: {:?}", json_path);

        // Also update the clips.json index (lightweight reference)
        self.update_clips_index_v2(game_id, clip)?;

        Ok(())
    }

    /// Load comprehensive clip metadata (V2) for a specific clip
    ///
    /// Loads from individual JSON file alongside video.
    pub fn load_clip_metadata_v2(&self, clip_path: &str) -> Result<ClipMetadataV2> {
        let json_path = Path::new(clip_path).with_extension("json");

        if !json_path.exists() {
            return Err(StorageError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Clip metadata not found: {:?}", json_path),
            )));
        }

        let json = fs::read_to_string(&json_path)?;
        let clip = serde_json::from_str(&json)?;

        Ok(clip)
    }

    /// Load all V2 clip metadata for a game
    ///
    /// Returns list of all clips with full metadata for editor display.
    pub fn load_all_clips_v2(&self, game_id: &str) -> Result<Vec<ClipMetadataV2>> {
        let clips_dir = self.game_path(game_id).join("clips");

        if !clips_dir.exists() {
            return Ok(Vec::new());
        }

        let mut clips = Vec::new();

        for entry in fs::read_dir(clips_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only load JSON files
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(clip) = serde_json::from_str::<ClipMetadataV2>(&json) {
                        clips.push(clip);
                    }
                }
            }
        }

        // Sort by created_at (most recent first)
        clips.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(clips)
    }

    /// Update the clips.json index with lightweight reference
    ///
    /// This maintains backward compatibility with V1 while adding V2 support.
    fn update_clips_index_v2(&self, game_id: &str, clip: &ClipMetadataV2) -> Result<()> {
        let game_path = self.game_path(game_id);

        // Load existing V1 clips
        let mut v1_clips = self.load_clip_metadata(game_id).unwrap_or_default();

        // Convert V2 to V1 for index
        let v1_clip = ClipMetadata {
            file_path: clip.file_path.clone(),
            thumbnail_path: clip.thumbnail_path.clone(),
            event_type: clip.primary_event.event_type.clone(),
            event_time: clip.game_time_start,
            priority: clip.priority,
            duration: clip.clip_duration,
            created_at: clip.created_at,
        };

        // Add or update
        if let Some(pos) = v1_clips.iter().position(|c| c.file_path == v1_clip.file_path) {
            v1_clips[pos] = v1_clip;
        } else {
            v1_clips.push(v1_clip);
        }

        // Save index
        let clips_path = game_path.join("clips.json");
        let json = serde_json::to_string_pretty(&v1_clips)?;
        fs::write(clips_path, json)?;

        Ok(())
    }

    /// Delete V2 clip metadata (both video and JSON)
    pub fn delete_clip_v2(&self, game_id: &str, clip_path: &str) -> Result<()> {
        let video_path = Path::new(clip_path);
        let json_path = video_path.with_extension("json");
        let jpg_path = video_path.with_extension("jpg");

        // Delete video file
        if video_path.exists() {
            fs::remove_file(video_path)?;
            tracing::info!("Deleted video: {:?}", video_path);
        }

        // Delete JSON metadata
        if json_path.exists() {
            fs::remove_file(&json_path)?;
            tracing::info!("Deleted metadata: {:?}", json_path);
        }

        // Delete thumbnail
        if jpg_path.exists() {
            fs::remove_file(&jpg_path)?;
            tracing::info!("Deleted thumbnail: {:?}", jpg_path);
        }

        // Update clips.json index
        self.delete_clip_metadata(game_id, clip_path)?;

        Ok(())
    }

    /// Search clips by tags
    pub fn search_clips_by_tags(&self, game_id: &str, tags: &[String]) -> Result<Vec<ClipMetadataV2>> {
        let all_clips = self.load_all_clips_v2(game_id)?;

        let filtered = all_clips
            .into_iter()
            .filter(|clip| {
                // Clip must have at least one matching tag
                tags.iter().any(|tag| clip.tags.contains(tag))
            })
            .collect();

        Ok(filtered)
    }

    /// Get clips by priority threshold
    pub fn get_clips_by_priority(&self, game_id: &str, min_priority: u8) -> Result<Vec<ClipMetadataV2>> {
        let all_clips = self.load_all_clips_v2(game_id)?;

        let filtered = all_clips
            .into_iter()
            .filter(|clip| clip.priority >= min_priority)
            .collect();

        Ok(filtered)
    }

    /// Get favorite clips
    pub fn get_favorite_clips(&self, game_id: &str) -> Result<Vec<ClipMetadataV2>> {
        let all_clips = self.load_all_clips_v2(game_id)?;

        let filtered = all_clips
            .into_iter()
            .filter(|clip| {
                clip.annotations
                    .as_ref()
                    .map(|a| a.favorite)
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered)
    }

    // ========================================================================
    // Canvas Template Storage
    // ========================================================================

    /// Save a canvas template to the template library
    ///
    /// Templates are stored in: <base_path>/templates/<template_id>.json
    pub fn save_canvas_template(&self, template: &crate::video::CanvasTemplate) -> Result<()> {
        let templates_dir = self.base_path.join("templates");
        fs::create_dir_all(&templates_dir)?;

        let template_path = templates_dir.join(format!("{}.json", template.id));
        let json = serde_json::to_string_pretty(template)?;
        fs::write(template_path, json)?;

        tracing::info!("Saved canvas template: {} ({})", template.name, template.id);
        Ok(())
    }

    /// Load a canvas template by ID
    pub fn load_canvas_template(&self, template_id: &str) -> Result<crate::video::CanvasTemplate> {
        let template_path = self.base_path.join("templates").join(format!("{}.json", template_id));

        if !template_path.exists() {
            return Err(StorageError::GameNotFound(format!("Template not found: {}", template_id)));
        }

        let json = fs::read_to_string(template_path)?;
        let template = serde_json::from_str(&json)?;

        Ok(template)
    }

    /// List all available canvas templates
    ///
    /// Returns a list of template IDs and names
    pub fn list_canvas_templates(&self) -> Result<Vec<CanvasTemplateInfo>> {
        let templates_dir = self.base_path.join("templates");

        if !templates_dir.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();

        for entry in fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(template) = serde_json::from_str::<crate::video::CanvasTemplate>(&json) {
                        templates.push(CanvasTemplateInfo {
                            id: template.id.clone(),
                            name: template.name.clone(),
                            element_count: template.elements.len(),
                        });
                    }
                }
            }
        }

        // Sort by name
        templates.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(templates)
    }

    /// Delete a canvas template
    pub fn delete_canvas_template(&self, template_id: &str) -> Result<()> {
        let template_path = self.base_path.join("templates").join(format!("{}.json", template_id));

        if template_path.exists() {
            fs::remove_file(template_path)?;
            tracing::info!("Deleted canvas template: {}", template_id);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_games: usize,
    pub total_clips: usize,
    pub total_size_bytes: u64,
}

/// Canvas template metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasTemplateInfo {
    pub id: String,
    pub name: String,
    pub element_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_storage_creation() {
        let temp_dir = std::env::temp_dir().join("lolshorts_test");
        let storage = Storage::new(&temp_dir);
        assert!(storage.is_ok());

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_game_metadata() {
        let temp_dir = std::env::temp_dir().join("lolshorts_test2");
        let storage = Storage::new(&temp_dir).unwrap();

        let metadata = GameMetadata {
            game_id: "12345".to_string(),
            champion: "Yasuo".to_string(),
            game_mode: "Ranked".to_string(),
            start_time: Utc::now(),
            end_time: None,
            result: None,
            kda: None,
        };

        storage.save_game_metadata("12345", &metadata).unwrap();
        let loaded = storage.load_game_metadata("12345").unwrap();

        assert_eq!(loaded.game_id, "12345");
        assert_eq!(loaded.champion, "Yasuo");

        // Cleanup
        let _ = fs::remove_dir_all(temp_dir);
    }
}
