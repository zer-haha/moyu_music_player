use crate::db::connection::Database;
use crate::db::song_repo::SongDto;
use crate::errors::{AppError, AppResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistDto {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub source_folder: Option<String>,
    pub sort_order: i32,
    pub created_at: i64,
    pub updated_at: i64,
    pub track_count: i32,
}

pub fn get_all_playlists(db: &Database) -> AppResult<Vec<PlaylistDto>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT p.id, p.name, p.kind, p.source_folder, p.sort_order, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM playlist_songs ps WHERE ps.playlist_id = p.id) AS track_count
         FROM playlists p ORDER BY p.sort_order, p.id"
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let rows = stmt.query_map([], |row| {
        Ok(PlaylistDto {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: row.get(2)?,
            source_folder: row.get(3)?,
            sort_order: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
            track_count: row.get(7)?,
        })
    }).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut playlists = Vec::new();
    for row in rows {
        playlists.push(row.map_err(|e| AppError::DatabaseError(e.to_string()))?);
    }
    Ok(playlists)
}

pub fn create_playlist(db: &Database, name: &str, kind: &str, source_folder: Option<&str>) -> AppResult<PlaylistDto> {
    let conn = db.conn();
    let now = chrono::Utc::now().timestamp();
    let max_order: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) FROM playlists", [], |row| row.get(0)
    ).unwrap_or(0);

    conn.execute(
        "INSERT INTO playlists (name, kind, source_folder, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![name, kind, source_folder, max_order + 1, now],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let id = conn.last_insert_rowid();
    Ok(PlaylistDto {
        id,
        name: name.to_string(),
        kind: kind.to_string(),
        source_folder: source_folder.map(|s| s.to_string()),
        sort_order: max_order + 1,
        created_at: now,
        updated_at: now,
        track_count: 0,
    })
}

pub fn rename_playlist(db: &Database, id: i64, name: &str) -> AppResult<PlaylistDto> {
    let conn = db.conn();
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE playlists SET name = ?2, updated_at = ?3 WHERE id = ?1",
        params![id, name, now],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let pl = conn.query_row(
        "SELECT p.id, p.name, p.kind, p.source_folder, p.sort_order, p.created_at, p.updated_at,
                (SELECT COUNT(*) FROM playlist_songs ps WHERE ps.playlist_id = p.id) AS track_count
         FROM playlists p WHERE p.id = ?1",
        params![id],
        |row| Ok(PlaylistDto {
            id: row.get(0)?, name: row.get(1)?, kind: row.get(2)?,
            source_folder: row.get(3)?, sort_order: row.get(4)?,
            created_at: row.get(5)?, updated_at: row.get(6)?,
            track_count: row.get(7)?,
        })
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(pl)
}

pub fn delete_playlist(db: &Database, id: i64) -> AppResult<()> {
    // Don't allow deleting default playlist
    let conn = db.conn();
    let kind: String = conn.query_row(
        "SELECT kind FROM playlists WHERE id = ?1", params![id], |row| row.get(0)
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if kind == "default" {
        return Err(AppError::InvalidArgument("默认播放列表不能删除".to_string()));
    }

    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", params![id])
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn clear_playlist(db: &Database, id: i64) -> AppResult<()> {
    let conn = db.conn();
    conn.execute("DELETE FROM playlist_songs WHERE playlist_id = ?1", params![id])
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn get_playlist_songs(db: &Database, playlist_id: i64) -> AppResult<Vec<SongDto>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT s.id, s.path, s.path_key, s.file_name, s.title, s.artist, s.album, s.album_artist, s.genre, s.track_number, s.disc_number, s.duration_ms, s.format, s.extension, s.codec_hint, s.size_bytes, s.modified_time, s.added_at, s.updated_at, s.last_played_at, s.play_count, s.playable, s.missing, s.last_error
         FROM songs s
         INNER JOIN playlist_songs ps ON ps.song_id = s.id
         WHERE ps.playlist_id = ?1
         ORDER BY ps.sort_order, ps.added_at"
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let rows = stmt.query_map(params![playlist_id], |row| {
        let playable: i32 = row.get(21)?;
        let missing: i32 = row.get(22)?;
        Ok(SongDto {
            id: row.get(0)?, path: row.get(1)?, path_key: row.get(2)?,
            file_name: row.get(3)?, title: row.get(4)?, artist: row.get(5)?,
            album: row.get(6)?, album_artist: row.get(7)?, genre: row.get(8)?,
            track_number: row.get(9)?, disc_number: row.get(10)?,
            duration_ms: row.get(11)?, format: row.get(12)?, extension: row.get(13)?,
            codec_hint: row.get(14)?, size_bytes: row.get(15)?, modified_time: row.get(16)?,
            added_at: row.get(17)?, updated_at: row.get(18)?, last_played_at: row.get(19)?,
            play_count: row.get(20)?, playable: playable != 0, missing: missing != 0,
            last_error: row.get(23)?,
        })
    }).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut songs = Vec::new();
    for row in rows {
        songs.push(row.map_err(|e| AppError::DatabaseError(e.to_string()))?);
    }
    Ok(songs)
}

pub fn add_song_to_playlist(db: &Database, playlist_id: i64, song_id: i64) -> AppResult<bool> {
    let conn = db.conn();
    // Check if already in playlist
    let exists: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        params![playlist_id, song_id],
        |row| row.get(0),
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    if exists {
        return Ok(false);
    }

    let max_order: i32 = conn.query_row(
        "SELECT COALESCE(MAX(sort_order), 0) FROM playlist_songs WHERE playlist_id = ?1",
        params![playlist_id],
        |row| row.get(0),
    ).unwrap_or(0);

    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO playlist_songs (playlist_id, song_id, sort_order, added_at) VALUES (?1, ?2, ?3, ?4)",
        params![playlist_id, song_id, max_order + 1, now],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(true)
}

pub fn remove_song_from_playlist(db: &Database, playlist_id: i64, song_id: i64) -> AppResult<()> {
    let conn = db.conn();
    conn.execute(
        "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
        params![playlist_id, song_id],
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn reorder_playlist_songs(db: &Database, playlist_id: i64, song_ids: &[i64]) -> AppResult<()> {
    let conn = db.conn();
    for (i, song_id) in song_ids.iter().enumerate() {
        conn.execute(
            "UPDATE playlist_songs SET sort_order = ?3 WHERE playlist_id = ?1 AND song_id = ?2",
            params![playlist_id, song_id, i as i32],
        ).map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }
    Ok(())
}

/// 持久化播放列表侧栏排序
pub fn reorder_playlists(db: &Database, playlist_ids: &[i64]) -> AppResult<()> {
    let conn = db.conn();
    let now = chrono::Utc::now().timestamp();
    for (i, id) in playlist_ids.iter().enumerate() {
        conn.execute(
            "UPDATE playlists SET sort_order = ?2, updated_at = ?3 WHERE id = ?1",
            params![id, i as i32, now],
        )
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }
    Ok(())
}
