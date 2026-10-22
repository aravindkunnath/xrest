//! On-disk layout and (de)serialization.
//!
//! ```text
//! <workspace>/
//! ├── service.yaml
//! ├── environments.yaml
//! ├── endpoints/<id>.yaml
//! ├── scratchpad/<id>.yaml      (gitignored)
//! ├── .xrest/                   (gitignored: state.yaml, versions/)
//! └── .gitignore
//! ```

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::{Error, Result};
use crate::model::{Endpoint, Environment, ScratchRequest, ServiceFile, WorkspaceState};

pub const SERVICE_FILE: &str = "service.yaml";
pub const ENVIRONMENTS_FILE: &str = "environments.yaml";
pub const ENDPOINTS_DIR: &str = "endpoints";
pub const SCRATCH_DIR: &str = "scratchpad";
pub const LOCAL_DIR: &str = ".xrest";
pub const STATE_FILE: &str = "state.yaml";
pub const VERSIONS_DIR: &str = "versions";
pub const GITIGNORE_FILE: &str = ".gitignore";

/// Entries every workspace `.gitignore` must contain.
pub const GITIGNORE_ENTRIES: [&str; 2] = ["/scratchpad/", "/.xrest/"];

/// Ids become file names, so keep them to a safe alphabet.
pub fn validate_id(id: &str) -> Result<()> {
    let ok = !id.is_empty()
        && id.len() <= 128
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'));
    if ok {
        Ok(())
    } else {
        Err(Error::InvalidId(id.to_string()))
    }
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    let tmp = dir.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
        std::process::id()
    ));
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(Error::io(path, e));
    }
    Ok(())
}

pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
    serde_yaml_ng::from_str(&text).map_err(|e| Error::Yaml {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

pub fn write_yaml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let text = serde_yaml_ng::to_string(value).map_err(|e| Error::Yaml {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    write_atomic(path, text.as_bytes())
}

/// Create/extend `<root>/.gitignore` so it contains [`GITIGNORE_ENTRIES`]. Existing content and
/// existing entries are preserved.
pub fn ensure_gitignore(root: &Path) -> Result<()> {
    let path = root.join(GITIGNORE_FILE);
    let existing = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(Error::io(path, e)),
    };
    let has = |entry: &str| {
        let bare = entry.trim_matches('/');
        existing.lines().map(str::trim).any(|l| {
            let l = l.trim_start_matches('/').trim_end_matches('/');
            l == bare
        })
    };
    let missing: Vec<&str> = GITIGNORE_ENTRIES.into_iter().filter(|e| !has(e)).collect();
    if missing.is_empty() {
        return Ok(());
    }
    let mut out = existing;
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    if !out.contains("# XRest local-only state") {
        out.push_str(
            "# XRest local-only state (scratch requests, per-user state, version history)\n",
        );
    }
    for m in missing {
        out.push_str(m);
        out.push('\n');
    }
    write_atomic(&path, out.as_bytes())
}

fn yaml_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(rd) = fs::read_dir(dir) else {
        return vec![];
    };
    let mut files: Vec<PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension().is_some_and(|x| x == "yaml" || x == "yml")
                && !p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with('.'))
        })
        .collect();
    files.sort();
    files
}

pub struct Loaded {
    pub service: ServiceFile,
    pub environments: Vec<Environment>,
    pub endpoints: Vec<Endpoint>,
    pub scratch: Vec<ScratchRequest>,
    pub state: WorkspaceState,
    /// Non-fatal problems (unparsable endpoint files and the like).
    pub warnings: Vec<String>,
}

pub fn load(root: &Path) -> Result<Loaded> {
    let service_path = root.join(SERVICE_FILE);
    if !service_path.is_file() {
        return Err(Error::NotAWorkspace(root.to_path_buf()));
    }
    let service: ServiceFile = read_yaml(&service_path)?;
    let mut warnings = vec![];

    let env_path = root.join(ENVIRONMENTS_FILE);
    let environments = if env_path.is_file() {
        match read_yaml::<Option<Vec<Environment>>>(&env_path) {
            Ok(v) => v.unwrap_or_default(),
            Err(e) => {
                warnings.push(e.to_string());
                vec![]
            }
        }
    } else {
        vec![]
    };

    let mut endpoints = vec![];
    for p in yaml_files(&root.join(ENDPOINTS_DIR)) {
        match read_yaml::<Endpoint>(&p) {
            Ok(mut ep) => match validate_id(&ep.id) {
                Ok(()) => {
                    if ep.service_id.is_empty() {
                        ep.service_id = service.id.clone();
                    }
                    endpoints.push(ep)
                }
                Err(e) => warnings.push(format!("{}: {e}", p.display())),
            },
            Err(e) => warnings.push(e.to_string()),
        }
    }
    endpoints.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut scratch = vec![];
    for p in yaml_files(&root.join(SCRATCH_DIR)) {
        match read_yaml::<ScratchRequest>(&p) {
            Ok(s) => match validate_id(&s.id) {
                Ok(()) => scratch.push(s),
                Err(e) => warnings.push(format!("{}: {e}", p.display())),
            },
            Err(e) => warnings.push(e.to_string()),
        }
    }
    scratch.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| a.id.cmp(&b.id))
    });

    let state_path = root.join(LOCAL_DIR).join(STATE_FILE);
    let state = if state_path.is_file() {
        read_yaml(&state_path).unwrap_or_else(|e| {
            warnings.push(e.to_string());
            WorkspaceState::default()
        })
    } else {
        WorkspaceState::default()
    };

    Ok(Loaded {
        service,
        environments,
        endpoints,
        scratch,
        state,
        warnings,
    })
}

pub fn endpoint_path(root: &Path, id: &str) -> Result<PathBuf> {
    validate_id(id)?;
    Ok(root.join(ENDPOINTS_DIR).join(format!("{id}.yaml")))
}

pub fn scratch_path(root: &Path, id: &str) -> Result<PathBuf> {
    validate_id(id)?;
    Ok(root.join(SCRATCH_DIR).join(format!("{id}.yaml")))
}

pub fn save_service(root: &Path, service: &ServiceFile) -> Result<()> {
    write_yaml(&root.join(SERVICE_FILE), service)
}

pub fn save_environments(root: &Path, envs: &[Environment]) -> Result<()> {
    write_yaml(&root.join(ENVIRONMENTS_FILE), &envs)
}

pub fn save_endpoint(root: &Path, ep: &Endpoint) -> Result<()> {
    write_yaml(&endpoint_path(root, &ep.id)?, ep)
}

pub fn save_scratch(root: &Path, s: &ScratchRequest) -> Result<()> {
    ensure_gitignore(root)?;
    write_yaml(&scratch_path(root, &s.id)?, s)
}

pub fn save_state(root: &Path, state: &WorkspaceState) -> Result<()> {
    ensure_gitignore(root)?;
    write_yaml(&root.join(LOCAL_DIR).join(STATE_FILE), state)
}

pub fn remove_file_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(Error::io(path, e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_validation() {
        for ok in ["e-123", "a_b.c", "R1"] {
            assert!(validate_id(ok).is_ok(), "{ok}");
        }
        for bad in ["", ".hidden", "a/b", "..", "a\\b", "a b", "é"] {
            assert!(validate_id(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn gitignore_created_and_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        ensure_gitignore(dir.path()).unwrap();
        let first = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(first.contains("/scratchpad/\n"));
        assert!(first.contains("/.xrest/\n"));
        ensure_gitignore(dir.path()).unwrap();
        assert_eq!(
            first,
            fs::read_to_string(dir.path().join(".gitignore")).unwrap()
        );
    }

    #[test]
    fn gitignore_preserves_existing_and_detects_variants() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gitignore"), "target\nscratchpad/").unwrap();
        ensure_gitignore(dir.path()).unwrap();
        let s = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(s.starts_with("target\nscratchpad/\n"));
        assert_eq!(s.matches("scratchpad").count(), 1);
        assert!(s.contains("/.xrest/"));
    }
}
