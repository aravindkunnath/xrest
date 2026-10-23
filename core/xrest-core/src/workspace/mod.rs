//! The workspace: one service directory = endpoints + scratchpad + environments.
//!
//! `Workspace` is a facade over the on-disk files. Its methods are split by concern:
//!
//! - this file: the struct, opening/creating, settings, environments, format upgrades;
//! - [`endpoints`]: versioned endpoints;
//! - [`scratch`]: the scratchpad, promotion and cURL import;
//! - [`send`]: preparing requests (the logic itself lives in [`crate::resolve`]).

mod endpoints;
mod scratch;
mod send;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::exec::HttpOptions;
use crate::model::*;
use crate::policy::SendPolicy;
use crate::preflight::TokenCache;
use crate::secrets::{MemorySecretStore, SecretStore};
use crate::storage;
use crate::versions::{clamp_max_versions, FileVersionStore, VersionStore, DEFAULT_VERSIONS_LIMIT};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub struct Workspace {
    root: PathBuf,
    service: ServiceFile,
    environments: Vec<Environment>,
    endpoints: Vec<Endpoint>,
    scratch: Vec<ScratchRequest>,
    state: WorkspaceState,
    warnings: Vec<String>,
    versions: Box<dyn VersionStore>,
    max_versions: usize,
    tokens: Arc<TokenCache>,
    secrets: Arc<dyn SecretStore>,
    http: HttpOptions,
}

impl Workspace {
    // ---- lifecycle -------------------------------------------------------------------------

    /// Open an existing workspace directory (must contain `service.yaml`).
    ///
    /// Fails with [`Error::UnsupportedFormat`] if a newer XRest wrote it. Older workspaces open
    /// as-is and are upgraded to [`FORMAT_VERSION`] the first time something is saved.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let loaded = storage::load(&root)?;
        let versions = Box::new(FileVersionStore::new(
            root.join(storage::LOCAL_DIR).join(storage::VERSIONS_DIR),
        ));
        Ok(Self {
            root,
            service: loaded.service,
            environments: loaded.environments,
            endpoints: loaded.endpoints,
            scratch: loaded.scratch,
            state: loaded.state,
            warnings: loaded.warnings,
            versions,
            max_versions: DEFAULT_VERSIONS_LIMIT,
            tokens: Arc::new(TokenCache::default()),
            secrets: Arc::new(MemorySecretStore::default()),
            http: HttpOptions::default(),
        })
    }

    /// Initialise a new workspace in `root` (created if missing) and open it.
    pub fn create(root: impl AsRef<Path>, name: &str) -> Result<Self> {
        let root = root.as_ref();
        if root.join(storage::SERVICE_FILE).exists() {
            return Err(Error::AlreadyExists(root.to_path_buf()));
        }
        let service = ServiceFile {
            format_version: FORMAT_VERSION,
            id: format!("s-{}", uuid::Uuid::new_v4()),
            name: name.to_string(),
            auth_type: Some(AuthKind::None),
            auth: Some(AuthConfig {
                active: true,
                ..Default::default()
            }),
            ..Default::default()
        };
        let env = |n: &str, is_unsafe: bool| Environment {
            name: n.to_string(),
            is_unsafe,
            variables: vec![NameValue::new("BASE_URL", "")],
        };
        storage::save_service(root, &service)?;
        storage::save_environments(
            root,
            &[env("DEV", false), env("STAGE", false), env("PROD", true)],
        )?;
        std::fs::create_dir_all(root.join(storage::ENDPOINTS_DIR))
            .map_err(|e| Error::io(root.join(storage::ENDPOINTS_DIR), e))?;
        storage::ensure_gitignore(root)?;
        Self::open(root)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn service(&self) -> &ServiceFile {
        &self.service
    }

    /// The format version of the files on disk (may be older than [`FORMAT_VERSION`] until the
    /// first save).
    pub fn format_version(&self) -> u32 {
        self.service.format_version
    }

    /// Non-fatal problems found while loading (e.g. unparsable endpoint files).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn set_secret_store(&mut self, store: Arc<dyn SecretStore>) {
        self.secrets = store;
    }

    /// Every distinct secret key referenced by the service, environments, endpoints and scratch
    /// requests: `{{secret.KEY}}` placeholders in any string, and any `secretKey` field. Sorted.
    pub fn secret_keys_in_use(&self) -> Vec<String> {
        let mut keys = std::collections::BTreeSet::new();
        let mut scan = |v: serde_json::Value| collect_secret_keys(&v, &mut keys);
        scan(serde_json::to_value(&self.service).unwrap_or_default());
        scan(serde_json::to_value(&self.environments).unwrap_or_default());
        scan(serde_json::to_value(&self.endpoints).unwrap_or_default());
        scan(serde_json::to_value(&self.scratch).unwrap_or_default());
        keys.into_iter().collect()
    }

    pub fn set_http_options(&mut self, opts: HttpOptions) {
        self.http = opts;
    }

    pub fn set_max_versions(&mut self, n: usize) {
        self.max_versions = clamp_max_versions(n);
    }

    pub fn clear_token_cache(&self) {
        self.tokens.clear();
    }

    /// Upgrade an older workspace to [`FORMAT_VERSION`] before writing anything in the current
    /// format. Called by every method that writes request data; a no-op once current.
    fn ensure_current_format(&mut self) -> Result<()> {
        if self.service.format_version >= FORMAT_VERSION {
            return Ok(());
        }
        // `service.yaml` no longer carries the selected environment; keep the user's choice.
        if self.state.selected_environment.is_none() {
            if let Some(sel) = self.service.selected_environment.clone() {
                let state = WorkspaceState {
                    selected_environment: Some(sel),
                };
                storage::save_state(&self.root, &state)?;
                self.state = state;
            }
        }
        let mut service = self.service.clone();
        service.format_version = FORMAT_VERSION;
        storage::save_service(&self.root, &service)?;
        self.service = service;
        Ok(())
    }

    // ---- environments ----------------------------------------------------------------------

    pub fn environments(&self) -> &[Environment] {
        &self.environments
    }

    pub fn update_environments(&mut self, envs: Vec<Environment>) -> Result<()> {
        self.ensure_current_format()?;
        storage::save_environments(&self.root, &envs)?;
        self.environments = envs;
        Ok(())
    }

    /// The active environment: saved choice, else the legacy `selectedEnvironment` from
    /// `service.yaml`, else the first environment.
    pub fn selected_environment(&self) -> Option<&Environment> {
        let wanted = self
            .state
            .selected_environment
            .as_deref()
            .or(self.service.selected_environment.as_deref());
        wanted
            .and_then(|n| self.environments.iter().find(|e| e.name == n))
            .or_else(|| self.environments.first())
    }

    pub fn select_environment(&mut self, name: &str) -> Result<()> {
        if !self.environments.iter().any(|e| e.name == name) {
            return Err(Error::UnknownEnvironment(name.to_string()));
        }
        let mut next = self.state.clone();
        next.selected_environment = Some(name.to_string());
        storage::save_state(&self.root, &next)?;
        self.state = next;
        Ok(())
    }

    pub fn send_policy(&self) -> SendPolicy {
        SendPolicy::for_environment(self.selected_environment())
    }
}

fn collect_secret_keys(v: &serde_json::Value, out: &mut std::collections::BTreeSet<String>) {
    use serde_json::Value;
    match v {
        Value::String(s) => {
            for name in crate::interpolate::placeholders(s) {
                if let Some(k) = name.strip_prefix("secret.") {
                    out.insert(k.to_string());
                }
            }
        }
        Value::Array(a) => a.iter().for_each(|x| collect_secret_keys(x, out)),
        Value::Object(m) => {
            for (k, x) in m {
                match x {
                    Value::String(s) if k == "secretKey" && !s.is_empty() => {
                        out.insert(s.clone());
                    }
                    _ => collect_secret_keys(x, out),
                }
            }
        }
        _ => {}
    }
}
