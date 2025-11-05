-- Initial database schema for LoLShorts

-- Games table: stores game session information
CREATE TABLE IF NOT EXISTS games (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT UNIQUE NOT NULL,
    champion TEXT NOT NULL,
    game_mode TEXT NOT NULL,
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    kda TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_games_game_id ON games(game_id);
CREATE INDEX idx_games_start_time ON games(start_time DESC);

-- Clips table: stores video clip information
CREATE TABLE IF NOT EXISTS clips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id INTEGER NOT NULL,
    event_type TEXT NOT NULL,
    event_time REAL NOT NULL,
    priority INTEGER NOT NULL,
    file_path TEXT NOT NULL,
    thumbnail_path TEXT,
    duration REAL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE
);

CREATE INDEX idx_clips_game_id ON clips(game_id);
CREATE INDEX idx_clips_priority ON clips(priority DESC);
CREATE INDEX idx_clips_event_time ON clips(event_time);

-- User settings table
CREATE TABLE IF NOT EXISTS user_settings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT UNIQUE NOT NULL,
    auto_record BOOLEAN DEFAULT TRUE,
    record_quality TEXT DEFAULT '1080p',
    output_dir TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
