use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppErrorDto {
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
    pub recoverable: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Audio core missing: {0}")]
    AudioCoreMissing(String),

    #[error("Audio core load failed: {0}")]
    AudioCoreLoadFailed(String),

    #[error("Audio plugin load failed: {0}")]
    AudioPluginLoadFailed(String),

    #[error("Audio init failed: {0}")]
    AudioInitFailed(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("File access denied: {0}")]
    FileAccessDenied(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Corrupted file: {0}")]
    CorruptedFile(String),

    #[error("Playback error: {0}")]
    PlaybackError(String),

    #[error("Seek failed: {0}")]
    SeekFailed(String),

    #[error("No active stream")]
    NoActiveStream,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Scan cancelled")]
    ScanCancelled,

    #[error("Scan failed: {0}")]
    ScanFailed(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl AppError {
    pub fn code(&self) -> &str {
        match self {
            AppError::AudioCoreMissing(_) => "AudioCoreMissing",
            AppError::AudioCoreLoadFailed(_) => "AudioCoreLoadFailed",
            AppError::AudioPluginLoadFailed(_) => "AudioPluginLoadFailed",
            AppError::AudioInitFailed(_) => "AudioInitFailed",
            AppError::FileNotFound(_) => "FileNotFound",
            AppError::FileAccessDenied(_) => "FileAccessDenied",
            AppError::UnsupportedFormat(_) => "UnsupportedFormat",
            AppError::CorruptedFile(_) => "CorruptedFile",
            AppError::PlaybackError(_) => "PlaybackError",
            AppError::SeekFailed(_) => "SeekFailed",
            AppError::NoActiveStream => "NoActiveStream",
            AppError::DatabaseError(_) => "DatabaseError",
            AppError::ScanCancelled => "ScanCancelled",
            AppError::ScanFailed(_) => "ScanFailed",
            AppError::InvalidArgument(_) => "InvalidArgument",
            AppError::InternalError(_) => "InternalError",
        }
    }

    pub fn to_dto(&self) -> AppErrorDto {
        AppErrorDto {
            code: self.code().to_string(),
            message: self.to_string(),
            detail: None,
            recoverable: matches!(
                self,
                AppError::PlaybackError(_)
                    | AppError::FileNotFound(_)
                    | AppError::SeekFailed(_)
                    | AppError::ScanCancelled
            ),
        }
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::DatabaseError(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::InternalError(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
