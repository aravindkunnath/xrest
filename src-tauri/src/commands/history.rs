use crate::core::history::HistoryService;
use crate::core::traits::PathProvider;
use crate::core::types::HistoryEntry;
use crate::infra::paths::TauriPathProvider;
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
    let service = HistoryService::new(conn);
    service.get_history(limit, offset)
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    let paths = TauriPathProvider::new(&app)?;
    let db_path = paths.history_db_path()?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let service = HistoryService::new(conn);
    service.clear()
}
