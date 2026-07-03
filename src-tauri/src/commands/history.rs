use xrest_core::traits::{HistoryRepository, PathProvider};
use xrest_core::types::HistoryEntry;
use xrest_infra::history::SqliteHistoryRepository;
use xrest_infra::paths::TauriPathProvider;
use rusqlite::Connection;
use tauri::AppHandle;

#[tauri::command]
pub fn get_history(
    app: AppHandle,
    limit: usize,
    offset: usize,
) -> Result<Vec<HistoryEntry>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let db_path = paths.history_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let repo = SqliteHistoryRepository::new(conn);
    repo.get_history(limit, offset)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    let paths = TauriPathProvider::new(&app)?;
    let db_path = paths.history_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let repo = SqliteHistoryRepository::new(conn);
    repo.clear()
}
