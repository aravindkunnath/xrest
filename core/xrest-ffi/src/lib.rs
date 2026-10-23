//! UniFFI surface over `xrest-core`.
//!
//! The API is deliberately coarse: one object ([`XRestWorkspace`]) plus plain records. Records
//! mirror the core types (the core stays free of any FFI dependency); conversions live here.
//! Swift bindings are generated from this crate; see `macos/scripts/generate-bindings.sh`.

use std::sync::{Arc, Mutex, MutexGuard};

use xrest_core as core;

uniffi::setup_scaffolding!();

// ---- errors -------------------------------------------------------------------------------

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum XRestError {
    /// The selected environment is unsafe and `confirmed` was false.
    #[error("environment {environment:?} is unsafe: sending requires confirmation")]
    ConfirmationRequired { environment: String },
    #[error("{message}")]
    NotFound { message: String },
    #[error("{message}")]
    InvalidInput { message: String },
    #[error("{message}")]
    Io { message: String },
    #[error("{message}")]
    Http { message: String },
    #[error("{message}")]
    Preflight { message: String },
    /// The workspace was written by a newer XRest.
    #[error("this workspace uses format {found}; this XRest supports up to {supported}. Update XRest to open it.")]
    UnsupportedFormat { found: u32, supported: u32 },
    #[error("missing secrets: {}", keys.join(", "))]
    MissingSecrets { keys: Vec<String> },
    #[error("{message}")]
    NotImplemented { message: String },
}

impl From<core::Error> for XRestError {
    fn from(e: core::Error) -> Self {
        let message = e.to_string();
        match e {
            core::Error::ConfirmationRequired { environment } => {
                XRestError::ConfirmationRequired { environment }
            }
            core::Error::NotFound { .. } => XRestError::NotFound { message },
            core::Error::UnsupportedFormat {
                found, supported, ..
            } => XRestError::UnsupportedFormat { found, supported },
            core::Error::NotAWorkspace(_)
            | core::Error::AlreadyExists(_)
            | core::Error::InvalidId(_)
            | core::Error::InvalidUrl { .. }
            | core::Error::UnknownEnvironment(_)
            | core::Error::Curl(_)
            | core::Error::InvalidBody(_)
            | core::Error::InvalidInput(_) => XRestError::InvalidInput { message },
            core::Error::Io { .. } | core::Error::Yaml { .. } => XRestError::Io { message },
            core::Error::MissingSecrets(keys) => XRestError::MissingSecrets { keys },
            core::Error::Preflight(_) => XRestError::Preflight { message },
            core::Error::Http(_) => XRestError::Http { message },
            core::Error::NotImplemented(_) => XRestError::NotImplemented { message },
        }
    }
}

type Result<T> = std::result::Result<T, XRestError>;

// ---- records ------------------------------------------------------------------------------

#[derive(Debug, Clone, uniffi::Record)]
pub struct NameValue {
    pub name: String,
    pub value: String,
    pub enabled: bool,
    pub secret_key: Option<String>,
    /// `plain` or `secret` (informational).
    pub kind: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Preflight {
    pub enabled: bool,
    pub method: String,
    pub url: String,
    pub body: String,
    pub body_type: String,
    pub body_params: Vec<NameValue>,
    pub headers: Vec<NameValue>,
    pub cache_token: bool,
    pub cache_duration: String,
    pub cache_duration_key: String,
    pub cache_duration_unit: String,
    pub token_key: String,
    pub token_header: Option<String>,
}

/// A typed request body. File paths are relative to the workspace root and must stay inside it.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum RequestBody {
    /// No body. (Not called `None`, which would be ambiguous with Swift's `Optional.none`.)
    Empty,
    /// Text as-is; without `content_type` the type is sniffed (JSON-looking → JSON).
    Raw {
        content: String,
        content_type: Option<String>,
    },
    Json {
        content: String,
    },
    /// `application/x-www-form-urlencoded`.
    Form {
        fields: Vec<NameValue>,
    },
    /// `multipart/form-data`; fields with kind `file` upload the file at `value`.
    Multipart {
        fields: Vec<NameValue>,
    },
    /// The bytes of one file.
    Binary {
        path: String,
        content_type: Option<String>,
    },
}

/// Per-request transport options; `None` means the app default.
#[derive(Debug, Clone, Default, uniffi::Record)]
pub struct RequestOptions {
    /// Milliseconds; 0 means the default.
    pub timeout_ms: Option<u64>,
    pub follow_redirects: Option<bool>,
    /// `false` accepts invalid TLS certificates.
    pub verify_tls: Option<bool>,
    /// Proxy URL; empty forces a direct connection.
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct RequestSpec {
    pub method: String,
    pub url: String,
    pub authenticated: bool,
    pub params: Vec<NameValue>,
    /// Values for `{name}` tokens in the URL path.
    pub path_params: Vec<NameValue>,
    pub headers: Vec<NameValue>,
    pub body: RequestBody,
    pub options: RequestOptions,
    pub preflight: Option<Preflight>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct Endpoint {
    pub id: String,
    pub name: String,
    pub spec: RequestSpec,
    pub last_version: i32,
    /// Unix seconds.
    pub last_updated: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ScratchRequest {
    pub id: String,
    pub name: String,
    pub spec: RequestSpec,
    /// Unix seconds.
    pub updated_at: u64,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct EndpointVersion {
    pub version: i32,
    pub last_updated: u64,
    pub spec: RequestSpec,
}

#[derive(Debug, Clone, uniffi::Record)]
/// Named variable set. (Not called `Environment` to avoid clashing with SwiftUI's `Environment`.)
pub struct EnvironmentConfig {
    pub name: String,
    pub is_unsafe: bool,
    pub variables: Vec<NameValue>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub is_authenticated: bool,
    /// `none`, `basic`, `bearer` or `apikey`.
    pub auth_kind: String,
    /// On-disk format version (older workspaces are upgraded on the first save).
    pub format_version: u32,
    pub basic_user: String,
    pub basic_pass: String,
    pub bearer_token: String,
    pub api_key_name: String,
    pub api_key_value: String,
    /// `header` or `query` (empty means header).
    pub api_key_location: String,
    pub preflight: Option<Preflight>,
}

/// The editable service settings (`update_service_settings`). `id`, git URL and format version
/// are not editable.
#[derive(Debug, Clone, uniffi::Record)]
pub struct ServiceSettings {
    pub name: String,
    pub is_authenticated: bool,
    /// `none`, `basic`, `bearer` or `apikey`.
    pub auth_type: String,
    pub basic_user: String,
    pub basic_pass: String,
    pub bearer_token: String,
    pub api_key_name: String,
    pub api_key_value: String,
    pub api_key_location: String,
    pub preflight: Option<Preflight>,
}

/// Text with variables resolved and every secret shown as `••••••`.
#[derive(Debug, Clone, uniffi::Record)]
pub struct InterpolationPreview {
    pub resolved: String,
    /// Plain variable names, then `secret.KEY` for secrets with no value.
    pub unresolved: Vec<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct SendPolicy {
    pub environment: Option<String>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct HeaderPair {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<HeaderPair>,
    pub content_type: String,
    /// UTF-8 text, or base64 when `body_is_base64`.
    pub body: String,
    pub body_is_base64: bool,
    /// Re-indented JSON when the body is JSON, else `None`.
    pub pretty_body: Option<String>,
    pub size_bytes: u64,
    pub time_ms: u64,
    pub final_url: String,
    pub environment: Option<String>,
    pub unresolved_variables: Vec<String>,
}

/// The outcome of "Test Preflight": the exchange and the extracted token. `error` is set when the
/// request failed, returned non-2xx, or the token could not be extracted.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PreflightTestResult {
    pub method: String,
    pub url: String,
    pub request_headers: Vec<HeaderPair>,
    pub request_body: String,
    pub status: Option<u16>,
    pub response_headers: Vec<HeaderPair>,
    pub response_body: String,
    pub token: Option<String>,
    pub error: Option<String>,
    pub time_ms: u64,
}

impl From<core::PreflightTestResult> for PreflightTestResult {
    fn from(r: core::PreflightTestResult) -> Self {
        let pairs = |v: Vec<(String, String)>| {
            v.into_iter()
                .map(|(name, value)| HeaderPair { name, value })
                .collect()
        };
        Self {
            method: r.method,
            url: r.url,
            request_headers: pairs(r.request_headers),
            request_body: r.request_body,
            status: r.status,
            response_headers: pairs(r.response_headers),
            response_body: r.response_body,
            token: r.token,
            error: r.error,
            time_ms: r.time_ms,
        }
    }
}

// ---- conversions --------------------------------------------------------------------------

impl From<core::NameValue> for NameValue {
    fn from(v: core::NameValue) -> Self {
        Self {
            name: v.name,
            value: v.value,
            enabled: v.enabled,
            secret_key: v.secret_key.filter(|s| !s.is_empty()),
            kind: v.kind,
        }
    }
}

impl From<NameValue> for core::NameValue {
    fn from(v: NameValue) -> Self {
        Self {
            name: v.name,
            value: v.value,
            enabled: v.enabled,
            secret_key: v.secret_key.filter(|s| !s.is_empty()),
            kind: if v.kind.is_empty() {
                "plain".into()
            } else {
                v.kind
            },
        }
    }
}

fn nv_in(v: Vec<NameValue>) -> Vec<core::NameValue> {
    v.into_iter().map(Into::into).collect()
}

fn nv_out(v: Vec<core::NameValue>) -> Vec<NameValue> {
    v.into_iter().map(Into::into).collect()
}

impl From<core::Preflight> for Preflight {
    fn from(p: core::Preflight) -> Self {
        Self {
            enabled: p.enabled,
            method: p.method,
            url: p.url,
            body: p.body,
            body_type: p.body_type,
            body_params: nv_out(p.body_params),
            headers: nv_out(p.headers),
            cache_token: p.cache_token,
            cache_duration: p.cache_duration,
            cache_duration_key: p.cache_duration_key,
            cache_duration_unit: p.cache_duration_unit,
            token_key: p.token_key,
            token_header: p.token_header,
        }
    }
}

impl From<Preflight> for core::Preflight {
    fn from(p: Preflight) -> Self {
        Self {
            enabled: p.enabled,
            method: p.method,
            url: p.url,
            body: p.body,
            body_type: p.body_type,
            body_params: nv_in(p.body_params),
            headers: nv_in(p.headers),
            cache_token: p.cache_token,
            cache_duration: p.cache_duration,
            cache_duration_key: p.cache_duration_key,
            cache_duration_unit: p.cache_duration_unit,
            token_key: p.token_key,
            token_header: p.token_header,
        }
    }
}

impl From<core::Body> for RequestBody {
    fn from(b: core::Body) -> Self {
        match b {
            core::Body::None => RequestBody::Empty,
            core::Body::Raw {
                content,
                content_type,
            } => RequestBody::Raw {
                content,
                content_type,
            },
            core::Body::Json { content } => RequestBody::Json { content },
            core::Body::Form { fields } => RequestBody::Form {
                fields: nv_out(fields),
            },
            core::Body::Multipart { fields } => RequestBody::Multipart {
                fields: nv_out(fields),
            },
            core::Body::Binary { path, content_type } => RequestBody::Binary { path, content_type },
        }
    }
}

impl From<RequestBody> for core::Body {
    fn from(b: RequestBody) -> Self {
        match b {
            RequestBody::Empty => core::Body::None,
            RequestBody::Raw {
                content,
                content_type,
            } => core::Body::Raw {
                content,
                content_type: content_type.filter(|c| !c.trim().is_empty()),
            },
            RequestBody::Json { content } => core::Body::Json { content },
            RequestBody::Form { fields } => core::Body::Form {
                fields: nv_in(fields),
            },
            RequestBody::Multipart { fields } => core::Body::Multipart {
                fields: nv_in(fields),
            },
            RequestBody::Binary { path, content_type } => core::Body::Binary {
                path,
                content_type: content_type.filter(|c| !c.trim().is_empty()),
            },
        }
    }
}

impl From<core::RequestOptions> for RequestOptions {
    fn from(o: core::RequestOptions) -> Self {
        Self {
            timeout_ms: o.timeout_ms,
            follow_redirects: o.follow_redirects,
            verify_tls: o.verify_tls,
            proxy: o.proxy,
        }
    }
}

impl From<RequestOptions> for core::RequestOptions {
    fn from(o: RequestOptions) -> Self {
        Self {
            timeout_ms: o.timeout_ms.filter(|ms| *ms > 0),
            follow_redirects: o.follow_redirects,
            verify_tls: o.verify_tls,
            proxy: o.proxy,
        }
    }
}

impl From<core::RequestSpec> for RequestSpec {
    fn from(s: core::RequestSpec) -> Self {
        Self {
            method: s.method,
            url: s.url,
            authenticated: s.authenticated,
            params: nv_out(s.params),
            path_params: nv_out(s.path_params),
            headers: nv_out(s.headers),
            body: s.body.into(),
            options: s.options.into(),
            preflight: s.preflight.map(Into::into),
        }
    }
}

impl From<RequestSpec> for core::RequestSpec {
    fn from(s: RequestSpec) -> Self {
        Self {
            method: s.method,
            url: s.url,
            authenticated: s.authenticated,
            params: nv_in(s.params),
            path_params: nv_in(s.path_params),
            headers: nv_in(s.headers),
            body: s.body.into(),
            options: s.options.into(),
            preflight: s.preflight.map(Into::into),
        }
    }
}

impl From<core::Endpoint> for Endpoint {
    fn from(e: core::Endpoint) -> Self {
        let spec = e.spec().into();
        Self {
            id: e.id,
            name: e.name,
            spec,
            last_version: e.last_version,
            last_updated: e.metadata.last_updated,
        }
    }
}

impl From<core::ScratchRequest> for ScratchRequest {
    fn from(s: core::ScratchRequest) -> Self {
        Self {
            id: s.id,
            name: s.name,
            spec: s.spec.into(),
            updated_at: s.updated_at,
        }
    }
}

impl From<core::EndpointVersion> for EndpointVersion {
    fn from(v: core::EndpointVersion) -> Self {
        Self {
            version: v.version,
            last_updated: v.last_updated,
            spec: v.config.spec().into(),
        }
    }
}

impl From<core::Environment> for EnvironmentConfig {
    fn from(e: core::Environment) -> Self {
        Self {
            name: e.name,
            is_unsafe: e.is_unsafe,
            variables: nv_out(e.variables),
        }
    }
}

impl From<EnvironmentConfig> for core::Environment {
    fn from(e: EnvironmentConfig) -> Self {
        Self {
            name: e.name,
            is_unsafe: e.is_unsafe,
            variables: nv_in(e.variables),
        }
    }
}

impl From<core::ServiceSettings> for ServiceSettings {
    fn from(s: core::ServiceSettings) -> Self {
        Self {
            name: s.name,
            is_authenticated: s.is_authenticated,
            auth_type: s.auth_type.as_str().to_string(),
            basic_user: s.basic_user,
            basic_pass: s.basic_pass,
            bearer_token: s.bearer_token,
            api_key_name: s.api_key_name,
            api_key_value: s.api_key_value,
            api_key_location: s.api_key_location,
            preflight: s.preflight.map(Into::into),
        }
    }
}

impl TryFrom<ServiceSettings> for core::ServiceSettings {
    type Error = XRestError;

    fn try_from(s: ServiceSettings) -> Result<Self> {
        let auth_type = match s.auth_type.trim().to_ascii_lowercase().as_str() {
            "" | "none" => core::AuthKind::None,
            "basic" => core::AuthKind::Basic,
            "bearer" => core::AuthKind::Bearer,
            "apikey" => core::AuthKind::Apikey,
            other => {
                return Err(XRestError::InvalidInput {
                    message: format!("unknown auth type {other:?}"),
                })
            }
        };
        Ok(Self {
            name: s.name,
            is_authenticated: s.is_authenticated,
            auth_type,
            basic_user: s.basic_user,
            basic_pass: s.basic_pass,
            bearer_token: s.bearer_token,
            api_key_name: s.api_key_name,
            api_key_value: s.api_key_value,
            api_key_location: s.api_key_location,
            preflight: s.preflight.map(Into::into),
        })
    }
}

impl From<core::HttpResponse> for HttpResponse {
    fn from(r: core::HttpResponse) -> Self {
        Self {
            status: r.status,
            status_text: r.status_text,
            headers: r
                .headers
                .into_iter()
                .map(|(name, value)| HeaderPair { name, value })
                .collect(),
            content_type: r.content_type,
            body: r.body,
            body_is_base64: r.body_is_base64,
            pretty_body: r.pretty_body,
            size_bytes: r.size_bytes,
            time_ms: r.time_ms,
            final_url: r.final_url,
            environment: r.environment,
            unresolved_variables: r.unresolved_variables,
        }
    }
}

// ---- secrets ------------------------------------------------------------------------------

/// Implemented by the app (e.g. over the Keychain) to supply secret values for
/// `{{secret.KEY}}` placeholders and `secretKey` variables. Returns `None` when the key is
/// unknown. Called synchronously from the sending thread, so it should be quick.
#[uniffi::export(with_foreign)]
pub trait SecretProvider: Send + Sync {
    fn get(&self, key: String) -> Option<String>;
}

/// Looks keys up in the foreign provider first, then in the in-memory store.
struct SecretChain {
    provider: Mutex<Option<Arc<dyn SecretProvider>>>,
    memory: core::MemorySecretStore,
}

impl core::SecretStore for SecretChain {
    fn get(&self, key: &str) -> Option<String> {
        // Clone the Arc so the foreign call runs without holding our lock.
        let provider = self
            .provider
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        provider
            .and_then(|p| p.get(key.to_string()))
            .or_else(|| self.memory.get(key))
    }
}

// ---- the workspace object -----------------------------------------------------------------

/// One open workspace (a service directory with its endpoints and scratchpad).
///
/// Thread-safe: methods may be called from any thread. Synchronous methods only touch local
/// files; [`XRestWorkspace::send`] is async and does not hold the internal lock while waiting
/// on the network.
#[derive(uniffi::Object)]
pub struct XRestWorkspace {
    inner: Mutex<core::Workspace>,
    secrets: Arc<SecretChain>,
}

impl XRestWorkspace {
    fn wrap(mut ws: core::Workspace) -> Arc<Self> {
        let secrets = Arc::new(SecretChain {
            provider: Mutex::new(None),
            memory: core::MemorySecretStore::default(),
        });
        ws.set_secret_store(secrets.clone());
        Arc::new(Self {
            inner: Mutex::new(ws),
            secrets,
        })
    }

    fn lock(&self) -> MutexGuard<'_, core::Workspace> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[uniffi::export(async_runtime = "tokio")]
impl XRestWorkspace {
    /// Open an existing workspace directory (must contain `service.yaml`).
    #[uniffi::constructor]
    pub fn open_workspace(path: String) -> Result<Arc<Self>> {
        Ok(Self::wrap(core::Workspace::open(path)?))
    }

    /// Initialise a new workspace in `path` and open it.
    #[uniffi::constructor]
    pub fn create_workspace(path: String, name: String) -> Result<Arc<Self>> {
        Ok(Self::wrap(core::Workspace::create(path, &name)?))
    }

    pub fn service(&self) -> ServiceInfo {
        let ws = self.lock();
        let s = ws.service();
        let c = ws.service_settings();
        ServiceInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            root_path: ws.root().to_string_lossy().into_owned(),
            is_authenticated: s.is_authenticated,
            auth_kind: s.effective_auth_kind().as_str().to_string(),
            format_version: ws.format_version(),
            basic_user: c.basic_user,
            basic_pass: c.basic_pass,
            bearer_token: c.bearer_token,
            api_key_name: c.api_key_name,
            api_key_value: c.api_key_value,
            api_key_location: c.api_key_location,
            preflight: c.preflight.map(Into::into),
        }
    }

    /// Save the service name, auth and preflight. Keeps the id and git URL; clears the token
    /// cache. Fails with `InvalidInput` for an empty name or unknown auth type.
    pub fn update_service_settings(&self, settings: ServiceSettings) -> Result<()> {
        Ok(self.lock().update_service_settings(settings.try_into()?)?)
    }

    /// Non-fatal problems found while loading the workspace.
    pub fn warnings(&self) -> Vec<String> {
        self.lock().warnings().to_vec()
    }

    // environments

    pub fn environments(&self) -> Vec<EnvironmentConfig> {
        self.lock()
            .environments()
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }

    /// Replace all environments. Names are trimmed, non-empty and unique (else `InvalidInput`).
    /// The selection is kept if its name still exists, else it moves to the first environment.
    pub fn save_environments(&self, envs: Vec<EnvironmentConfig>) -> Result<()> {
        Ok(self
            .lock()
            .save_environments(envs.into_iter().map(Into::into).collect())?)
    }

    /// Enabled variable names of the selected environment, for autocomplete.
    pub fn variable_names(&self) -> Vec<String> {
        self.lock().variable_names()
    }

    /// Resolve `text` in the selected environment for display. Secrets (looked up through the
    /// provider, then the memory store) are masked as `••••••`; their values are never returned.
    pub fn preview(&self, text: String) -> InterpolationPreview {
        let p = self.lock().preview(&text);
        InterpolationPreview {
            resolved: p.resolved,
            unresolved: p.unresolved,
        }
    }

    pub fn selected_environment(&self) -> Option<String> {
        self.lock().selected_environment().map(|e| e.name.clone())
    }

    pub fn select_environment(&self, name: String) -> Result<()> {
        Ok(self.lock().select_environment(&name)?)
    }

    /// Whether a send in the current environment needs `confirmed = true`.
    pub fn send_policy(&self) -> SendPolicy {
        let p = self.lock().send_policy();
        SendPolicy {
            environment: p.environment,
            requires_confirmation: p.requires_confirmation,
        }
    }

    // endpoints

    pub fn list_endpoints(&self) -> Vec<Endpoint> {
        self.lock()
            .endpoints()
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }

    pub fn get_endpoint(&self, id: String) -> Result<Endpoint> {
        Ok(self.lock().endpoint(&id)?.clone().into())
    }

    pub fn create_endpoint(&self, name: String, spec: RequestSpec) -> Result<Endpoint> {
        Ok(self.lock().create_endpoint(&name, spec.into())?.into())
    }

    /// Save an endpoint. Records a new version when the request changed.
    pub fn update_endpoint(&self, id: String, name: String, spec: RequestSpec) -> Result<Endpoint> {
        Ok(self
            .lock()
            .update_endpoint(&id, Some(&name), spec.into())?
            .into())
    }

    pub fn delete_endpoint(&self, id: String) -> Result<()> {
        Ok(self.lock().delete_endpoint(&id)?)
    }

    /// Version history, newest first.
    pub fn endpoint_versions(&self, id: String, limit: u32) -> Result<Vec<EndpointVersion>> {
        Ok(self
            .lock()
            .endpoint_versions(&id, limit as usize)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    // scratchpad

    pub fn list_scratch(&self) -> Vec<ScratchRequest> {
        self.lock()
            .scratch_requests()
            .iter()
            .cloned()
            .map(Into::into)
            .collect()
    }

    pub fn get_scratch(&self, id: String) -> Result<ScratchRequest> {
        Ok(self.lock().scratch(&id)?.clone().into())
    }

    pub fn create_scratch(&self, name: String, spec: RequestSpec) -> Result<ScratchRequest> {
        Ok(self.lock().create_scratch(&name, spec.into())?.into())
    }

    pub fn update_scratch(
        &self,
        id: String,
        name: String,
        spec: RequestSpec,
    ) -> Result<ScratchRequest> {
        Ok(self
            .lock()
            .update_scratch(&id, Some(&name), spec.into())?
            .into())
    }

    pub fn delete_scratch(&self, id: String) -> Result<()> {
        Ok(self.lock().delete_scratch(&id)?)
    }

    /// Move a scratch request into the versioned endpoints (the scratch file is removed).
    pub fn promote_scratch(&self, id: String, name: Option<String>) -> Result<Endpoint> {
        Ok(self.lock().promote_scratch(&id, name.as_deref())?.into())
    }

    /// Parse a cURL command into a new scratch request.
    pub fn import_curl_as_scratch(&self, command: String) -> Result<ScratchRequest> {
        Ok(self.lock().import_curl_as_scratch(&command)?.into())
    }

    // secrets and caches

    /// Store a secret in memory for `{{secret.KEY}}` placeholders (tests and fallback). The
    /// provider set with [`XRestWorkspace::set_secret_provider`] is consulted first.
    pub fn set_secret(&self, key: String, value: String) {
        self.secrets.memory.set(key, value);
    }

    /// Plug in the platform credential store. Lookup order: provider, then `set_secret`.
    pub fn set_secret_provider(&self, provider: Arc<dyn SecretProvider>) {
        *self
            .secrets
            .provider
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(provider);
    }

    /// Every distinct secret key referenced by the service, environments, endpoints and scratch
    /// requests, sorted.
    pub fn secret_keys_in_use(&self) -> Vec<String> {
        self.lock().secret_keys_in_use()
    }

    pub fn clear_token_cache(&self) {
        self.lock().clear_token_cache();
    }

    // sending

    /// Resolve and send `request` in the selected environment.
    ///
    /// Fails with [`XRestError::ConfirmationRequired`] if the environment is unsafe and
    /// `confirmed` is false. A non-2xx status is a normal response, not an error.
    pub async fn send(&self, request: RequestSpec, confirmed: bool) -> Result<HttpResponse> {
        let prepared = {
            let ws = self.lock();
            ws.prepare_send(&request.into(), confirmed)?
        };
        Ok(prepared.execute().await?.into())
    }

    /// Run a preflight request once (`None`: the service preflight) and report the exchange and
    /// token. Uses the selected environment, never touches the token cache, and applies the same
    /// unsafe-environment confirmation as [`XRestWorkspace::send`].
    pub async fn test_preflight(
        &self,
        preflight: Option<Preflight>,
        confirmed: bool,
    ) -> Result<PreflightTestResult> {
        let prepared = {
            let ws = self.lock();
            ws.prepare_test_preflight(preflight.map(Into::into), confirmed)?
        };
        Ok(prepared.execute().await?.into())
    }
}
