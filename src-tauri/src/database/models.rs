use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub game_id: String,
    pub champion: String,
    pub game_mode: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub kda: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub game_id: i64,
    pub event_type: String,
    pub event_time: f64,
    pub priority: i32,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}
