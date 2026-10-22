//! Request history.
//!
//! TODO(history): port `internal/adapters/history.go` (SQLite at `~/.xrest/history.db`, table
//! `history`, columns per `models.HistoryEntry`). Deferred on purpose: choose between `rusqlite`
//! (bundled) and keeping the existing Go schema so the legacy database remains readable.

use crate::error::{Error, Result};
use crate::model::HttpResponse;

/// Recorded send. Shape intentionally minimal until the storage format is decided.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub method: String,
    pub url: String,
    pub response: HttpResponse,
}

pub trait HistoryStore: Send + Sync {
    fn record(&self, entry: &HistoryEntry) -> Result<()>;
}

/// Placeholder store: `record` reports that history is not implemented.
pub struct UnimplementedHistory;

impl HistoryStore for UnimplementedHistory {
    fn record(&self, _entry: &HistoryEntry) -> Result<()> {
        Err(Error::NotImplemented("request history (SQLite)"))
    }
}
