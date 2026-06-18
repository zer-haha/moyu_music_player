use crate::audio::audio_state::*;
use crate::audio::bass_ffi::*;
use crate::audio::bass_loader::{BassFunctions, BassLoader};
use crate::errors::AppError;
use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use std::sync::atomic::{AtomicBool, Ordering};

pub enum AudioCommand {
    Play {
        song_id: i64,
        path: String,
        reply: crossbeam_channel::Sender<Result<AudioStateDto, String>>,
    },
    Pause {
        reply: crossbeam_channel::Sender<Result<AudioStateDto, String>>,
    },
    Resume {
        reply: crossbeam_channel::Sender<Result<AudioStateDto, String>>,
    },
    Stop {
        reply: crossbeam_channel::Sender<Result<AudioStateDto, String>>,
    },
    Seek {
        seconds: f64,
        reply: crossbeam_channel::Sender<Result<AudioStateDto, String>>,
    },
    SetVolume {
        volume: f32,
        reply: crossbeam_channel::Sender<Result<AudioStateDto, String>>,
    },
    TogglePause {
        reply: crossbeam_channel::Sender<Result<AudioStateDto, String>>,
    },
    GetState {
        reply: crossbeam_channel::Sender<AudioStateDto>,
    },
    SetPlayMode {
        mode: PlayMode,
        reply: crossbeam_channel::Sender<AudioStateDto>,
    },
    GetDiagnostics {
        reply: crossbeam_channel::Sender<AudioDiagnosticsDto>,
    },
    Ended,
    Shutdown,
}

pub struct AudioWorker {
    funcs: Option<BassFunctions>,
    loader: Option<BassLoader>,
    current_stream: Option<HSTREAM>,
    state: AudioState,
    bass_init_ok: bool,
    bass_version: Option<u32>,
    audio_core_path: Option<String>,
    last_bass_error: Option<i32>,
    app_handle: AppHandle,
    cmd_rx: Receiver<AudioCommand>,
    end_flag: Arc<AtomicBool>,
    last_recovery_at: Option<Instant>,
}

impl AudioWorker {
    pub fn new(
        app_handle: AppHandle,
        cmd_rx: Receiver<AudioCommand>,
    ) -> Self {
        Self {
            funcs: None,
            loader: None,
            current_stream: None,
            state: AudioState::default(),
            bass_init_ok: false,
            bass_version: None,
            audio_core_path: None,
            last_bass_error: None,
            app_handle,
            cmd_rx,
            end_flag: Arc::new(AtomicBool::new(false)),
            last_recovery_at: None,
        }
    }

    pub fn run(mut self) {
        // Initialize BASS
        if let Err(e) = self.init_bass() {
            tracing::error!("BASS initialization failed: {}", e);
            self.state.last_error = Some(e.to_string());
        }

        // Main loop
        loop {
            // Use recv_timeout to periodically check state
            match self.cmd_rx.recv_timeout(std::time::Duration::from_millis(300)) {
                Ok(cmd) => {
                    if matches!(cmd, AudioCommand::Shutdown) {
                        self.shutdown();
                        break;
                    }
                    self.handle_command(cmd);
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    if self.current_stream.is_some() {
                        let was_playing = self.state.status == PlaybackStatus::Playing;
                        self.update_position();
                        if was_playing && self.state.status == PlaybackStatus::Ended {
                            if self.is_premature_stop() {
                                let _ = self.try_recover_playback("poll_premature_stop");
                            } else {
                                self.handle_track_ended();
                            }
                        } else if was_playing {
                            self.check_playback_stall();
                        }
                    }
                    self.push_state();
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    self.shutdown();
                    break;
                }
            }
        }
    }

    fn init_bass(&mut self) -> Result<(), AppError> {
        if self.bass_init_ok {
            return Ok(());
        }

        let audio_core_dir = if let Some(ref path) = self.audio_core_path {
            PathBuf::from(path)
        } else {
            let dir = crate::audio::bass_loader::resolve_audio_core_dir(&self.app_handle)?;
            self.audio_core_path = Some(dir.to_string_lossy().to_string());
            dir
        };

        if self.loader.is_none() {
            let loader = BassLoader::load(&audio_core_dir)?;
            let funcs = loader.load_functions()?;
            self.loader = Some(loader);
            self.funcs = Some(funcs);
        }

        let funcs = self.funcs.as_ref().ok_or_else(|| {
            AppError::AudioCoreMissing("BASS 未加载".to_string())
        })?;

        // 跟随系统默认输出设备（蓝牙耳机切换时自动跟踪）
        if let Some(set_config) = funcs.bass_set_config {
            unsafe {
                (set_config)(BASS_CONFIG_DEV_DEFAULT, 1);
                (set_config)(BASS_CONFIG_DEV_NONSTOP, 1);
            }
        }

        // BASS_Init(-1, 44100, 0, 0, NULL)
        let result = unsafe { (funcs.bass_init)(-1, 44100, 0, std::ptr::null_mut(), std::ptr::null()) };
        if result == 0 {
            let err = unsafe { (funcs.bass_error_get_code)() };
            self.last_bass_error = Some(err);
            return Err(AppError::AudioInitFailed(format!(
                "BASS_Init 失败: {} ({})",
                bass_error_name(err as u32),
                err
            )));
        }

        self.bass_version = Some(unsafe { (funcs.bass_get_version)() });
        self.bass_init_ok = true;
        self.state.loaded_plugins.clear();

        // Load plugins
        let plugins = vec![
            ("bassflac", "bassflac.dll"),
            ("bass_aac", "bass_aac.dll"),
            ("bassopus", "bassopus.dll"),
            ("bassape", "bassape.dll"),
            ("basswma", "basswma.dll"),
            ("bassalac", "bassalac.dll"),
        ];

        for (name, filename) in plugins {
            let dll_path = audio_core_dir.join(filename);
            if !dll_path.exists() {
                tracing::warn!("Plugin {} not found at {}", filename, dll_path.display());
                self.state.loaded_plugins.push(PluginStatus {
                    name: name.to_string(),
                    filename: filename.to_string(),
                    loaded: false,
                    error_code: None,
                    error_message: Some(format!("文件不存在: {}", dll_path.display())),
                });
                continue;
            }

            let wide_path: Vec<u16> = std::ffi::OsStr::new(&dll_path.to_string_lossy().as_ref())
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let handle = unsafe { (funcs.bass_plugin_load)(wide_path.as_ptr(), BASS_UNICODE) };
            if handle == 0 {
                let err = unsafe { (funcs.bass_error_get_code)() };
                tracing::warn!("Plugin {} load failed: error {}", filename, err);
                self.state.loaded_plugins.push(PluginStatus {
                    name: name.to_string(),
                    filename: filename.to_string(),
                    loaded: false,
                    error_code: Some(err),
                    error_message: Some(bass_error_message(err as u32)),
                });
            } else {
                tracing::info!("Plugin {} loaded successfully", filename);
                self.state.loaded_plugins.push(PluginStatus {
                    name: name.to_string(),
                    filename: filename.to_string(),
                    loaded: true,
                    error_code: None,
                    error_message: None,
                });
            }
        }

        tracing::info!("BASS initialized successfully, version: {:?}", self.bass_version);
        Ok(())
    }

    /// 释放 BASS 输出（保留 DLL 加载），用于设备切换后重建
    fn free_bass_output(&mut self) {
        if let Some(funcs) = self.funcs.as_ref() {
            if let Some(stream) = self.current_stream.take() {
                unsafe { (funcs.bass_stream_free)(stream); }
            }
            if self.bass_init_ok {
                unsafe { (funcs.bass_free)(); }
            }
        }
        self.bass_init_ok = false;
        self.current_stream = None;
        self.state.loaded_plugins.clear();
    }

    fn is_premature_stop(&self) -> bool {
        self.state.duration_ms > 0 && self.state.position_ms + 1500 < self.state.duration_ms
    }

    fn is_recoverable_error(code: i32) -> bool {
        matches!(
            code as u32,
            BASS_ERROR_DEVICE
                | BASS_ERROR_INIT
                | BASS_ERROR_START
                | BASS_ERROR_HANDLE
                | BASS_ERROR_BUFLOST
                | BASS_ERROR_DRIVER
        )
    }

    fn can_attempt_recovery(&mut self) -> bool {
        match self.last_recovery_at {
            Some(t) => t.elapsed() > Duration::from_secs(2),
            None => true,
        }
    }

    fn mark_recovery_attempt(&mut self) {
        self.last_recovery_at = Some(Instant::now());
    }

    /// 尝试 BASS_Start 重启输出（WASAPI 设备恢复）
    fn try_bass_start(&self) -> bool {
        let funcs = match self.funcs.as_ref() {
            Some(f) => f,
            None => return false,
        };
        if let Some(bass_start) = funcs.bass_start {
            unsafe { bass_start() != 0 }
        } else {
            false
        }
    }

    /// 蓝牙耳机等设备切换后恢复播放
    fn try_recover_playback(&mut self, reason: &str) -> Result<(), AppError> {
        if !self.can_attempt_recovery() {
            return Err(AppError::PlaybackError("恢复冷却中".to_string()));
        }
        self.mark_recovery_attempt();

        let song_id = self.state.current_song_id;
        let path = self.state.current_path.clone();
        let position_secs = self.state.position_ms as f64 / 1000.0;
        let was_playing = matches!(
            self.state.status,
            PlaybackStatus::Playing | PlaybackStatus::Ended
        );

        tracing::warn!("尝试恢复音频输出 ({})", reason);

        // 第一步：尝试 BASS_Start（不重载曲目）
        if self.try_bass_start() {
            if let Some(stream) = self.current_stream {
                if let Some(funcs) = self.funcs.as_ref() {
                    let play_ok = unsafe { (funcs.bass_channel_play)(stream, 0) } != 0;
                    if play_ok {
                        self.state.status = PlaybackStatus::Playing;
                        self.state.last_error = None;
                        self.emit_device_recovered();
                        tracing::info!("BASS_Start 恢复成功");
                        return Ok(());
                    }
                }
            }
        }

        // 第二步：重建 BASS 设备并重新加载当前曲目
        let (Some(song_id), Some(path)) = (song_id, path) else {
            return Err(AppError::NoActiveStream);
        };

        self.free_bass_output();
        self.init_bass()?;

        if was_playing {
            self.do_play(song_id, &path)?;
            if position_secs > 0.5 {
                let _ = self.do_seek(position_secs);
            }
        } else {
            self.do_play(song_id, &path)?;
            let _ = self.do_pause();
            if position_secs > 0.5 {
                let _ = self.do_seek(position_secs);
            }
        }

        self.emit_device_recovered();
        tracing::info!("音频设备已重建并恢复播放");
        Ok(())
    }

    fn emit_device_recovered(&self) {
        let _ = self.app_handle.emit(
            "audio://device_recovered",
            serde_json::json!({
                "song_id": self.state.current_song_id,
                "path": self.state.current_path,
            }),
        );
    }

    fn check_playback_stall(&mut self) {
        if self.state.status != PlaybackStatus::Playing {
            return;
        }
        let funcs = match self.funcs.as_ref() {
            Some(f) => f,
            None => return,
        };
        let stream = match self.current_stream {
            Some(s) => s,
            None => return,
        };
        let active = unsafe { (funcs.bass_channel_is_active)(stream) };
        if active == BASS_ACTIVE_STOPPED || active == BASS_ACTIVE_STALLED {
            let _ = self.try_recover_playback("channel_stalled");
        }
    }

    fn play_with_recovery(
        &mut self,
        song_id: i64,
        path: &str,
    ) -> Result<AudioStateDto, AppError> {
        match self.do_play(song_id, path) {
            Ok(dto) => Ok(dto),
            Err(e) => {
                let should_retry = match &e {
                    AppError::PlaybackError(msg) => {
                        msg.contains("设备") || msg.contains("通道") || msg.contains("缓冲")
                    }
                    _ => false,
                };
                if !should_retry {
                    if let Some(code) = self.last_bass_error {
                        if !Self::is_recoverable_error(code) {
                            return Err(e);
                        }
                    } else {
                        return Err(e);
                    }
                }
                self.try_recover_playback("play_failed")?;
                self.do_play(song_id, path)
            }
        }
    }

    fn handle_command(&mut self, cmd: AudioCommand) {
        match cmd {
            AudioCommand::Play { song_id, path, reply } => {
                let result = self.play_with_recovery(song_id, &path);
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            AudioCommand::Pause { reply } => {
                let result = self.do_pause();
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            AudioCommand::Resume { reply } => {
                let result = self.do_resume();
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            AudioCommand::Stop { reply } => {
                let result = self.do_stop();
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            AudioCommand::Seek { seconds, reply } => {
                let result = self.do_seek(seconds);
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            AudioCommand::SetVolume { volume, reply } => {
                let result = self.do_set_volume(volume);
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            AudioCommand::TogglePause { reply } => {
                let result = if self.state.status == PlaybackStatus::Playing {
                    self.do_pause()
                } else {
                    self.do_resume()
                };
                let _ = reply.send(result.map_err(|e| e.to_string()));
            }
            AudioCommand::GetState { reply } => {
                self.update_position();
                let _ = reply.send(AudioStateDto::from(&self.state));
            }
            AudioCommand::SetPlayMode { mode, reply } => {
                self.state.play_mode = mode;
                let _ = reply.send(AudioStateDto::from(&self.state));
            }
            AudioCommand::GetDiagnostics { reply } => {
                self.update_position();
                let _ = reply.send(AudioDiagnosticsDto {
                    bass_loaded: self.funcs.is_some(),
                    bass_version: self.bass_version,
                    bass_init_ok: self.bass_init_ok,
                    audio_core_path: self.audio_core_path.clone(),
                    plugins: self.state.loaded_plugins.clone(),
                    current_state: AudioStateDto::from(&self.state),
                    last_bass_error: self.last_bass_error,
                });
            }
            AudioCommand::Ended => {
                self.handle_track_ended();
            }
            AudioCommand::Shutdown => {}
        }
    }

    fn do_play(&mut self, song_id: i64, path: &str) -> Result<AudioStateDto, AppError> {
        let funcs = self.funcs.as_ref().ok_or_else(|| {
            AppError::AudioCoreMissing("BASS 未初始化".to_string())
        })?;

        // Check file exists
        if !std::path::Path::new(path).exists() {
            self.state.status = PlaybackStatus::Error;
            self.state.last_error = Some("文件不存在".to_string());
            return Err(AppError::FileNotFound(path.to_string()));
        }

        self.state.status = PlaybackStatus::Loading;
        self.state.current_song_id = Some(song_id);
        self.state.current_path = Some(path.to_string());
        self.state.last_error = None;

        // Free old stream
        if let Some(old_stream) = self.current_stream.take() {
            unsafe { (funcs.bass_stream_free)(old_stream); }
        }

        // Create new stream with Unicode path
        let wide_path: Vec<u16> = std::ffi::OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let stream = unsafe {
            (funcs.bass_stream_create_file)(
                0, // not memory
                wide_path.as_ptr(),
                0, // offset
                0, // length
                BASS_UNICODE,
            )
        };

        if stream == 0 {
            let err = unsafe { (funcs.bass_error_get_code)() };
            self.last_bass_error = Some(err);
            self.state.status = PlaybackStatus::Error;
            self.state.last_error = Some(bass_error_message(err as u32));
            tracing::error!("BASS_StreamCreateFile failed for '{}': {} ({})", path, bass_error_name(err as u32), err);
            return Err(AppError::PlaybackError(bass_error_message(err as u32)));
        }

        // Set volume
        let vol = if self.state.muted { 0.0 } else { self.state.volume };
        unsafe { (funcs.bass_channel_set_attribute)(stream, BASS_ATTRIB_VOL, vol); }

        // Get duration
        let length = unsafe { (funcs.bass_channel_get_length)(stream, 0) };
        let duration_secs = unsafe { (funcs.bass_channel_bytes2seconds)(stream, length) };
        self.state.duration_ms = (duration_secs * 1000.0) as u64;

        // Set end sync callback
        let end_flag = self.end_flag.clone();
        unsafe {
            (funcs.bass_channel_set_sync)(stream, BASS_SYNC_END, 0, end_sync_proc, std::ptr::null_mut());
        }

        // Play
        let result = unsafe { (funcs.bass_channel_play)(stream, 0) };
        if result == 0 {
            let err = unsafe { (funcs.bass_error_get_code)() };
            self.last_bass_error = Some(err);
            self.state.status = PlaybackStatus::Error;
            self.state.last_error = Some(bass_error_message(err as u32));
            unsafe { (funcs.bass_stream_free)(stream); }
            return Err(AppError::PlaybackError(bass_error_message(err as u32)));
        }

        self.current_stream = Some(stream);
        self.state.status = PlaybackStatus::Playing;
        self.state.position_ms = 0;

        tracing::info!("Playing: {} (duration: {}ms)", path, self.state.duration_ms);
        Ok(AudioStateDto::from(&self.state))
    }

    fn do_pause(&mut self) -> Result<AudioStateDto, AppError> {
        let funcs = self.funcs.as_ref().ok_or_else(|| {
            AppError::AudioCoreMissing("BASS 未初始化".to_string())
        })?;

        if let Some(stream) = self.current_stream {
            unsafe { (funcs.bass_channel_pause)(stream); }
            self.state.status = PlaybackStatus::Paused;
        }
        Ok(AudioStateDto::from(&self.state))
    }

    fn do_resume(&mut self) -> Result<AudioStateDto, AppError> {
        let funcs = self.funcs.as_ref().ok_or_else(|| {
            AppError::AudioCoreMissing("BASS 未初始化".to_string())
        })?;

        if let Some(stream) = self.current_stream {
            let result = unsafe { (funcs.bass_channel_play)(stream, 0) };
            if result != 0 {
                self.state.status = PlaybackStatus::Playing;
                return Ok(AudioStateDto::from(&self.state));
            }

            let err = unsafe { (funcs.bass_error_get_code)() };
            self.last_bass_error = Some(err);
            tracing::warn!("恢复播放失败 ({}), 尝试重建音频设备", err);

            self.try_recover_playback("resume_failed")?;
        } else if self.state.current_song_id.is_some() && self.state.current_path.is_some() {
            self.try_recover_playback("resume_no_stream")?;
        }
        Ok(AudioStateDto::from(&self.state))
    }

    fn do_stop(&mut self) -> Result<AudioStateDto, AppError> {
        let funcs = self.funcs.as_ref().ok_or_else(|| {
            AppError::AudioCoreMissing("BASS 未初始化".to_string())
        })?;

        if let Some(stream) = self.current_stream {
            unsafe { (funcs.bass_channel_stop)(stream); }
        }
        self.state.status = PlaybackStatus::Stopped;
        self.state.position_ms = 0;
        Ok(AudioStateDto::from(&self.state))
    }

    fn do_seek(&mut self, seconds: f64) -> Result<AudioStateDto, AppError> {
        let funcs = self.funcs.as_ref().ok_or_else(|| {
            AppError::AudioCoreMissing("BASS 未初始化".to_string())
        })?;

        let stream = self.current_stream.ok_or(AppError::NoActiveStream)?;

        // Clamp seconds
        let max_secs = self.state.duration_ms as f64 / 1000.0;
        let clamped = seconds.max(0.0).min(max_secs);

        let byte_pos = unsafe { (funcs.bass_channel_seconds2bytes)(stream, clamped) };
        let result = unsafe { (funcs.bass_channel_set_position)(stream, byte_pos, 0) };

        if result == 0 {
            let err = unsafe { (funcs.bass_error_get_code)() };
            self.last_bass_error = Some(err);
            return Err(AppError::SeekFailed(bass_error_message(err as u32)));
        }

        // Read back position
        self.update_position();
        Ok(AudioStateDto::from(&self.state))
    }

    fn do_set_volume(&mut self, volume: f32) -> Result<AudioStateDto, AppError> {
        let funcs = self.funcs.as_ref().ok_or_else(|| {
            AppError::AudioCoreMissing("BASS 未初始化".to_string())
        })?;

        self.state.volume = volume.clamp(0.0, 1.0);

        if let Some(stream) = self.current_stream {
            let vol = if self.state.muted { 0.0 } else { self.state.volume };
            unsafe { (funcs.bass_channel_set_attribute)(stream, BASS_ATTRIB_VOL, vol); }
        }
        Ok(AudioStateDto::from(&self.state))
    }

    fn handle_track_ended(&mut self) {
        self.state.status = PlaybackStatus::Ended;
        self.state.position_ms = self.state.duration_ms;

        let _ = self.app_handle.emit("audio://ended", serde_json::json!({
            "song_id": self.state.current_song_id,
            "path": self.state.current_path,
            "reason": "natural_end"
        }));

        self.push_state();
    }

    fn update_position(&mut self) {
        if let (Some(funcs), Some(stream)) = (self.funcs.as_ref(), self.current_stream) {
            let active = unsafe { (funcs.bass_channel_is_active)(stream) };
            let pos = unsafe { (funcs.bass_channel_get_position)(stream, 0) };
            let secs = unsafe { (funcs.bass_channel_bytes2seconds)(stream, pos) };
            self.state.position_ms = (secs * 1000.0) as u64;

            match active {
                BASS_ACTIVE_PLAYING => {
                    if self.state.status != PlaybackStatus::Paused {
                        self.state.status = PlaybackStatus::Playing;
                    }
                }
                BASS_ACTIVE_PAUSED => {
                    self.state.status = PlaybackStatus::Paused;
                }
                BASS_ACTIVE_STOPPED => {
                    if self.state.status == PlaybackStatus::Playing {
                        self.state.status = PlaybackStatus::Ended;
                    }
                }
                _ => {}
            }
        }
    }

    fn push_state(&self) {
        let dto = AudioStateDto::from(&self.state);
        let _ = self.app_handle.emit("audio://state", dto);
    }

    fn shutdown(&mut self) {
        tracing::info!("Audio worker shutting down");
        self.free_bass_output();
    }
}

unsafe extern "C" fn end_sync_proc(_handle: HSTREAM, _channel: DWORD, _data: DWORD, _user: *mut c_void) {
    // The actual ended handling is done via the Ended command
    // We emit via the event system in the worker's main loop
}
