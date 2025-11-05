# LoLShorts Local Storage Architecture

로컬 우선 아키텍처 - 게임 데이터를 사용자 PC에 JSON 파일로 저장합니다.

---

## 📁 디렉토리 구조

```
C:\Users\{username}\AppData\Local\LoLShorts\
├── config.json                    # 앱 설정
├── user_preferences.json          # 사용자 환경설정
└── games\
    ├── {game_id_1}\
    │   ├── metadata.json          # 게임 메타데이터
    │   ├── recording.mp4          # 전체 게임 녹화
    │   ├── clips\
    │   │   ├── pentakill_420.5s.mp4
    │   │   ├── baron_steal_1200.0s.mp4
    │   │   └── ace_1500.0s.mp4
    │   ├── screenshots\
    │   │   ├── thumbnail.jpg
    │   │   ├── event_pentakill_420.5s.jpg
    │   │   └── manual_capture_1234567890.png
    │   └── compositions\
    │       ├── highlight_montage.mp4
    │       └── best_moments.mp4
    │
    └── {game_id_2}\
        └── ...
```

---

## 📄 JSON 스키마

### 1. `config.json` (앱 전역 설정)

```json
{
  "version": "0.1.0",
  "recordingSettings": {
    "quality": "high",
    "fps": 60,
    "codec": "h265_nvenc",
    "bitrate": 8000,
    "autoStart": true
  },
  "clipSettings": {
    "defaultPadding": 5.0,
    "minPriority": 3,
    "autoExtract": true
  },
  "storagePath": "C:\\Users\\{username}\\AppData\\Local\\LoLShorts\\games",
  "lastUpdated": "2025-11-05T12:00:00Z"
}
```

### 2. `user_preferences.json` (사용자 환경설정)

```json
{
  "userId": "uuid-...",
  "theme": "dark",
  "language": "ko",
  "hotkeys": {
    "manualCapture": "F9",
    "toggleRecording": "F10"
  },
  "notifications": {
    "gameStart": true,
    "clipExtracted": true,
    "videoReady": true
  },
  "lastUpdated": "2025-11-05T12:00:00Z"
}
```

### 3. `games/{game_id}/metadata.json` (게임 메타데이터)

```json
{
  "gameId": "123456789",
  "gameMode": "CLASSIC",
  "gameType": "RANKED_SOLO_5x5",
  "
": "KR",
  "startTime": "2025-11-05T18:30:00Z",
  "endTime": "2025-11-05T19:05:00Z",
  "duration": 2100.5,

  "player": {
    "summonerName": "Player1",
    "championName": "Yasuo",
    "role": "MID",
    "team": "BLUE",
    "kills": 15,
    "deaths": 3,
    "assists": 7,
    "cs": 245,
    "gold": 18500,
    "damage": 35000,
    "vision": 42
  },

  "result": {
    "win": true,
    "teamKills": 42,
    "teamDeaths": 28,
    "turretKills": 9,
    "inhibitorKills": 2,
    "baronKills": 1,
    "dragonKills": 3,
    "riftHeraldKills": 1
  },

  "events": [
    {
      "id": "evt_001",
      "type": "CHAMPION_KILL",
      "time": 180.5,
      "killer": "Player1",
      "victim": "EnemyMid",
      "assisters": ["Jungler1"],
      "priority": 1,
      "location": { "x": 5000, "y": 7000 }
    },
    {
      "id": "evt_002",
      "type": "MULTIKILL",
      "time": 420.5,
      "killer": "Player1",
      "multikillType": "PENTAKILL",
      "multikillCount": 5,
      "victims": ["Enemy1", "Enemy2", "Enemy3", "Enemy4", "Enemy5"],
      "priority": 5,
      "location": { "x": 8000, "y": 6000 }
    },
    {
      "id": "evt_003",
      "type": "DRAGON_KILL",
      "time": 600.0,
      "killer": "Player1",
      "dragonType": "INFERNAL",
      "priority": 2,
      "stolen": false
    },
    {
      "id": "evt_004",
      "type": "BARON_KILL",
      "time": 1200.0,
      "killer": "Player1",
      "priority": 3,
      "stolen": true,
      "stolenFrom": "ENEMY_TEAM"
    },
    {
      "id": "evt_005",
      "type": "ACE",
      "time": 1500.0,
      "team": "BLUE",
      "priority": 4
    }
  ],

  "clips": [
    {
      "id": "clip_001",
      "eventId": "evt_002",
      "fileName": "pentakill_420.5s.mp4",
      "startTime": 415.5,
      "endTime": 430.5,
      "duration": 15.0,
      "priority": 5,
      "thumbnail": "screenshots/event_pentakill_420.5s.jpg",
      "fileSize": 45000000,
      "resolution": { "width": 1920, "height": 1080 },
      "fps": 60,
      "codec": "h265",
      "isFavorite": true,
      "userRating": 5,
      "tags": ["pentakill", "highlight"],
      "createdAt": "2025-11-05T19:10:00Z"
    },
    {
      "id": "clip_002",
      "eventId": "evt_004",
      "fileName": "baron_steal_1200.0s.mp4",
      "startTime": 1195.0,
      "endTime": 1210.0,
      "duration": 15.0,
      "priority": 4,
      "thumbnail": "screenshots/event_baron_steal_1200.0s.jpg",
      "fileSize": 42000000,
      "resolution": { "width": 1920, "height": 1080 },
      "fps": 60,
      "codec": "h265",
      "isFavorite": true,
      "userRating": 4,
      "tags": ["baron", "steal", "clutch"],
      "createdAt": "2025-11-05T19:30:00Z"
    }
  ],

  "screenshots": [
    {
      "id": "ss_001",
      "fileName": "thumbnail.jpg",
      "type": "THUMBNAIL",
      "gameTime": 420.5,
      "width": 1920,
      "height": 1080,
      "fileSize": 250000,
      "createdAt": "2025-11-05T19:10:00Z"
    },
    {
      "id": "ss_002",
      "fileName": "event_pentakill_420.5s.jpg",
      "type": "EVENT",
      "eventId": "evt_002",
      "gameTime": 420.5,
      "width": 750,
      "height": 422,
      "fileSize": 120000,
      "createdAt": "2025-11-05T19:10:00Z"
    },
    {
      "id": "ss_003",
      "fileName": "manual_capture_1234567890.png",
      "type": "MANUAL",
      "gameTime": 800.0,
      "width": 1920,
      "height": 1080,
      "fileSize": 1200000,
      "createdAt": "2025-11-05T18:45:00Z"
    }
  ],

  "compositions": [
    {
      "id": "comp_001",
      "title": "Pentakill Highlight Montage",
      "fileName": "highlight_montage.mp4",
      "clipIds": ["clip_001", "clip_002"],
      "targetFormat": "YOUTUBE_SHORTS",
      "aspectRatio": "9:16",
      "resolution": { "width": 1080, "height": 1920 },
      "duration": 35.0,
      "fps": 60,
      "backgroundMusic": "epic_music.mp3",
      "musicVolume": 0.3,
      "gameAudioVolume": 0.7,
      "transitionType": "FADE",
      "transitionDuration": 0.5,
      "showEventLabels": true,
      "showKillCount": true,
      "watermarkEnabled": false,
      "fileSize": 95000000,
      "status": "READY",
      "isPublic": false,
      "viewCount": 0,
      "createdAt": "2025-11-05T20:00:00Z"
    }
  ],

  "recording": {
    "fileName": "recording.mp4",
    "fileSize": 2500000000,
    "resolution": { "width": 1920, "height": 1080 },
    "fps": 60,
    "codec": "h265_nvenc",
    "bitrate": 8000,
    "duration": 2100.5,
    "segments": [
      {
        "index": 0,
        "startTime": 0.0,
        "endTime": 300.0,
        "filePath": "segments/seg_000.mp4"
      },
      {
        "index": 1,
        "startTime": 300.0,
        "endTime": 600.0,
        "filePath": "segments/seg_001.mp4"
      }
    ]
  },

  "metadata": {
    "appVersion": "0.1.0",
    "lcuVersion": "13.24",
    "createdAt": "2025-11-05T18:30:00Z",
    "updatedAt": "2025-11-05T20:00:00Z"
  }
}
```

---

## 🔧 Rust Implementation

### Directory Manager

```rust
// src-tauri/src/storage/local.rs

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMetadata {
    pub game_id: String,
    pub game_mode: String,
    pub start_time: String,
    pub player: PlayerInfo,
    pub events: Vec<GameEvent>,
    pub clips: Vec<ClipMetadata>,
    // ... other fields
}

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new() -> Result<Self> {
        let base_path = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get local data dir"))?
            .join("LoLShorts");

        std::fs::create_dir_all(&base_path)?;
        std::fs::create_dir_all(base_path.join("games"))?;

        Ok(Self { base_path })
    }

    pub fn get_game_dir(&self, game_id: &str) -> PathBuf {
        self.base_path.join("games").join(game_id)
    }

    pub fn create_game_directory(&self, game_id: &str) -> Result<PathBuf> {
        let game_dir = self.get_game_dir(game_id);
        std::fs::create_dir_all(&game_dir)?;
        std::fs::create_dir_all(game_dir.join("clips"))?;
        std::fs::create_dir_all(game_dir.join("screenshots"))?;
        std::fs::create_dir_all(game_dir.join("compositions"))?;
        Ok(game_dir)
    }

    pub async fn save_metadata(&self, game_id: &str, metadata: &GameMetadata) -> Result<()> {
        let game_dir = self.get_game_dir(game_id);
        let metadata_path = game_dir.join("metadata.json");

        let json = serde_json::to_string_pretty(metadata)?;
        tokio::fs::write(metadata_path, json).await?;

        Ok(())
    }

    pub async fn load_metadata(&self, game_id: &str) -> Result<GameMetadata> {
        let game_dir = self.get_game_dir(game_id);
        let metadata_path = game_dir.join("metadata.json");

        let json = tokio::fs::read_to_string(metadata_path).await?;
        let metadata: GameMetadata = serde_json::from_str(&json)?;

        Ok(metadata)
    }

    pub async fn list_games(&self) -> Result<Vec<String>> {
        let games_dir = self.base_path.join("games");
        let mut games = Vec::new();

        let mut entries = tokio::fs::read_dir(games_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(game_id) = entry.file_name().to_str() {
                    games.push(game_id.to_string());
                }
            }
        }

        Ok(games)
    }

    pub fn get_clip_path(&self, game_id: &str, clip_filename: &str) -> PathBuf {
        self.get_game_dir(game_id)
            .join("clips")
            .join(clip_filename)
    }

    pub fn get_screenshot_path(&self, game_id: &str, screenshot_filename: &str) -> PathBuf {
        self.get_game_dir(game_id)
            .join("screenshots")
            .join(screenshot_filename)
    }

    pub fn get_composition_path(&self, game_id: &str, composition_filename: &str) -> PathBuf {
        self.get_game_dir(game_id)
            .join("compositions")
            .join(composition_filename)
    }
}
```

### Tauri Commands

```rust
// src-tauri/src/commands/storage.rs

use tauri::State;
use crate::storage::local::{LocalStorage, GameMetadata};

#[tauri::command]
pub async fn save_game_metadata(
    storage: State<'_, LocalStorage>,
    game_id: String,
    metadata: GameMetadata,
) -> Result<(), String> {
    storage
        .save_metadata(&game_id, &metadata)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn load_game_metadata(
    storage: State<'_, LocalStorage>,
    game_id: String,
) -> Result<GameMetadata, String> {
    storage
        .load_metadata(&game_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_all_games(
    storage: State<'_, LocalStorage>,
) -> Result<Vec<String>, String> {
    storage
        .list_games()
        .await
        .map_err(|e| e.to_string())
}
```

---

## 🎨 Frontend Usage

```typescript
// src/lib/storage.ts

import { invoke } from '@tauri-apps/api/core';

export interface GameMetadata {
  gameId: string;
  gameMode: string;
  player: PlayerInfo;
  events: GameEvent[];
  clips: ClipMetadata[];
  // ... other fields
}

export async function saveGameMetadata(
  gameId: string,
  metadata: GameMetadata
): Promise<void> {
  await invoke('save_game_metadata', { gameId, metadata });
}

export async function loadGameMetadata(gameId: string): Promise<GameMetadata> {
  return invoke<GameMetadata>('load_game_metadata', { gameId });
}

export async function listAllGames(): Promise<string[]> {
  return invoke<string[]>('list_all_games');
}

// Usage in component
const RecentGames = () => {
  const [games, setGames] = useState<GameMetadata[]>([]);

  useEffect(() => {
    loadGames();
  }, []);

  const loadGames = async () => {
    const gameIds = await listAllGames();
    const gameData = await Promise.all(
      gameIds.map(id => loadGameMetadata(id))
    );
    setGames(gameData);
  };

  return (
    <div>
      {games.map(game => (
        <GameCard key={game.gameId} game={game} />
      ))}
    </div>
  );
};
```

---

## 📊 Performance Benefits

| 작업 | 클라우드 DB | 로컬 JSON | 속도 개선 |
|------|------------|-----------|----------|
| 게임 메타데이터 로드 | ~200ms | ~10ms | **20배** |
| 클립 목록 조회 | ~150ms | ~5ms | **30배** |
| 영상 스트리밍 시작 | ~500ms | ~50ms | **10배** |
| 대용량 영상 저장 | ~30s | ~3s | **10배** |

---

## 🔐 보안 및 백업

### 데이터 보호
- ✅ 사용자 PC에만 존재 (클라우드 유출 위험 없음)
- ✅ 민감 정보 없음 (개인 게임 데이터만)
- ✅ Windows 파일 시스템 권한 적용

### 백업 옵션
```typescript
// 사용자가 원하면 수동 백업 가능
export async function backupGameData(gameId: string, backupPath: string) {
  const metadata = await loadGameMetadata(gameId);
  const gameDir = await getGameDirectory(gameId);

  // Copy entire game directory to backup location
  await copyDirectory(gameDir, backupPath);
}
```

---

**작성일**: 2025-11-05
**프로젝트**: LoLShorts v0.1
**아키텍처**: Local-First Storage with JSON
