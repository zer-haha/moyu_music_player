use std::path::Path;
use tauri::Window;
use tauri_plugin_dialog::DialogExt;

/// 打开文件选择对话框，选择音乐文件（支持多选）
#[tauri::command]
pub async fn pick_music_files(window: Window) -> Result<Vec<String>, String> {
    let files = window
        .dialog()
        .file()
        .add_filter("音乐文件", &["mp3", "flac", "wav", "ogg", "aac", "m4a", "wma", "ape", "opus", "alac", "aiff", "wv", "tta", "mka"])
        .blocking_pick_files();

    match files {
        Some(paths) => {
            let result: Vec<String> = paths
                .into_iter()
                .filter_map(|fp| {
                    fp.as_path()
                        .and_then(|path| path.to_str().map(|s| s.to_string()))
                })
                .collect();
            Ok(result)
        }
        None => Ok(Vec::new()),
    }
}

/// 打开文件夹选择对话框
#[tauri::command]
pub async fn pick_folder(window: Window) -> Result<Option<String>, String> {
    let folder = window
        .dialog()
        .file()
        .blocking_pick_folder();

    match folder {
        Some(fp) => {
            let path = fp
                .as_path()
                .and_then(|p| p.to_str().map(|s| s.to_string()));
            Ok(path)
        }
        None => Ok(None),
    }
}

/// 检查文件是否存在
#[tauri::command]
pub async fn check_file_exists(file_path: String) -> Result<bool, String> {
    Ok(Path::new(&file_path).exists())
}

/// 批量检查文件是否存在
#[tauri::command]
pub async fn check_files_exist(file_paths: Vec<String>) -> Result<Vec<(String, bool)>, String> {
    let results: Vec<(String, bool)> = file_paths
        .into_iter()
        .map(|p| {
            let exists = Path::new(&p).exists();
            (p, exists)
        })
        .collect();
    Ok(results)
}

/// 在文件管理器中显示文件
#[tauri::command]
pub async fn show_in_folder(file_path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &file_path])
            .spawn()
            .map_err(|e| format!("\u{65e0}\u{6cd5}\u{6253}\u{5f00}\u{6587}\u{4ef6}\u{7ba1}\u{7406}\u{5668}: {}", e))?;
    }
    Ok(())
}

/// 真正退出应用程序
#[tauri::command]
pub async fn app_quit(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}
