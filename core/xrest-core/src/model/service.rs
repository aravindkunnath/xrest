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

/// An environment: named variable set plus the unsafe guardrail flag and its rules.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub name: String,
    #[serde(default)]
    pub is_unsafe: bool,
    #[serde(default)]
    pub variables: Vec<NameValue>,
    /// Display color, one of [`PALETTE`]. Each platform maps the name to its own shade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Which sends to an unsafe environment need confirmation.
    #[serde(default, skip_serializing_if = "ConfirmScope::is_every")]
    pub confirm: ConfirmScope,
    /// Team minimum for the confirmation countdown, in seconds (0 = none). The app uses the
    /// longer of this and the user's own setting. At most [`MAX_MIN_COUNTDOWN_SECONDS`].
    #[serde(default, skip_serializing_if = "is_zero")]
    pub min_countdown_seconds: u32,
}

/// Upper bound for [`Environment::min_countdown_seconds`].
pub const MAX_MIN_COUNTDOWN_SECONDS: u32 = 30;

/// Color names for environments and workspaces. Names, not hex, so each platform can pick
/// shades that work in light and dark mode.
pub const PALETTE: &[&str] = &[
    "red", "orange", "amber", "green", "teal", "blue", "purple", "pink", "graphite",
];

/// Workspace icon names (platforms map them to their own symbols).
pub const WORKSPACE_ICONS: &[&str] = &["card", "server", "person", "box", "globe", "bolt"];

fn is_zero(v: &u32) -> bool {
    *v == 0
}

/// Which requests to an unsafe environment ask for confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfirmScope {
    /// Every request.
    #[default]
    Every,
    /// Only requests that may change data: anything but `GET`, `HEAD` and `OPTIONS`.
    Changes,
}

impl ConfirmScope {
    fn is_every(&self) -> bool {
        *self == ConfirmScope::Every
    }

    /// Whether a request with `method` needs confirmation under this scope.
    pub fn applies_to(self, method: &str) -> bool {
        match self {
            ConfirmScope::Every => true,
            ConfirmScope::Changes => !matches!(
                method.trim().to_ascii_uppercase().as_str(),
                "GET" | "HEAD" | "OPTIONS"
            ),
        }
    }
}

/// Workspace color and icon (`appearance:` in service.yaml).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceAppearance {
    /// One of [`PALETTE`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// One of [`WORKSPACE_ICONS`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl WorkspaceAppearance {
    pub fn is_empty(&self) -> bool {
        self.color.is_none() && self.icon.is_none()
    }
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
    #[serde(default, skip_serializing_if = "WorkspaceAppearance::is_empty")]
    pub appearance: WorkspaceAppearance,
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

/// The editable part of `service.yaml` (everything except `id`, `gitUrl` and the format version).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ServiceSettings {
    pub name: String,
    pub is_authenticated: bool,
    pub auth_type: AuthKind,
    pub basic_user: String,
    pub basic_pass: String,
    pub bearer_token: String,
    pub api_key_name: String,
    pub api_key_value: String,
    pub api_key_location: String,
    pub preflight: Option<Preflight>,
}

impl ServiceSettings {
    pub fn from_service(s: &ServiceFile) -> Self {
        let auth = s.auth.clone().unwrap_or_default();
        Self {
            name: s.name.clone(),
            is_authenticated: s.is_authenticated,
            auth_type: s.effective_auth_kind(),
            basic_user: auth.basic_user,
            basic_pass: auth.basic_pass,
            bearer_token: auth.bearer_token,
            api_key_name: auth.api_key_name,
            api_key_value: auth.api_key_value,
            api_key_location: auth.api_key_location,
            preflight: s.preflight.clone(),
        }
    }
}

/// Per-user, per-workspace state (`.xrest/state.yaml`, gitignored).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_environment: Option<String>,
    /// Whether saves commit to git by themselves. Per user; defaults to manual.
    #[serde(default, skip_serializing_if = "GitMode::is_manual")]
    pub git_mode: GitMode,
    /// Unix time of the last successful pull/push/sync.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sync: Option<u64>,
}

/// `auto`: every successful save commits the saved file(s). `manual`: only explicit commits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GitMode {
    Auto,
    #[default]
    Manual,
}

impl GitMode {
    fn is_manual(&self) -> bool {
        *self == GitMode::Manual
    }
}
