//! What a request is: method, URL, params, headers, a typed body, options and preflight.

use serde::{Deserialize, Deserializer, Serialize};

use super::yes;

fn plain() -> String {
    "plain".to_string()
}

/// A name/value row used for params, headers, variables, form fields and preflight form bodies.
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
    /// `plain` (default). In a multipart body, `file` means `value` is a file path.
    #[serde(rename = "type", default = "plain")]
    pub kind: String,
}

impl NameValue {
    pub const KIND_PLAIN: &'static str = "plain";
    pub const KIND_FILE: &'static str = "file";

    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            enabled: true,
            secret_key: None,
            kind: plain(),
        }
    }

    /// A multipart file field: `value` is a path (relative to the workspace root).
    pub fn file(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            kind: Self::KIND_FILE.into(),
            ..Self::new(name, path)
        }
    }

    pub fn is_file(&self) -> bool {
        self.kind.eq_ignore_ascii_case(Self::KIND_FILE)
    }

    /// The secret key if one is set and non-empty (the Go app wrote empty strings).
    pub fn secret(&self) -> Option<&str> {
        self.secret_key.as_deref().filter(|s| !s.is_empty())
    }
}

/// A typed request body.
///
/// On disk this is a map tagged by `type`:
///
/// ```yaml
/// body: { type: json, content: '{"a": 1}' }
/// body: { type: raw, content: '<x/>', contentType: application/xml }
/// body: { type: form, fields: [{ name: q, value: rust }] }          # x-www-form-urlencoded
/// body: { type: multipart, fields: [{ name: f, value: data/a.png, type: file }] }
/// body: { type: binary, path: data/blob.bin, contentType: image/png }
/// ```
///
/// A plain string (format version 0) is read as `raw` with no explicit content type. File
/// paths are relative to the workspace root and must stay inside it.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Body {
    #[default]
    None,
    /// Text sent as-is. Without `content_type` the type is sniffed (JSON-looking → JSON,
    /// else `text/plain`), matching the legacy behaviour.
    Raw {
        #[serde(default)]
        content: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
    /// Text sent as `application/json`.
    Json {
        #[serde(default)]
        content: String,
    },
    /// `application/x-www-form-urlencoded` fields.
    Form {
        #[serde(default)]
        fields: Vec<NameValue>,
    },
    /// `multipart/form-data`; fields with `type: file` upload the file at `value`.
    Multipart {
        #[serde(default)]
        fields: Vec<NameValue>,
    },
    /// The bytes of one file.
    Binary {
        #[serde(default)]
        path: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        content_type: Option<String>,
    },
}

impl Body {
    pub fn is_none(&self) -> bool {
        matches!(self, Body::None)
    }

    /// Body for a legacy (format 0) string: empty → none, else raw text.
    pub fn from_text(content: impl Into<String>) -> Self {
        let content = content.into();
        if content.is_empty() {
            Body::None
        } else {
            Body::Raw {
                content,
                content_type: None,
            }
        }
    }
}

/// Deserialize a body written either as a typed map (format 1) or a plain string (format 0).
pub(crate) fn de_body<'de, D: Deserializer<'de>>(d: D) -> Result<Body, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Compat {
        Legacy(String),
        Typed(Body),
    }
    Ok(match Option::<Compat>::deserialize(d)? {
        None => Body::None,
        Some(Compat::Legacy(s)) => Body::from_text(s),
        Some(Compat::Typed(b)) => b,
    })
}

/// Per-request transport options. Every field is optional: unset means "use the app default"
/// ([`crate::HttpOptions`]).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestOptions {
    /// Total request timeout in milliseconds; `0` means the default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,
    /// `false` accepts invalid TLS certificates.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify_tls: Option<bool>,
    /// Proxy URL (variables allowed). An empty string forces a direct connection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
}

impl RequestOptions {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
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
    /// Values for `{name}` tokens in the URL path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_params: Vec<NameValue>,
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
}

impl Default for RequestSpec {
    fn default() -> Self {
        Self {
            method: "GET".into(),
            url: String::new(),
            authenticated: false,
            params: vec![],
            path_params: vec![],
            headers: vec![],
            body: Body::None,
            options: RequestOptions::default(),
            preflight: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(yaml: &str) -> RequestSpec {
        serde_yaml_ng::from_str(yaml).unwrap()
    }

    #[test]
    fn legacy_string_body_reads_as_raw() {
        let s = spec("method: POST\nurl: http://x\nbody: '{\"a\":1}'\n");
        assert_eq!(s.body, Body::from_text(r#"{"a":1}"#));
        assert_eq!(spec("url: http://x\nbody: ''\n").body, Body::None);
        assert_eq!(spec("url: http://x\nbody:\n").body, Body::None);
        assert_eq!(spec("url: http://x\n").body, Body::None);
    }

    #[test]
    fn typed_bodies_round_trip() {
        let bodies = [
            Body::Json {
                content: "{}".into(),
            },
            Body::Raw {
                content: "<x/>".into(),
                content_type: Some("application/xml".into()),
            },
            Body::Form {
                fields: vec![NameValue::new("q", "rust")],
            },
            Body::Multipart {
                fields: vec![NameValue::new("a", "1"), NameValue::file("f", "data/a.png")],
            },
            Body::Binary {
                path: "data/blob.bin".into(),
                content_type: None,
            },
        ];
        for body in bodies {
            let s = RequestSpec {
                url: "http://x".into(),
                body: body.clone(),
                ..Default::default()
            };
            let yaml = serde_yaml_ng::to_string(&s).unwrap();
            assert_eq!(spec(&yaml).body, body, "{yaml}");
            let json = serde_json::to_string(&s).unwrap();
            let back: RequestSpec = serde_json::from_str(&json).unwrap();
            assert_eq!(back.body, body, "{json}");
        }
    }

    #[test]
    fn on_disk_shape_is_tagged_camel_case() {
        let s = RequestSpec {
            url: "http://x".into(),
            body: Body::Binary {
                path: "a.bin".into(),
                content_type: Some("image/png".into()),
            },
            options: RequestOptions {
                timeout_ms: Some(5000),
                ..Default::default()
            },
            ..Default::default()
        };
        let yaml = serde_yaml_ng::to_string(&s).unwrap();
        assert!(yaml.contains("type: binary"), "{yaml}");
        assert!(yaml.contains("contentType: image/png"), "{yaml}");
        assert!(yaml.contains("timeoutMs: 5000"), "{yaml}");
    }

    #[test]
    fn empty_body_and_options_are_omitted() {
        let yaml = serde_yaml_ng::to_string(&RequestSpec {
            url: "http://x".into(),
            ..Default::default()
        })
        .unwrap();
        assert!(!yaml.contains("body"), "{yaml}");
        assert!(!yaml.contains("options"), "{yaml}");
    }

    #[test]
    fn path_params_round_trip_and_are_omitted_when_empty() {
        let s = RequestSpec {
            url: "http://x/{id}".into(),
            path_params: vec![NameValue::new("id", "7")],
            ..Default::default()
        };
        let yaml = serde_yaml_ng::to_string(&s).unwrap();
        assert!(yaml.contains("pathParams"), "{yaml}");
        assert_eq!(spec(&yaml).path_params, s.path_params);
        assert!(spec("url: http://x\n").path_params.is_empty());
        let plain = serde_yaml_ng::to_string(&RequestSpec::default()).unwrap();
        assert!(!plain.contains("pathParams"), "{plain}");
    }

    #[test]
    fn unknown_body_type_is_an_error() {
        let r: Result<RequestSpec, _> =
            serde_yaml_ng::from_str("url: http://x\nbody: { type: graphql }\n");
        assert!(r.is_err());
    }
}
