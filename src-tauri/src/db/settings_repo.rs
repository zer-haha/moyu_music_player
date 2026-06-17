use crate::db::connection::Database;
use crate::errors::{AppError, AppResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub fn get_setting(db: &Database, key: &str) -> AppResult<Option<String>> {
    let conn = db.conn();
    let result = conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

pub fn set_setting(db: &Database, key: &str, value: &str) -> AppResult<()> {
    let conn = db.conn();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
        params![key, value, now],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn get_all_settings(db: &Database) -> AppResult<HashMap<String, String>> {
    let conn = db.conn();
    let mut stmt = conn.prepare("SELECT key, value FROM app_settings")
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut map = HashMap::new();
    for row in rows {
        let (k, v) = row.map_err(|e| AppError::DatabaseError(e.to_string()))?;
        map.insert(k, v);
    }
    Ok(map)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackStateDto {
    pub current_playlist_id: Option<i64>,
    pub current_song_id: Option<i64>,
    pub position_ms: i64,
    pub volume: f64,
    pub muted: bool,
    pub play_mode: String,
}

pub fn get_playback_state(db: &Database) -> AppResult<PlaybackStateDto> {
    let conn = db.conn();
    let result = conn.query_row(
        "SELECT current_playlist_id, current_song_id, position_ms, volume, muted, play_mode FROM playback_state WHERE id = 1",
        [],
        |row| {
            let muted: i32 = row.get(4)?;
            Ok(PlaybackStateDto {
                current_playlist_id: row.get(0)?,
                current_song_id: row.get(1)?,
                position_ms: row.get(2)?,
                volume: row.get(3)?,
                muted: muted != 0,
                play_mode: row.get(5)?,
            })
        },
    );
    match result {
        Ok(s) => Ok(s),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(PlaybackStateDto {
            current_playlist_id: Some(1),
            current_song_id: None,
            position_ms: 0,
            volume: 0.8,
            muted: false,
            play_mode: "list_loop".to_string(),
        }),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

pub fn save_playback_state(db: &Database, state: &PlaybackStateDto) -> AppResult<()> {
    let conn = db.conn();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO playback_state (id, current_playlist_id, current_song_id, position_ms, volume, muted, play_mode, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            state.current_playlist_id,
            state.current_song_id,
            state.position_ms,
            state.volume,
            state.muted as i32,
            state.play_mode,
            now,
        ],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}
