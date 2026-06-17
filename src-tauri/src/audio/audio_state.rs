use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackStatus {
    Idle,
    Loading,
    Playing,
    Paused,
    Stopped,
    Ended,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlayMode {
    Sequence,
    ListLoop,
    SingleLoop,
    Random,
}

impl Default for PlayMode {
    fn default() -> Self {
        PlayMode::ListLoop
    }
}

impl PlayMode {
    pub fn next(&self) -> Self {
        match self {
            PlayMode::Sequence => PlayMode::ListLoop,
            PlayMode::ListLoop => PlayMode::SingleLoop,
            PlayMode::SingleLoop => PlayMode::Random,
            PlayMode::Random => PlayMode::Sequence,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            PlayMode::Sequence => "顺序播放",
            PlayMode::ListLoop => "列表循环",
            PlayMode::SingleLoop => "单曲循环",
            PlayMode::Random => "随机播放",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginStatus {
    pub name: String,
    pub filename: String,
    pub loaded: bool,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioState {
    pub current_song_id: Option<i64>,
    pub current_path: Option<String>,
    pub status: PlaybackStatus,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub volume: f32,
    pub muted: bool,
    pub play_mode: PlayMode,
    pub last_error: Option<String>,
    pub loaded_plugins: Vec<PluginStatus>,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            current_song_id: None,
            current_path: None,
            status: PlaybackStatus::Idle,
            position_ms: 0,
            duration_ms: 0,
            volume: 0.8,
            muted: false,
            play_mode: PlayMode::ListLoop,
            last_error: None,
            loaded_plugins: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStateDto {
    pub current_song_id: Option<i64>,
    pub path: Option<String>,
    pub status: PlaybackStatus,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub volume: f32,
    pub muted: bool,
    pub play_mode: PlayMode,
    pub error: Option<String>,
}

impl From<&AudioState> for AudioStateDto {
    fn from(s: &AudioState) -> Self {
        Self {
            current_song_id: s.current_song_id,
            path: s.current_path.clone(),
            status: s.status.clone(),
            position_ms: s.position_ms,
            duration_ms: s.duration_ms,
            volume: if s.muted { 0.0 } else { s.volume },
            muted: s.muted,
            play_mode: s.play_mode.clone(),
            error: s.last_error.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDiagnosticsDto {
    pub bass_loaded: bool,
    pub bass_version: Option<u32>,
    pub bass_init_ok: bool,
    pub audio_core_path: Option<String>,
    pub plugins: Vec<PluginStatus>,
    pub current_state: AudioStateDto,
    pub last_bass_error: Option<i32>,
}
