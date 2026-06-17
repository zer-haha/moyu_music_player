use crate::db::connection::Database;
use crate::db::song_repo;
use crate::errors::{AppError, AppResult};
use crate::library::metadata;
use crate::library::supported_formats;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanTaskDto {
    pub task_id: String,
    pub root_path: String,
    pub playlist_id: i64,
    pub status: String,
    pub scanned_files: u32,
    pub matched_audio_files: u32,
    pub added: u32,
    pub updated: u32,
    pub skipped_duplicate: u32,
    pub failed: u32,
    pub current_path: Option<String>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanStatusDto {
    pub task_id: String,
    pub status: String,
    pub scanned_files: u32,
    pub matched_audio_files: u32,
    pub added: u32,
    pub skipped_duplicate: u32,
    pub failed: u32,
    pub current_path: Option<String>,
}

pub struct ScanTask {
    pub task_id: String,
    pub root_path: PathBuf,
    pub playlist_id: i64,
    pub cancelled: Arc<AtomicBool>,
}

impl ScanTask {
    pub fn new(root_path: PathBuf, playlist_id: i64) -> Self {
        Self {
            task_id: uuid::Uuid::new_v4().to_string(),
            root_path,
            playlist_id,
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub fn scan_folder(
    db: Database,
    app_handle: AppHandle,
    task: ScanTask,
) -> ScanTaskDto {
    let task_id = task.task_id.clone();
    let root_path = task.root_path.to_string_lossy().to_string();
    let playlist_id = task.playlist_id;
    let started_at = chrono::Utc::now().timestamp();

    let _ = app_handle.emit("scan://started", serde_json::json!({
        "task_id": task_id,
        "folder": root_path,
    }));

    let mut scanned_files: u32 = 0;
    let mut matched_audio_files: u32 = 0;
    let mut added: u32 = 0;
    let mut updated: u32 = 0;
    let mut skipped_duplicate: u32 = 0;
    let mut failed: u32 = 0;

    for entry in WalkDir::new(&task.root_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if task.cancelled.load(Ordering::Relaxed) {
            let _ = app_handle.emit("scan://cancelled", serde_json::json!({
                "task_id": task_id,
                "added": added,
            }));
            return ScanTaskDto {
                task_id: task_id.clone(),
                root_path: root_path.clone(),
                playlist_id,
                status: "cancelled".to_string(),
                scanned_files,
                matched_audio_files,
                added,
                updated,
                skipped_duplicate,
                failed,
                current_path: None,
                started_at,
                finished_at: Some(chrono::Utc::now().timestamp()),
            };
        }

        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        scanned_files += 1;

        if !supported_formats::is_supported_audio_file(path) {
            continue;
        }

        matched_audio_files += 1;
        let path_str = path.to_string_lossy().to_string();

        let _ = app_handle.emit("scan://progress", serde_json::json!({
            "task_id": task_id,
            "folder": root_path,
            "scanned_files": scanned_files,
            "matched_audio_files": matched_audio_files,
            "added": added,
            "skipped_duplicate": skipped_duplicate,
            "failed": failed,
            "current_path": path_str,
        }));

        let path_key = song_repo::normalize_path_key(&path_str);

        // Check if already in database
        match song_repo::get_song_by_path_key(&db, &path_key) {
            Ok(Some(existing)) => {
                // Check if file changed
                if existing.size_bytes > 0 {
                    let meta = std::fs::metadata(&path_str).ok();
                    let current_size = meta.as_ref().map(|m| m.len() as i64).unwrap_or(0);
                    let current_modified = meta
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    if existing.size_bytes == current_size && existing.modified_time == current_modified {
                        // No change, just add to playlist
                        let _ = crate::db::playlist_repo::add_song_to_playlist(&db, playlist_id, existing.id);
                        skipped_duplicate += 1;
                        continue;
                    }
                }
                // File changed, re-read metadata
                skipped_duplicate += 1;
                let _ = crate::db::playlist_repo::add_song_to_playlist(&db, playlist_id, existing.id);
            }
            Ok(None) => {
                // New file, read metadata and insert
                match std::panic::catch_unwind(|| metadata::read_metadata(&path_str)) {
                    Ok(mut song) => {
                        match song_repo::insert_song(&db, &song) {
                            Ok(id) => {
                                song.id = id;
                                let _ = crate::db::playlist_repo::add_song_to_playlist(&db, playlist_id, id);
                                added += 1;
                            }
                            Err(e) => {
                                tracing::error!("Failed to insert song '{}': {}", path_str, e);
                                failed += 1;
                            }
                        }
                    }
                    Err(_) => {
                        tracing::error!("Panic reading metadata for '{}'", path_str);
                        failed += 1;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Database error checking path_key '{}': {}", path_key, e);
                failed += 1;
            }
        }
    }

    let _ = app_handle.emit("scan://finished", serde_json::json!({
        "task_id": task_id,
        "folder": root_path,
        "scanned_files": scanned_files,
        "matched_audio_files": matched_audio_files,
        "added": added,
        "skipped_duplicate": skipped_duplicate,
        "failed": failed,
    }));

    let _ = app_handle.emit("library://changed", serde_json::json!({}));
    let _ = app_handle.emit("playlist://changed", serde_json::json!({}));

    ScanTaskDto {
        task_id: task_id.clone(),
        root_path: root_path.clone(),
        playlist_id,
        status: "finished".to_string(),
        scanned_files,
        matched_audio_files,
        added,
        updated,
        skipped_duplicate,
        failed,
        current_path: None,
        started_at,
        finished_at: Some(chrono::Utc::now().timestamp()),
    }
}

pub fn add_files(
    db: &Database,
    playlist_id: i64,
    paths: Vec<String>,
) -> (u32, u32, u32) {
    let mut added: u32 = 0;
    let mut skipped: u32 = 0;
    let mut failed: u32 = 0;

    for path_str in paths {
        let path = std::path::Path::new(&path_str);
        if !path.is_file() || !supported_formats::is_supported_audio_file(path) {
            continue;
        }

        let path_key = song_repo::normalize_path_key(&path_str);

        match song_repo::get_song_by_path_key(db, &path_key) {
            Ok(Some(existing)) => {
                let _ = crate::db::playlist_repo::add_song_to_playlist(db, playlist_id, existing.id);
                skipped += 1;
            }
            Ok(None) => {
                match std::panic::catch_unwind(|| metadata::read_metadata(&path_str)) {
                    Ok(song) => {
                        match song_repo::insert_song(db, &song) {
                            Ok(id) => {
                                let _ = crate::db::playlist_repo::add_song_to_playlist(db, playlist_id, id);
                                added += 1;
                            }
                            Err(_) => failed += 1,
                        }
                    }
                    Err(_) => failed += 1,
                }
            }
            Err(_) => failed += 1,
        }
    }

    (added, skipped, failed)
}
