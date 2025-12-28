use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime, Instant};
use tokio::time;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Live Client Data API endpoint
const LIVE_CLIENT_API: &str = "https://127.0.0.1:2999/liveclientdata";

/// Event types that trigger automatic recording
#[derive(Debug, Clone, PartialEq)]
pub enum EventTrigger {
    ChampionKill,
    Multikill(u8), // Double, Triple, Quadra, Penta
    DragonKill,
    BaronKill,
    TurretKill,
    InhibitorKill,
    Ace,
    Steal,      // Dragon/Baron steal
}

impl EventTrigger {
    /// Get clip priority (1-5)
    pub fn priority(&self) -> u8 {
        match self {
            EventTrigger::ChampionKill => 1,
            EventTrigger::Multikill(2) => 2, // Double
            EventTrigger::Multikill(3) => 3, // Triple
            EventTrigger::Multikill(4) => 4, // Quadra
            EventTrigger::Multikill(5) => 5, // Penta
            EventTrigger::DragonKill => 2,
            EventTrigger::BaronKill => 3,
            EventTrigger::TurretKill => 1,
            EventTrigger::InhibitorKill => 2,
            EventTrigger::Ace => 4,
            EventTrigger::Steal => 4,
            _ => 1,
        }
    }

    /// Get recommended clip duration before event (seconds)
    pub fn pre_duration(&self) -> u32 {
        match self {
            EventTrigger::Multikill(_) => 15, // Need setup time
            EventTrigger::Steal => 20,        // Need fight context
            _ => 10,
        }
    }

    /// Get recommended clip duration after event (seconds)
    pub fn post_duration(&self) -> u32 {
        match self {
            EventTrigger::Ace => 10, // Show aftermath
            EventTrigger::BaronKill => 5,
            EventTrigger::Multikill(_) => 5,
            _ => 3,
        }
    }
}

/// Live Client API response structures
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AllGameData {
    #[serde(rename = "activePlayer", default)]
    pub active_player: ActivePlayer,
    #[serde(rename = "allPlayers", default)]
    pub all_players: Vec<Player>,
    #[serde(default)]
    pub events: Events,
    #[serde(rename = "gameData", default)]
    pub game_data: GameData,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ActivePlayer {
    #[serde(rename = "championName", default)]
    pub champion_name: String,
    #[serde(rename = "summonerName", default)]
    pub summoner_name: String,
    #[serde(default)]
    pub level: u32,
    #[serde(rename = "currentGold", default)]
    pub current_gold: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Player {
    #[serde(rename = "championName", default)]
    pub champion_name: String,
    #[serde(rename = "summonerName", default)]
    pub summoner_name: String,
    #[serde(default)]
    pub team: String,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub scores: Scores,
    #[serde(rename = "isDead", default)]
    pub is_dead: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Scores {
    #[serde(default)]
    pub kills: u32,
    #[serde(default)]
    pub deaths: u32,
    #[serde(default)]
    pub assists: u32,
    #[serde(rename = "creepScore", default)]
    pub creep_score: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Events {
    #[serde(rename = "Events", default)]
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GameEvent {
    #[serde(rename = "EventID", default)]
    pub event_id: u32,
    #[serde(rename = "EventName", default)]
    pub event_name: String,
    #[serde(rename = "EventTime", default)]
    pub event_time: f32,
    #[serde(rename = "KillerName", default)]
    pub killer_name: Option<String>,
    #[serde(rename = "VictimName", default)]
    pub victim_name: Option<String>,
    #[serde(rename = "Assisters", default)]
    pub assisters: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GameData {
    #[serde(rename = "gameMode", default)]
    pub game_mode: String,
    #[serde(rename = "gameTime", default)]
    pub game_time: f32,
    #[serde(rename = "mapName", default)]
    pub map_name: String,
    #[serde(rename = "mapNumber", default)]
    pub map_number: u32,
}

/// Monitor for Live Client events with optimized event streaming
pub struct LiveClientMonitor {
    client: Client,
    config: EventStreamConfig,
    last_event_id: Arc<tokio::sync::Mutex<u32>>,
    player_name: Option<String>,
    recent_kills: Arc<tokio::sync::Mutex<Vec<KillRecord>>>,
    game_state_cache: Arc<RwLock<GameStateCache>>,
    last_full_data_fetch: Arc<tokio::sync::Mutex<Option<Instant>>>,
    performance_stats: Arc<RwLock<EventStreamStats>>,
}

/// Performance statistics for monitoring event stream efficiency
#[derive(Debug, Clone)]
#[allow(dead_code)] // Future performance monitoring infrastructure
pub struct EventStreamStats {
    #[allow(dead_code)] // Future cache efficiency tracking
    pub events_processed: u64,
    #[allow(dead_code)] // Future API usage analytics
    pub api_calls_made: u64,
    #[allow(dead_code)] // Future cache performance tracking
    pub cache_hits: u64,
    #[allow(dead_code)] // Future error rate monitoring
    pub errors: u64,
    #[allow(dead_code)] // Future performance metrics
    pub average_response_time_ms: f64,
    #[allow(dead_code)] // Future stats reset functionality
    pub last_stats_reset: Instant,
}

impl Default for EventStreamStats {
    fn default() -> Self {
        Self {
            events_processed: 0,
            api_calls_made: 0,
            cache_hits: 0,
            errors: 0,
            average_response_time_ms: 0.0,
            last_stats_reset: Instant::now(),
        }
    }
}

impl EventStreamStats {
    fn new() -> Self {
        Self {
            last_stats_reset: Instant::now(),
            ..Default::default()
        }
    }

    #[allow(dead_code)] // Future stats reset functionality
    fn reset(&mut self) {
        *self = Self::new();
    }

    fn update_response_time(&mut self, response_time_ms: u64) {
        let total_time = self.average_response_time_ms * (self.api_calls_made as f64) + response_time_ms as f64;
        self.api_calls_made += 1;
        self.average_response_time_ms = total_time / self.api_calls_made as f64;
    }

    #[allow(dead_code)] // Future performance efficiency monitoring
    fn get_efficiency_score(&self) -> f64 {
        if self.api_calls_made == 0 {
            return 0.0;
        }
        // Cache hit ratio * response efficiency
        let cache_ratio = self.cache_hits as f64 / self.api_calls_made as f64;
        let response_efficiency = 1000.0 / (self.average_response_time_ms.max(1.0));
        cache_ratio * response_efficiency
    }
}

#[derive(Debug, Clone)]
struct KillRecord {
    killer: String,
    timestamp: SystemTime,
}

/// Optimized game state cache to reduce redundant API calls
#[derive(Debug, Clone)]
struct GameStateCache {
    data: Option<AllGameData>,
    last_updated: Option<Instant>,
    ttl: Duration,
}

impl GameStateCache {
    fn new(ttl: Duration) -> Self {
        Self {
            data: None,
            last_updated: None,
            ttl,
        }
    }

    #[allow(dead_code)] // Future cache validation functionality
    fn is_valid(&self) -> bool {
        self.last_updated
            .map_or(false, |t| t.elapsed() < self.ttl)
    }

    fn update(&mut self, data: AllGameData) {
        self.data = Some(data);
        self.last_updated = Some(Instant::now());
    }

    #[allow(dead_code)] // Future cache access functionality
    fn get(&self) -> Option<&AllGameData> {
        if self.is_valid() {
            self.data.as_ref()
        } else {
            None
        }
    }
}

/// Event streaming configuration
#[derive(Debug, Clone, Copy)]
pub struct EventStreamConfig {
    /// Polling interval for event-only endpoint (faster)
    event_poll_interval: Duration,
    /// Polling interval for full game data (slower, fallback)
    full_data_interval: Duration,
    /// Cache TTL for game state
    #[allow(dead_code)] // Future cache tuning capability
    pub cache_ttl: Duration,
    /// Connection timeout
    connection_timeout: Duration,
}

impl Default for EventStreamConfig {
    fn default() -> Self {
        Self {
            event_poll_interval: Duration::from_millis(250),  // 4x faster for events
            full_data_interval: Duration::from_secs(2),       // Slower for full data
            cache_ttl: Duration::from_millis(500),            // 500ms cache
            connection_timeout: Duration::from_secs(1),
        }
    }
}

impl LiveClientMonitor {
    pub fn new() -> Result<Self> {
        Self::with_config(EventStreamConfig::default())
    }

    pub fn with_config(config: EventStreamConfig) -> Result<Self> {
        // Create HTTP client that accepts self-signed certificates
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(config.connection_timeout)
            .build()?;

        Ok(Self {
            client,
            config,
            last_event_id: Arc::new(tokio::sync::Mutex::new(0)),
            player_name: None,
            recent_kills: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            game_state_cache: Arc::new(RwLock::new(GameStateCache::new(config.cache_ttl))),
            last_full_data_fetch: Arc::new(tokio::sync::Mutex::new(None)),
            performance_stats: Arc::new(RwLock::new(EventStreamStats::new())),
        })
    }

    /// Get performance metrics for monitoring
    #[allow(dead_code)] // Future performance monitoring API
    pub async fn get_performance_stats(&self) -> EventStreamStats {
        self.performance_stats.read().await.clone()
    }

    /// Reset performance statistics
    #[allow(dead_code)] // Future stats management
    pub async fn reset_performance_stats(&self) {
        self.performance_stats.write().await.reset();
    }

    /// Start optimized monitoring for events with intelligent polling
    pub async fn start_monitoring<F>(&mut self, mut on_event: F) -> Result<()>
    where
        F: FnMut(EventTrigger, GameEvent) + Send + 'static,
    {
        info!("Starting optimized Live Client monitor...");

        let mut event_interval = time::interval(self.config.event_poll_interval);
        let mut full_data_interval = time::interval(self.config.full_data_interval);

        // Initialize player name with retry logic (game may still be loading)
        // League games can take 30-60+ seconds to fully load, especially on slower systems
        const MAX_INIT_RETRIES: u32 = 30;
        const INIT_RETRY_DELAY: Duration = Duration::from_secs(3);

        for attempt in 1..=MAX_INIT_RETRIES {
            match self.fetch_game_data().await {
                Ok(initial_data) => {
                    let player_name = initial_data.active_player.summoner_name.clone();
                    info!("✅ Connected to Live Client API - Monitoring player: {}", player_name);
                    self.player_name = Some(player_name);
                    self.game_state_cache.write().await.update(initial_data);
                    break;
                }
                Err(e) => {
                    if attempt < MAX_INIT_RETRIES {
                        info!("⏳ Waiting for game to load... (attempt {}/{}) - {}", attempt, MAX_INIT_RETRIES, e);
                        time::sleep(INIT_RETRY_DELAY).await;
                    } else {
                        tracing::warn!("Failed to fetch initial game data after {} attempts - will retry during polling", MAX_INIT_RETRIES);
                    }
                }
            }
        }

        info!("🔄 Starting event monitoring loop (polling every {}ms, full refresh every {}ms)",
            self.config.event_poll_interval.as_millis(),
            self.config.full_data_interval.as_millis());

        loop {
            tokio::select! {
                // High-frequency event polling (lightweight)
                _ = event_interval.tick() => {
                    if let Err(e) = self.poll_events_only(&mut on_event).await {
                        debug!("Event polling failed: {}", e);
                    }
                }

                // Low-frequency full data refresh (heavyweight)
                _ = full_data_interval.tick() => {
                    if let Err(e) = self.refresh_full_game_data(&mut on_event).await {
                        debug!("Full data refresh failed: {}", e);
                    }
                }
            }
        }
    }

    /// Lightweight event polling using eventlist endpoint
    async fn poll_events_only<F>(&mut self, on_event: &mut F) -> Result<()>
    where
        F: FnMut(EventTrigger, GameEvent) + Send + 'static,
    {
        let start_time = Instant::now();

        match self.fetch_event_list().await {
            Ok(events) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                self.update_api_stats(true, response_time).await;

                // Process only new events
                self.process_event_list(events, on_event).await?;
            }
            Err(e) => {
                self.update_api_stats(false, start_time.elapsed().as_millis() as u64).await;
                debug!("Event polling failed: {}", e);
            }
        }

        Ok(())
    }

    /// Full game data refresh (fallback and cache update)
    async fn refresh_full_game_data<F>(&mut self, on_event: &mut F) -> Result<()>
    where
        F: FnMut(EventTrigger, GameEvent) + Send + 'static,
    {
        let start_time = Instant::now();

        match self.fetch_game_data().await {
            Ok(data) => {
                let response_time = start_time.elapsed().as_millis() as u64;

                // Update cache
                {
                    let mut cache = self.game_state_cache.write().await;
                    cache.update(data.clone());
                }

                // Update last fetch time
                *self.last_full_data_fetch.lock().await = Some(Instant::now());

                // Update player name if needed (may have failed during initial connect)
                if self.player_name.is_none() {
                    self.player_name = Some(data.active_player.summoner_name.clone());
                    info!("✅ Live Client API connected - Monitoring player: {}", data.active_player.summoner_name);
                }

                // Process events
                self.process_events(data, on_event).await?;

                self.update_api_stats(true, response_time).await;
            }
            Err(e) => {
                self.update_api_stats(false, start_time.elapsed().as_millis() as u64).await;
                debug!("Full data refresh failed: {}", e);
            }
        }

        Ok(())
    }

    /// Fetch only the event list (lightweight endpoint)
    async fn fetch_event_list(&self) -> Result<Vec<GameEvent>> {
        let url = format!("{}/eventlist", LIVE_CLIENT_API);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send event list request")?;

        if response.status().is_success() {
            let events: Vec<GameEvent> = response
                .json()
                .await
                .context("Failed to parse event list")?;
            Ok(events)
        } else {
            Err(anyhow::anyhow!(
                "Event list endpoint returned status {}",
                response.status()
            ))
        }
    }

    /// Update API performance statistics
    async fn update_api_stats(&self, success: bool, response_time_ms: u64) {
        let mut stats = self.performance_stats.write().await;

        if success {
            stats.update_response_time(response_time_ms);
        } else {
            stats.errors += 1;
        }
    }

    /// Get cached game data or fetch if needed
    #[allow(dead_code)] // Future cache optimization
    async fn get_cached_game_data(&self) -> Result<Option<AllGameData>> {
        {
            let cache = self.game_state_cache.read().await;
            if let Some(data) = cache.get() {
                // Record cache hit
                self.performance_stats.write().await.cache_hits += 1;
                return Ok(Some(data.clone()));
            }
        }

        // Cache miss, need to fetch
        Ok(None)
    }

    /// Fetch current game data
    async fn fetch_game_data(&self) -> Result<AllGameData> {
        let url = format!("{}/allgamedata", LIVE_CLIENT_API);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Live Client API")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "API returned status: {}",
                response.status()
            ));
        }

        // Get response text first for debugging
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        // Try to parse JSON
        let data: AllGameData = serde_json::from_str(&response_text)
            .map_err(|e| {
                // Log the first 500 chars of response for debugging
                let preview = if response_text.len() > 500 {
                    format!("{}...", &response_text[..500])
                } else {
                    response_text.clone()
                };
                tracing::warn!("JSON parse error: {}. Response preview: {}", e, preview);
                anyhow::anyhow!("Failed to parse game data: {}", e)
            })?;

        // Validate that essential data is present (game may still be loading)
        if data.active_player.summoner_name.is_empty() {
            return Err(anyhow::anyhow!("Game still loading: summoner name not available yet"));
        }

        Ok(data)
    }

    /// Process event list (optimized for lightweight polling)
    async fn process_event_list<F>(&self, events: Vec<GameEvent>, on_event: &mut F) -> Result<()>
    where
        F: FnMut(EventTrigger, GameEvent) + Send + 'static,
    {
        let mut last_id = self.last_event_id.lock().await;
        let player_name = match &self.player_name {
            Some(name) => name.clone(),
            None => return Ok(()), // Skip if we don't know the player name yet
        };

        for event in events {
            // Skip already processed events
            if event.event_id <= *last_id {
                continue;
            }

            debug!("New event: {} at {}s", event.event_name, event.event_time);

            // Detect event triggers with cached player name
            if let Some(trigger) = self.detect_trigger(&event, &player_name).await {
                info!(
                    "Event trigger detected: {:?} (priority: {})",
                    trigger,
                    trigger.priority()
                );
                on_event(trigger, event.clone());

                // Update performance stats
                {
                    let mut stats = self.performance_stats.write().await;
                    stats.events_processed += 1;
                }
            }

            *last_id = event.event_id;
        }

        Ok(())
    }

    /// Process events and detect triggers (full data version)
    async fn process_events<F>(&self, data: AllGameData, on_event: &mut F) -> Result<()>
    where
        F: FnMut(EventTrigger, GameEvent),
    {
        let mut last_id = self.last_event_id.lock().await;
        let player_name = match self.player_name.as_ref() {
            Some(name) => name,
            None => {
                tracing::warn!("Player name not initialized - skipping event processing");
                return Ok(());
            }
        };

        for event in &data.events.events {
            // Skip already processed events
            if event.event_id <= *last_id {
                continue;
            }

            debug!("New event: {} at {}s", event.event_name, event.event_time);

            // Detect event triggers
            if let Some(trigger) = self.detect_trigger(event, player_name).await {
                info!(
                    "Event trigger detected: {:?} (priority: {})",
                    trigger,
                    trigger.priority()
                );
                on_event(trigger, event.clone());
            }

            *last_id = event.event_id;
        }

        Ok(())
    }

    /// Detect if an event should trigger recording
    async fn detect_trigger(&self, event: &GameEvent, player_name: &str) -> Option<EventTrigger> {
        match event.event_name.as_str() {
            "ChampionKill" => {
                if let Some(killer) = &event.killer_name {
                    if killer == player_name {
                        // Player got a kill
                        let multikill = self.check_multikill(killer).await;

                        if multikill >= 2 {
                            Some(EventTrigger::Multikill(multikill))
                        } else {
                            Some(EventTrigger::ChampionKill)
                        }
                    } else if let Some(assisters) = &event.assisters {
                        if assisters.contains(&player_name.to_string()) {
                            // Player got an assist
                            Some(EventTrigger::ChampionKill)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "DragonKill" => {
                if event.killer_name.as_deref() == Some(player_name) {
                    Some(EventTrigger::DragonKill)
                } else {
                    None
                }
            }
            "BaronKill" => {
                if event.killer_name.as_deref() == Some(player_name) {
                    Some(EventTrigger::BaronKill)
                } else {
                    None
                }
            }
            "TurretKilled" => {
                if let Some(killer) = &event.killer_name {
                    if killer == player_name {
                        Some(EventTrigger::TurretKill)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "InhibKilled" => {
                if let Some(killer) = &event.killer_name {
                    if killer == player_name {
                        Some(EventTrigger::InhibitorKill)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "Ace" => Some(EventTrigger::Ace),
            _ => None,
        }
    }

    /// Check if recent kills form a multikill
    async fn check_multikill(&self, killer: &str) -> u8 {
        let mut kills = self.recent_kills.lock().await;
        let now = SystemTime::now();

        // Add new kill
        kills.push(KillRecord {
            killer: killer.to_string(),
            timestamp: now,
        });

        // Remove old kills (>10 seconds)
        kills.retain(|k| {
            now.duration_since(k.timestamp)
                .unwrap_or(Duration::from_secs(100))
                < Duration::from_secs(10)
        });

        // Count kills by this player in the window
        let kill_count = kills.iter().filter(|k| k.killer == killer).count() as u8;

        // Return multikill level
        match kill_count {
            5.. => 5, // Pentakill
            4 => 4,   // Quadrakill
            3 => 3,   // Triple kill
            2 => 2,   // Double kill
            _ => 1,
        }
    }

    /// Check if Live Client API is available (optimized with cached connection)
    #[allow(dead_code)] // Future connection health checking
    pub async fn is_available(&self) -> bool {
        // Try eventlist endpoint first (lightweight)
        let url = format!("{}/eventlist", LIVE_CLIENT_API);

        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => {
                // Fallback to full endpoint
                let url = format!("{}/allgamedata", LIVE_CLIENT_API);
                matches!(self.client.get(&url).send().await, Ok(response) if response.status().is_success())
            }
        }
    }

    /// Detect if current game session is a replay (not a live game)
    ///
    /// Detection logic:
    /// 1. In a live game, the activePlayer matches one of the allPlayers
    /// 2. In a replay, the activePlayer summoner name is empty or doesn't match any participant
    /// 3. Additionally, we can check if the game mode or other indicators suggest replay
    ///
    /// Returns: Some(true) for replay, Some(false) for live game, None if detection fails
    pub async fn detect_replay_mode(&self) -> Option<bool> {
        // Fetch game data
        let data = match self.fetch_game_data().await {
            Ok(data) => data,
            Err(e) => {
                debug!("Failed to fetch game data for replay detection: {}", e);
                return None;
            }
        };

        // Get active player name
        let active_player_name = &data.active_player.summoner_name;

        // Check 1: Empty or placeholder active player name suggests replay mode
        if active_player_name.is_empty() || active_player_name == "Spectator" {
            info!("Replay detected: Active player name is empty or 'Spectator'");
            return Some(true);
        }

        // Check 2: Active player not in allPlayers list
        let is_participant = data.all_players.iter().any(|p| {
            &p.summoner_name == active_player_name
        });

        if !is_participant {
            info!(
                "Replay detected: Active player '{}' not found in game participants",
                active_player_name
            );
            return Some(true);
        }

        // Check 3: In replay mode, currentGold is usually 0 or static
        // (This is a heuristic - live games have fluctuating gold)
        // We'll use participant check as primary method

        info!("Live game detected: Active player '{}' is a participant", active_player_name);
        Some(false)
    }

    /// Get the current active player name (if known)
    pub fn get_active_player_name(&self) -> Option<&str> {
        self.player_name.as_deref()
    }

    /// Get all players in the current game
    pub async fn get_all_players(&self) -> Result<Vec<String>> {
        let data = self.fetch_game_data().await?;
        Ok(data.all_players.iter().map(|p| p.summoner_name.clone()).collect())
    }

    /// Get performance metrics for external monitoring
    #[allow(dead_code)] // Future performance dashboard integration
    pub async fn get_live_client_metrics(&self) -> LiveClientMetrics {
        let stats = self.performance_stats.read().await.clone();
        LiveClientMetrics {
            events_processed: stats.events_processed,
            api_calls_made: stats.api_calls_made,
            cache_hits: stats.cache_hits,
            errors: stats.errors,
            average_response_time_ms: stats.average_response_time_ms,
            efficiency_score: stats.get_efficiency_score(),
            uptime_seconds: stats.last_stats_reset.elapsed().as_secs(),
        }
    }

    /// Configure event streaming settings for optimal performance
    #[allow(dead_code)] // Future performance tuning
    pub async fn configure_streaming(&mut self, config: EventStreamConfig) {
        self.config = config.clone();

        // Update cache TTL
        {
            let mut cache = self.game_state_cache.write().await;
            *cache = GameStateCache::new(config.cache_ttl);
        }

        info!("Updated Live Client streaming configuration:");
        info!("  Event polling interval: {}ms", config.event_poll_interval.as_millis());
        info!("  Full data interval: {}ms", config.full_data_interval.as_millis());
        info!("  Cache TTL: {}ms", config.cache_ttl.as_millis());
    }
}

/// Public metrics struct for external monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)] // Future performance dashboard feature
pub struct LiveClientMetrics {
    pub events_processed: u64,
    pub api_calls_made: u64,
    pub cache_hits: u64,
    pub errors: u64,
    pub average_response_time_ms: f64,
    pub efficiency_score: f64,
    pub uptime_seconds: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_trigger_priority() {
        assert_eq!(EventTrigger::ChampionKill.priority(), 1);
        assert_eq!(EventTrigger::Multikill(2).priority(), 2);
        assert_eq!(EventTrigger::Multikill(5).priority(), 5);
        assert_eq!(EventTrigger::BaronKill.priority(), 3);
        assert_eq!(EventTrigger::Ace.priority(), 4);
    }

    #[test]
    fn test_event_trigger_duration() {
        let trigger = EventTrigger::Multikill(3);
        assert_eq!(trigger.pre_duration(), 15);
        assert_eq!(trigger.post_duration(), 5);

        let trigger = EventTrigger::Steal;
        assert_eq!(trigger.pre_duration(), 20);
        assert_eq!(trigger.post_duration(), 3);
    }

    #[tokio::test]
    async fn test_live_client_creation() {
        let monitor = LiveClientMonitor::new();
        assert!(monitor.is_ok());
    }
}