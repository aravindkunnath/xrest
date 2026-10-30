//! Execution history: every send is recorded in a SQLite database owned by the app (never inside
//! a workspace).
//!
//! [`SqliteHistory`] is one `Mutex<Connection>` in WAL mode, shared by every workspace through an
//! `Arc<dyn HistoryStore>`. Secrets never reach the database: [`record_send`] masks sensitive
//! headers and every resolved secret value before an entry is built, and [`SqliteHistory`] caps
//! bodies at [`MAX_BODY_BYTES`].

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use base64::Engine;
use rusqlite::{params_from_iter, types::Value, Connection, OptionalExtension, Row};

use crate::error::{Error, Result};
use crate::exec::RecordedRequest;
use crate::interpolate::SECRET_MASK;
use crate::model::HttpResponse;

/// What masked values are replaced with.
pub const MASK: &str = SECRET_MASK;
/// Bodies longer than this are cut when stored (`truncated = true`).
pub const MAX_BODY_BYTES: usize = 1024 * 1024;
/// Secrets shorter than this are not masked: they would blank out ordinary text.
pub const MIN_MASKED_SECRET_LEN: usize = 4;

/// Header names whose values are never stored.
const SENSITIVE_HEADERS: [&str; 4] = [
    "authorization",
    "proxy-authorization",
    "cookie",
    "set-cookie",
];

/// One recorded send. `id` and `created_at` are assigned by the store when `id == 0` /
/// `created_at == 0`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoryEntry {
    pub id: i64,
    /// Unix milliseconds.
    pub created_at: i64,
    pub service_id: String,
    pub service_name: String,
    pub endpoint_id: Option<String>,
    pub environment: Option<String>,
    pub method: String,
    /// Final URL (after interpolation and auth query params), secrets masked.
    pub url: String,
    pub request_headers: Vec<(String, String)>,
    pub request_body: String,
    /// `None` when the request never produced a response (transport error).
    pub status: Option<u16>,
    pub status_text: String,
    pub response_headers: Vec<(String, String)>,
    /// UTF-8 text, or base64 when `body_is_base64`.
    pub response_body: String,
    pub body_is_base64: bool,
    pub truncated: bool,
    pub time_ms: u64,
    pub size_bytes: u64,
    pub error: Option<String>,
}

/// A listing row: an entry without bodies or headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistorySummary {
    pub id: i64,
    pub created_at: i64,
    pub service_id: String,
    pub service_name: String,
    pub endpoint_id: Option<String>,
    pub environment: Option<String>,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub truncated: bool,
    pub time_ms: u64,
    pub size_bytes: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoryFilter {
    pub service_id: Option<String>,
    pub endpoint_id: Option<String>,
    pub method: Option<String>,
    /// 2, 3, 4 or 5 for that status class; 0 for entries without a response (errors).
    pub status_class: Option<u8>,
}

pub trait HistoryStore: Send + Sync {
    /// Store `entry` (bodies capped) and return its id.
    fn record(&self, entry: &HistoryEntry) -> Result<i64>;
    /// Newest first.
    fn list(&self, filter: &HistoryFilter, limit: u32, offset: u32) -> Result<Vec<HistorySummary>>;
    fn get(&self, id: i64) -> Result<Option<HistoryEntry>>;
    fn count(&self, filter: &HistoryFilter) -> Result<u64>;
    fn clear(&self) -> Result<()>;
}

fn db_err(e: rusqlite::Error) -> Error {
    Error::InvalidInput(format!("history database: {e}"))
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at INTEGER NOT NULL,
    service_id TEXT NOT NULL,
    service_name TEXT NOT NULL,
    endpoint_id TEXT,
    environment TEXT,
    method TEXT NOT NULL,
    url TEXT NOT NULL,
    request_headers TEXT NOT NULL,
    request_body TEXT NOT NULL,
    status INTEGER,
    status_text TEXT NOT NULL,
    response_headers TEXT NOT NULL,
    response_body TEXT NOT NULL,
    body_is_base64 INTEGER NOT NULL,
    truncated INTEGER NOT NULL,
    time_ms INTEGER NOT NULL,
    size_bytes INTEGER NOT NULL,
    error TEXT
);
CREATE INDEX IF NOT EXISTS history_service ON history (service_id, id);
CREATE INDEX IF NOT EXISTS history_endpoint ON history (endpoint_id, id);
";

/// Schema version stored in `PRAGMA user_version`. Bump it and extend [`migrate`] on a change.
pub const SCHEMA_VERSION: i64 = 1;

pub struct SqliteHistory {
    conn: Mutex<Connection>,
}

impl SqliteHistory {
    /// Open (creating the file, its parent directory and the schema) in WAL mode.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
        }
        let conn = Connection::open(path).map_err(db_err)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(db_err)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(db_err)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(db_err)?;
        // Deleted rows are overwritten with zeros, so `clear` really erases bodies.
        conn.pragma_update(None, "secure_delete", "ON")
            .map_err(db_err)?;
        migrate(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn conn(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Create the schema in a fresh database, or check an existing one is not from the future.
fn migrate(conn: &Connection) -> Result<()> {
    let found: i64 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(db_err)?;
    if found > SCHEMA_VERSION {
        return Err(Error::InvalidInput(format!(
            "the history database uses schema version {found}, but this version of XRest supports up to {SCHEMA_VERSION}; update XRest"
        )));
    }
    // Version 0 is a new (or pre-versioning) database; the schema statements are idempotent.
    conn.execute_batch(SCHEMA).map_err(db_err)?;
    if found < SCHEMA_VERSION {
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(db_err)?;
    }
    Ok(())
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Cut `s` to at most [`MAX_BODY_BYTES`] on a char boundary. Returns whether it was cut.
fn cap_text(s: &str) -> (&str, bool) {
    if s.len() <= MAX_BODY_BYTES {
        return (s, false);
    }
    let mut end = MAX_BODY_BYTES;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    (&s[..end], true)
}

fn cap_response_body(e: &HistoryEntry) -> (String, bool) {
    if e.body_is_base64 {
        let engine = base64::engine::general_purpose::STANDARD;
        // Cheap check first: base64 is 4 chars per 3 bytes.
        if e.response_body.len() <= MAX_BODY_BYTES / 3 * 4 {
            return (e.response_body.clone(), false);
        }
        if let Ok(bytes) = engine.decode(&e.response_body) {
            if bytes.len() > MAX_BODY_BYTES {
                return (engine.encode(&bytes[..MAX_BODY_BYTES]), true);
            }
        }
        return (e.response_body.clone(), false);
    }
    let (t, cut) = cap_text(&e.response_body);
    (t.to_string(), cut)
}

fn where_clause(f: &HistoryFilter) -> (String, Vec<Value>) {
    let mut conds = vec![];
    let mut args = vec![];
    if let Some(v) = &f.service_id {
        conds.push("service_id = ?");
        args.push(Value::Text(v.clone()));
    }
    if let Some(v) = &f.endpoint_id {
        conds.push("endpoint_id = ?");
        args.push(Value::Text(v.clone()));
    }
    if let Some(v) = &f.method {
        conds.push("method = ? COLLATE NOCASE");
        args.push(Value::Text(v.clone()));
    }
    match f.status_class {
        Some(0) => conds.push("status IS NULL"),
        Some(c @ 1..=9) => {
            conds.push("status >= ? AND status < ?");
            args.push(Value::Integer(i64::from(c) * 100));
            args.push(Value::Integer(i64::from(c) * 100 + 100));
        }
        _ => {}
    }
    if conds.is_empty() {
        (String::new(), args)
    } else {
        (format!(" WHERE {}", conds.join(" AND ")), args)
    }
}

fn headers_json(h: &[(String, String)]) -> String {
    serde_json::to_string(h).unwrap_or_else(|_| "[]".into())
}

fn headers_from(s: String) -> Vec<(String, String)> {
    serde_json::from_str(&s).unwrap_or_default()
}

fn summary_from(r: &Row) -> rusqlite::Result<HistorySummary> {
    Ok(HistorySummary {
        id: r.get(0)?,
        created_at: r.get(1)?,
        service_id: r.get(2)?,
        service_name: r.get(3)?,
        endpoint_id: r.get(4)?,
        environment: r.get(5)?,
        method: r.get(6)?,
        url: r.get(7)?,
        status: r.get::<_, Option<i64>>(8)?.map(|s| s as u16),
        truncated: r.get(9)?,
        time_ms: r.get::<_, i64>(10)? as u64,
        size_bytes: r.get::<_, i64>(11)? as u64,
        error: r.get(12)?,
    })
}

const SUMMARY_COLS: &str = "id, created_at, service_id, service_name, endpoint_id, environment, \
     method, url, status, truncated, time_ms, size_bytes, error";

impl HistoryStore for SqliteHistory {
    fn record(&self, e: &HistoryEntry) -> Result<i64> {
        let (resp_body, resp_cut) = cap_response_body(e);
        let (req_body, req_cut) = cap_text(&e.request_body);
        let created_at = if e.created_at == 0 {
            now_ms()
        } else {
            e.created_at
        };
        let conn = self.conn();
        conn.execute(
            "INSERT INTO history (created_at, service_id, service_name, endpoint_id, environment, \
             method, url, request_headers, request_body, status, status_text, response_headers, \
             response_body, body_is_base64, truncated, time_ms, size_bytes, error) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
            rusqlite::params![
                created_at,
                e.service_id,
                e.service_name,
                e.endpoint_id,
                e.environment,
                e.method,
                e.url,
                headers_json(&e.request_headers),
                req_body,
                e.status.map(i64::from),
                e.status_text,
                headers_json(&e.response_headers),
                resp_body,
                e.body_is_base64,
                e.truncated || resp_cut || req_cut,
                e.time_ms as i64,
                e.size_bytes as i64,
                e.error,
            ],
        )
        .map_err(db_err)?;
        Ok(conn.last_insert_rowid())
    }

    fn list(&self, f: &HistoryFilter, limit: u32, offset: u32) -> Result<Vec<HistorySummary>> {
        let (wh, mut args) = where_clause(f);
        args.push(Value::Integer(i64::from(limit)));
        args.push(Value::Integer(i64::from(offset)));
        let sql =
            format!("SELECT {SUMMARY_COLS} FROM history{wh} ORDER BY id DESC LIMIT ? OFFSET ?");
        let conn = self.conn();
        let mut stmt = conn.prepare_cached(&sql).map_err(db_err)?;
        let rows = stmt
            .query_map(params_from_iter(args), summary_from)
            .map_err(db_err)?;
        rows.collect::<rusqlite::Result<_>>().map_err(db_err)
    }

    fn get(&self, id: i64) -> Result<Option<HistoryEntry>> {
        let conn = self.conn();
        conn.query_row(
            "SELECT id, created_at, service_id, service_name, endpoint_id, environment, method, \
             url, request_headers, request_body, status, status_text, response_headers, \
             response_body, body_is_base64, truncated, time_ms, size_bytes, error \
             FROM history WHERE id = ?",
            [id],
            |r| {
                Ok(HistoryEntry {
                    id: r.get(0)?,
                    created_at: r.get(1)?,
                    service_id: r.get(2)?,
                    service_name: r.get(3)?,
                    endpoint_id: r.get(4)?,
                    environment: r.get(5)?,
                    method: r.get(6)?,
                    url: r.get(7)?,
                    request_headers: headers_from(r.get(8)?),
                    request_body: r.get(9)?,
                    status: r.get::<_, Option<i64>>(10)?.map(|s| s as u16),
                    status_text: r.get(11)?,
                    response_headers: headers_from(r.get(12)?),
                    response_body: r.get(13)?,
                    body_is_base64: r.get(14)?,
                    truncated: r.get(15)?,
                    time_ms: r.get::<_, i64>(16)? as u64,
                    size_bytes: r.get::<_, i64>(17)? as u64,
                    error: r.get(18)?,
                })
            },
        )
        .optional()
        .map_err(db_err)
    }

    fn count(&self, f: &HistoryFilter) -> Result<u64> {
        let (wh, args) = where_clause(f);
        let conn = self.conn();
        let n: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM history{wh}"),
                params_from_iter(args),
                |r| r.get(0),
            )
            .map_err(db_err)?;
        Ok(n as u64)
    }

    fn clear(&self) -> Result<()> {
        let conn = self.conn();
        conn.execute_batch("DELETE FROM history").map_err(db_err)?;
        // Fold the WAL into the main file and rewrite it, so no deleted body lingers on disk.
        conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
            .map_err(db_err)?;
        conn.execute_batch("VACUUM").map_err(db_err)
    }
}

// ---- recording a send ------------------------------------------------------------------------

/// Where and as whom a send is recorded.
#[derive(Clone)]
pub struct HistorySink {
    pub store: Arc<dyn HistoryStore>,
    pub service_id: String,
    pub service_name: String,
    pub endpoint_id: Option<String>,
}

/// Forms a secret can take inside a URL or form body, besides its raw value.
fn secret_variants(secret: &str) -> Vec<String> {
    let mut v = vec![secret.to_string()];
    let form: String = url::form_urlencoded::byte_serialize(secret.as_bytes()).collect();
    let pct = percent_encode(secret);
    let mut extra = vec![form, pct];
    extra.extend(url_forms(secret));
    for s in extra {
        if !s.is_empty() && !v.contains(&s) {
            v.push(s);
        }
    }
    v
}

/// How the `url` crate serializes `secret` in each URL position (path, path segment, query,
/// fragment, userinfo), which differs from form and strict percent encoding.
fn url_forms(secret: &str) -> Vec<String> {
    let Ok(base) = url::Url::parse("http://h/") else {
        return vec![];
    };
    let mut out = vec![];
    let mut u = base.clone();
    u.set_path(secret);
    out.push(u.path().trim_start_matches('/').to_string());
    let mut u = base.clone();
    if let Ok(mut seg) = u.path_segments_mut() {
        seg.push(secret);
    }
    out.push(u.path().trim_start_matches('/').to_string());
    let mut u = base.clone();
    u.set_query(Some(secret));
    out.extend(u.query().map(str::to_string));
    let mut u = base.clone();
    u.query_pairs_mut().append_pair("k", secret);
    out.extend(
        u.query()
            .and_then(|q| q.strip_prefix("k="))
            .map(str::to_string),
    );
    let mut u = base.clone();
    u.set_fragment(Some(secret));
    out.extend(u.fragment().map(str::to_string));
    let mut u = base.clone();
    let _ = u.set_username(secret);
    out.push(u.username().to_string());
    let mut u = base;
    let _ = u.set_password(Some(secret));
    out.extend(u.password().map(str::to_string));
    out
}

fn percent_encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Replaces secret values, longest first.
pub(crate) struct Masker(Vec<String>);

impl Masker {
    pub(crate) fn new<'a>(secrets: impl IntoIterator<Item = &'a String>) -> Self {
        let mut all: Vec<String> = secrets
            .into_iter()
            .filter(|s| s.len() >= MIN_MASKED_SECRET_LEN)
            .flat_map(|s| secret_variants(s))
            .collect();
        all.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        all.dedup();
        Self(all)
    }

    pub(crate) fn mask(&self, text: &str) -> String {
        let mut out = text.to_string();
        for s in &self.0 {
            if out.contains(s.as_str()) {
                out = out.replace(s.as_str(), MASK);
            }
        }
        out
    }

    fn headers(&self, headers: &[(String, String)]) -> Vec<(String, String)> {
        headers
            .iter()
            .map(|(k, v)| {
                let v = if SENSITIVE_HEADERS.iter().any(|h| k.eq_ignore_ascii_case(h)) {
                    MASK.to_string()
                } else {
                    self.mask(v)
                };
                (k.clone(), v)
            })
            .collect()
    }
}

/// Build the masked entry for one send and store it. Returns a warning on failure; never panics
/// or fails the send.
pub(crate) fn record_send(
    sink: &HistorySink,
    req: &RecordedRequest,
    secrets: &[String],
    environment: Option<&str>,
    result: &Result<HttpResponse>,
) -> Option<String> {
    let m = Masker::new(secrets);
    let mut e = HistoryEntry {
        service_id: sink.service_id.clone(),
        service_name: sink.service_name.clone(),
        endpoint_id: sink.endpoint_id.clone(),
        environment: environment.map(str::to_string),
        method: req.method.clone(),
        url: m.mask(&req.url),
        request_headers: m.headers(&req.headers),
        request_body: m.mask(&req.body),
        ..Default::default()
    };
    match result {
        Ok(r) => {
            e.status = Some(r.status);
            e.status_text = m.mask(&r.status_text);
            e.response_headers = m.headers(&r.headers);
            e.body_is_base64 = r.body_is_base64;
            e.response_body = if r.body_is_base64 {
                r.body.clone()
            } else {
                m.mask(&r.body)
            };
            e.time_ms = r.time_ms;
            e.size_bytes = r.size_bytes;
            e.url = m.mask(&r.final_url);
        }
        Err(err) => e.error = Some(m.mask(&err.to_string())),
    }
    sink.store
        .record(&e)
        .err()
        .map(|err| format!("could not record request history: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_text_is_masked() {
        let dir = tempfile::tempdir().unwrap();
        let h = Arc::new(SqliteHistory::open(dir.path().join("h.sqlite")).unwrap());
        let sink = HistorySink {
            store: h.clone(),
            service_id: "s".into(),
            service_name: "S".into(),
            endpoint_id: None,
        };
        let req = RecordedRequest {
            method: "GET".into(),
            url: "http://x/".into(),
            headers: vec![],
            body: String::new(),
        };
        let resp = HttpResponse {
            status: 200,
            status_text: "echo a b!c".into(),
            headers: vec![],
            content_type: String::new(),
            body: String::new(),
            body_is_base64: false,
            pretty_body: None,
            size_bytes: 0,
            time_ms: 0,
            final_url: "http://x/".into(),
            environment: None,
            unresolved_variables: vec![],
            cookies: vec![],
        };
        assert!(record_send(&sink, &req, &["a b!c".to_string()], None, &Ok(resp)).is_none());
        let e = h.get(1).unwrap().unwrap();
        assert!(!e.status_text.contains("a b!c"), "{}", e.status_text);
    }
}
