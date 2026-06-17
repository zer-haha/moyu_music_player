use crate::app_state::AppState;
use crate::audio::audio_state::{AudioDiagnosticsDto, AudioStateDto, PlayMode};
use crate::errors::AppErrorDto;
use tauri::State;

type CmdResult<T> = Result<T, AppErrorDto>;

#[tauri::command]
pub async fn audio_play(
    state: State<'_, AppState>,
    song_id: i64,
    path: String,
) -> CmdResult<AudioStateDto> {
    state.audio.play(song_id, path).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_pause(state: State<'_, AppState>) -> CmdResult<AudioStateDto> {
    state.audio.pause().map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_resume(state: State<'_, AppState>) -> CmdResult<AudioStateDto> {
    state.audio.resume().map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_toggle_pause(state: State<'_, AppState>) -> CmdResult<AudioStateDto> {
    state.audio.toggle_pause().map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_stop(state: State<'_, AppState>) -> CmdResult<AudioStateDto> {
    state.audio.stop().map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_seek(
    state: State<'_, AppState>,
    seconds: f64,
) -> CmdResult<AudioStateDto> {
    state.audio.seek(seconds).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_set_volume(
    state: State<'_, AppState>,
    volume: f32,
) -> CmdResult<AudioStateDto> {
    state.audio.set_volume(volume).map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_get_state(state: State<'_, AppState>) -> CmdResult<AudioStateDto> {
    state.audio.get_state().map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_get_diagnostics(
    state: State<'_, AppState>,
) -> CmdResult<AudioDiagnosticsDto> {
    state.audio.get_diagnostics().map_err(|e| e.to_dto())
}

#[tauri::command]
pub async fn audio_set_play_mode(
    state: State<'_, AppState>,
    mode: PlayMode,
) -> CmdResult<AudioStateDto> {
    state.audio.set_play_mode(mode).map_err(|e| e.to_dto())
}
