use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Game metadata stored in metadata.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: String,
    pub champion: String,
    pub game_mode: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub result: Option<GameResult>,
    pub kda: Option<KDA>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameResult {
    Win,
    Loss,
    Remake,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KDA {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
}

impl KDA {
    pub fn ratio(&self) -> f64 {
        if self.deaths == 0 {
            (self.kills + self.assists) as f64
        } else {
            (self.kills + self.assists) as f64 / self.deaths as f64
        }
    }
}

/// Event data stored in events.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    pub event_id: u64,
    pub event_type: EventType,
    pub timestamp: f64,  // Game time in seconds
    pub priority: u8,     // 1-5, higher is more important
    pub participants: Vec<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    ChampionKill,
    Multikill(u8),  // 2=double, 3=triple, 4=quadra, 5=penta
    TurretKill,
    InhibitorKill,
    DragonKill,
    BaronKill,
    Ace,
    FirstBlood,
    Custom(String),
}

impl EventType {
    pub fn default_priority(&self) -> u8 {
        match self {
            EventType::ChampionKill => 1,
            EventType::Multikill(2) => 2,  // Double kill
            EventType::Multikill(3) => 3,  // Triple kill
            EventType::Multikill(4) => 4,  // Quadra kill
            EventType::Multikill(5) => 5,  // Penta kill
            EventType::Multikill(_) => 3,
            EventType::TurretKill => 2,
            EventType::InhibitorKill => 3,
            EventType::DragonKill => 3,
            EventType::BaronKill => 4,
            EventType::Ace => 4,
            EventType::FirstBlood => 3,
            EventType::Custom(_) => 2,
        }
    }
}

/// Clip metadata stored in clips.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipMetadata {
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub event_type: EventType,
    pub event_time: f64,  // Game time when event occurred
    pub priority: u8,
    pub duration: f64,    // Clip duration in seconds
    pub created_at: DateTime<Utc>,
}
