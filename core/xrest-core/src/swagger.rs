//! OpenAPI 3.x / Swagger 2.0 import (JSON or YAML).

use std::time::Duration;

use crate::error::{Error, Result};
use serde_json::Value;

use crate::model::{Body, NameValue, RequestSpec};

/// Placeholder base URL used when the spec declares no server.
pub const PLACEHOLDER_BASE_URL: &str = "https://api.example.com";
/// Largest response [`fetch_text`] accepts.
pub const MAX_FETCH_BYTES: usize = 10 * 1024 * 1024;
/// Total timeout of [`fetch_text`].
pub const FETCH_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedEndpoint {
    pub name: String,
    pub method: String,
    pub path: String,
    pub spec: RequestSpec,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedSpec {
    pub title: String,
    pub base_url: Option<String>,
    pub endpoints: Vec<ImportedEndpoint>,
    /// Imported values that were dropped (see [`clean`]); surfaced as workspace warnings.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportSummary {
    pub added: u32,
    pub skipped: u32,
}

const METHODS: [&str; 7] = ["GET", "PUT", "POST", "DELETE", "OPTIONS", "HEAD", "PATCH"];
const MAX_REF_DEPTH: usize = 16;

/// Imported text must not carry XRest interpolation (`{{VAR}}`, `{{secret.KEY}}`) into the
/// workspace: a hostile spec could otherwise make a later send resolve and leak a secret. Such a
/// value is dropped (becomes empty) and a warning names the field.
fn clean(warnings: &mut Vec<String>, what: &str, value: &str) -> String {
    if value.contains("{{") {
        warnings.push(format!(
            "ignored imported value containing '{{{{' in {what}"
        ));
        String::new()
    } else {
        value.to_string()
    }
}

/// Parse an OpenAPI 3.x or Swagger 2.0 document (JSON or YAML). Anything else is
/// [`Error::InvalidInput`].
pub fn parse(content: &str) -> Result<ImportedSpec> {
    let text = content.trim_start_matches('\u{feff}').trim();
    if text.is_empty() {
        return Err(Error::InvalidInput("the API description is empty".into()));
    }
    let doc: Value = serde_json::from_str(text)
        .or_else(|_| serde_yaml_ng::from_str(text))
        .map_err(|e| Error::InvalidInput(format!("not valid JSON or YAML: {e}")))?;
    let v2 = match (doc.get("openapi"), doc.get("swagger")) {
        (Some(v), _) if version_starts(v, "3") => false,
        (_, Some(v)) if version_starts(v, "2") => true,
        _ => {
            return Err(Error::InvalidInput(
                "not an OpenAPI 3.x or Swagger 2.0 document".into(),
            ))
        }
    };
    let title = doc
        .pointer("/info/title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    let mut warnings = vec![];
    let title = clean(&mut warnings, "the API title", &title);
    let base_url = if v2 {
        v2_base_url(&doc)
    } else {
        v3_base_url(&doc)
    }
    .filter(|b| {
        let bad = b.contains("{{");
        if bad {
            warnings.push("ignored imported value containing '{{' in the base URL".into());
        }
        !bad
    });

    let mut endpoints = vec![];
    if let Some(paths) = doc.get("paths").and_then(Value::as_object) {
        for (p, item) in paths {
            let item = deref(&doc, item);
            for method in METHODS {
                let Some(op) = item.get(method.to_lowercase()).map(|o| deref(&doc, o)) else {
                    continue;
                };
                if !op.is_object() {
                    continue;
                }
                endpoints.push(build_endpoint(&mut warnings, &doc, v2, p, item, method, op));
            }
        }
    }
    Ok(ImportedSpec {
        title,
        base_url,
        endpoints,
        warnings,
    })
}

fn version_starts(v: &Value, major: &str) -> bool {
    v.as_str()
        .map(|s| s.trim().starts_with(major))
        .or_else(|| v.as_f64().map(|f| f.floor().to_string() == major))
        .unwrap_or(false)
}

fn trim_base(s: &str) -> Option<String> {
    let s = s.trim().trim_end_matches('/');
    (!s.is_empty()).then(|| s.to_string())
}

fn v2_base_url(doc: &Value) -> Option<String> {
    let host = doc.get("host")?.as_str()?.trim();
    if host.is_empty() {
        return None;
    }
    let scheme = doc
        .pointer("/schemes/0")
        .and_then(Value::as_str)
        .unwrap_or("https");
    let base = doc.get("basePath").and_then(Value::as_str).unwrap_or("");
    let base = base.trim_matches('/');
    let url = if base.is_empty() {
        format!("{scheme}://{host}")
    } else {
        format!("{scheme}://{host}/{base}")
    };
    trim_base(&url)
}

fn v3_base_url(doc: &Value) -> Option<String> {
    let server = doc.pointer("/servers/0")?;
    let mut url = server.get("url")?.as_str()?.to_string();
    if let Some(vars) = server.get("variables").and_then(Value::as_object) {
        for (name, var) in vars {
            if let Some(d) = var.get("default").map(scalar_text) {
                url = url.replace(&format!("{{{name}}}"), &d);
            }
        }
    }
    trim_base(&url)
}

/// Follow `$ref: "#/..."` chains (local refs only). Unresolvable refs return the node itself.
fn deref<'a>(doc: &'a Value, node: &'a Value) -> &'a Value {
    let mut cur = node;
    for _ in 0..MAX_REF_DEPTH {
        let Some(r) = cur.get("$ref").and_then(Value::as_str) else {
            return cur;
        };
        let Some(ptr) = r.strip_prefix('#') else {
            return cur;
        };
        match doc.pointer(ptr) {
            Some(next) => cur = next,
            None => return cur,
        }
    }
    cur
}

fn scalar_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Default, else example, of a parameter or schema property (Swagger 2 keeps them on the
/// parameter, OpenAPI 3 on its schema).
fn default_text(doc: &Value, node: &Value) -> String {
    let schema = node.get("schema").map(|s| deref(doc, s));
    let candidates = [
        node.get("default"),
        schema.and_then(|s| s.get("default")),
        node.get("example"),
        node.get("x-example"),
        schema.and_then(|s| s.get("example")),
    ];
    candidates
        .into_iter()
        .flatten()
        .find(|v| !v.is_null())
        .map(scalar_text)
        .unwrap_or_default()
}

fn build_endpoint(
    warnings: &mut Vec<String>,
    doc: &Value,
    v2: bool,
    path: &str,
    item: &Value,
    method: &str,
    op: &Value,
) -> ImportedEndpoint {
    let text = |k: &str| {
        op.get(k)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
    };
    let path = clean(warnings, &format!("path of {method} operation"), path);
    let mut name = None;
    for key in ["summary", "operationId"] {
        if let Some(t) = text(key) {
            let t = clean(warnings, &format!("{key} of {method} {path}"), t);
            if !t.is_empty() {
                name = Some(t);
                break;
            }
        }
    }
    let name = name.unwrap_or_else(|| format!("{method} {path}").trim().to_string());
    let path = path.as_str();

    // Path-item parameters first; an operation parameter with the same (in, name) overrides.
    let mut params: Vec<&Value> = vec![];
    let item_params = item.get("parameters").and_then(Value::as_array);
    let op_params = op.get("parameters").and_then(Value::as_array);
    for p in item_params.into_iter().chain(op_params).flatten() {
        let p = deref(doc, p);
        let key = (
            p.get("in").and_then(Value::as_str),
            p.get("name").and_then(Value::as_str),
        );
        if let Some(slot) = params.iter_mut().find(|q| {
            (
                q.get("in").and_then(Value::as_str),
                q.get("name").and_then(Value::as_str),
            ) == key
        }) {
            *slot = p;
        } else {
            params.push(p);
        }
    }

    let mut spec = RequestSpec {
        method: method.to_string(),
        url: format!("{{{{BASE_URL}}}}{path}"),
        ..Default::default()
    };
    let at = format!("{method} {path}");
    let mut form: Vec<NameValue> = vec![];
    let mut body = Body::None;
    for p in params {
        let Some(pname) = p
            .get("name")
            .and_then(Value::as_str)
            .filter(|n| !n.is_empty())
        else {
            continue;
        };
        if pname.contains("{{") {
            clean(warnings, &format!("parameter name of {at}"), pname);
            continue;
        }
        let mut row = || {
            NameValue::new(
                pname,
                clean(
                    warnings,
                    &format!("parameter {pname} default of {at}"),
                    &default_text(doc, p),
                ),
            )
        };
        match p.get("in").and_then(Value::as_str) {
            Some("path") => spec.path_params.push(row()),
            Some("query") => spec.params.push(row()),
            Some("header") => spec.headers.push(row()),
            Some("formData") => form.push(row()),
            Some("body") if v2 => {
                body = json_body(
                    doc,
                    p.get("schema").map(|s| deref(doc, s)),
                    p.get("x-example"),
                );
            }
            _ => {}
        }
    }
    if !form.is_empty() {
        body = Body::Form { fields: form };
    }
    if !v2 {
        if let Some(b) = op.get("requestBody").map(|b| deref(doc, b)) {
            body = v3_body(doc, b);
        }
    }
    spec.body = match body {
        Body::Json { content } => Body::Json {
            content: clean(warnings, &format!("JSON body of {at}"), &content),
        },
        Body::Form { fields } => Body::Form {
            fields: fields
                .into_iter()
                .filter_map(|f| {
                    if f.name.contains("{{") {
                        clean(warnings, &format!("form field name of {at}"), &f.name);
                        return None;
                    }
                    let value = clean(
                        warnings,
                        &format!("form field {} of {at}", f.name),
                        &f.value,
                    );
                    Some(NameValue::new(f.name, value))
                })
                .collect(),
        },
        other => other,
    };
    ImportedEndpoint {
        name,
        method: method.to_string(),
        path: path.to_string(),
        spec,
    }
}

fn json_body(doc: &Value, schema: Option<&Value>, example: Option<&Value>) -> Body {
    let ex = example.or_else(|| schema.and_then(|s| deref(doc, s).get("example")));
    match ex {
        Some(v) if !v.is_null() => Body::Json {
            content: serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string()),
        },
        _ => Body::None,
    }
}

fn v3_body(doc: &Value, body: &Value) -> Body {
    let Some(content) = body.get("content").and_then(Value::as_object) else {
        return Body::None;
    };
    let is_json = |m: &str| {
        let m = m
            .split(';')
            .next()
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        m == "application/json" || m.ends_with("+json")
    };
    if let Some((_, media)) = content.iter().find(|(m, _)| is_json(m)) {
        let schema = media.get("schema").map(|s| deref(doc, s));
        let example = media.get("example").or_else(|| {
            media
                .get("examples")
                .and_then(Value::as_object)
                .and_then(|m| m.values().next())
                .map(|e| deref(doc, e))
                .and_then(|e| e.get("value"))
        });
        return json_body(doc, schema, example);
    }
    if let Some((_, media)) = content
        .iter()
        .find(|(m, _)| m.eq_ignore_ascii_case("application/x-www-form-urlencoded"))
    {
        let schema = media.get("schema").map(|s| deref(doc, s));
        let fields: Vec<NameValue> = schema
            .and_then(|s| s.get("properties"))
            .and_then(Value::as_object)
            .map(|props| {
                props
                    .iter()
                    .map(|(k, v)| NameValue::new(k, default_text(doc, &deref(doc, v).clone())))
                    .collect()
            })
            .unwrap_or_default();
        return Body::Form { fields };
    }
    Body::None
}

/// Fetch a spec from an `http(s)` URL as text: at most [`MAX_FETCH_BYTES`], [`FETCH_TIMEOUT`]
/// total, UTF-8 only.
pub async fn fetch_text(url: &str) -> Result<String> {
    fetch_text_with(url, MAX_FETCH_BYTES, FETCH_TIMEOUT).await
}

/// [`fetch_text`] with explicit limits. The size cap is enforced while streaming, so an
/// oversized (or endless) body is cut off instead of buffered.
pub async fn fetch_text_with(url: &str, max_bytes: usize, timeout: Duration) -> Result<String> {
    let parsed = url::Url::parse(url.trim()).map_err(|e| Error::InvalidUrl {
        url: url.to_string(),
        message: e.to_string(),
    })?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(Error::InvalidUrl {
            url: url.to_string(),
            message: "only http and https URLs can be fetched".into(),
        });
    }
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .user_agent(concat!("XRest/", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|e| Error::Http(e.to_string()))?;
    let mut resp = client
        .get(parsed)
        .send()
        .await
        .map_err(|e| Error::Http(describe(&e)))?;
    if !resp.status().is_success() {
        return Err(Error::Http(format!("HTTP {}", resp.status())));
    }
    let too_big = || {
        Error::InvalidInput(format!(
            "the response is larger than {} MiB",
            (max_bytes / (1024 * 1024)).max(1)
        ))
    };
    if resp.content_length().is_some_and(|n| n > max_bytes as u64) {
        return Err(too_big());
    }
    let mut buf: Vec<u8> = vec![];
    while let Some(chunk) = resp.chunk().await.map_err(|e| Error::Http(describe(&e)))? {
        if buf.len() + chunk.len() > max_bytes {
            return Err(too_big());
        }
        buf.extend_from_slice(&chunk);
    }
    String::from_utf8(buf).map_err(|_| Error::Http("the response is not valid UTF-8 text".into()))
}

fn describe(e: &reqwest::Error) -> String {
    use std::error::Error as _;
    let mut s = e.to_string();
    let mut src = e.source();
    while let Some(c) = src {
        s.push_str(": ");
        s.push_str(&c.to_string());
        src = c.source();
    }
    s
}
