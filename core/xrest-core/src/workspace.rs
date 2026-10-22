//! The workspace: one service directory = endpoints + scratchpad + environments.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::curl;
use crate::error::{Error, Result};
use crate::exec::{
    build_url, HttpOptions, PreparedSend, ResolvedAuth, ResolvedPreflight, ResolvedRequest,
};
use crate::interpolate::{interpolate, resolve_secrets, unresolved, Variables};
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

impl AuthKind {
    pub fn as_str(self) -> &'static str {
        match self {
            AuthKind::None => "none",
            AuthKind::Basic => "basic",
            AuthKind::Bearer => "bearer",
            AuthKind::Apikey => "apikey",
        }
    }
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

    /// Non-fatal problems found while loading (e.g. unparsable endpoint files).
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn set_secret_store(&mut self, store: Arc<dyn SecretStore>) {
        self.secrets = store;
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

    // ---- environments ----------------------------------------------------------------------

    pub fn environments(&self) -> &[Environment] {
        &self.environments
    }

    pub fn update_environments(&mut self, envs: Vec<Environment>) -> Result<()> {
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

    // ---- endpoints -------------------------------------------------------------------------

    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    pub fn endpoint(&self, id: &str) -> Result<&Endpoint> {
        self.endpoints
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| Error::NotFound {
                what: "endpoint",
                id: id.to_string(),
            })
    }

    fn sort_endpoints(&mut self) {
        self.endpoints.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    pub fn create_endpoint(&mut self, name: &str, spec: RequestSpec) -> Result<Endpoint> {
        let id = format!("e-{}", uuid::Uuid::new_v4());
        let now = now_secs();
        let mut ep = Endpoint {
            id,
            service_id: self.service.id.clone(),
            name: name.to_string(),
            method: String::new(),
            url: String::new(),
            authenticated: false,
            auth_type: self.service.effective_auth_kind().as_str().to_string(),
            metadata: EndpointMetadata {
                version: "1.0".into(),
                last_updated: now,
            },
            params: vec![],
            headers: vec![],
            body: String::new(),
            preflight: None,
            last_version: 0,
        };
        ep.apply_spec(spec);
        let v = self.versions.add(
            &ep.id,
            RequestConfig::from_endpoint(&ep),
            now,
            self.max_versions,
        )?;
        ep.last_version = v.version;
        storage::save_endpoint(&self.root, &ep)?;
        self.endpoints.push(ep.clone());
        self.sort_endpoints();
        Ok(ep)
    }

    /// Update an endpoint. A new version snapshot is recorded when the request config changed.
    pub fn update_endpoint(
        &mut self,
        id: &str,
        name: Option<&str>,
        spec: RequestSpec,
    ) -> Result<Endpoint> {
        let mut ep = self.endpoint(id)?.clone();
        let before = RequestConfig::from_endpoint(&ep);
        let renamed = name.is_some_and(|n| n != ep.name);
        if let Some(n) = name {
            ep.name = n.to_string();
        }
        ep.apply_spec(spec);
        let after = RequestConfig::from_endpoint(&ep);
        if before == after && !renamed {
            return Ok(ep);
        }
        let now = now_secs();
        ep.metadata.last_updated = now;
        if before != after {
            storage::ensure_gitignore(&self.root)?;
            let v = self.versions.add(id, after, now, self.max_versions)?;
            ep.last_version = v.version;
        }
        storage::save_endpoint(&self.root, &ep)?;
        if let Some(slot) = self.endpoints.iter_mut().find(|e| e.id == id) {
            *slot = ep.clone();
        }
        self.sort_endpoints();
        Ok(ep)
    }

    pub fn delete_endpoint(&mut self, id: &str) -> Result<()> {
        self.endpoint(id)?;
        storage::remove_file_if_exists(&storage::endpoint_path(&self.root, id)?)?;
        self.versions.clear(id)?;
        self.endpoints.retain(|e| e.id != id);
        Ok(())
    }

    /// Version history, newest first.
    pub fn endpoint_versions(&self, id: &str, limit: usize) -> Result<Vec<EndpointVersion>> {
        self.endpoint(id)?;
        self.versions.list(id, limit)
    }

    // ---- scratchpad ------------------------------------------------------------------------

    pub fn scratch_requests(&self) -> &[ScratchRequest] {
        &self.scratch
    }

    pub fn scratch(&self, id: &str) -> Result<&ScratchRequest> {
        self.scratch
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| Error::NotFound {
                what: "scratch request",
                id: id.to_string(),
            })
    }

    fn sort_scratch(&mut self) {
        self.scratch.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    pub fn create_scratch(&mut self, name: &str, spec: RequestSpec) -> Result<ScratchRequest> {
        let now = now_secs();
        let s = ScratchRequest {
            id: format!("r-{}", uuid::Uuid::new_v4()),
            name: name.to_string(),
            spec,
            created_at: now,
            updated_at: now,
        };
        storage::save_scratch(&self.root, &s)?;
        self.scratch.push(s.clone());
        self.sort_scratch();
        Ok(s)
    }

    pub fn update_scratch(
        &mut self,
        id: &str,
        name: Option<&str>,
        spec: RequestSpec,
    ) -> Result<ScratchRequest> {
        let mut s = self.scratch(id)?.clone();
        let unchanged = s.spec == spec && name.is_none_or(|n| n == s.name);
        if unchanged {
            return Ok(s);
        }
        if let Some(n) = name {
            s.name = n.to_string();
        }
        s.spec = spec;
        s.updated_at = now_secs();
        storage::save_scratch(&self.root, &s)?;
        if let Some(slot) = self.scratch.iter_mut().find(|x| x.id == id) {
            *slot = s.clone();
        }
        self.sort_scratch();
        Ok(s)
    }

    pub fn delete_scratch(&mut self, id: &str) -> Result<()> {
        self.scratch(id)?;
        storage::remove_file_if_exists(&storage::scratch_path(&self.root, id)?)?;
        self.scratch.retain(|s| s.id != id);
        Ok(())
    }

    /// Turn a scratch request into a versioned endpoint (version 1) and remove the scratch file.
    /// `name` overrides the scratch request's name.
    pub fn promote_scratch(&mut self, id: &str, name: Option<&str>) -> Result<Endpoint> {
        let s = self.scratch(id)?.clone();
        let name = name
            .filter(|n| !n.trim().is_empty())
            .map(str::to_string)
            .or_else(|| (!s.name.trim().is_empty()).then(|| s.name.clone()))
            .unwrap_or_else(|| curl::endpoint_name(&s.spec.url));
        let ep = self.create_endpoint(&name, s.spec)?;
        // The endpoint exists on disk now; only then drop the scratch file.
        self.delete_scratch(id)?;
        Ok(ep)
    }

    // ---- cURL import -----------------------------------------------------------------------

    fn spec_from_curl(&self, parsed: &curl::ParsedCurl) -> RequestSpec {
        RequestSpec {
            method: parsed.method.clone(),
            url: parsed.url.clone(),
            authenticated: self.service.is_authenticated,
            params: parsed.params.clone(),
            headers: parsed.headers.clone(),
            body: parsed.body.clone(),
            preflight: None,
        }
    }

    pub fn import_curl_as_scratch(&mut self, command: &str) -> Result<ScratchRequest> {
        let parsed = curl::parse(command)?;
        let spec = self.spec_from_curl(&parsed);
        self.create_scratch(&parsed.name, spec)
    }

    pub fn import_curl_as_endpoint(&mut self, command: &str) -> Result<Endpoint> {
        let parsed = curl::parse(command)?;
        let spec = self.spec_from_curl(&parsed);
        self.create_endpoint(&parsed.name, spec)
    }

    // ---- sending ---------------------------------------------------------------------------

    pub fn prepare_send_endpoint(&self, id: &str, confirmed: bool) -> Result<PreparedSend> {
        self.prepare_send(&self.endpoint(id)?.spec(), confirmed)
    }

    pub fn prepare_send_scratch(&self, id: &str, confirmed: bool) -> Result<PreparedSend> {
        self.prepare_send(&self.scratch(id)?.spec, confirmed)
    }

    fn variables(&self, env: Option<&Environment>) -> Variables {
        let mut vars = Variables::new();
        for v in env.map(|e| e.variables.as_slice()).unwrap_or_default() {
            if !v.enabled || v.name.is_empty() {
                continue;
            }
            let value = match v.secret() {
                Some(key) => format!("{{{{secret.{key}}}}}"),
                None => v.value.clone(),
            };
            vars.insert(v.name.clone(), value);
        }
        vars
    }

    /// Resolve `spec` against the selected environment and service auth.
    ///
    /// Enforces the unsafe-environment policy: returns [`Error::ConfirmationRequired`] unless
    /// `confirmed` is true. Nothing touches the network until [`PreparedSend::execute`].
    pub fn prepare_send(&self, spec: &RequestSpec, confirmed: bool) -> Result<PreparedSend> {
        let env = self.selected_environment();
        SendPolicy::for_environment(env).check(confirmed)?;

        let vars = self.variables(env);
        let mut r = Resolver {
            vars: &vars,
            secrets: self.secrets.as_ref(),
            unresolved: BTreeSet::new(),
            missing: BTreeSet::new(),
        };

        let params: Vec<(String, String)> = spec
            .params
            .iter()
            .filter(|p| p.enabled && !p.name.is_empty())
            .map(|p| (r.text(&p.name), r.text(&p.value)))
            .collect();
        let headers = r.headers(&spec.headers);
        let url = build_url(&r.text(&spec.url), &params)?;
        let method = normalize_method(&spec.method);
        let body = r.text(&spec.body);

        let use_service_auth = spec.authenticated && self.service.is_authenticated;
        let auth = if use_service_auth {
            self.resolve_auth(&mut r)
        } else {
            ResolvedAuth::None
        };

        let scope = format!(
            "{}|{}",
            self.service.id,
            env.map(|e| e.name.as_str()).unwrap_or("")
        );
        let pf_cfg = spec.preflight.as_ref().filter(|p| p.enabled).or_else(|| {
            use_service_auth
                .then_some(self.service.preflight.as_ref())
                .flatten()
                .filter(|p| p.enabled)
        });
        let preflight = pf_cfg
            .map(|cfg| resolve_preflight(cfg, &mut r, &scope))
            .transpose()?;

        if !r.missing.is_empty() {
            return Err(Error::MissingSecrets(r.missing.into_iter().collect()));
        }

        Ok(PreparedSend {
            environment: env.map(|e| e.name.clone()),
            request: ResolvedRequest {
                method,
                url,
                headers,
                body,
            },
            auth,
            preflight,
            unresolved: r.unresolved.into_iter().collect(),
            tokens: self.tokens.clone(),
            http: self.http.clone(),
        })
    }

    fn resolve_auth(&self, r: &mut Resolver) -> ResolvedAuth {
        let Some(cfg) = self.service.auth.as_ref() else {
            return ResolvedAuth::None;
        };
        match self.service.effective_auth_kind() {
            AuthKind::None => ResolvedAuth::None,
            AuthKind::Bearer => ResolvedAuth::Bearer(r.text(&cfg.bearer_token)),
            AuthKind::Basic => ResolvedAuth::Basic {
                user: r.text(&cfg.basic_user),
                pass: r.text(&cfg.basic_pass),
            },
            AuthKind::Apikey => ResolvedAuth::ApiKey {
                name: r.text(&cfg.api_key_name),
                value: r.text(&cfg.api_key_value),
                in_query: cfg.api_key_location.eq_ignore_ascii_case("query"),
            },
        }
    }
}

fn normalize_method(m: &str) -> String {
    let m = m.trim().to_ascii_uppercase();
    if m.is_empty() {
        "GET".into()
    } else {
        m
    }
}

/// Variable + secret resolution context for one send.
struct Resolver<'a> {
    vars: &'a Variables,
    secrets: &'a dyn SecretStore,
    unresolved: BTreeSet<String>,
    missing: BTreeSet<String>,
}

impl Resolver<'_> {
    fn text(&mut self, s: &str) -> String {
        let interpolated = interpolate(s, self.vars);
        self.unresolved.extend(unresolved(&interpolated));
        let (out, missing) = resolve_secrets(&interpolated, &|k| self.secrets.get(k));
        self.missing.extend(missing);
        out
    }

    fn headers(&mut self, rows: &[NameValue]) -> Vec<(String, String)> {
        rows.iter()
            .filter(|h| h.enabled && !h.name.is_empty())
            .map(|h| (self.text(&h.name), self.text(&h.value)))
            .collect()
    }
}

fn resolve_preflight(cfg: &Preflight, r: &mut Resolver, scope: &str) -> Result<ResolvedPreflight> {
    let method = normalize_method(&cfg.method);
    let url = build_url(&r.text(&cfg.url), &[])?;
    let mut headers = r.headers(&cfg.headers);
    let body = if cfg.body_type.eq_ignore_ascii_case("urlencoded") {
        let mut ser = url::form_urlencoded::Serializer::new(String::new());
        for p in cfg
            .body_params
            .iter()
            .filter(|p| p.enabled && !p.name.is_empty())
        {
            ser.append_pair(&r.text(&p.name), &r.text(&p.value));
        }
        if !crate::exec::has_header(&headers, "content-type") {
            headers.push((
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into(),
            ));
        }
        ser.finish()
    } else {
        r.text(&cfg.body)
    };
    let cache_key = format!("{scope}|{method}:{url}");
    Ok(ResolvedPreflight {
        request: ResolvedRequest {
            method,
            url,
            headers,
            body,
        },
        config: cfg.clone(),
        cache_key,
    })
}
