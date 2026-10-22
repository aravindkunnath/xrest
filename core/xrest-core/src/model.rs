//! Domain model and serde mapping for the on-disk YAML format.
//!
//! Field names are `camelCase` on disk to stay compatible with the files written by the
//! legacy Go/Wails app (see `docs/architecture.md` for the exact differences).

use serde::{Deserialize, Serialize};

fn yes() -> bool {
    true
}

fn plain() -> String {
    "plain".to_string()
}

fn is_zero(v: &i32) -> bool {
    *v == 0
}

/// A name/value row used for params, headers, variables and preflight form bodies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameValue {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// For variables: when set the value is looked up in the secret store under this key.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_key: Option<String>,
    #[serde(rename = "type", default = "plain")]
    pub kind: String,
}

impl NameValue {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            enabled: true,
            secret_key: None,
            kind: plain(),
        }
    }

    /// The secret key if one is set and non-empty (the Go app wrote empty strings).
    pub fn secret(&self) -> Option<&str> {
        self.secret_key.as_deref().filter(|s| !s.is_empty())
    }
}

/// Service-level authentication type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthKind {
    Basic,
    Bearer,
    Apikey,
    /// `none`, empty, or any unknown value.
    #[default]
    #[serde(other)]
    None,
}

/// Service-level auth configuration (`auth:` in service.yaml).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConfig {
    #[serde(rename = "type", default)]
    pub kind: AuthKind,
    /// Written as `true` by the Go app and never read by it; the core ignores it.
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub basic_user: String,
    #[serde(default)]
    pub basic_pass: String,
    #[serde(default)]
    pub bearer_token: String,
    #[serde(default)]
    pub api_key_name: String,
    #[serde(default)]
    pub api_key_value: String,
    /// `"header"` (default) or `"query"`.
    #[serde(default)]
    pub api_key_location: String,
}

/// Preflight auth request: acquire a token, cache it, inject it into the real request.
///
/// This is the flat "storage" shape used by the Go app in `service.yaml` and endpoint files.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preflight {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub body: String,
    /// `"raw"` (default) or `"urlencoded"` (uses `body_params`).
    #[serde(default)]
    pub body_type: String,
    #[serde(default)]
    pub body_params: Vec<NameValue>,
    #[serde(default)]
    pub headers: Vec<NameValue>,
    #[serde(default)]
    pub cache_token: bool,
    /// `"derived"` (take the lifetime from `cache_duration_key`) or a fixed number in
    /// `cache_duration_unit`s.
    #[serde(default)]
    pub cache_duration: String,
    #[serde(default)]
    pub cache_duration_key: String,
    /// `seconds` (default), `minutes`, `hours` or `days`.
    #[serde(default)]
    pub cache_duration_unit: String,
    /// Dot path of the token in the JSON response body (default `access_token`).
    #[serde(default)]
    pub token_key: String,
    /// Header to write the token to (default `Authorization`).
    #[serde(default)]
    pub token_header: Option<String>,
}

/// A secret-free, environment-independent description of one HTTP request.
/// Shared by endpoints, scratch requests and version snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestSpec {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub url: String,
    /// Whether service-level auth is injected into this request.
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default)]
    pub params: Vec<NameValue>,
    #[serde(default)]
    pub headers: Vec<NameValue>,
    #[serde(default)]
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight: Option<Preflight>,
}

impl Default for RequestSpec {
    fn default() -> Self {
        Self {
            method: "GET".into(),
            url: String::new(),
            authenticated: false,
            params: vec![],
            headers: vec![],
            body: String::new(),
            preflight: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointMetadata {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub last_updated: u64,
}

impl Default for EndpointMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".into(),
            last_updated: 0,
        }
    }
}

/// A versioned endpoint: `endpoints/<id>.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub id: String,
    #[serde(default)]
    pub service_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default)]
    pub auth_type: String,
    #[serde(default)]
    pub metadata: EndpointMetadata,
    #[serde(default)]
    pub params: Vec<NameValue>,
    #[serde(default)]
    pub headers: Vec<NameValue>,
    #[serde(default)]
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight: Option<Preflight>,
    /// Live save counter; the history itself lives in the [`crate::versions::VersionStore`].
    #[serde(default, skip_serializing_if = "is_zero")]
    pub last_version: i32,
}

impl Endpoint {
    pub fn spec(&self) -> RequestSpec {
        RequestSpec {
            method: self.method.clone(),
            url: self.url.clone(),
            authenticated: self.authenticated,
            params: self.params.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
            preflight: self.preflight.clone(),
        }
    }

    pub fn apply_spec(&mut self, spec: RequestSpec) {
        self.method = spec.method;
        self.url = spec.url;
        self.authenticated = spec.authenticated;
        self.params = spec.params;
        self.headers = spec.headers;
        self.body = spec.body;
        self.preflight = spec.preflight;
    }
}

/// A loose, unversioned request: `scratchpad/<id>.yaml` (gitignored by default).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScratchRequest {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(flatten)]
    pub spec: RequestSpec,
    /// Unix seconds.
    #[serde(default)]
    pub created_at: u64,
    /// Unix seconds.
    #[serde(default)]
    pub updated_at: u64,
}

/// An environment: named variable set plus the unsafe guardrail flag.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub name: String,
    #[serde(default)]
    pub is_unsafe: bool,
    #[serde(default)]
    pub variables: Vec<NameValue>,
}

/// Legacy endpoint index entry in `service.yaml`. Read for compatibility, never written.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EndpointStub {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
}

/// `service.yaml`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceFile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_authenticated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<AuthKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight: Option<Preflight>,
    /// Legacy; read-only.
    #[serde(default, skip_serializing)]
    pub endpoints: Vec<EndpointStub>,
    /// Legacy absolute path of the service; read-only (the workspace root is authoritative).
    #[serde(default, skip_serializing)]
    pub directory: Option<String>,
    /// Legacy; now stored in `.xrest/state.yaml`. Used as a fallback when reading.
    #[serde(default, skip_serializing)]
    pub selected_environment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_url: Option<String>,
}

impl ServiceFile {
    /// Effective auth kind, preferring the detailed `auth` block.
    pub fn effective_auth_kind(&self) -> AuthKind {
        match (&self.auth, self.auth_type) {
            (Some(a), _) if a.kind != AuthKind::None => a.kind,
            (_, Some(k)) => k,
            _ => AuthKind::None,
        }
    }
}

/// Per-user, per-workspace state (`.xrest/state.yaml`, gitignored).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_environment: Option<String>,
}

/// Snapshot of an endpoint's request config stored per version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestConfig {
    pub method: String,
    pub url: String,
    pub authenticated: bool,
    pub auth_type: String,
    pub params: Vec<NameValue>,
    pub headers: Vec<NameValue>,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflight: Option<Preflight>,
}

impl RequestConfig {
    pub fn from_endpoint(ep: &Endpoint) -> Self {
        Self {
            method: ep.method.clone(),
            url: ep.url.clone(),
            authenticated: ep.authenticated,
            auth_type: ep.auth_type.clone(),
            params: ep.params.clone(),
            headers: ep.headers.clone(),
            body: ep.body.clone(),
            preflight: ep.preflight.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointVersion {
    pub version: i32,
    pub config: RequestConfig,
    pub last_updated: u64,
}

/// Result of a send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub content_type: String,
    /// UTF-8 text, or base64 when `body_is_base64`.
    pub body: String,
    pub body_is_base64: bool,
    /// Re-indented JSON (key order and number formatting preserved) when the body is JSON.
    pub pretty_body: Option<String>,
    pub size_bytes: u64,
    pub time_ms: u64,
    pub final_url: String,
    /// Environment the request was resolved against.
    pub environment: Option<String>,
    /// `{{VAR}}` placeholders that had no value and were sent verbatim.
    pub unresolved_variables: Vec<String>,
}
