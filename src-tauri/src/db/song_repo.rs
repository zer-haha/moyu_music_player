use crate::db::connection::Database;
use crate::errors::{AppError, AppResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongDto {
    pub id: i64,
    pub path: String,
    pub path_key: String,
    pub file_name: String,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub track_number: Option<i32>,
    pub disc_number: Option<i32>,
    pub duration_ms: i64,
    pub format: Option<String>,
    pub extension: Option<String>,
    pub codec_hint: Option<String>,
    pub size_bytes: i64,
    pub modified_time: i64,
    pub added_at: i64,
    pub updated_at: i64,
    pub last_played_at: Option<i64>,
    pub play_count: i64,
    pub playable: bool,
    pub missing: bool,
    pub last_error: Option<String>,
}

pub fn normalize_path_key(path: &str) -> String {
    path.replace('/', "\\")
        .to_lowercase()
        .split('\\')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\\")
}

pub fn insert_song(db: &Database, song: &SongDto) -> AppResult<i64> {
    let conn = db.conn();
    conn.execute(
        "INSERT OR REPLACE INTO songs (path, path_key, file_name, title, artist, album, album_artist, genre, track_number, disc_number, duration_ms, format, extension, codec_hint, size_bytes, modified_time, added_at, updated_at, playable, missing, last_error)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)",
        params![
            song.path, song.path_key, song.file_name, song.title, song.artist,
            song.album, song.album_artist, song.genre, song.track_number, song.disc_number,
            song.duration_ms, song.format, song.extension, song.codec_hint,
            song.size_bytes, song.modified_time, song.added_at, song.updated_at,
            song.playable as i32, song.missing as i32, song.last_error,
        ],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(conn.last_insert_rowid())
}

pub fn get_song_by_id(db: &Database, id: i64) -> AppResult<Option<SongDto>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, path, path_key, file_name, title, artist, album, album_artist, genre, track_number, disc_number, duration_ms, format, extension, codec_hint, size_bytes, modified_time, added_at, updated_at, last_played_at, play_count, playable, missing, last_error FROM songs WHERE id = ?1"
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let result = stmt.query_row(params![id], |row| Ok(row_to_song(row)));
    match result {
        Ok(song) => Ok(Some(song)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

pub fn get_song_by_path_key(db: &Database, path_key: &str) -> AppResult<Option<SongDto>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, path, path_key, file_name, title, artist, album, album_artist, genre, track_number, disc_number, duration_ms, format, extension, codec_hint, size_bytes, modified_time, added_at, updated_at, last_played_at, play_count, playable, missing, last_error FROM songs WHERE path_key = ?1"
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let result = stmt.query_row(params![path_key], |row| Ok(row_to_song(row)));
    match result {
        Ok(song) => Ok(Some(song)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DatabaseError(e.to_string())),
    }
}

pub fn mark_song_unplayable(db: &Database, song_id: i64, error: &str) -> AppResult<()> {
    let conn = db.conn();
    conn.execute(
        "UPDATE songs SET playable = 0, last_error = ?2, updated_at = ?3 WHERE id = ?1",
        params![song_id, error, chrono::Utc::now().timestamp()],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn mark_song_missing(db: &Database, song_id: i64) -> AppResult<()> {
    let conn = db.conn();
    conn.execute(
        "UPDATE songs SET missing = 1, playable = 0, last_error = '文件不存在', updated_at = ?2 WHERE id = ?1",
        params![song_id, chrono::Utc::now().timestamp()],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn delete_song(db: &Database, song_id: i64) -> AppResult<()> {
    let conn = db.conn();
    conn.execute("DELETE FROM songs WHERE id = ?1", params![song_id])
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn update_play_count(db: &Database, song_id: i64) -> AppResult<()> {
    let conn = db.conn();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE songs SET play_count = play_count + 1, last_played_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![song_id, now],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

fn row_to_song(row: &rusqlite::Row<'_>) -> SongDto {
    let playable: i32 = row.get(21).unwrap_or(1);
    let missing: i32 = row.get(22).unwrap_or(0);
    SongDto {
        id: row.get(0).unwrap_or(0),
        path: row.get(1).unwrap_or_default(),
        path_key: row.get(2).unwrap_or_default(),
        file_name: row.get(3).unwrap_or_default(),
        title: row.get(4).unwrap_or_default(),
        artist: row.get(5).unwrap_or(None),
        album: row.get(6).unwrap_or(None),
        album_artist: row.get(7).unwrap_or(None),
        genre: row.get(8).unwrap_or(None),
        track_number: row.get(9).unwrap_or(None),
        disc_number: row.get(10).unwrap_or(None),
        duration_ms: row.get(11).unwrap_or(0),
        format: row.get(12).unwrap_or(None),
        extension: row.get(13).unwrap_or(None),
        codec_hint: row.get(14).unwrap_or(None),
        size_bytes: row.get(15).unwrap_or(0),
        modified_time: row.get(16).unwrap_or(0),
        added_at: row.get(17).unwrap_or(0),
        updated_at: row.get(18).unwrap_or(0),
        last_played_at: row.get(19).unwrap_or(None),
        play_count: row.get(20).unwrap_or(0),
        playable: playable != 0,
        missing: missing != 0,
        last_error: row.get(23).unwrap_or(None),
    }
}
