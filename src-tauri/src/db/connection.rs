use crate::errors::{AppError, AppResult};
use parking_lot::Mutex;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Manager;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl Database {
    pub fn new(app_handle: &tauri::AppHandle) -> AppResult<Self> {
        let db_path = resolve_db_path(app_handle)?;
        tracing::info!("Database path: {}", db_path.display());

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::DatabaseError(format!("无法创建数据库目录: {}", e))
            })?;
        }

        let conn = Connection::open(&db_path).map_err(|e| {
            AppError::DatabaseError(format!("打开数据库失败: {}", e))
        })?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;").map_err(|e| {
            AppError::DatabaseError(format!("设置 PRAGMA 失败: {}", e))
        })?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };

        db.run_migrations()?;
        Ok(db)
    }

    pub fn conn(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    fn run_migrations(&self) -> AppResult<()> {
        crate::db::migrations::run_migrations(&self.conn())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: self.conn.clone(),
            db_path: self.db_path.clone(),
        }
    }
}

fn resolve_db_path(app_handle: &tauri::AppHandle) -> AppResult<PathBuf> {
    // Try EXE sibling data/ directory
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let data_dir = exe_dir.join("data");
            if data_dir.exists() || std::fs::create_dir_all(&data_dir).is_ok() {
                // Check if writable
                let test_file = data_dir.join(".write_test");
                if std::fs::write(&test_file, "test").is_ok() {
                    let _ = std::fs::remove_file(&test_file);
                    return Ok(data_dir.join("music.db"));
                }
            }
        }
    }

    // Fallback to app data directory
    let app_data = app_handle.path().app_data_dir().map_err(|e| {
        AppError::DatabaseError(format!("无法获取应用数据目录: {}", e))
    })?;
    std::fs::create_dir_all(&app_data).map_err(|e| {
        AppError::DatabaseError(format!("无法创建数据目录: {}", e))
    })?;
    Ok(app_data.join("music.db"))
}
