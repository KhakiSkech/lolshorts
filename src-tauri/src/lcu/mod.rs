pub mod commands;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LcuError {
    #[error("League client not found")]
    ClientNotFound,
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid lockfile format")]
    InvalidLockfile,
}

pub type Result<T> = std::result::Result<T, LcuError>;

/// Lockfile data parsed from League client lockfile
#[derive(Debug, Clone)]
pub struct LockfileData {
    pub process_name: String,
    pub pid: u32,
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

impl LockfileData {
    /// Parse lockfile format: ProcessName:PID:PORT:PASSWORD:PROTOCOL
    pub fn parse(content: &str) -> Result<Self> {
        let parts: Vec<&str> = content.trim().split(':').collect();

        if parts.len() != 5 {
            return Err(LcuError::InvalidLockfile);
        }

        Ok(Self {
            process_name: parts[0].to_string(),
            pid: parts[1].parse().map_err(|_| LcuError::InvalidLockfile)?,
            port: parts[2].parse().map_err(|_| LcuError::InvalidLockfile)?,
            password: parts[3].to_string(),
            protocol: parts[4].to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub game_id: String,
    pub champion: String,
    pub game_mode: String,
    pub game_time: f64,
}

/// Game flow phase from LCU API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum GameFlowPhase {
    None,
    Lobby,
    Matchmaking,
    CheckedIntoTournament,
    ReadyCheck,
    ChampSelect,
    GameStart,
    FailedToLaunch,
    InProgress,
    Reconnect,
    WaitingForStats,
    PreEndOfGame,
    EndOfGame,
    TerminatedInError,
}

/// Game session response from /lol-gameflow/v1/session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSession {
    pub phase: GameFlowPhase,
    #[serde(rename = "gameData")]
    pub game_data: Option<GameData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    #[serde(rename = "gameId")]
    pub game_id: i64,
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    #[serde(rename = "gameTime")]
    pub game_time: f64,
}

pub struct LcuClient {
    http_client: Option<reqwest::Client>,
    lockfile_data: Option<LockfileData>,
}

impl LcuClient {
    pub fn new() -> Self {
        Self {
            http_client: None,
            lockfile_data: None,
        }
    }

    /// Get the lockfile path by checking multiple possible locations
    pub fn get_lockfile_path() -> Result<PathBuf> {
        // List of possible lockfile locations
        let mut possible_paths = vec![
            // Standard installation in C:\Riot Games
            PathBuf::from("C:\\Riot Games\\League of Legends\\lockfile"),
            // Program Files locations
            PathBuf::from("C:\\Program Files\\Riot Games\\League of Legends\\lockfile"),
            PathBuf::from("C:\\Program Files (x86)\\Riot Games\\League of Legends\\lockfile"),
        ];

        // Add LocalAppData location if environment variable exists
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            possible_paths.push(
                PathBuf::from(local_app_data)
                    .join("Riot Games")
                    .join("League of Legends")
                    .join("lockfile"),
            );
        }

        // Try each path
        for path in possible_paths {
            if path.exists() {
                tracing::info!("Found lockfile at: {}", path.display());
                return Ok(path);
            }
        }

        Err(LcuError::ClientNotFound)
    }

    /// Read and parse the lockfile
    pub fn read_lockfile() -> Result<LockfileData> {
        let lockfile_path = Self::get_lockfile_path()?;
        let content = fs::read_to_string(lockfile_path)?;
        LockfileData::parse(&content)
    }

    /// Connect to the League client by reading lockfile
    pub async fn connect(&mut self) -> Result<()> {
        let lockfile = Self::read_lockfile()?;

        // Create HTTP client that accepts self-signed certificates
        let http_client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()
            .map_err(|e| LcuError::Connection(e.to_string()))?;

        self.lockfile_data = Some(lockfile);
        self.http_client = Some(http_client);

        tracing::info!(
            "Connected to LCU on port {}",
            self.lockfile_data.as_ref().unwrap().port
        );

        Ok(())
    }

    /// Get the base URL for LCU API
    fn get_base_url(&self) -> Result<String> {
        let lockfile = self
            .lockfile_data
            .as_ref()
            .ok_or(LcuError::Connection("Not connected".to_string()))?;

        Ok(format!("https://127.0.0.1:{}", lockfile.port))
    }

    /// Get current game information
    pub async fn get_current_game(&self) -> Result<Option<GameInfo>> {
        let session = self.get_game_session().await?;

        match session.phase {
            GameFlowPhase::InProgress | GameFlowPhase::Reconnect => {
                if let Some(game_data) = session.game_data {
                    Ok(Some(GameInfo {
                        game_id: game_data.game_id.to_string(),
                        champion: "Unknown".to_string(), // Need to fetch from another endpoint
                        game_mode: game_data.game_mode,
                        game_time: game_data.game_time,
                    }))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Get game session from LCU API
    pub async fn get_game_session(&self) -> Result<GameSession> {
        let client = self
            .http_client
            .as_ref()
            .ok_or(LcuError::Connection("Not connected".to_string()))?;
        let lockfile = self
            .lockfile_data
            .as_ref()
            .ok_or(LcuError::Connection("Not connected".to_string()))?;

        let base_url = self.get_base_url()?;
        let url = format!("{}/lol-gameflow/v1/session", base_url);

        let response = client
            .get(&url)
            .basic_auth("riot", Some(&lockfile.password))
            .send()
            .await
            .map_err(|e| LcuError::Api(e.to_string()))?;

        if !response.status().is_success() {
            return Err(LcuError::Api(format!("HTTP {}", response.status())));
        }

        let session: GameSession = response
            .json()
            .await
            .map_err(|e| LcuError::Api(e.to_string()))?;

        Ok(session)
    }

    /// Check if a game is in progress
    pub async fn is_in_game(&self) -> Result<bool> {
        let session = self.get_game_session().await?;

        matches!(
            session.phase,
            GameFlowPhase::InProgress | GameFlowPhase::Reconnect
        )
        .then_some(true)
        .ok_or(LcuError::Api("Failed to check game state".to_string()))
        .or(Ok(false))
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        self.lockfile_data.is_some() && self.http_client.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lcu_client_creation() {
        let client = LcuClient::new();
        assert!(!client.is_connected());
    }

    #[test]
    fn test_lockfile_parse_valid() {
        let content = "LeagueClient:12345:54321:secret123:https";
        let lockfile = LockfileData::parse(content).unwrap();

        assert_eq!(lockfile.process_name, "LeagueClient");
        assert_eq!(lockfile.pid, 12345);
        assert_eq!(lockfile.port, 54321);
        assert_eq!(lockfile.password, "secret123");
        assert_eq!(lockfile.protocol, "https");
    }

    #[test]
    fn test_lockfile_parse_with_whitespace() {
        let content = "  LeagueClient:12345:54321:secret123:https  \n";
        let lockfile = LockfileData::parse(content).unwrap();

        assert_eq!(lockfile.process_name, "LeagueClient");
        assert_eq!(lockfile.port, 54321);
    }

    #[test]
    fn test_lockfile_parse_invalid_format() {
        let content = "InvalidFormat";
        let result = LockfileData::parse(content);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LcuError::InvalidLockfile));
    }

    #[test]
    fn test_lockfile_parse_invalid_port() {
        let content = "LeagueClient:12345:notanumber:secret123:https";
        let result = LockfileData::parse(content);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LcuError::InvalidLockfile));
    }

    #[test]
    fn test_lockfile_parse_missing_fields() {
        let content = "LeagueClient:12345:54321";
        let result = LockfileData::parse(content);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LcuError::InvalidLockfile));
    }

    #[test]
    fn test_gameflow_phase_deserialization() {
        // Test that GameFlowPhase can be deserialized from JSON
        let json = r#""InProgress""#;
        let phase: GameFlowPhase = serde_json::from_str(json).unwrap();
        assert!(matches!(phase, GameFlowPhase::InProgress));
    }

    // Note: The following tests require a running League client
    // They are commented out for automated testing
    // Uncomment and run manually when League is running

    // #[tokio::test]
    // async fn test_lcu_connection() {
    //     let mut client = LcuClient::new();
    //     let result = client.connect().await;
    //
    //     if result.is_ok() {
    //         assert!(client.is_connected());
    //     } else {
    //         // If League is not running, this is expected
    //         println!("League client not running: {:?}", result);
    //     }
    // }

    // #[tokio::test]
    // async fn test_get_game_session() {
    //     let mut client = LcuClient::new();
    //     if client.connect().await.is_ok() {
    //         let session = client.get_game_session().await;
    //         assert!(session.is_ok());
    //         println!("Game session: {:?}", session);
    //     }
    // }

    // #[tokio::test]
    // async fn test_is_in_game() {
    //     let mut client = LcuClient::new();
    //     if client.connect().await.is_ok() {
    //         let in_game = client.is_in_game().await;
    //         assert!(in_game.is_ok());
    //         println!("In game: {:?}", in_game);
    //     }
    // }
}
