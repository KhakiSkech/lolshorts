use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time;
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
    ClutchPlay, // 1v2+, low HP survival
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
            EventTrigger::ClutchPlay => 3,
            _ => 1,
        }
    }

    /// Get recommended clip duration before event (seconds)
    pub fn pre_duration(&self) -> u32 {
        match self {
            EventTrigger::Multikill(_) => 15, // Need setup time
            EventTrigger::Steal => 20,        // Need fight context
            EventTrigger::ClutchPlay => 20,
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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AllGameData {
    #[serde(rename = "activePlayer")]
    pub active_player: ActivePlayer,
    #[serde(rename = "allPlayers")]
    pub all_players: Vec<Player>,
    pub events: Events,
    #[serde(rename = "gameData")]
    pub game_data: GameData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActivePlayer {
    #[serde(rename = "championName")]
    pub champion_name: String,
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    pub level: u32,
    #[serde(rename = "currentGold")]
    pub current_gold: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Player {
    #[serde(rename = "championName")]
    pub champion_name: String,
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    pub team: String,
    pub level: u32,
    pub scores: Scores,
    #[serde(rename = "isDead")]
    pub is_dead: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Scores {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    #[serde(rename = "creepScore")]
    pub creep_score: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Events {
    #[serde(rename = "Events")]
    pub events: Vec<GameEvent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameEvent {
    #[serde(rename = "EventID")]
    pub event_id: u32,
    #[serde(rename = "EventName")]
    pub event_name: String,
    #[serde(rename = "EventTime")]
    pub event_time: f32,
    #[serde(rename = "KillerName")]
    pub killer_name: Option<String>,
    #[serde(rename = "VictimName")]
    pub victim_name: Option<String>,
    #[serde(rename = "Assisters")]
    pub assisters: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GameData {
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    #[serde(rename = "gameTime")]
    pub game_time: f32,
    #[serde(rename = "mapName")]
    pub map_name: String,
    #[serde(rename = "mapNumber")]
    pub map_number: u32,
}

/// Monitor for Live Client events
pub struct LiveClientMonitor {
    client: Client,
    last_event_id: Arc<tokio::sync::Mutex<u32>>,
    player_name: Option<String>,
    recent_kills: Arc<tokio::sync::Mutex<Vec<KillRecord>>>,
}

#[derive(Debug, Clone)]
struct KillRecord {
    killer: String,
    timestamp: SystemTime,
}

impl LiveClientMonitor {
    pub fn new() -> Result<Self> {
        // Create HTTP client that accepts self-signed certificates
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(2))
            .build()?;

        Ok(Self {
            client,
            last_event_id: Arc::new(tokio::sync::Mutex::new(0)),
            player_name: None,
            recent_kills: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        })
    }

    /// Start monitoring for events
    pub async fn start_monitoring<F>(&mut self, mut on_event: F) -> Result<()>
    where
        F: FnMut(EventTrigger, GameEvent) + Send + 'static,
    {
        info!("Starting Live Client monitor...");

        let mut interval = time::interval(Duration::from_millis(500)); // Check 2x per second

        loop {
            interval.tick().await;

            match self.fetch_game_data().await {
                Ok(data) => {
                    // Store player name on first fetch
                    if self.player_name.is_none() {
                        self.player_name = Some(data.active_player.summoner_name.clone());
                        info!("Monitoring player: {}", data.active_player.summoner_name);
                    }

                    // Process new events
                    self.process_events(data, &mut on_event).await?;
                }
                Err(e) => {
                    // Game might not be running, this is normal
                    debug!("Live Client not available: {}", e);
                }
            }
        }
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

        let data = response
            .json::<AllGameData>()
            .await
            .context("Failed to parse game data")?;

        Ok(data)
    }

    /// Process events and detect triggers
    async fn process_events<F>(&self, data: AllGameData, on_event: &mut F) -> Result<()>
    where
        F: FnMut(EventTrigger, GameEvent),
    {
        let mut last_id = self.last_event_id.lock().await;
        let player_name = self.player_name.as_ref().unwrap();

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
                    } else if event.victim_name.as_deref() == Some(player_name) {
                        // Player died - might want to save if it was a close fight
                        None // TODO: Detect clutch plays
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

    /// Check if Live Client API is available
    pub async fn is_available(&self) -> bool {
        self.fetch_game_data().await.is_ok()
    }
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
