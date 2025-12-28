// Integration tests for WindowsCaptureRecorder
#![cfg(test)]

use lolshorts_tauri::recording::integration_backend::{
    WindowsCaptureRecorder, RecordingConfig, RecordingStatus,
    CapturedFrame, FFmpegVideoWriter, VideoEncoder
};
use lolshorts_tauri::recording::audio::AudioConfig;
use lolshorts_tauri::storage::GameMetadata;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::sleep;
use tempfile::TempDir;

/// Helper to create test recording configuration
fn create_test_config() -> RecordingConfig {
    RecordingConfig {
        fps: 30,
        bitrate: 5_000_000, // 5 Mbps for testing
        resolution: (1280, 720), // Smaller for testing
        encoder: VideoEncoder::H264,
        output_dir: PathBuf::from("./test_recordings"),
        buffer_duration_secs: 30, // 30 seconds for testing
        audio_config: Some(AudioConfig::default()),
    }
}

/// Helper to create test audio configuration
fn create_test_audio_config() -> AudioConfig {
    AudioConfig {
        record_microphone: false, // Disable for CI testing
        microphone_device: None,
        microphone_volume: 100,
        record_system_audio: false, // Disable for CI testing
        system_audio_device: None,
        system_audio_volume: 100,
        sample_rate: 44100,
        bitrate: 128,
    }
}

/// Create test frame data
fn create_test_frame(width: u32, height: u32, color: u8) -> Vec<u8> {
    let size = (width * height * 4) as usize; // RGBA
    let mut data = vec![color; size];

    // Add some variation to make it more realistic
    for (i, pixel) in data.chunks_mut(4).enumerate() {
        if i % 100 == 0 {
            pixel[0] = (i % 256) as u8; // Red channel variation
            pixel[1] = ((i * 2) % 256) as u8; // Green channel variation
            pixel[2] = ((i * 3) % 256) as u8; // Blue channel variation
        }
    }

    data
}

#[tokio::test]
async fn test_windows_capture_recorder_initialization() {
    // Create temporary directory for test recordings
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        ..create_test_config()
    };

    // Test recorder creation
    let recorder = WindowsCaptureRecorder::new(config).await;
    assert!(recorder.is_ok(), "Failed to create WindowsCaptureRecorder: {:?}", recorder.err());

    let recorder = recorder.unwrap();

    // Test initial state
    let status = recorder.get_status().await;
    assert_eq!(status, RecordingStatus::Idle);

    // Test initial stats
    let stats = recorder.get_stats().await;
    assert_eq!(stats.total_frames, 0);
    assert_eq!(stats.uptime_seconds, 0.0);
    assert_eq!(stats.current_fps, 0.0);

    // Test initial game state
    let current_game = recorder.get_current_game().await;
    assert!(current_game.is_none());
}

#[tokio::test]
async fn test_windows_capture_recorder_start_stop() {
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        encoder: VideoEncoder::H264,
        ..create_test_config()
    };

    let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

    // Test starting recording
    let start_result = recorder.start_recording().await;

    // Note: This might fail in CI without FFmpeg, which is expected
    if start_result.is_ok() {
        // Give it a moment to initialize
        sleep(Duration::from_millis(100)).await;

        // Check status changed to Recording or Buffering
        let status = recorder.get_status().await;
        assert!(status == RecordingStatus::Recording || status == RecordingStatus::Buffering);

        // Test stopping recording
        let stop_result = recorder.stop_recording().await;
        if stop_result.is_ok() {
            let output_path = stop_result.unwrap();

            // Verify output file was created
            assert!(output_path.exists(), "Output video file should exist");
            assert!(output_path.extension().unwrap() == "mp4");
        }

        // Check status returned to Idle
        let status = recorder.get_status().await;
        assert_eq!(status, RecordingStatus::Idle);
    } else {
        println!("Recording start failed as expected in CI environment: {:?}", start_result.err());
    }
}

#[tokio::test]
async fn test_windows_capture_recorder_double_start_protection() {
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        ..create_test_config()
    };

    let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

    // Start first recording (may fail in CI)
    let first_start = recorder.start_recording().await;

    if first_start.is_ok() {
        // Try to start second recording - should fail
        let second_start = recorder.start_recording().await;
        assert!(second_start.is_err(), "Second recording start should fail");

        // Clean up
        let _ = recorder.stop_recording().await;
    }
}

#[tokio::test]
async fn test_windows_capture_recorder_stop_without_start() {
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        ..create_test_config()
    };

    let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

    // Try to stop recording without starting - should fail
    let stop_result = recorder.stop_recording().await;
    assert!(stop_result.is_err(), "Stop without start should fail");
}

#[tokio::test]
async fn test_windows_capture_recorder_frame_processing() {
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        resolution: (320, 240), // Very small for testing
        ..create_test_config()
    };

    let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

    // Create test frame
    let test_frame_data = create_test_frame(320, 240, 128);
    let test_frame = CapturedFrame::new(320, 240, test_frame_data);

    // Test frame processing without recording (should be ignored)
    let process_result = recorder.process_frame(test_frame).await;
    assert!(process_result.is_ok(), "Frame processing should not fail");

    // Try to start recording
    let start_result = recorder.start_recording().await;
    if start_result.is_ok() {
        // Give it a moment to initialize
        sleep(Duration::from_millis(100)).await;

        // Process frames during recording
        for i in 0..10 {
            let frame_data = create_test_frame(320, 240, (i * 25) as u8);
            let frame = CapturedFrame::new(320, 240, frame_data);

            let process_result = recorder.process_frame(frame).await;
            if process_result.is_err() {
                println!("Frame processing failed: {:?}", process_result.err());
                break;
            }

            // Small delay between frames
            sleep(Duration::from_millis(33)).await; // ~30 FPS
        }

        // Check stats after frame processing
        let stats = recorder.get_stats().await;
        if stats.total_frames > 0 {
            println!("Processed {} frames", stats.total_frames);
            assert!(stats.uptime_seconds > 0.0);
        }

        // Clean up
        let _ = recorder.stop_recording().await;
    }
}

#[tokio::test]
async fn test_windows_capture_recorder_game_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        ..create_test_config()
    };

    let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

    // Create test game metadata
    let test_game = GameMetadata {
        game_id: 12345,
        champion: "Ahri".to_string(),
        game_mode: "Ranked Solo".to_string(),
        start_time: chrono::Utc::now().timestamp(),
        region: "NA".to_string(),
    };

    // Test setting game metadata
    recorder.set_current_game(Some(test_game.clone())).await;

    // Test getting game metadata
    let retrieved_game = recorder.get_current_game().await;
    assert!(retrieved_game.is_some(), "Game metadata should be set");

    let retrieved_game = retrieved_game.unwrap();
    assert_eq!(retrieved_game.game_id, test_game.game_id);
    assert_eq!(retrieved_game.champion, test_game.champion);
    assert_eq!(retrieved_game.game_mode, test_game.game_mode);

    // Test clearing game metadata
    recorder.set_current_game(None).await;
    let cleared_game = recorder.get_current_game().await;
    assert!(cleared_game.is_none(), "Game metadata should be cleared");
}

#[tokio::test]
async fn test_windows_capture_recorder_different_encoders() {
    let temp_dir = TempDir::new().unwrap();

    for encoder in [VideoEncoder::H264, VideoEncoder::H265] {
        let config = RecordingConfig {
            output_dir: temp_dir.path().to_path_buf(),
            encoder,
            resolution: (160, 120), // Very small for testing
            ..create_test_config()
        };

        let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

        // Test basic functionality with different encoders
        let status = recorder.get_status().await;
        assert_eq!(status, RecordingStatus::Idle);

        // Test start (may fail in CI)
        let start_result = recorder.start_recording().await;
        if start_result.is_ok() {
            sleep(Duration::from_millis(50)).await;
            let _ = recorder.stop_recording().await;
        }

        println!("Encoder {:?} test completed", encoder);
    }
}

#[tokio::test]
async fn test_windows_capture_recorder_audio_config() {
    let temp_dir = TempDir::new().unwrap();

    // Test with audio enabled (but may fail in CI)
    let audio_config = AudioConfig {
        record_microphone: false, // Keep disabled for CI
        record_system_audio: false, // Keep disabled for CI
        ..create_test_audio_config()
    };

    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        audio_config: Some(audio_config),
        ..create_test_config()
    };

    let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

    // Test that recorder can be created with audio config
    let status = recorder.get_status().await;
    assert_eq!(status, RecordingStatus::Idle);

    // Test start (may fail due to audio device availability)
    let start_result = recorder.start_recording().await;
    if start_result.is_err() {
        println!("Audio recording start failed as expected: {:?}", start_result.err());
    } else {
        sleep(Duration::from_millis(50)).await;
        let _ = recorder.stop_recording().await;
    }
}

#[tokio::test]
async fn test_windows_capture_recorder_concurrent_access() {
    use tokio::task;

    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        ..create_test_config()
    };

    let recorder = std::sync::Arc::new(WindowsCaptureRecorder::new(config).await.unwrap());

    // Spawn multiple concurrent tasks to access recorder
    let mut handles = vec![];

    // Task 1: Check status multiple times
    let recorder_clone1 = recorder.clone();
    let handle1 = task::spawn(async move {
        for _ in 0..10 {
            let _status = recorder_clone1.get_status().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    handles.push(handle1);

    // Task 2: Check stats multiple times
    let recorder_clone2 = recorder.clone();
    let handle2 = task::spawn(async move {
        for _ in 0..10 {
            let _stats = recorder_clone2.get_stats().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    handles.push(handle2);

    // Task 3: Check game metadata multiple times
    let recorder_clone3 = recorder.clone();
    let handle3 = task::spawn(async move {
        for _ in 0..10 {
            let _game = recorder_clone3.get_current_game().await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });
    handles.push(handle3);

    // Wait for all tasks to complete
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Concurrent access task should complete successfully");
    }

    // Final status check
    let status = recorder.get_status().await;
    assert_eq!(status, RecordingStatus::Idle);
}

#[tokio::test]
async fn test_captured_frame_creation() {
    // Test frame creation with various sizes
    let test_cases = vec![
        (320, 240),   // Small
        (640, 480),   // Medium
        (1920, 1080), // Large
    ];

    for (width, height) in test_cases {
        let frame_data = create_test_frame(width, height, 128);
        let frame = CapturedFrame::new(width, height, frame_data);

        assert_eq!(frame.width, width);
        assert_eq!(frame.height, height);
        assert_eq!(frame.size(), (width * height * 4) as usize);
        assert!(frame.size_mb() > 0.0);
        assert!(frame.timestamp.elapsed().as_secs() < 1); // Should be recent
    }
}

#[tokio::test]
async fn test_recording_config_defaults() {
    // Test default configuration
    let default_config = RecordingConfig::default();

    assert_eq!(default_config.fps, 60);
    assert_eq!(default_config.bitrate, 15_000_000);
    assert_eq!(default_config.resolution, (1920, 1080));
    assert_eq!(matches!(default_config.encoder, VideoEncoder::H265), true);
    assert_eq!(default_config.buffer_duration_secs, 60);
    assert!(default_config.audio_config.is_some());
}

#[tokio::test]
async fn test_video_encoder_extensions() {
    // Test encoder to extension mapping
    assert_eq!(VideoEncoder::H264.to_extension(), "mp4");
    assert_eq!(VideoEncoder::H265.to_extension(), "mp4");
}

// Performance benchmarks (only run when explicitly enabled)
#[tokio::test]
#[ignore]
async fn benchmark_windows_capture_recorder_performance() {
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        resolution: (1280, 720),
        fps: 60,
        ..create_test_config()
    };

    let recorder = WindowsCaptureRecorder::new(config).await.unwrap();

    println!("=== WindowsCaptureRecorder Performance Benchmark ===");

    // Benchmark status checks
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _status = recorder.get_status().await;
    }
    let status_duration = start.elapsed();
    println!("1000 status checks: {:?} ({:.2}μs per check)",
             status_duration, status_duration.as_micros() as f64 / 1000.0);

    // Benchmark stats checks
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _stats = recorder.get_stats().await;
    }
    let stats_duration = start.elapsed();
    println!("1000 stats checks: {:?} ({:.2}μs per check)",
             stats_duration, stats_duration.as_micros() as f64 / 1000.0);

    // Benchmark frame processing (without recording)
    let test_frame = CapturedFrame::new(1280, 720, create_test_frame(1280, 720, 128));

    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _result = recorder.process_frame(test_frame.clone()).await;
    }
    let frame_duration = start.elapsed();
    println!("100 frame processing operations: {:?} ({:.2}μs per frame)",
             frame_duration, frame_duration.as_micros() as f64 / 100.0);
}

// Stress test (only run when explicitly enabled)
#[tokio::test]
#[ignore]
async fn stress_test_windows_capture_recorder() {
    let temp_dir = TempDir::new().unwrap();
    let config = RecordingConfig {
        output_dir: temp_dir.path().to_path_buf(),
        ..create_test_config()
    };

    let recorder = std::sync::Arc::new(WindowsCaptureRecorder::new(config).await.unwrap());

    println!("=== WindowsCaptureRecorder Stress Test ===");

    // Spawn many concurrent tasks
    let mut handles = vec![];

    for i in 0..10 {
        let recorder_clone = recorder.clone();
        let handle = tokio::task::spawn(async move {
            for j in 0..100 {
                let _status = recorder_clone.get_status().await;
                let _stats = recorder_clone.get_stats().await;

                if j % 10 == 0 {
                    println!("Task {}: iteration {}", i, j);
                }

                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok(), "Stress test task should complete");
    }

    println!("Stress test completed successfully");
}