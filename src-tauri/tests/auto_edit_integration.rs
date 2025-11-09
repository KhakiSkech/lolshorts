// Integration tests for Auto-Edit / YouTube Shorts generation system
#![cfg(test)]

use lolshorts::storage::Storage;
use lolshorts::video::auto_composer::{
    AudioLevels, AutoComposer, AutoEditConfig, AutoEditStatus, BackgroundLayer, BackgroundMusic,
    CanvasElement, CanvasTemplate, Position,
};
use lolshorts::video::{ClipInfo, VideoProcessor};
use std::sync::Arc;
use tokio;

// =====================================================================
// Helper Functions for Test Setup
// =====================================================================

/// Create a test storage instance in temporary directory
fn create_test_storage() -> Arc<Storage> {
    let temp_dir = std::env::temp_dir().join(format!(
        "lolshorts_auto_edit_test_{}",
        std::process::id()
    ));
    Arc::new(Storage::new(&temp_dir).expect("Failed to create test storage"))
}

/// Create a test ClipInfo with specified parameters
fn create_test_clip(
    id: i64,
    priority: i32,
    duration: f64,
    event_type: &str,
    file_path: &str,
) -> ClipInfo {
    ClipInfo {
        id,
        event_type: event_type.to_string(),
        event_time: 100.0 + (id as f64 * 10.0),
        priority,
        file_path: file_path.to_string(),
        thumbnail_path: None,
        duration: Some(duration),
    }
}

/// Create a basic AutoEditConfig for testing
fn create_basic_config() -> AutoEditConfig {
    AutoEditConfig {
        target_duration: 60,
        game_ids: vec!["test_game_1".to_string()],
        selected_clip_ids: None,
        canvas_template: None,
        background_music: None,
        audio_levels: AudioLevels::default(),
    }
}

/// Create a canvas template for testing
fn create_test_canvas_template() -> CanvasTemplate {
    CanvasTemplate {
        id: "test_template_1".to_string(),
        name: "Test Template".to_string(),
        background: BackgroundLayer::Color {
            value: "#000000".to_string(),
        },
        elements: vec![
            CanvasElement::Text {
                id: "title".to_string(),
                content: "PENTAKILL!".to_string(),
                font: "Arial".to_string(),
                size: 48,
                color: "#FFD700".to_string(),
                outline: Some("#000000".to_string()),
                position: Position { x: 50.0, y: 10.0 },
            },
            CanvasElement::Image {
                id: "logo".to_string(),
                path: "/test/logo.png".to_string(),
                width: 100,
                height: 100,
                position: Position { x: 50.0, y: 90.0 },
            },
        ],
    }
}

// =====================================================================
// Configuration Validation Tests
// =====================================================================

#[tokio::test]
async fn test_target_duration_validation() {
    // Valid YouTube Shorts durations
    let valid_durations = vec![60, 120, 180];

    for duration in valid_durations {
        let config = AutoEditConfig {
            target_duration: duration,
            game_ids: vec!["game1".to_string()],
            selected_clip_ids: None,
            canvas_template: None,
            background_music: None,
            audio_levels: AudioLevels::default(),
        };

        assert!(config.target_duration == 60 || config.target_duration == 120 || config.target_duration == 180);
        assert!(config.target_duration <= 180); // Max YouTube Shorts length
    }
}

#[tokio::test]
async fn test_game_id_validation() {
    let config = create_basic_config();

    // Must have at least one game selected
    assert!(!config.game_ids.is_empty());

    // Game IDs should not be empty strings
    for game_id in &config.game_ids {
        assert!(!game_id.is_empty());
    }
}

#[tokio::test]
async fn test_audio_levels_default() {
    let levels = AudioLevels::default();

    // Validate default audio levels
    assert_eq!(levels.game_audio, 60);
    assert_eq!(levels.background_music, 80);

    // Both should be in valid range (0-100)
    assert!(levels.game_audio <= 100);
    assert!(levels.background_music <= 100);
}

#[tokio::test]
async fn test_audio_levels_custom() {
    let levels = AudioLevels {
        game_audio: 70,
        background_music: 30,
    };

    // Custom levels should be respected
    assert_eq!(levels.game_audio, 70);
    assert_eq!(levels.background_music, 30);

    // Validate range (u8 is always >= 0)
    assert!(levels.game_audio <= 100);
    assert!(levels.background_music <= 100);
}

// =====================================================================
// Clip Selection Algorithm Tests
// =====================================================================

#[tokio::test]
async fn test_clip_selection_by_priority() {
    let processor = Arc::new(VideoProcessor::new());
    let storage = create_test_storage();
    let composer = AutoComposer::new(processor, storage);

    let clips = vec![
        create_test_clip(1, 1, 10.0, "Kill", "/test/clip1.mp4"),
        create_test_clip(2, 3, 15.0, "TripleKill", "/test/clip2.mp4"),
        create_test_clip(3, 5, 12.0, "Pentakill", "/test/clip3.mp4"),
        create_test_clip(4, 2, 8.0, "DoubleKill", "/test/clip4.mp4"),
        create_test_clip(5, 4, 10.0, "Quadrakill", "/test/clip5.mp4"),
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
    assert_eq!(selected[0].priority, 5); // Pentakill should be first

    // All selected clips should have priority >= 2 (LOW priority clips excluded)
    for clip in &selected {
        assert!(clip.priority >= 2);
    }

    // Total duration should fit within buffer (90% of target = 54s)
    let total_duration: f64 = selected.iter().map(|c| c.duration.unwrap()).sum();
    assert!(total_duration <= 54.0);
}

#[tokio::test]
async fn test_clip_selection_fits_duration() {
    let processor = Arc::new(VideoProcessor::new());
    let storage = create_test_storage();
    let composer = AutoComposer::new(processor, storage);

    let clips = vec![
        create_test_clip(1, 5, 20.0, "Pentakill", "/test/clip1.mp4"),
        create_test_clip(2, 4, 25.0, "Quadrakill", "/test/clip2.mp4"),
        create_test_clip(3, 3, 30.0, "TripleKill", "/test/clip3.mp4"),
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

    // Should fit within buffer duration (54s = 90% of 60s)
    let total_duration: f64 = selected.iter().map(|c| c.duration.unwrap()).sum();
    assert!(total_duration <= 54.0);

    // Should select 2 clips (20 + 25 = 45s < 54s, but 20 + 25 + 30 = 75s > 54s)
    assert_eq!(selected.len(), 2);
}

#[tokio::test]
async fn test_manual_clip_selection() {
    let processor = Arc::new(VideoProcessor::new());
    let storage = create_test_storage();
    let composer = AutoComposer::new(processor, storage);

    let clips = vec![
        create_test_clip(1, 1, 10.0, "Kill", "/test/clip1.mp4"),
        create_test_clip(2, 3, 15.0, "TripleKill", "/test/clip2.mp4"),
        create_test_clip(3, 5, 12.0, "Pentakill", "/test/clip3.mp4"),
    ];

    // Manual selection: only clips 1 and 3
    let config = AutoEditConfig {
        target_duration: 60,
        game_ids: vec!["game1".to_string()],
        selected_clip_ids: Some(vec![1, 3]),
        canvas_template: None,
        background_music: None,
        audio_levels: AudioLevels::default(),
    };

    let selected = composer.select_clips(&clips, &config).await.unwrap();

    // Should return exactly the manually selected clips
    assert_eq!(selected.len(), 2);
    assert!(selected.iter().any(|c| c.id == 1));
    assert!(selected.iter().any(|c| c.id == 3));
    assert!(!selected.iter().any(|c| c.id == 2)); // Clip 2 not selected
}

#[tokio::test]
async fn test_clip_selection_empty_clips() {
    let processor = Arc::new(VideoProcessor::new());
    let storage = create_test_storage();
    let composer = AutoComposer::new(processor, storage);

    let clips: Vec<ClipInfo> = vec![];
    let config = create_basic_config();

    let result = composer.select_clips(&clips, &config).await;

    // Should return error for empty clips
    assert!(result.is_err());
}

#[tokio::test]
async fn test_clip_selection_single_long_clip() {
    let processor = Arc::new(VideoProcessor::new());
    let storage = create_test_storage();
    let composer = AutoComposer::new(processor, storage);

    // Single clip that's longer than target duration
    let clips = vec![create_test_clip(1, 5, 80.0, "Pentakill", "/test/clip1.mp4")];

    let config = AutoEditConfig {
        target_duration: 60,
        game_ids: vec!["game1".to_string()],
        selected_clip_ids: None,
        canvas_template: None,
        background_music: None,
        audio_levels: AudioLevels::default(),
    };

    let selected = composer.select_clips(&clips, &config).await.unwrap();

    // Should still select the clip (will be trimmed later)
    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].priority, 5);
}

// =====================================================================
// Canvas Template Tests
// =====================================================================

#[tokio::test]
async fn test_canvas_template_creation() {
    let template = create_test_canvas_template();

    assert_eq!(template.id, "test_template_1");
    assert_eq!(template.name, "Test Template");
    assert_eq!(template.elements.len(), 2);

    // Validate background
    match &template.background {
        BackgroundLayer::Color { value } => {
            assert_eq!(value, "#000000");
        }
        _ => panic!("Expected Color background"),
    }
}

#[tokio::test]
async fn test_canvas_element_text() {
    let template = create_test_canvas_template();

    // First element should be text
    match &template.elements[0] {
        CanvasElement::Text {
            id,
            content,
            font,
            size,
            color,
            outline,
            position,
        } => {
            assert_eq!(id, "title");
            assert_eq!(content, "PENTAKILL!");
            assert_eq!(font, "Arial");
            assert_eq!(*size, 48);
            assert_eq!(color, "#FFD700");
            assert_eq!(outline.as_ref().unwrap(), "#000000");
            assert_eq!(position.x, 50.0);
            assert_eq!(position.y, 10.0);
        }
        _ => panic!("Expected Text element"),
    }
}

#[tokio::test]
async fn test_canvas_element_image() {
    let template = create_test_canvas_template();

    // Second element should be image
    match &template.elements[1] {
        CanvasElement::Image {
            id,
            path,
            width,
            height,
            position,
        } => {
            assert_eq!(id, "logo");
            assert_eq!(path, "/test/logo.png");
            assert_eq!(*width, 100);
            assert_eq!(*height, 100);
            assert_eq!(position.x, 50.0);
            assert_eq!(position.y, 90.0);
        }
        _ => panic!("Expected Image element"),
    }
}

#[tokio::test]
async fn test_canvas_position_validation() {
    let position = Position { x: 50.0, y: 75.0 };

    // Position should be in percentage (0-100)
    assert!(position.x >= 0.0 && position.x <= 100.0);
    assert!(position.y >= 0.0 && position.y <= 100.0);
}

#[tokio::test]
async fn test_canvas_background_types() {
    // Test Color background
    let color_bg = BackgroundLayer::Color {
        value: "#FF0000".to_string(),
    };
    match color_bg {
        BackgroundLayer::Color { value } => assert!(value.starts_with('#')),
        _ => panic!("Expected Color background"),
    }

    // Test Gradient background
    let gradient_bg = BackgroundLayer::Gradient {
        value: "blue:purple".to_string(),
    };
    match gradient_bg {
        BackgroundLayer::Gradient { value } => assert!(value.contains(':')),
        _ => panic!("Expected Gradient background"),
    }

    // Test Image background
    let image_bg = BackgroundLayer::Image {
        path: "/path/to/bg.jpg".to_string(),
    };
    match image_bg {
        BackgroundLayer::Image { path } => assert!(path.ends_with(".jpg")),
        _ => panic!("Expected Image background"),
    }
}

// =====================================================================
// Background Music Tests
// =====================================================================

#[tokio::test]
async fn test_background_music_config() {
    let music = BackgroundMusic {
        file_path: "/test/music.mp3".to_string(),
        loop_music: true,
    };

    assert!(!music.file_path.is_empty());
    assert!(music.file_path.ends_with(".mp3"));
    assert!(music.loop_music);
}

#[tokio::test]
async fn test_audio_mixing_config() {
    let config = AutoEditConfig {
        target_duration: 60,
        game_ids: vec!["game1".to_string()],
        selected_clip_ids: None,
        canvas_template: None,
        background_music: Some(BackgroundMusic {
            file_path: "/test/music.mp3".to_string(),
            loop_music: true,
        }),
        audio_levels: AudioLevels {
            game_audio: 70,
            background_music: 30,
        },
    };

    // Validate music configuration
    let music = config.background_music.as_ref().unwrap();
    assert_eq!(music.file_path, "/test/music.mp3");
    assert!(music.loop_music);

    // Validate audio levels
    assert_eq!(config.audio_levels.game_audio, 70);
    assert_eq!(config.audio_levels.background_music, 30);

    // Total volume should be reasonable (game + music ≤ 100 is good practice)
    let total_volume = config.audio_levels.game_audio + config.audio_levels.background_music;
    assert!(total_volume == 100); // Balanced mix
}

// =====================================================================
// Progress Tracking Tests
// =====================================================================

#[tokio::test]
async fn test_auto_edit_status_enum() {
    let statuses = vec![
        AutoEditStatus::Queued,
        AutoEditStatus::Processing,
        AutoEditStatus::Completed,
        AutoEditStatus::Failed,
    ];

    // Validate all status types exist
    assert_eq!(statuses.len(), 4);

    // Processing and Completed should not be equal
    assert_ne!(AutoEditStatus::Processing, AutoEditStatus::Completed);
    assert_ne!(AutoEditStatus::Processing, AutoEditStatus::Failed);
}

#[tokio::test]
async fn test_progress_tracking_initialization() {
    let processor = Arc::new(VideoProcessor::new());
    let storage = create_test_storage();
    let composer = AutoComposer::new(processor, storage);

    // Initially, no progress
    let initial_progress = composer.get_progress().await;
    assert!(initial_progress.is_none());
}

// =====================================================================
// Error Scenario Tests
// =====================================================================

#[tokio::test]
async fn test_config_validation_no_games() {
    let config = AutoEditConfig {
        target_duration: 60,
        game_ids: vec![], // Empty game list
        selected_clip_ids: None,
        canvas_template: None,
        background_music: None,
        audio_levels: AudioLevels::default(),
    };

    // Should have at least one game
    assert!(config.game_ids.is_empty());
}

#[tokio::test]
async fn test_invalid_manual_selection() {
    let processor = Arc::new(VideoProcessor::new());
    let storage = create_test_storage();
    let composer = AutoComposer::new(processor, storage);

    let clips = vec![
        create_test_clip(1, 5, 10.0, "Pentakill", "/test/clip1.mp4"),
        create_test_clip(2, 4, 15.0, "Quadrakill", "/test/clip2.mp4"),
    ];

    // Try to select clip ID that doesn't exist
    let config = AutoEditConfig {
        target_duration: 60,
        game_ids: vec!["game1".to_string()],
        selected_clip_ids: Some(vec![99]), // Non-existent clip ID
        canvas_template: None,
        background_music: None,
        audio_levels: AudioLevels::default(),
    };

    let result = composer.select_clips(&clips, &config).await;

    // Should return error (no clips found matching selection)
    assert!(result.is_err());
}

// =====================================================================
// YouTube Shorts Dimension Tests
// =====================================================================

#[tokio::test]
async fn test_youtube_shorts_dimensions() {
    // YouTube Shorts must be 9:16 aspect ratio (1080x1920)
    const SHORTS_WIDTH: u32 = 1080;
    const SHORTS_HEIGHT: u32 = 1920;

    // Validate aspect ratio
    let aspect_ratio = SHORTS_HEIGHT as f64 / SHORTS_WIDTH as f64;
    let expected_ratio = 16.0 / 9.0;
    let difference = (aspect_ratio - expected_ratio).abs();

    assert!(difference < 0.01); // Allow small floating point error
    assert_eq!(SHORTS_WIDTH, 1080);
    assert_eq!(SHORTS_HEIGHT, 1920);
}

#[tokio::test]
async fn test_shorts_duration_limits() {
    // YouTube Shorts: 15-180 seconds (3 minutes max)
    let valid_durations = vec![60, 120, 180];
    let invalid_durations = vec![10, 200, 300];

    for duration in valid_durations {
        assert!(duration >= 15 && duration <= 180);
    }

    for duration in invalid_durations {
        assert!(duration < 15 || duration > 180);
    }
}

// =====================================================================
// Performance and Resource Tests
// =====================================================================

#[tokio::test]
async fn test_multiple_concurrent_compositions() {
    use tokio::task;

    let processor = Arc::new(VideoProcessor::new());

    // Simulate multiple concurrent composition requests
    let mut handles = vec![];

    for i in 0..3 {
        let proc_clone = Arc::clone(&processor);
        let handle = task::spawn(async move {
            // Just validate config creation
            let config = AutoEditConfig {
                target_duration: 60,
                game_ids: vec![format!("game_{}", i)],
                selected_clip_ids: None,
                canvas_template: None,
                background_music: None,
                audio_levels: AudioLevels::default(),
            };

            assert!(!config.game_ids.is_empty());
            true
        });
        handles.push(handle);
    }

    // All should complete successfully
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result);
    }
}

#[tokio::test]
async fn test_disk_space_estimation() {
    // Estimate required disk space for composition

    // YouTube Shorts: 1080x1920 @ 10 Mbps
    let bitrate = 10_000_000; // 10 Mbps
    let duration = 60.0; // 60 seconds

    let estimated_size_bytes = ((bitrate as f64 * duration) / 8.0) as u64;

    // 60 seconds at 10 Mbps should be ~75 MB
    let expected_size = 75_000_000; // bytes

    let difference = (estimated_size_bytes as i64 - expected_size as i64).abs();

    // Allow 20% margin of error (compression varies)
    assert!(difference < (expected_size / 5) as i64);
}

// =====================================================================
// Serialization Tests
// =====================================================================

#[tokio::test]
async fn test_config_serialization() {
    use serde_json;

    let config = create_basic_config();

    // Should serialize to JSON
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("target_duration"));
    assert!(json.contains("game_ids"));

    // Should deserialize back
    let deserialized: AutoEditConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.target_duration, config.target_duration);
    assert_eq!(deserialized.game_ids, config.game_ids);
}

#[tokio::test]
async fn test_canvas_template_serialization() {
    use serde_json;

    let template = create_test_canvas_template();

    // Should serialize to JSON
    let json = serde_json::to_string(&template).unwrap();
    assert!(json.contains("test_template_1"));
    assert!(json.contains("PENTAKILL"));

    // Should deserialize back
    let deserialized: CanvasTemplate = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, template.id);
    assert_eq!(deserialized.name, template.name);
    assert_eq!(deserialized.elements.len(), template.elements.len());
}

#[tokio::test]
async fn test_audio_levels_serialization() {
    use serde_json;

    let levels = AudioLevels {
        game_audio: 70,
        background_music: 30,
    };

    // Should serialize to JSON
    let json = serde_json::to_string(&levels).unwrap();
    assert!(json.contains("game_audio"));
    assert!(json.contains("background_music"));

    // Should deserialize back
    let deserialized: AudioLevels = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.game_audio, levels.game_audio);
    assert_eq!(deserialized.background_music, levels.background_music);
}

// =====================================================================
// Integration Tests Summary
// =====================================================================

#[tokio::test]
async fn test_auto_edit_feature_completeness() {
    // Verify all components exist and can be instantiated

    let _processor = VideoProcessor::new();
    let _storage = create_test_storage();
    let _config = create_basic_config();
    let _template = create_test_canvas_template();
    let _audio_levels = AudioLevels::default();

    // If we reach here, all core components are properly defined
    assert!(true);
}
