//! The workspace: one service directory = endpoints + scratchpad + environments.
//!
//! `Workspace` is a facade over the on-disk files. Its methods are split by concern:
//!
//! - this file: the struct, opening/creating, settings, environments, format upgrades;
//! - [`endpoints`]: versioned endpoints;
//! - [`git`]: git status/commit/sync and auto-commit on save;
//! - [`scratch`]: the scratchpad, promotion and cURL import;
//! - [`send`]: preparing requests (the logic itself lives in [`crate::resolve`]).

mod endpoints;
mod git;
mod import;
mod scratch;
mod send;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use crate::exec::HttpOptions;
use crate::history::{HistorySink, HistoryStore};
use crate::interpolate::InterpolationPreview;
use crate::model::*;
use crate::policy::SendPolicy;
use crate::preflight::TokenCache;
use crate::secrets::{MemorySecretStore, SecretStore};
use crate::storage::{self, LocalOverrides};
use crate::variables::{GlobalVariables, VariableLayers, VariableReport};
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
    /// `variables.yaml`: shared by every environment.
    variables: Vec<NameValue>,
    /// `.xrest/overrides.yaml`: this user's overrides, per environment.
    overrides: LocalOverrides,
    /// Global variables shared by every open workspace (attached by the app).
    globals: Option<Arc<GlobalVariables>>,
    warnings: Vec<String>,
    versions: Box<dyn VersionStore>,
    max_versions: usize,
    tokens: Arc<TokenCache>,
    secrets: Arc<dyn SecretStore>,
    http: HttpOptions,
    history: Option<Arc<dyn HistoryStore>>,
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
            variables: loaded.variables,
            overrides: loaded.overrides,
            globals: None,
            warnings: loaded.warnings,
            versions,
            max_versions: DEFAULT_VERSIONS_LIMIT,
            tokens: Arc::new(TokenCache::default()),
            secrets: Arc::new(MemorySecretStore::default()),
            http: HttpOptions::default(),
            history: None,
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
        let env = |n: &str, is_unsafe: bool, color: &str| Environment {
            name: n.to_string(),
            is_unsafe,
            variables: vec![NameValue::new("BASE_URL", "")],
            color: Some(color.to_string()),
            ..Default::default()
        };
        storage::save_service(root, &service)?;
        storage::save_environments(
            root,
            &[
                env("DEV", false, "green"),
                env("STAGE", false, "amber"),
                env("PROD", true, "red"),
            ],
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

    /// The editable service settings (name, auth, preflight).
    pub fn service_settings(&self) -> ServiceSettings {
        ServiceSettings::from_service(&self.service)
    }

    /// Write the service settings to `service.yaml`, keeping `id`, `gitUrl` and stamping the
    /// format version. Clears the token cache, since auth or preflight may have changed.
    pub fn update_service_settings(&mut self, settings: ServiceSettings) -> Result<()> {
        let name = settings.name.trim().to_string();
        if name.is_empty() {
            return Err(Error::InvalidInput("service name cannot be empty".into()));
        }
        self.ensure_current_format()?;
        let mut service = self.service.clone();
        service.name = name;
        service.is_authenticated = settings.is_authenticated;
        service.auth_type = Some(settings.auth_type);
        service.auth = Some(AuthConfig {
            kind: settings.auth_type,
            active: true,
            basic_user: settings.basic_user,
            basic_pass: settings.basic_pass,
            bearer_token: settings.bearer_token,
            api_key_name: settings.api_key_name,
            api_key_value: settings.api_key_value,
            api_key_location: settings.api_key_location,
        });
        service.preflight = settings.preflight;
        storage::save_service(&self.root, &service)?;
        self.service = service;
        self.tokens.clear();
        self.auto_commit(&[storage::SERVICE_FILE], "Update service settings");
        Ok(())
    }

    /// Record every send in `history` (shared between workspaces; not stored in the workspace).
    pub fn set_history(&mut self, history: Arc<dyn HistoryStore>) {
        self.history = Some(history);
    }

    /// Where a send from this workspace is recorded, if a history is attached.
    pub fn history_sink(&self, endpoint_id: Option<&str>) -> Option<HistorySink> {
        self.history.as_ref().map(|store| HistorySink {
            store: store.clone(),
            service_id: self.service.id.clone(),
            service_name: self.service.name.clone(),
            endpoint_id: endpoint_id.map(str::to_string),
        })
    }

    /// Add a non-fatal problem to [`Workspace::warnings`].
    pub fn push_warning(&mut self, warning: String) {
        self.warnings.push(warning);
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
        scan(serde_json::to_value(&self.variables).unwrap_or_default());
        scan(serde_json::to_value(&self.overrides).unwrap_or_default());
        if let Some(g) = &self.globals {
            scan(serde_json::to_value(g.get()).unwrap_or_default());
        }
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
                    ..self.state.clone()
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
        self.auto_commit(&[storage::ENVIRONMENTS_FILE], "Update environments");
        Ok(())
    }

    /// Replace all environments. Names are trimmed and must be non-empty and unique
    /// ([`Error::InvalidInput`] otherwise, nothing written). The selection is kept if its name
    /// still exists, else it moves to the first environment.
    pub fn save_environments(&mut self, mut envs: Vec<Environment>) -> Result<()> {
        let mut seen = std::collections::HashSet::new();
        for e in &mut envs {
            e.name = e.name.trim().to_string();
            if e.name.is_empty() {
                return Err(Error::InvalidInput(
                    "environment names cannot be empty".into(),
                ));
            }
            if !seen.insert(e.name.clone()) {
                return Err(Error::InvalidInput(format!(
                    "duplicate environment name {:?}",
                    e.name
                )));
            }
            e.color = checked_name(e.color.take(), PALETTE, "color")?;
            e.min_countdown_seconds = e.min_countdown_seconds.min(MAX_MIN_COUNTDOWN_SECONDS);
        }
        self.ensure_current_format()?;
        let current = self.selected_environment().map(|e| e.name.clone());
        storage::save_environments(&self.root, &envs)?;
        self.environments = envs;
        let keep = current.filter(|c| self.environments.iter().any(|e| &e.name == c));
        let next = keep.or_else(|| self.environments.first().map(|e| e.name.clone()));
        if self.state.selected_environment != next {
            let state = WorkspaceState {
                selected_environment: next,
                ..self.state.clone()
            };
            storage::save_state(&self.root, &state)?;
            self.state = state;
        }
        self.auto_commit(&[storage::ENVIRONMENTS_FILE], "Update environments");
        Ok(())
    }

    /// Every enabled variable name available in the selected environment, from all layers,
    /// most specific first, then the built-ins (for autocomplete).
    pub fn variable_names(&self) -> Vec<String> {
        let env = self.selected_environment();
        self.variable_layers(env).names(env)
    }

    // ---- variable layers -------------------------------------------------------------------

    /// The list layers for `env`: globals (if attached), workspace variables and this user's
    /// overrides for that environment.
    pub fn variable_layers(&self, env: Option<&Environment>) -> VariableLayers {
        VariableLayers {
            global: self.globals.as_ref().map(|g| g.get()).unwrap_or_default(),
            workspace: self.variables.clone(),
            local: env
                .and_then(|e| self.overrides.get(&e.name))
                .cloned()
                .unwrap_or_default(),
        }
    }

    /// Attach the app's global variables (shared by every open workspace).
    pub fn set_global_variables(&mut self, globals: Arc<GlobalVariables>) {
        self.globals = Some(globals);
    }

    /// Variables shared by every environment (`variables.yaml`).
    pub fn workspace_variables(&self) -> &[NameValue] {
        &self.variables
    }

    /// Replace the workspace variables. Names are trimmed; rows with no name are dropped.
    pub fn save_workspace_variables(&mut self, vars: Vec<NameValue>) -> Result<()> {
        let vars = clean_rows(vars);
        self.ensure_current_format()?;
        storage::save_variables(&self.root, &vars)?;
        self.variables = vars;
        self.auto_commit(&[storage::VARIABLES_FILE], "Update workspace variables");
        Ok(())
    }

    /// This user's overrides for `environment` (empty if none).
    pub fn local_overrides(&self, environment: &str) -> &[NameValue] {
        self.overrides
            .get(environment)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Replace this user's overrides for `environment`; an empty list removes them. Stored in
    /// `.xrest/overrides.yaml`, never committed. Overrides are keyed by environment name, so
    /// renaming an environment leaves its old overrides behind (they reappear if the name
    /// comes back).
    pub fn save_local_overrides(&mut self, environment: &str, rows: Vec<NameValue>) -> Result<()> {
        if !self.environments.iter().any(|e| e.name == environment) {
            return Err(Error::UnknownEnvironment(environment.to_string()));
        }
        let rows = clean_rows(rows);
        let mut next = self.overrides.clone();
        if rows.is_empty() {
            next.remove(environment);
        } else {
            next.insert(environment.to_string(), rows);
        }
        storage::save_overrides(&self.root, &next)?;
        self.overrides = next;
        Ok(())
    }

    /// Where each variable's value comes from in `environment` (the selected one when `None`).
    pub fn variable_report(&self, environment: Option<&str>) -> Result<Vec<VariableReport>> {
        let env = match environment {
            Some(name) => Some(
                self.environments
                    .iter()
                    .find(|e| e.name == name)
                    .ok_or_else(|| Error::UnknownEnvironment(name.to_string()))?,
            ),
            None => self.selected_environment(),
        };
        Ok(self.variable_layers(env).report(env))
    }

    // ---- appearance ------------------------------------------------------------------------

    pub fn appearance(&self) -> &WorkspaceAppearance {
        &self.service.appearance
    }

    /// Set the workspace color and icon. Unknown names are [`Error::InvalidInput`].
    pub fn set_appearance(&mut self, appearance: WorkspaceAppearance) -> Result<()> {
        let appearance = WorkspaceAppearance {
            color: checked_name(appearance.color, PALETTE, "color")?,
            icon: checked_name(appearance.icon, WORKSPACE_ICONS, "icon")?,
        };
        self.ensure_current_format()?;
        let mut service = self.service.clone();
        service.appearance = appearance;
        storage::save_service(&self.root, &service)?;
        self.service = service;
        self.auto_commit(&[storage::SERVICE_FILE], "Update workspace appearance");
        Ok(())
    }

    /// Resolve `text` against the selected environment for display. Secret values are masked,
    /// never returned; unresolved names are listed. Only secret lookup touches anything outside
    /// memory.
    pub fn preview(&self, text: &str) -> InterpolationPreview {
        let env = self.selected_environment();
        let vars = self.variable_layers(env).resolve_static(env);
        crate::interpolate::preview(text, &vars, &|k| self.secrets.contains(k))
    }

    /// The URL `spec` would be sent to, for display: variables interpolated, enabled path and
    /// query params applied exactly as [`Workspace::prepare_send`] does, secrets masked as
    /// `••••••`. No network, no file reads, no policy check; secrets are only tested for
    /// existence. Unresolved variables, `{name}` path params and missing secrets are listed.
    pub fn preview_request_url(&self, spec: &RequestSpec) -> InterpolationPreview {
        let env = self.selected_environment();
        crate::resolve::preview_request_url(
            env,
            &self.variable_layers(env),
            self.secrets.as_ref(),
            spec,
        )
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

    /// Whether the selected environment is unsafe at all (any method).
    pub fn send_policy(&self) -> SendPolicy {
        SendPolicy::for_environment(self.selected_environment())
    }

    /// The policy for sending a `method` request in the selected environment: whether it needs
    /// confirmation and the workspace's minimum countdown.
    pub fn send_policy_for(&self, method: &str) -> SendPolicy {
        SendPolicy::for_request(self.selected_environment(), method)
    }
}

/// Trim names and drop rows without one.
fn clean_rows(rows: Vec<NameValue>) -> Vec<NameValue> {
    rows.into_iter()
        .map(|mut r| {
            r.name = r.name.trim().to_string();
            r
        })
        .filter(|r| !r.name.is_empty())
        .collect()
}

/// `None` and empty stay `None`; anything else must be in `allowed`.
fn checked_name(value: Option<String>, allowed: &[&str], what: &str) -> Result<Option<String>> {
    match value.map(|v| v.trim().to_ascii_lowercase()) {
        None => Ok(None),
        Some(v) if v.is_empty() => Ok(None),
        Some(v) if allowed.contains(&v.as_str()) => Ok(Some(v)),
        Some(v) => Err(Error::InvalidInput(format!(
            "unknown {what} {v:?}; expected one of {}",
            allowed.join(", ")
        ))),
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
