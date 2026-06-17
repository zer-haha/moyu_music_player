use crate::app_state::AppState;
use crate::db::{playlist_repo, song_repo};
use crate::errors::AppErrorDto;
use crate::library::scanner;
use serde::{Deserialize, Serialize};
use tauri::State;

type CmdResult<T> = Result<T, AppErrorDto>;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddResultDto {
    pub added: u32,
    pub skipped: u32,
    pub failed: u32,
}

#[tauri::command]
pub async fn library_add_files(
    state: State<'_, AppState>,
    paths: Vec<String>,
    playlist_id: i64,
) -> CmdResult<AddResultDto> {
    let (added, skipped, failed) = scanner::add_files(&state.db, playlist_id, paths);
    Ok(AddResultDto { added, skipped, failed })
}

#[tauri::command]
pub async fn library_remove_song_from_playlist(
    state: State<'_, AppState>,
    playlist_id: i64,
    song_id: i64,
) -> CmdResult<()> {
    playlist_repo::remove_song_from_playlist(&state.db, playlist_id, song_id)
        .map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn library_delete_song(
    state: State<'_, AppState>,
    song_id: i64,
) -> CmdResult<()> {
    song_repo::delete_song(&state.db, song_id).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn library_mark_song_unplayable(
    state: State<'_, AppState>,
    song_id: i64,
    error: String,
) -> CmdResult<()> {
    song_repo::mark_song_unplayable(&state.db, song_id, &error)
        .map_err(|e| e.to_dto())
}
