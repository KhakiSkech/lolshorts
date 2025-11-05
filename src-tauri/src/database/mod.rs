pub mod models;
pub mod migrations;

use sqlx::{sqlite::SqlitePool, migrate::MigrateDatabase, Sqlite, Row};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(String),
    #[error("Query error: {0}")]
    Query(String),
}

pub type Result<T> = std::result::Result<T, DatabaseError>;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Initialize database connection and run migrations
    pub async fn new(db_path: &str) -> Result<Self> {
        let db_url = format!("sqlite:{}", db_path);

        // Create database if it doesn't exist
        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            tracing::info!("Creating database: {}", db_path);
            Sqlite::create_database(&db_url)
                .await
                .map_err(|e| DatabaseError::Migration(e.to_string()))?;
        }

        // Connect to database
        let pool = SqlitePool::connect(&db_url).await?;

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| DatabaseError::Migration(e.to_string()))?;

        tracing::info!("Database initialized successfully");

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Insert a new game record
    pub async fn insert_game(&self, game: &models::GameRecord) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO games (game_id, champion, game_mode, start_time, end_time, kda)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&game.game_id)
        .bind(&game.champion)
        .bind(&game.game_mode)
        .bind(&game.start_time)
        .bind(&game.end_time)
        .bind(&game.kda)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    /// Insert a new clip record
    pub async fn insert_clip(&self, clip: &models::ClipRecord) -> Result<i64> {
        let id = sqlx::query(
            r#"
            INSERT INTO clips (game_id, event_type, event_time, priority, file_path, thumbnail_path, duration)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(clip.game_id)
        .bind(&clip.event_type)
        .bind(clip.event_time)
        .bind(clip.priority)
        .bind(&clip.file_path)
        .bind(&clip.thumbnail_path)
        .bind(clip.duration)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    /// Get all clips for a game
    pub async fn get_clips_by_game(&self, game_id: i64) -> Result<Vec<models::ClipRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, game_id, event_type, event_time, priority, file_path, thumbnail_path, duration, created_at
            FROM clips
            WHERE game_id = ?
            ORDER BY priority DESC, event_time ASC
            "#,
        )
        .bind(game_id)
        .fetch_all(&self.pool)
        .await?;

        let clips = rows
            .into_iter()
            .map(|row| models::ClipRecord {
                id: row.try_get("id").ok(),
                game_id: row.get("game_id"),
                event_type: row.get("event_type"),
                event_time: row.get("event_time"),
                priority: row.get("priority"),
                file_path: row.get("file_path"),
                thumbnail_path: row.try_get("thumbnail_path").ok(),
                duration: row.try_get("duration").ok(),
                created_at: row.try_get("created_at").ok(),
            })
            .collect();

        Ok(clips)
    }

    /// Delete a clip by ID
    pub async fn delete_clip(&self, clip_id: i64) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM clips WHERE id = ?
            "#,
        )
        .bind(clip_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get recent games
    pub async fn get_recent_games(&self, limit: i64) -> Result<Vec<models::GameRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, game_id, champion, game_mode, start_time, end_time, kda, created_at
            FROM games
            ORDER BY start_time DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let games = rows
            .into_iter()
            .map(|row| models::GameRecord {
                id: row.try_get("id").ok(),
                game_id: row.get("game_id"),
                champion: row.get("champion"),
                game_mode: row.get("game_mode"),
                start_time: row.get("start_time"),
                end_time: row.try_get("end_time").ok(),
                kda: row.try_get("kda").ok(),
                created_at: row.try_get("created_at").ok(),
            })
            .collect();

        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new(":memory:").await;
        assert!(db.is_ok());
    }
}
