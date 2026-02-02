use crate::types::{Header, HistoryEntry};
use rusqlite::{params, Connection, Result};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

pub struct HistoryService {
    pub conn: Connection,
}

impl HistoryService {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn init(&self) -> Result<(), String> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS history (
                id TEXT PRIMARY KEY,
                service_id TEXT,
                endpoint_id TEXT,
                method TEXT NOT NULL,
                url TEXT NOT NULL,
                request_headers TEXT NOT NULL,
                request_body TEXT NOT NULL,
                response_status INTEGER NOT NULL,
                response_status_text TEXT NOT NULL,
                response_headers TEXT NOT NULL,
                response_body TEXT NOT NULL,
                time_elapsed INTEGER NOT NULL,
                size INTEGER NOT NULL,
                created_at TEXT NOT NULL
            )",
                [],
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn save(&self, entry: HistoryEntry) -> Result<(), String> {
        let request_headers =
            serde_json::to_string(&entry.request_headers).map_err(|e| e.to_string())?;
        let response_headers =
            serde_json::to_string(&entry.response_headers).map_err(|e| e.to_string())?;

        self.conn
            .execute(
                "INSERT INTO history (
                id, service_id, endpoint_id, method, url, 
                request_headers, request_body, 
                response_status, response_status_text, 
                response_headers, response_body, 
                time_elapsed, size, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
                params![
                    entry.id,
                    entry.service_id,
                    entry.endpoint_id,
                    entry.method,
                    entry.url,
                    request_headers,
                    entry.request_body,
                    entry.response_status,
                    entry.response_status_text,
                    response_headers,
                    entry.response_body,
                    entry.time_elapsed,
                    entry.size,
                    entry.created_at,
                ],
            )
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_history(&self, limit: usize, offset: usize) -> Result<Vec<HistoryEntry>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT 
                    id, service_id, endpoint_id, method, url, 
                    request_headers, request_body, 
                    response_status, response_status_text, 
                    response_headers, response_body, 
                    time_elapsed, size, created_at 
                FROM history 
                ORDER BY created_at DESC 
                LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| e.to_string())?;

        let history_iter = stmt
            .query_map(params![limit, offset], |row| {
                let request_headers_raw: String = row.get(5)?;
                let response_headers_raw: String = row.get(9)?;

                let request_headers: Vec<Header> =
                    serde_json::from_str(&request_headers_raw).unwrap_or_default();
                let response_headers: Vec<Header> =
                    serde_json::from_str(&response_headers_raw).unwrap_or_default();

                Ok(HistoryEntry {
                    id: row.get(0)?,
                    service_id: row.get(1)?,
                    endpoint_id: row.get(2)?,
                    method: row.get(3)?,
                    url: row.get(4)?,
                    request_headers,
                    request_body: row.get(6)?,
                    response_status: row.get(7)?,
                    response_status_text: row.get(8)?,
                    response_headers,
                    response_body: row.get(10)?,
                    time_elapsed: row.get(11)?,
                    size: row.get(12)?,
                    created_at: row.get(13)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut history = Vec::new();
        for entry in history_iter {
            history.push(entry.map_err(|e| e.to_string())?);
        }

        Ok(history)
    }

    pub fn clear(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM history", [])
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}

// Wrapper functions for Tauri commands
pub fn get_db_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let path = app.path().app_config_dir().map_err(|e| e.to_string())?;
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    Ok(path.join("history.db"))
}

pub fn init_db<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let service = HistoryService::new(conn);
    service.init()
}

pub fn save_history<R: Runtime>(app: &AppHandle<R>, entry: HistoryEntry) -> Result<(), String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let service = HistoryService::new(conn);
    service.save(entry)
}

pub fn get_history<R: Runtime>(
    app: &AppHandle<R>,
    limit: usize,
    offset: usize,
) -> Result<Vec<HistoryEntry>, String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let service = HistoryService::new(conn);
    service.get_history(limit, offset)
}

pub fn clear_history<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let db_path = get_db_path(app)?;
    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;
    let service = HistoryService::new(conn);
    service.clear()
}
