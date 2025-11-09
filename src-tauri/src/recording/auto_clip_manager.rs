#![allow(clippy::unnecessary_cast)]
use anyhow::{Context as AnyhowContext, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex as TokioMutex, RwLock as TokioRwLock};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use super::live_client::{EventTrigger, LiveClientMonitor};
use super::windows_backend::WindowsRecorder;
use super::GameEvent; // Use the recording module's GameEvent
use crate::settings::models::RecordingSettings;
use crate::storage::{
    models::{ClipMetadata, EventData, EventType},
    Storage,
};

/// Queued event with timestamp for merging logic
#[derive(Debug, Clone)]
struct QueuedEvent {
    trigger: EventTrigger,
    event: GameEvent,
    received_at: Instant,
}

/// Event window after merging consecutive events
#[derive(Debug, Clone)]
struct EventWindow {
    primary_trigger: EventTrigger,
    events: Vec<GameEvent>,
    start_time: f32, // Game time in seconds
    end_time: f32,   // Game time in seconds
    priority: u8,    // Highest priority in window
}

/// Auto Clip Manager - Bridges event detection with automatic clip saving
///
/// Architecture:
/// LiveClientMonitor → AutoClipManager → WindowsRecorder + Storage
///                           ↓
///                      Settings (filter)
///
/// Responsibilities:
/// 1. Event Queue: Receive events from LiveClientMonitor
/// 2. Event Filtering: Apply settings filters (event types, priority, game modes)
/// 3. Event Merging: Combine consecutive events within threshold
/// 4. Clip Window Calculation: Calculate pre/post durations from settings or defaults
/// 5. Automatic Saving: Trigger WindowsRecorder.save_clip() for filtered events
/// 6. Metadata Generation: Create rich metadata for each saved clip
pub struct AutoClipManager {
    /// Recording backend reference
    recorder: Arc<TokioRwLock<WindowsRecorder>>,

    /// Storage reference
    storage: Arc<Storage>,

    /// Settings reference
    settings: Arc<TokioRwLock<RecordingSettings>>,

    /// Event queue for merging
    event_queue: Arc<TokioMutex<VecDeque<QueuedEvent>>>,

    /// Current game ID for clip organization
    current_game_id: Arc<TokioRwLock<Option<String>>>,

    /// Processing lock to prevent concurrent clip saves
    processing_lock: Arc<TokioMutex<()>>,

    /// Event monitoring task handle
    monitor_task: Arc<TokioMutex<Option<JoinHandle<()>>>>,

    /// Cancellation token for stopping the monitoring task
    cancel_token: CancellationToken,
}

impl AutoClipManager {
    /// Create a new Auto Clip Manager
    pub fn new(
        recorder: Arc<TokioRwLock<WindowsRecorder>>,
        storage: Arc<Storage>,
        settings: Arc<TokioRwLock<RecordingSettings>>,
    ) -> Self {
        Self {
            recorder,
            storage,
            settings,
            event_queue: Arc::new(TokioMutex::new(VecDeque::new())),
            current_game_id: Arc::new(TokioRwLock::new(None)),
            processing_lock: Arc::new(TokioMutex::new(())),
            monitor_task: Arc::new(TokioMutex::new(None)),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Set the current game ID for clip organization
    pub async fn set_current_game(&self, game_id: Option<String>) {
        let mut current = self.current_game_id.write().await;
        *current = game_id.clone();

        if let Some(ref id) = game_id {
            info!("Auto Clip Manager: tracking game {}", id);
        } else {
            info!("Auto Clip Manager: game ended, clearing queue");
            // Clear event queue when game ends
            let mut queue = self.event_queue.lock().await;
            queue.clear();
        }
    }

    /// Check if event monitoring is active
    pub async fn is_monitoring(&self) -> bool {
        let task_guard = self.monitor_task.lock().await;
        task_guard.is_some()
    }

    /// Start event monitoring from Live Client API
    ///
    /// This spawns a background task that continuously polls the Live Client API
    /// for game events and automatically processes them through the clip pipeline.
    pub async fn start_event_monitoring(&self) -> Result<()> {
        // Check if already monitoring
        let mut task_guard = self.monitor_task.lock().await;
        if task_guard.is_some() {
            info!("Event monitoring already running");
            return Ok(());
        }

        info!("Starting event monitoring...");

        // Create a new LiveClientMonitor
        let mut monitor = LiveClientMonitor::new().context("Failed to create LiveClientMonitor")?;

        // Clone Arc references for the monitoring task
        let event_queue = Arc::clone(&self.event_queue);
        let settings = Arc::clone(&self.settings);
        let recorder = Arc::clone(&self.recorder);
        let storage = Arc::clone(&self.storage);
        let current_game_id = Arc::clone(&self.current_game_id);
        let processing_lock = Arc::clone(&self.processing_lock);
        let cancel_token = self.cancel_token.clone();

        // Spawn monitoring task
        let handle = tokio::spawn(async move {
            info!("Event monitoring task started");

            // Create callback closure that processes events
            let callback =
                move |trigger: EventTrigger, live_event: super::live_client::GameEvent| {
                    // Convert live_client::GameEvent to recording::GameEvent
                    let event = convert_live_event(live_event, &trigger);

                    // Clone Arc references for the async block
                    let event_queue = Arc::clone(&event_queue);
                    let settings = Arc::clone(&settings);
                    let recorder = Arc::clone(&recorder);
                    let storage = Arc::clone(&storage);
                    let current_game_id = Arc::clone(&current_game_id);
                    let processing_lock = Arc::clone(&processing_lock);

                    // Spawn a task to process the event asynchronously
                    tokio::spawn(async move {
                        // Create a temporary AutoClipManager instance for processing
                        let temp_manager = AutoClipManager {
                            recorder,
                            storage,
                            settings,
                            event_queue,
                            current_game_id,
                            processing_lock,
                            monitor_task: Arc::new(TokioMutex::new(None)),
                            cancel_token: CancellationToken::new(),
                        };

                        if let Err(e) = temp_manager
                            .process_event(trigger.clone(), event.clone())
                            .await
                        {
                            error!("Failed to process event {:?}: {}", trigger, e);
                        }
                    });
                };

            // Run the monitor until cancelled
            let monitoring = monitor.start_monitoring(callback);
            tokio::select! {
                result = monitoring => {
                    if let Err(e) = result {
                        error!("Event monitoring error: {}", e);
                    }
                }
                _ = cancel_token.cancelled() => {
                    info!("Event monitoring cancelled");
                }
            }

            info!("Event monitoring task stopped");
        });

        *task_guard = Some(handle);
        info!("Event monitoring started successfully");

        Ok(())
    }

    /// Stop event monitoring
    ///
    /// This cancels the background monitoring task and waits for it to finish.
    pub async fn stop_event_monitoring(&self) -> Result<()> {
        info!("Stopping event monitoring...");

        // Cancel the monitoring task
        self.cancel_token.cancel();

        // Get and wait for the task to finish
        let mut task_guard = self.monitor_task.lock().await;
        if let Some(handle) = task_guard.take() {
            handle.await.context("Failed to join monitoring task")?;
            info!("Event monitoring stopped successfully");
        } else {
            info!("Event monitoring was not running");
        }

        Ok(())
    }

    /// Process an event from LiveClientMonitor
    ///
    /// This is the main entry point called by the event detection callback.
    /// Events are filtered, queued, merged, and automatically saved.
    pub async fn process_event(&self, trigger: EventTrigger, event: GameEvent) -> Result<()> {
        debug!(
            "Auto Clip Manager: processing event {} (priority: {})",
            event.event_name,
            trigger.priority()
        );

        // Check if we should record this event based on settings
        if !self.should_record_event(&trigger, &event).await? {
            debug!(
                "Event filtered out by settings: {} (priority: {})",
                event.event_name,
                trigger.priority()
            );
            return Ok(());
        }

        // Add event to queue
        let queued = QueuedEvent {
            trigger: trigger.clone(),
            event: event.clone(),
            received_at: Instant::now(),
        };

        {
            let mut queue = self.event_queue.lock().await;
            queue.push_back(queued);
        }

        // Check if we should merge events or save immediately
        let settings = self.settings.read().await;

        if settings.clip_timing.merge_consecutive_events {
            // Wait for merge window to close before processing
            self.try_process_merged_events().await?;
        } else {
            // Save immediately without merging
            self.save_single_event(trigger, event).await?;
        }

        Ok(())
    }

    /// Check if event should be recorded based on settings
    async fn should_record_event(&self, trigger: &EventTrigger, _event: &GameEvent) -> Result<bool> {
        let settings = self.settings.read().await;

        // Check priority threshold
        let event_priority = trigger.priority();
        if event_priority < settings.event_filter.min_priority {
            return Ok(false);
        }

        // Check event type filters
        let should_record = match trigger {
            EventTrigger::ChampionKill => settings.event_filter.record_kills,
            EventTrigger::Multikill(_) => settings.event_filter.record_multikills,
            EventTrigger::DragonKill => settings.event_filter.record_dragon,
            EventTrigger::BaronKill => settings.event_filter.record_baron,
            EventTrigger::TurretKill => settings.event_filter.record_turret,
            EventTrigger::InhibitorKill => settings.event_filter.record_inhibitor,
            EventTrigger::Ace => settings.event_filter.record_ace,
            EventTrigger::Steal => settings.event_filter.record_steal,
            EventTrigger::ClutchPlay => true, // Always record clutch plays if detected
        };

        Ok(should_record)
    }

    /// Try to process merged events if merge window has closed
    async fn try_process_merged_events(&self) -> Result<()> {
        let settings = self.settings.read().await;
        let merge_threshold = settings.clip_timing.merge_time_threshold as u64;
        drop(settings);

        let mut queue = self.event_queue.lock().await;

        // Check if oldest event is outside merge window
        if let Some(oldest) = queue.front() {
            let age = oldest.received_at.elapsed().as_secs();

            if age >= merge_threshold {
                // Merge window closed - process events
                let events_to_merge: Vec<QueuedEvent> = queue.drain(..).collect();
                drop(queue);

                if !events_to_merge.is_empty() {
                    self.process_event_window(events_to_merge).await?;
                }
            }
        }

        Ok(())
    }

    /// Process a window of merged events
    async fn process_event_window(&self, events: Vec<QueuedEvent>) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        // Create event window
        let window = self.merge_events(&events);

        info!(
            "Merged {} events into window: {:?} (priority: {}, duration: {:.1}s)",
            events.len(),
            window.primary_trigger,
            window.priority,
            window.end_time - window.start_time
        );

        // Save the merged clip
        self.save_event_window(window).await?;

        Ok(())
    }

    /// Merge consecutive events into a single window
    fn merge_events(&self, events: &[QueuedEvent]) -> EventWindow {
        // Find highest priority event
        let primary_event = events.iter().max_by_key(|e| e.trigger.priority()).unwrap();

        let priority = primary_event.trigger.priority();

        // Calculate time range
        let start_time = events
            .iter()
            .map(|e| e.event.event_time)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let end_time = events
            .iter()
            .map(|e| e.event.event_time)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        EventWindow {
            primary_trigger: primary_event.trigger.clone(),
            events: events.iter().map(|e| e.event.clone()).collect(),
            start_time: start_time as f32,
            end_time: end_time as f32,
            priority,
        }
    }

    /// Save a single event without merging
    async fn save_single_event(&self, trigger: EventTrigger, event: GameEvent) -> Result<()> {
        // Prevent concurrent saves
        let _lock = self.processing_lock.lock().await;

        let settings = self.settings.read().await;

        // Calculate clip window duration
        let clip_window = self.calculate_clip_window(&trigger, &settings);
        drop(settings);

        let total_duration = clip_window.pre_duration as f64 + clip_window.post_duration as f64;

        info!(
            "Saving clip for event: {} (priority: {}, duration: {:.1}s)",
            event.event_name,
            trigger.priority(),
            total_duration
        );

        // Generate clip ID
        let clip_id = format!("{}_{}", event.event_name, event.event_time as u32);

        // Save clip via WindowsRecorder
        let clip_path = self
            .recorder
            .read()
            .await
            .save_clip(&event, clip_id.clone(), trigger.priority(), total_duration)
            .await
            .context("Failed to save clip via recorder")?;

        info!("Clip saved: {:?}", clip_path);

        // Save metadata to storage
        self.save_clip_metadata(&clip_id, &event, trigger.priority(), &clip_path)
            .await?;

        Ok(())
    }

    /// Save an event window (merged events)
    async fn save_event_window(&self, window: EventWindow) -> Result<()> {
        // Prevent concurrent saves
        let _lock = self.processing_lock.lock().await;

        let settings = self.settings.read().await;

        // Calculate clip window for primary event
        let clip_window = self.calculate_clip_window(&window.primary_trigger, &settings);
        drop(settings);

        // Extend duration to cover the full event window
        let event_window_duration = window.end_time - window.start_time;
        let total_duration = clip_window.pre_duration as f64
            + event_window_duration as f64
            + clip_window.post_duration as f64;

        info!(
            "Saving merged clip: {:?} ({} events, priority: {}, duration: {:.1}s)",
            window.primary_trigger,
            window.events.len(),
            window.priority,
            total_duration
        );

        // Use primary event for clip generation
        let primary_event = &window.events[0];
        let clip_id = format!(
            "merged_{}_{}",
            window.start_time as u32, window.end_time as u32
        );

        // Save clip via WindowsRecorder
        let clip_path = self
            .recorder
            .read()
            .await
            .save_clip(
                primary_event,
                clip_id.clone(),
                window.priority,
                total_duration,
            )
            .await
            .context("Failed to save merged clip")?;

        info!("Merged clip saved: {:?}", clip_path);

        // Save metadata to storage
        self.save_clip_metadata(&clip_id, primary_event, window.priority, &clip_path)
            .await?;

        // Save all events in the window to storage
        let game_id = self.current_game_id.read().await;
        if let Some(ref game_id) = *game_id {
            let event_data: Vec<EventData> = window
                .events
                .iter()
                .map(|e| {
                    // Collect participants (killer + assisters)
                    let mut participants = Vec::new();
                    if let Some(ref killer) = e.killer_name {
                        participants.push(killer.clone());
                    }
                    participants.extend_from_slice(&e.assisters);

                    EventData {
                        event_id: e.event_id,
                        event_type: trigger_to_event_type(&window.primary_trigger),
                        timestamp: e.event_time as f64,
                        priority: window.priority,
                        participants,
                        details: None,
                    }
                })
                .collect();

            self.storage
                .save_events(game_id, &event_data)
                .context("Failed to save event data")?;
        }

        Ok(())
    }

    /// Calculate clip window (pre/post durations) based on settings and event type
    fn calculate_clip_window(
        &self,
        trigger: &EventTrigger,
        settings: &RecordingSettings,
    ) -> ClipWindow {
        // Map EventTrigger to settings event type string
        let event_type = match trigger {
            EventTrigger::Multikill(_) => "multikill",
            EventTrigger::Steal => "steal",
            _ => "kill", // Default for other events
        };

        // Get event-specific timing or use defaults
        let timing = settings.clip_timing.get_timing_for_event(event_type);

        ClipWindow {
            pre_duration: timing.pre_duration,
            post_duration: timing.post_duration,
        }
    }

    /// Save clip metadata to storage
    async fn save_clip_metadata(
        &self,
        clip_id: &str,
        event: &GameEvent,
        priority: u8,
        clip_path: &std::path::Path,
    ) -> Result<()> {
        let game_id = self.current_game_id.read().await;

        if let Some(ref game_id) = *game_id {
            let metadata = ClipMetadata {
                file_path: clip_path.to_string_lossy().to_string(),
                thumbnail_path: None,
                event_type: EventType::Custom(event.event_name.clone()),
                event_time: event.event_time as f64,
                priority,
                duration: 0.0, // Will be calculated by video processor
                created_at: chrono::Utc::now(),
            };

            self.storage
                .save_clip_metadata(game_id, &metadata)
                .context("Failed to save clip metadata")?;

            info!("Clip metadata saved: {} (game: {})", clip_id, game_id);
        } else {
            warn!("No current game ID set - clip metadata not saved");
        }

        Ok(())
    }
}

/// Clip window timing configuration
#[derive(Debug, Clone)]
struct ClipWindow {
    pre_duration: u32,  // Seconds before event
    post_duration: u32, // Seconds after event
}

/// Convert LiveClientMonitor's EventTrigger to storage's EventType
fn trigger_to_event_type(trigger: &EventTrigger) -> EventType {
    match trigger {
        EventTrigger::ChampionKill => EventType::ChampionKill,
        EventTrigger::Multikill(n) => EventType::Multikill(*n),
        EventTrigger::DragonKill => EventType::DragonKill,
        EventTrigger::BaronKill => EventType::BaronKill,
        EventTrigger::TurretKill => EventType::TurretKill,
        EventTrigger::InhibitorKill => EventType::InhibitorKill,
        EventTrigger::Ace => EventType::Ace,
        EventTrigger::Steal => EventType::Custom("Steal".to_string()),
        EventTrigger::ClutchPlay => EventType::Custom("ClutchPlay".to_string()),
    }
}

/// Convert live_client::GameEvent to recording::GameEvent
fn convert_live_event(
    live_event: super::live_client::GameEvent,
    trigger: &EventTrigger,
) -> GameEvent {
    GameEvent {
        event_id: live_event.event_id as u64,
        event_name: live_event.event_name,
        event_time: live_event.event_time as f64,
        killer_name: live_event.killer_name,
        victim_name: live_event.victim_name,
        assisters: live_event.assisters.unwrap_or_default(),
        priority: trigger.priority(),
        timestamp: Instant::now(), // Use current time as event timestamp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::models::RecordingSettings;

    fn create_test_event(event_name: &str, event_time: f64) -> GameEvent {
        GameEvent {
            event_id: 1,
            event_name: event_name.to_string(),
            event_time,
            killer_name: Some("TestPlayer".to_string()),
            victim_name: Some("Enemy".to_string()),
            assisters: vec![],
            priority: 3,
            timestamp: Instant::now(),
        }
    }

    #[test]
    fn test_merge_events() {
        // Create test events at different times
        let events = vec![
            QueuedEvent {
                trigger: EventTrigger::ChampionKill,
                event: create_test_event("ChampionKill", 100.0),
                received_at: Instant::now(),
            },
            QueuedEvent {
                trigger: EventTrigger::Multikill(2),
                event: create_test_event("ChampionKill", 105.0),
                received_at: Instant::now(),
            },
            QueuedEvent {
                trigger: EventTrigger::Multikill(3),
                event: create_test_event("ChampionKill", 108.0),
                received_at: Instant::now(),
            },
        ];

        // Create manager (will need test doubles for dependencies)
        let temp_dir = std::env::temp_dir().join("lolshorts_test_acm");
        let recorder = Arc::new(TokioRwLock::new(
            WindowsRecorder::new(temp_dir.clone()).unwrap(),
        ));
        let storage = Arc::new(Storage::new(&temp_dir).unwrap());
        let settings = Arc::new(TokioRwLock::new(RecordingSettings::default()));

        let manager = AutoClipManager::new(recorder, storage, settings);

        // Test merge logic
        let window = manager.merge_events(&events);

        assert_eq!(window.events.len(), 3);
        assert_eq!(window.start_time, 100.0);
        assert_eq!(window.end_time, 108.0);
        assert_eq!(window.priority, 3); // Triple kill priority

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let temp_dir = std::env::temp_dir().join("lolshorts_test_filter");
        let recorder = Arc::new(TokioRwLock::new(
            WindowsRecorder::new(temp_dir.clone()).unwrap(),
        ));
        let storage = Arc::new(Storage::new(&temp_dir).unwrap());

        // Create settings with specific filters
        let mut settings = RecordingSettings::default();
        settings.event_filter.record_kills = false; // Disable kills
        settings.event_filter.record_multikills = true;
        settings.event_filter.min_priority = 2;

        let manager = AutoClipManager::new(recorder, storage, Arc::new(TokioRwLock::new(settings)));

        // Single kill should be filtered out
        let single_kill = create_test_event("ChampionKill", 100.0);
        let should_record = manager
            .should_record_event(&EventTrigger::ChampionKill, &single_kill)
            .await
            .unwrap();
        assert!(!should_record);

        // Double kill should pass (multikills enabled, priority 2)
        let double_kill = create_test_event("ChampionKill", 105.0);
        let should_record = manager
            .should_record_event(&EventTrigger::Multikill(2), &double_kill)
            .await
            .unwrap();
        assert!(should_record);

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
