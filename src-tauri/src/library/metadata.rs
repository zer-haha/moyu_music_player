use crate::db::song_repo::SongDto;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::Path;

pub fn read_metadata(path: &str) -> SongDto {
    let file_path = Path::new(path);
    let file_name = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let extension = file_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    let (size_bytes, modified_time) = match std::fs::metadata(path) {
        Ok(meta) => {
            let size = meta.len() as i64;
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);
            (size, modified)
        }
        Err(_) => (0, 0),
    };

    let now = chrono::Utc::now().timestamp();
    let path_key = crate::db::song_repo::normalize_path_key(path);

    let mut song = SongDto {
        id: 0,
        path: path.to_string(),
        path_key,
        file_name,
        title: String::new(),
        artist: None,
        album: None,
        album_artist: None,
        genre: None,
        track_number: None,
        disc_number: None,
        duration_ms: 0,
        format: extension.clone(),
        extension,
        codec_hint: None,
        size_bytes,
        modified_time,
        added_at: now,
        updated_at: now,
        last_played_at: None,
        play_count: 0,
        playable: true,
        missing: false,
        last_error: None,
    };

    // Default title to filename
    song.title = song.file_name.clone();

    match Probe::open(path) {
        Ok(probe) => match probe.read() {
            Ok(tagged_file) => {
                // Read properties
                let properties = tagged_file.properties();
                let duration = properties.duration();
                song.duration_ms = duration.as_millis() as i64;

                // Read tags
                if let Some(tag) = tagged_file.primary_tag().or_else(|| tagged_file.first_tag()) {
                    if let Some(title) = tag.title() {
                        let t = title.to_string().trim().to_string();
                        if !t.is_empty() {
                            song.title = t;
                        }
                    }
                    if let Some(artist) = tag.artist() {
                        let a = artist.to_string().trim().to_string();
                        if !a.is_empty() {
                            song.artist = Some(a);
                        }
                    }
                    if let Some(album) = tag.album() {
                        let a = album.to_string().trim().to_string();
                        if !a.is_empty() {
                            song.album = Some(a);
                        }
                    }
                    if let Some(genre) = tag.genre() {
                        let g = genre.to_string().trim().to_string();
                        if !g.is_empty() {
                            song.genre = Some(g);
                        }
                    }

                    // Track number
                    song.track_number = tag.get_string(&lofty::tag::ItemKey::TrackNumber)
                        .and_then(|s| s.parse::<i32>().ok());

                    // Disc number
                    song.disc_number = tag.get_string(&lofty::tag::ItemKey::DiscNumber)
                        .and_then(|s| s.parse::<i32>().ok());

                    // Album artist
                    song.album_artist = tag.get_string(&lofty::tag::ItemKey::AlbumArtist)
                        .map(|s| s.to_string().trim().to_string())
                        .filter(|s| !s.is_empty());
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read tags for '{}': {}", path, e);
                song.last_error = Some(format!("标签读取失败: {}", e));
            }
        },
        Err(e) => {
            tracing::warn!("Failed to open file for metadata '{}': {}", path, e);
            song.last_error = Some(format!("文件打开失败: {}", e));
        }
    }

    song
}
