//! Endpoint version history.
//!
//! The Go app kept history in a global SQLite database (`~/.xrest/history.db`) and never wrote it
//! back into the endpoint YAML. SQLite is deferred in the Rust core (see [`crate::history`]), so
//! versions are kept in the workspace as `.xrest/versions/<endpoint-id>.json` (gitignored,
//! per-user). Semantics are ported from `SqliteVersionRepository`: the next version is
//! `MAX(version) + 1`, and only the newest `max_versions` are retained (FIFO prune).

use std::fs;
use std::path::PathBuf;

use crate::error::{Error, Result};
#[cfg(test)]
use crate::model::Body;
use crate::model::{EndpointVersion, RequestConfig};

pub const MIN_VERSIONS_LIMIT: usize = 5;
pub const MAX_VERSIONS_LIMIT: usize = 50;
pub const DEFAULT_VERSIONS_LIMIT: usize = 50;

pub fn clamp_max_versions(n: usize) -> usize {
    n.clamp(MIN_VERSIONS_LIMIT, MAX_VERSIONS_LIMIT)
}

pub trait VersionStore: Send + Sync {
    /// Append a snapshot (`MAX+1`) and prune to the newest `max_versions`.
    fn add(
        &self,
        endpoint_id: &str,
        config: RequestConfig,
        now: u64,
        max_versions: usize,
    ) -> Result<EndpointVersion>;
    /// Newest `limit` versions, newest first.
    fn list(&self, endpoint_id: &str, limit: usize) -> Result<Vec<EndpointVersion>>;
    fn count(&self, endpoint_id: &str) -> Result<usize>;
    fn clear(&self, endpoint_id: &str) -> Result<()>;
}

/// JSON-file-per-endpoint implementation.
pub struct FileVersionStore {
    dir: PathBuf,
}

impl FileVersionStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn path(&self, endpoint_id: &str) -> PathBuf {
        self.dir.join(format!("{endpoint_id}.json"))
    }

    fn read(&self, endpoint_id: &str) -> Result<Vec<EndpointVersion>> {
        let path = self.path(endpoint_id);
        match fs::read_to_string(&path) {
            Ok(s) => serde_json::from_str(&s).map_err(|e| Error::Yaml {
                path: path.clone(),
                message: e.to_string(),
            }),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
            Err(e) => Err(Error::io(path, e)),
        }
    }

    fn write(&self, endpoint_id: &str, versions: &[EndpointVersion]) -> Result<()> {
        fs::create_dir_all(&self.dir).map_err(|e| Error::io(&self.dir, e))?;
        let path = self.path(endpoint_id);
        let json = serde_json::to_string_pretty(versions).map_err(|e| Error::Yaml {
            path: path.clone(),
            message: e.to_string(),
        })?;
        crate::storage::write_atomic(&path, json.as_bytes())
    }
}

impl VersionStore for FileVersionStore {
    fn add(
        &self,
        endpoint_id: &str,
        config: RequestConfig,
        now: u64,
        max_versions: usize,
    ) -> Result<EndpointVersion> {
        crate::storage::validate_id(endpoint_id)?;
        let max_versions = clamp_max_versions(max_versions);
        let mut all = self.read(endpoint_id)?; // stored oldest-first
        let next = all.iter().map(|v| v.version).max().unwrap_or(0) + 1;
        let entry = EndpointVersion {
            version: next,
            config,
            last_updated: now,
        };
        all.push(entry.clone());
        if all.len() > max_versions {
            let excess = all.len() - max_versions;
            all.drain(..excess);
        }
        self.write(endpoint_id, &all)?;
        Ok(entry)
    }

    fn list(&self, endpoint_id: &str, limit: usize) -> Result<Vec<EndpointVersion>> {
        crate::storage::validate_id(endpoint_id)?;
        let mut all = self.read(endpoint_id)?;
        all.sort_by_key(|v| std::cmp::Reverse(v.version));
        all.truncate(limit);
        Ok(all)
    }

    fn count(&self, endpoint_id: &str) -> Result<usize> {
        crate::storage::validate_id(endpoint_id)?;
        Ok(self.read(endpoint_id)?.len())
    }

    fn clear(&self, endpoint_id: &str) -> Result<()> {
        crate::storage::validate_id(endpoint_id)?;
        let path = self.path(endpoint_id);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::io(path, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(url: &str) -> RequestConfig {
        RequestConfig {
            method: "GET".into(),
            url: url.into(),
            authenticated: false,
            auth_type: "none".into(),
            params: vec![],
            path_params: vec![],
            headers: vec![],
            body: Body::None,
            options: Default::default(),
            preflight: None,
        }
    }

    #[test]
    fn increments_and_lists_newest_first() {
        let dir = tempfile::tempdir().unwrap();
        let s = FileVersionStore::new(dir.path());
        for i in 1..=3 {
            let v = s.add("e-1", cfg(&format!("u{i}")), 100 + i, 50).unwrap();
            assert_eq!(v.version as u64, i);
        }
        let list = s.list("e-1", 2).unwrap();
        assert_eq!(
            list.iter().map(|v| v.version).collect::<Vec<_>>(),
            vec![3, 2]
        );
        assert_eq!(s.count("e-1").unwrap(), 3);
        assert_eq!(s.count("other").unwrap(), 0);
    }

    #[test]
    fn prunes_fifo_but_keeps_counting_up() {
        let dir = tempfile::tempdir().unwrap();
        let s = FileVersionStore::new(dir.path());
        for i in 0..8 {
            s.add("e-1", cfg(&format!("u{i}")), i, 5).unwrap();
        }
        let list = s.list("e-1", 100).unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].version, 8);
        assert_eq!(list[4].version, 4);
        assert_eq!(s.add("e-1", cfg("x"), 9, 5).unwrap().version, 9);
    }

    #[test]
    fn clear_removes_history() {
        let dir = tempfile::tempdir().unwrap();
        let s = FileVersionStore::new(dir.path());
        s.add("e-1", cfg("a"), 1, 5).unwrap();
        s.clear("e-1").unwrap();
        assert_eq!(s.count("e-1").unwrap(), 0);
        s.clear("e-1").unwrap();
    }

    #[test]
    fn rejects_path_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let s = FileVersionStore::new(dir.path());
        assert!(matches!(
            s.add("../evil", cfg("a"), 1, 5),
            Err(Error::InvalidId(_))
        ));
    }
}
