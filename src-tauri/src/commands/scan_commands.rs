use crate::app_state::AppState;
use crate::errors::AppErrorDto;
use crate::library::scanner::{self, ScanTask, ScanTaskDto, ScanStatusDto};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tauri::State;

type CmdResult<T> = Result<T, AppErrorDto>;

#[tauri::command]
pub async fn scan_start_folder(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    path: String,
    playlist_id: i64,
) -> CmdResult<ScanTaskDto> {
    let db = state.db.clone();
    let root_path = PathBuf::from(&path);

    if !root_path.exists() {
        return Err(AppErrorDto {
            code: "FileNotFound".to_string(),
            message: format!("文件夹不存在: {}", path),
            detail: None,
            recoverable: true,
        });
    }

    let task = ScanTask::new(root_path, playlist_id);
    let cancelled = task.cancelled.clone();
    let task_id = task.task_id.clone();

    // Store cancellation token
    {
        let mut tokens = state.scan_tokens.lock().unwrap();
        tokens.insert(task_id.clone(), cancelled);
    }

    // Run scan in background thread
    let result = std::thread::Builder::new()
        .name("scan-worker".to_string())
        .spawn(move || {
            scanner::scan_folder(db, app_handle, task)
        })
        .map_err(|e| AppErrorDto {
            code: "ScanFailed".to_string(),
            message: format!("启动扫描失败: {}", e),
            detail: None,
            recoverable: true,
        })?;

    // Wait for result (this blocks the command but scan runs in separate thread)
    // For a non-blocking approach we'd use events, but this is simpler for now
    let task_result = result.join().map_err(|_| AppErrorDto {
        code: "ScanFailed".to_string(),
        message: "扫描线程异常退出".to_string(),
        detail: None,
        recoverable: true,
    })?;

    Ok(task_result)
}

#[tauri::command]
pub async fn scan_cancel(
    state: State<'_, AppState>,
    task_id: String,
) -> CmdResult<()> {
    let tokens = state.scan_tokens.lock().unwrap();
    if let Some(cancelled) = tokens.get(&task_id) {
        cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    Ok(())
}
