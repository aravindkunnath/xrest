//! Versioned endpoints, scratch requests and version snapshots.

use serde::{Deserialize, Serialize};

use super::request::{de_body, Body, NameValue, Preflight, RequestOptions, RequestSpec};

fn is_zero(v: &i32) -> bool {
    *v == 0
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
    #[serde(
        default,
        deserialize_with = "de_body",
        skip_serializing_if = "Body::is_none"
    )]
    pub body: Body,
    #[serde(default, skip_serializing_if = "RequestOptions::is_empty")]
    pub options: RequestOptions,
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
            options: self.options.clone(),
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
        self.options = spec.options;
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
    #[serde(
        default,
        deserialize_with = "de_body",
        skip_serializing_if = "Body::is_none"
    )]
    pub body: Body,
    #[serde(default, skip_serializing_if = "RequestOptions::is_empty")]
    pub options: RequestOptions,
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
            options: ep.options.clone(),
            preflight: ep.preflight.clone(),
        }
    }

    pub fn spec(&self) -> RequestSpec {
        RequestSpec {
            method: self.method.clone(),
            url: self.url.clone(),
            authenticated: self.authenticated,
            params: self.params.clone(),
            headers: self.headers.clone(),
            body: self.body.clone(),
            options: self.options.clone(),
            preflight: self.preflight.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_version_snapshot_json_still_loads() {
        // Shape written before typed bodies existed.
        let json = r#"{"version":3,"lastUpdated":10,"config":{"method":"POST","url":"http://x",
            "authenticated":false,"authType":"none","params":[],"headers":[],"body":"hello"}}"#;
        let v: EndpointVersion = serde_json::from_str(json).unwrap();
        assert_eq!(v.config.body, Body::from_text("hello"));
        assert!(v.config.options.is_empty());
    }

    #[test]
    fn scratch_flattens_spec_with_typed_body() {
        let yaml = "id: r-1\nname: n\nurl: http://x\nbody:\n  type: json\n  content: '{}'\n\
                    options:\n  verifyTls: false\ncreatedAt: 1\nupdatedAt: 2\n";
        let s: ScratchRequest = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(
            s.spec.body,
            Body::Json {
                content: "{}".into()
            }
        );
        assert_eq!(s.spec.options.verify_tls, Some(false));
        let legacy: ScratchRequest =
            serde_yaml_ng::from_str("id: r-2\nurl: http://x\nbody: plain text\n").unwrap();
        assert_eq!(legacy.spec.body, Body::from_text("plain text"));
    }
}
