//! The workspace's service: `service.yaml`, auth, environments and per-user state.

use serde::{Deserialize, Serialize};

use super::request::{NameValue, Preflight};

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
    /// On-disk format of the whole workspace; `0` (absent) for legacy Go workspaces.
    /// See [`crate::FORMAT_VERSION`].
    #[serde(default)]
    pub format_version: u32,
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
