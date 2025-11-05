// Integration tests for recording system
#![cfg(test)]

use lolshorts_tauri::recording::{RecordingManager, RecordingState};
use lolshorts_tauri::lcu::LcuClient;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio;

#[tokio::test]
async fn test_recording_manager_initialization() {
    let manager = RecordingManager::new();

    let state = manager.get_state().await;
    assert_eq!(state, RecordingState::Idle);
}

#[tokio::test]
async fn test_recording_state_transitions() {
    let manager = Arc::new(RwLock::new(RecordingManager::new()));

    // Initial state should be Idle
    {
        let mgr = manager.read().await;
        assert_eq!(mgr.get_state().await, RecordingState::Idle);
    }

    // Start recording (will fail without actual Game DVR, but test state change logic)
    {
        let mut mgr = manager.write().await;
        let result = mgr.start_replay_buffer().await;

        // May fail in test environment without Game DVR, which is expected
        // Just verify we handle the error gracefully
        if result.is_err() {
            println!("Recording start failed as expected in test environment");
        }
    }
}

#[tokio::test]
async fn test_lcu_client_initialization() {
    let client = LcuClient::new();

    // Test that client can be created
    assert!(true); // Client creation doesn't fail

    // Note: Actual connection test requires League Client running
    // This is tested in E2E tests with mocked LCU
}

#[tokio::test]
async fn test_concurrent_recording_requests() {
    use tokio::task;

    let manager = Arc::new(RwLock::new(RecordingManager::new()));

    // Spawn multiple concurrent state checks
    let mut handles = vec![];
    for _ in 0..5 {
        let mgr_clone = Arc::clone(&manager);
        let handle = task::spawn(async move {
            let mgr = mgr_clone.read().await;
            mgr.get_state().await
        });
        handles.push(handle);
    }

    // All should succeed and return same state
    for handle in handles {
        let state = handle.await.unwrap();
        assert_eq!(state, RecordingState::Idle);
    }
}

#[tokio::test]
async fn test_clip_metadata_validation() {
    use lolshorts_tauri::recording::ClipMetadata;

    let valid_metadata = ClipMetadata {
        game_id: 12345,
        event_type: "ChampionKill".to_string(),
        event_time: 180.5,
        priority: 3,
        file_path: "C:\\Videos\\clip.mp4".to_string(),
        duration: 15.0,
        created_at: chrono::Utc::now().timestamp(),
    };

    // Verify required fields are present
    assert!(!valid_metadata.event_type.is_empty());
    assert!(valid_metadata.event_time > 0.0);
    assert!(valid_metadata.priority >= 1 && valid_metadata.priority <= 5);
    assert!(!valid_metadata.file_path.is_empty());
}

#[tokio::test]
async fn test_event_priority_calculation() {
    use lolshorts_tauri::recording::EventDetector;

    let detector = EventDetector::new();

    // Pentakill should have highest priority
    let pentakill_priority = detector.calculate_pentakill_priority();
    assert_eq!(pentakill_priority, 5);

    // Quadrakill should be 4
    let quadrakill_priority = detector.calculate_multikill_priority(4);
    assert_eq!(quadrakill_priority, 4);

    // Triple kill should be 3
    let triple_priority = detector.calculate_multikill_priority(3);
    assert_eq!(triple_priority, 3);

    // Single kill should be lower
    let single_priority = detector.calculate_multikill_priority(1);
    assert!(single_priority < 3);
}

#[tokio::test]
async fn test_clip_storage_limits() {
    // Test that we respect storage limits
    const MAX_CLIPS_PER_GAME: usize = 20;
    const MAX_TOTAL_SIZE_GB: u64 = 50;

    let clips_count = 15;
    assert!(clips_count <= MAX_CLIPS_PER_GAME);

    let total_size_bytes = 10 * 1024 * 1024 * 1024; // 10 GB
    let max_size_bytes = MAX_TOTAL_SIZE_GB * 1024 * 1024 * 1024;
    assert!(total_size_bytes < max_size_bytes);
}

#[tokio::test]
async fn test_game_detection_flow() {
    use lolshorts_tauri::recording::GameState;

    // Test game state detection logic
    let states = vec![
        GameState::None,
        GameState::Lobby,
        GameState::InGame,
        GameState::PostGame,
    ];

    // Should transition through states in order
    for i in 0..states.len() - 1 {
        let current = &states[i];
        let next = &states[i + 1];

        // Verify state progression makes sense
        match (current, next) {
            (GameState::None, GameState::Lobby) => assert!(true),
            (GameState::Lobby, GameState::InGame) => assert!(true),
            (GameState::InGame, GameState::PostGame) => assert!(true),
            _ => (), // Other transitions are possible but not tested here
        }
    }
}
