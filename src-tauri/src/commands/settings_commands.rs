use crate::app_state::AppState;
use crate::db::settings_repo::{self, PlaybackStateDto};
use crate::errors::AppErrorDto;
use crate::settings::settings::WindowStateDto;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;

type CmdResult<T> = Result<T, AppErrorDto>;

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> CmdResult<HashMap<String, String>> {
    settings_repo::get_all_settings(&state.db).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn settings_update(
    state: State<'_, AppState>,
    settings: HashMap<String, String>,
) -> CmdResult<HashMap<String, String>> {
    for (key, value) in &settings {
        settings_repo::set_setting(&state.db, key, value).map_err(|e| e.to_dto())?;
    }
    settings_repo::get_all_settings(&state.db).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn settings_get_playback_state(
    state: State<'_, AppState>,
) -> CmdResult<PlaybackStateDto> {
    settings_repo::get_playback_state(&state.db).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn settings_save_playback_state(
    state: State<'_, AppState>,
    playback_state: PlaybackStateDto,
) -> CmdResult<()> {
    settings_repo::save_playback_state(&state.db, &playback_state).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn settings_get_db_path(state: State<'_, AppState>) -> CmdResult<String> {
    Ok(state.db.db_path().to_string_lossy().to_string())
}
