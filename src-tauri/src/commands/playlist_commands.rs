use crate::app_state::AppState;
use crate::db::playlist_repo::{self, PlaylistDto};
use crate::db::song_repo::SongDto;
use crate::errors::AppErrorDto;
use tauri::State;

type CmdResult<T> = Result<T, AppErrorDto>;

#[tauri::command]
pub async fn playlist_create(
    state: State<'_, AppState>,
    name: String,
) -> CmdResult<PlaylistDto> {
    playlist_repo::create_playlist(&state.db, &name, "custom", None)
        .map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn playlist_rename(
    state: State<'_, AppState>,
    id: i64,
    name: String,
) -> CmdResult<PlaylistDto> {
    playlist_repo::rename_playlist(&state.db, id, &name)
        .map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn playlist_delete(
    state: State<'_, AppState>,
    id: i64,
) -> CmdResult<()> {
    playlist_repo::delete_playlist(&state.db, id).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn playlist_clear(
    state: State<'_, AppState>,
    id: i64,
) -> CmdResult<()> {
    playlist_repo::clear_playlist(&state.db, id).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn playlist_get_all(
    state: State<'_, AppState>,
) -> CmdResult<Vec<PlaylistDto>> {
    playlist_repo::get_all_playlists(&state.db).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn playlist_get_songs(
    state: State<'_, AppState>,
    playlist_id: i64,
) -> CmdResult<Vec<SongDto>> {
    playlist_repo::get_playlist_songs(&state.db, playlist_id)
        .map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn playlist_song_reorder(
    state: State<'_, AppState>,
    playlist_id: i64,
    song_ids: Vec<i64>,
) -> CmdResult<()> {
    playlist_repo::reorder_playlist_songs(&state.db, playlist_id, &song_ids)
        .map_err(|e| e.to_dto())
}
