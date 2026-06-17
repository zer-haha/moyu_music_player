pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp3", "mp2", "mp1", "wav", "aiff", "aif", "ogg", "flac",
    "m4a", "aac", "mp4", "alac", "opus", "ape", "wma", "wv", "tta", "mka",
];

pub fn is_supported_audio_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn get_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}
