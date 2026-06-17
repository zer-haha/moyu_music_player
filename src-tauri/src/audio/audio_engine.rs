use crate::audio::audio_state::*;
use crate::audio::audio_worker::{AudioCommand, AudioWorker};
use crate::errors::{AppError, AppResult};
use crossbeam_channel::{bounded, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::AppHandle;

pub struct AudioEngine {
    cmd_tx: Sender<AudioCommand>,
    state: Arc<Mutex<AudioState>>,
}

impl AudioEngine {
    pub fn new(app_handle: AppHandle) -> Self {
        let (cmd_tx, cmd_rx) = bounded::<AudioCommand>(64);
        let state = Arc::new(Mutex::new(AudioState::default()));

        let worker = AudioWorker::new(app_handle, cmd_rx);
        std::thread::Builder::new()
            .name("audio-worker".to_string())
            .spawn(move || worker.run())
            .expect("Failed to spawn audio worker thread");

        Self { cmd_tx, state }
    }

    pub fn play(&self, song_id: i64, path: String) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::Play { song_id, path, reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(Ok(dto)) => Ok(dto),
            Ok(Err(e)) => Err(AppError::PlaybackError(e)),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn pause(&self) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::Pause { reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(Ok(dto)) => Ok(dto),
            Ok(Err(e)) => Err(AppError::PlaybackError(e)),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn resume(&self) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::Resume { reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(Ok(dto)) => Ok(dto),
            Ok(Err(e)) => Err(AppError::PlaybackError(e)),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn toggle_pause(&self) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::TogglePause { reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(Ok(dto)) => Ok(dto),
            Ok(Err(e)) => Err(AppError::PlaybackError(e)),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn stop(&self) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::Stop { reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(Ok(dto)) => Ok(dto),
            Ok(Err(e)) => Err(AppError::PlaybackError(e)),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn seek(&self, seconds: f64) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::Seek { seconds, reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(Ok(dto)) => Ok(dto),
            Ok(Err(e)) => Err(AppError::SeekFailed(e)),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn set_volume(&self, volume: f32) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::SetVolume { volume, reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(Ok(dto)) => Ok(dto),
            Ok(Err(e)) => Err(AppError::PlaybackError(e)),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn get_state(&self) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::GetState { reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(dto) => Ok(dto),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn set_play_mode(&self, mode: PlayMode) -> AppResult<AudioStateDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::SetPlayMode { mode, reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(dto) => Ok(dto),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn get_diagnostics(&self) -> AppResult<AudioDiagnosticsDto> {
        let (reply_tx, reply_rx) = bounded(1);
        self.cmd_tx.send(AudioCommand::GetDiagnostics { reply: reply_tx })
            .map_err(|_| AppError::InternalError("Audio worker disconnected".to_string()))?;
        match reply_rx.recv() {
            Ok(dto) => Ok(dto),
            Err(_) => Err(AppError::InternalError("Audio worker reply channel closed".to_string())),
        }
    }

    pub fn notify_ended(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Ended);
    }

    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Shutdown);
    }
}
