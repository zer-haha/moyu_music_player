use crate::errors::{AppError, AppResult};
use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> AppResult<()> {
    // Create migrations table
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );"
    ).map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let current_version = get_current_version(conn)?;

    if current_version < 1 {
        migration_001(conn)?;
    }

    Ok(())
}

fn get_current_version(conn: &Connection) -> AppResult<i64> {
    let version: i64 = conn
        .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |row| row.get(0))
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(version)
}

fn migration_001(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL,
            path_key TEXT NOT NULL UNIQUE,
            file_name TEXT NOT NULL,
            title TEXT NOT NULL,
            artist TEXT,
            album TEXT,
            album_artist TEXT,
            genre TEXT,
            track_number INTEGER,
            disc_number INTEGER,
            duration_ms INTEGER DEFAULT 0,
            format TEXT,
            extension TEXT,
            codec_hint TEXT,
            size_bytes INTEGER DEFAULT 0,
            modified_time INTEGER DEFAULT 0,
            added_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            last_played_at INTEGER,
            play_count INTEGER DEFAULT 0,
            playable INTEGER DEFAULT 1,
            missing INTEGER DEFAULT 0,
            last_error TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
        CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist);
        CREATE INDEX IF NOT EXISTS idx_songs_album ON songs(album);
        CREATE INDEX IF NOT EXISTS idx_songs_path_key ON songs(path_key);

        CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'custom',
            source_folder TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS playlist_songs (
            playlist_id INTEGER NOT NULL,
            song_id INTEGER NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            added_at INTEGER NOT NULL,
            PRIMARY KEY (playlist_id, song_id),
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_playlist_songs_playlist ON playlist_songs(playlist_id, sort_order);
        CREATE INDEX IF NOT EXISTS idx_playlist_songs_song ON playlist_songs(song_id);

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS playback_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            current_playlist_id INTEGER,
            current_song_id INTEGER,
            position_ms INTEGER DEFAULT 0,
            volume REAL DEFAULT 0.8,
            muted INTEGER DEFAULT 0,
            play_mode TEXT DEFAULT 'list_loop',
            updated_at INTEGER NOT NULL
        );"
    ).map_err(|e| AppError::DatabaseError(format!("Migration 001 failed: {}", e)))?;

    // Create default playlist
    let now = chrono::Utc::now().timestamp();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM playlists WHERE id = 1", [], |row| row.get(0))
        .unwrap_or(0);

    if count == 0 {
        conn.execute(
            "INSERT INTO playlists (id, name, kind, sort_order, created_at, updated_at) VALUES (1, '默认播放列表', 'default', 0, ?1, ?1)",
            [now],
        ).map_err(|e| AppError::DatabaseError(format!("创建默认播放列表失败: {}", e)))?;
    }

    // Initialize playback state
    let state_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM playback_state", [], |row| row.get(0))
        .unwrap_or(0);

    if state_count == 0 {
        conn.execute(
            "INSERT INTO playback_state (id, volume, play_mode, updated_at) VALUES (1, 0.8, 'list_loop', ?1)",
            [now],
        ).map_err(|e| AppError::DatabaseError(format!("初始化播放状态失败: {}", e)))?;
    }

    // Record migration
    conn.execute(
        "INSERT OR REPLACE INTO schema_migrations (version, applied_at) VALUES (1, ?1)",
        [now],
    ).map_err(|e| AppError::DatabaseError(format!("记录迁移失败: {}", e)))?;

    tracing::info!("Database migration 001 applied");
    Ok(())
}
