mod app_state;
mod audio;
mod commands;
mod db;
mod errors;
mod library;
mod settings;

use app_state::AppState;
use audio::AudioEngine;
use db::connection::Database;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Emitter, Manager};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

fn setup_logging() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    // Try to create logs directory
    let log_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("logs")))
        .unwrap_or_else(|| std::path::PathBuf::from("logs"));
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    // Keep the guard alive by leaking it (it's fine for a desktop app)
    std::mem::forget(_guard);

    tracing::info!("=== 墨鱼听歌 启动 ===");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    setup_logging();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize database
            let db = Database::new(&handle).unwrap_or_else(|e| {
                tracing::error!("Database initialization failed: {}", e);
                eprintln!("数据库初始化失败: {}", e);
                std::process::exit(1);
            });

            // Initialize audio engine
            let audio = AudioEngine::new(handle);

            let state = AppState {
                audio,
                db,
                scan_tokens: Mutex::new(HashMap::new()),
            };

            app.manage(state);

            // Setup tray icon
            let show_item = MenuItemBuilder::new("显示主窗口").id("show").build(app)?;
            let quit_item = MenuItemBuilder::new("退出").id("quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .item(&show_item).separator().item(&quit_item).build()?;
            let app2 = app.handle().clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("墨鱼听歌")
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                })
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.unminimize();
                            let _ = w.set_focus();
                        }
                    }
                    "quit" => { app.exit(0); }
                    _ => {}
                })
                .build(&app2)?;

            tracing::info!("App setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Audio commands
            commands::audio_commands::audio_play,
            commands::audio_commands::audio_pause,
            commands::audio_commands::audio_resume,
            commands::audio_commands::audio_toggle_pause,
            commands::audio_commands::audio_stop,
            commands::audio_commands::audio_seek,
            commands::audio_commands::audio_set_volume,
            commands::audio_commands::audio_get_state,
            commands::audio_commands::audio_get_diagnostics,
            commands::audio_commands::audio_set_play_mode,
            // Library commands
            commands::library_commands::library_add_files,
            commands::library_commands::library_remove_song_from_playlist,
            commands::library_commands::library_delete_song,
            commands::library_commands::library_mark_song_unplayable,
            // Playlist commands
            commands::playlist_commands::playlist_create,
            commands::playlist_commands::playlist_rename,
            commands::playlist_commands::playlist_delete,
            commands::playlist_commands::playlist_clear,
            commands::playlist_commands::playlist_get_all,
            commands::playlist_commands::playlist_get_songs,
            commands::playlist_commands::playlist_song_reorder,
            // Scan commands
            commands::scan_commands::scan_start_folder,
            commands::scan_commands::scan_cancel,
            // Settings commands
            commands::settings_commands::settings_get,
            commands::settings_commands::settings_update,
            commands::settings_commands::settings_get_playback_state,
            commands::settings_commands::settings_save_playback_state,
            commands::settings_commands::settings_get_db_path,
            // Utility commands
            commands::utility_commands::pick_music_files,
            commands::utility_commands::pick_folder,
            commands::utility_commands::check_file_exists,
            commands::utility_commands::check_files_exist,
            commands::utility_commands::show_in_folder,
            commands::utility_commands::app_quit,
        ])
        .run(tauri::generate_context!())
        .expect("启动音乐播放器失败");
}
