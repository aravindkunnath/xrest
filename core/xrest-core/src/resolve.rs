//! Resolution: turn a [`RequestSpec`] into a [`PreparedSend`] for one environment.
//!
//! Pure with respect to the workspace: everything it needs arrives in a [`SendContext`]. It
//! interpolates `{{VAR}}`s, resolves `{{secret.KEY}}`s, applies service auth and preflight,
//! reads files referenced by the body, and enforces the unsafe-environment policy first.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::codegen::{ExportBody, ExportPart, ExportRequest};
use crate::error::{Error, Result};
use crate::exec::{
    build_url, has_header, set_header, set_query, HttpOptions, PreparedPreflightTest, PreparedSend,
    ResolvedAuth, ResolvedBody, ResolvedPart, ResolvedPreflight, ResolvedRequest,
};
use crate::interpolate::{
    interpolate, resolve_secrets, unresolved, InterpolationPreview, Variables, SECRET_MASK,
};
use crate::model::{AuthKind, Body, Environment, NameValue, Preflight, RequestSpec, ServiceFile};
use crate::policy::SendPolicy;
use crate::preflight::TokenCache;
use crate::secrets::SecretStore;
use crate::variables::VariableLayers;

/// Largest file a request body may read (binary bodies and multipart file fields).
pub const MAX_BODY_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// What resolution needs from a workspace.
pub(crate) struct SendContext<'a> {
    pub root: &'a Path,
    pub service: &'a ServiceFile,
    pub environment: Option<&'a Environment>,
    /// Global, workspace and local variables for `environment`.
    pub layers: &'a VariableLayers,
    pub secrets: &'a dyn SecretStore,
    pub tokens: Arc<TokenCache>,
    pub http: &'a HttpOptions,
}

/// Resolve `spec` for sending. Returns [`Error::ConfirmationRequired`] for an unsafe environment
/// unless `confirmed`; nothing touches the network until [`PreparedSend::execute`].
pub(crate) fn prepare_send(
    ctx: SendContext<'_>,
    spec: &RequestSpec,
    confirmed: bool,
) -> Result<PreparedSend> {
    let env = ctx.environment;
    SendPolicy::for_request(env, &spec.method).check(confirmed)?;

    let vars = ctx.layers.resolve(env);
    let mut r = Resolver {
        vars: &vars,
        secrets: ctx.secrets,
        unresolved: BTreeSet::new(),
        missing: BTreeSet::new(),
        dot_segments: BTreeSet::new(),
        secret_values: BTreeSet::new(),
    };

    let params = r.pairs(&spec.params);
    let headers = r.pairs(&spec.headers);
    let raw_url = r.text(&spec.url);
    let raw_url = substitute_path_params(&raw_url, &spec.path_params, &mut r);
    if let Some(name) = r.dot_segments.iter().next() {
        // `url::Url` resolves `%2E%2E` like `..`, so the request would silently go elsewhere.
        return Err(Error::InvalidInput(format!(
            "path parameter {name} cannot be '.' or '..'"
        )));
    }
    let url = build_url(&raw_url, &params)?;
    let method = normalize_method(&spec.method)?;
    let body = resolve_body(&spec.body, &mut r, ctx.root)?;

    let mut options = spec.options.clone();
    if let Some(p) = &options.proxy {
        options.proxy = Some(r.text(p));
    }
    let http = ctx.http.merged(&options);

    let use_service_auth = spec.authenticated && ctx.service.is_authenticated;
    let auth = if use_service_auth {
        resolve_auth(ctx.service, &mut r)
    } else {
        ResolvedAuth::None
    };

    let scope = format!(
        "{}|{}",
        ctx.service.id,
        env.map(|e| e.name.as_str()).unwrap_or("")
    );
    let pf_cfg = spec.preflight.as_ref().filter(|p| p.enabled).or_else(|| {
        use_service_auth
            .then_some(ctx.service.preflight.as_ref())
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
        secret_values: r.secret_values.into_iter().collect(),
        tokens: ctx.tokens,
        http,
    })
}

/// Keeps every `{{secret.KEY}}` as its placeholder, whether or not the secret exists.
struct PlaceholderStore;

impl SecretStore for PlaceholderStore {
    fn get(&self, key: &str) -> Option<String> {
        Some(format!("{{{{secret.{key}}}}}"))
    }
}

/// What export needs from a workspace (no root: nothing is read from disk).
pub(crate) struct ExportContext<'a> {
    pub service: &'a ServiceFile,
    pub environment: Option<&'a Environment>,
    pub layers: &'a VariableLayers,
    pub secrets: &'a dyn SecretStore,
}

/// The request as it would be sent, for code export. Does no I/O (no preflight request, no file
/// reads: file fields keep their path) and applies no unsafe-environment policy. Without
/// `include_secrets` every secret stays a `{{secret.KEY}}` placeholder (a missing secret is
/// fine); with it, secrets are looked up and a missing one is [`Error::MissingSecrets`].
pub(crate) fn resolve_for_export(
    ctx: ExportContext<'_>,
    spec: &RequestSpec,
    include_secrets: bool,
) -> Result<ExportRequest> {
    let env = ctx.environment;
    // Built-ins get fresh values, so the exported request is runnable as-is.
    let vars = ctx.layers.resolve(env);
    let secrets: &dyn SecretStore = if include_secrets {
        ctx.secrets
    } else {
        &PlaceholderStore
    };
    let mut r = Resolver {
        vars: &vars,
        secrets,
        unresolved: BTreeSet::new(),
        missing: BTreeSet::new(),
        dot_segments: BTreeSet::new(),
        secret_values: BTreeSet::new(),
    };

    let params = r.pairs(&spec.params);
    let mut headers = r.pairs(&spec.headers);
    let raw_url = r.text(&spec.url);
    let raw_url = substitute_path_params(&raw_url, &spec.path_params, &mut r);
    if let Some(name) = r.dot_segments.iter().next() {
        return Err(Error::InvalidInput(format!(
            "path parameter {name} cannot be '.' or '..'"
        )));
    }
    let mut url = build_url(&raw_url, &params)?;
    let method = normalize_method(&spec.method)?;

    let mut body = match &spec.body {
        Body::None => ExportBody::Empty,
        Body::Raw { content, .. } | Body::Json { content } => ExportBody::Text(r.text(content)),
        Body::Form { fields } => ExportBody::Form(r.pairs(fields)),
        Body::Multipart { fields } => ExportBody::Multipart(
            fields
                .iter()
                .filter(|f| f.enabled && !f.name.is_empty())
                .map(|f| {
                    let name = r.text(&f.name);
                    let value = r.text(&f.value);
                    if f.is_file() {
                        ExportPart::File { name, path: value }
                    } else {
                        ExportPart::Text { name, value }
                    }
                })
                .collect(),
        ),
        Body::Binary { path, .. } => ExportBody::File(r.text(path)),
    };
    if let (ExportBody::Text(c), Body::Raw { .. } | Body::Json { .. }) = (&mut body, &spec.body) {
        if c.is_empty() {
            body = ExportBody::Empty;
        }
    }
    let default_ct = match (&spec.body, &body) {
        (_, ExportBody::Empty | ExportBody::Multipart(_)) => None,
        (Body::Json { .. }, _) => Some("application/json".to_string()),
        (Body::Raw { content_type, .. }, ExportBody::Text(c)) => Some(
            content_type
                .as_deref()
                .map(str::trim)
                .filter(|ct| !ct.is_empty())
                .map(|ct| r.text(ct))
                .unwrap_or_else(|| {
                    let t = c.trim_start();
                    if t.starts_with('{') || t.starts_with('[') {
                        "application/json".into()
                    } else {
                        "text/plain; charset=utf-8".into()
                    }
                }),
        ),
        (_, ExportBody::Form(_)) => Some("application/x-www-form-urlencoded".to_string()),
        (Body::Binary { content_type, .. }, _) => Some(
            content_type
                .as_deref()
                .map(str::trim)
                .filter(|ct| !ct.is_empty())
                .map(|ct| r.text(ct))
                .unwrap_or_else(|| "application/octet-stream".into()),
        ),
        _ => None,
    };

    let mut options = spec.options.clone();
    if let Some(p) = &options.proxy {
        options.proxy = Some(r.text(p));
    }

    let use_service_auth = spec.authenticated && ctx.service.is_authenticated;
    let pf_cfg = spec.preflight.as_ref().filter(|p| p.enabled).or_else(|| {
        use_service_auth
            .then_some(ctx.service.preflight.as_ref())
            .flatten()
            .filter(|p| p.enabled)
    });
    if let Some(cfg) = pf_cfg {
        let header = cfg
            .token_header
            .as_deref()
            .filter(|h| !h.is_empty())
            .unwrap_or("Authorization")
            .to_string();
        let value = if header.eq_ignore_ascii_case("authorization") {
            "Bearer <preflight token>".to_string()
        } else {
            "<preflight token>".to_string()
        };
        set_header(&mut headers, &header, value);
    }
    if use_service_auth {
        match resolve_auth(ctx.service, &mut r) {
            ResolvedAuth::None => {}
            ResolvedAuth::Bearer(t) => {
                set_header(&mut headers, "Authorization", format!("Bearer {t}"))
            }
            ResolvedAuth::Basic { user, pass } => {
                let value = if !include_secrets && format!("{user}{pass}").contains("{{secret.") {
                    // base64 would hide the placeholder.
                    format!("Basic base64({user}:{pass})")
                } else {
                    use base64::Engine;
                    let enc =
                        base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
                    format!("Basic {enc}")
                };
                set_header(&mut headers, "Authorization", value);
            }
            ResolvedAuth::ApiKey {
                name,
                value,
                in_query,
            } => {
                if in_query {
                    set_query(&mut url, &[(name, value)]);
                } else {
                    set_header(&mut headers, &name, value);
                }
            }
        }
    }
    if !r.missing.is_empty() {
        return Err(Error::MissingSecrets(r.missing.into_iter().collect()));
    }

    if matches!(body, ExportBody::Multipart(_)) {
        // The client writes the boundary parameter.
        headers.retain(|(k, _)| !k.eq_ignore_ascii_case("content-type"));
    } else if let Some(ct) = default_ct {
        if !has_header(&headers, "content-type") {
            headers.push(("Content-Type".into(), ct));
        }
    }

    Ok(ExportRequest {
        method,
        url: restore_placeholders(url.as_str()),
        headers,
        body,
        options,
    })
}

/// The URL layer percent-encodes `{{secret.KEY}}`; show those placeholders as written.
fn restore_placeholders(url: &str) -> String {
    const OPEN: &str = "%7B%7Bsecret.";
    const CLOSE: &str = "%7D%7D";
    let mut out = String::with_capacity(url.len());
    let mut rest = url;
    while let Some(i) = rest.find(OPEN) {
        let after = &rest[i + OPEN.len()..];
        let Some(j) = after.find(CLOSE) else { break };
        out.push_str(&rest[..i]);
        let key = &after[..j];
        out.push_str(&format!("{{{{secret.{key}}}}}"));
        rest = &after[j + CLOSE.len()..];
    }
    out.push_str(rest);
    out
}

/// Masks every secret that exists and never fetches a value.
struct MaskingStore<'a>(&'a dyn SecretStore);

impl SecretStore for MaskingStore<'_> {
    fn get(&self, key: &str) -> Option<String> {
        self.0.contains(key).then(|| SECRET_MASK.to_string())
    }
}

/// The request URL as `prepare_send` would build it, with secrets masked (see
/// [`crate::Workspace::preview_request_url`]). If the result is not a valid URL yet (for
/// example an unresolved `{{BASE_URL}}`), the text is shown as far as it resolved.
pub(crate) fn preview_request_url(
    env: Option<&Environment>,
    layers: &VariableLayers,
    secrets: &dyn SecretStore,
    spec: &RequestSpec,
) -> InterpolationPreview {
    // Built-ins stay as written: a preview must not show a value the send will not use.
    let vars = layers.resolve_static(env);
    let masking = MaskingStore(secrets);
    let mut r = Resolver {
        vars: &vars,
        secrets: &masking,
        unresolved: BTreeSet::new(),
        missing: BTreeSet::new(),
        dot_segments: BTreeSet::new(),
        secret_values: BTreeSet::new(),
    };
    let params = r.pairs(&spec.params);
    let raw_url = r.text(&spec.url);
    let raw_url = substitute_path_params(&raw_url, &spec.path_params, &mut r);
    let resolved = match build_url(&raw_url, &params) {
        // The URL layer percent-encodes the mask; show it as written.
        Ok(url) => url
            .as_str()
            .replace(&encode_segment(SECRET_MASK), SECRET_MASK),
        Err(_) => raw_url,
    };
    let mut unresolved: Vec<String> = r.unresolved.into_iter().collect();
    unresolved.extend(r.missing.into_iter().map(|k| format!("secret.{k}")));
    InterpolationPreview {
        resolved,
        unresolved,
    }
}

/// Resolve a preflight (the given one, else the service's) for a one-off test. Enforces the
/// unsafe-environment policy (judged by the preflight's own method) before anything else;
/// ignores `enabled` and the token cache.
pub(crate) fn prepare_test_preflight(
    ctx: SendContext<'_>,
    preflight: Option<Preflight>,
    confirmed: bool,
) -> Result<PreparedPreflightTest> {
    let env = ctx.environment;
    let cfg = preflight
        .or_else(|| ctx.service.preflight.clone())
        .ok_or_else(|| Error::Preflight("no preflight is configured".into()))?;
    let method = if cfg.method.trim().is_empty() {
        "POST"
    } else {
        cfg.method.as_str()
    };
    SendPolicy::for_request(env, method).check(confirmed)?;

    let vars = ctx.layers.resolve(env);
    let mut r = Resolver {
        vars: &vars,
        secrets: ctx.secrets,
        unresolved: BTreeSet::new(),
        missing: BTreeSet::new(),
        dot_segments: BTreeSet::new(),
        secret_values: BTreeSet::new(),
    };
    let resolved = resolve_preflight(&cfg, &mut r, "")?;
    if !r.missing.is_empty() {
        return Err(Error::MissingSecrets(r.missing.into_iter().collect()));
    }
    Ok(PreparedPreflightTest {
        request: resolved.request,
        config: cfg,
        http: ctx.http.clone(),
    })
}

pub(crate) use crate::model::normalize_method;

/// Variable + secret resolution context for one send.
struct Resolver<'a> {
    vars: &'a Variables,
    secrets: &'a dyn SecretStore,
    unresolved: BTreeSet<String>,
    missing: BTreeSet<String>,
    /// Path params whose value is `.` or `..`: the URL layer collapses even the encoded form.
    dot_segments: BTreeSet<String>,
    /// Every secret value that was substituted.
    secret_values: BTreeSet<String>,
}

impl Resolver<'_> {
    fn text(&mut self, s: &str) -> String {
        let interpolated = interpolate(s, self.vars);
        self.unresolved.extend(unresolved(&interpolated));
        let found = std::cell::RefCell::new(Vec::new());
        let (out, missing) = resolve_secrets(&interpolated, &|k| {
            let v = self.secrets.get(k);
            if let Some(v) = &v {
                found.borrow_mut().push(v.clone());
            }
            v
        });
        self.missing.extend(missing);
        self.secret_values.extend(found.into_inner());
        out
    }

    /// Enabled, named rows with both sides resolved.
    fn pairs(&mut self, rows: &[NameValue]) -> Vec<(String, String)> {
        rows.iter()
            .filter(|h| h.enabled && !h.name.is_empty())
            .map(|h| (self.text(&h.name), self.text(&h.value)))
            .collect()
    }
}

/// Replace `{name}` tokens in the URL path (not the scheme/host, query or fragment) with the
/// enabled path param of that name: value resolved like any text, then percent-encoded as one
/// path segment. `{{...}}` is left alone. Tokens without a param stay verbatim and are
/// reported as `{name}`.
fn substitute_path_params(url: &str, params: &[NameValue], r: &mut Resolver) -> String {
    let Some(scheme_end) = url.find("://") else {
        return url.to_string();
    };
    let path_start = url[scheme_end + 3..]
        .find(['/', '?', '#'])
        .map_or(url.len(), |i| scheme_end + 3 + i);
    let path_end = url[path_start..]
        .find(['?', '#'])
        .map_or(url.len(), |i| path_start + i);
    let path = &url[path_start..path_end];

    let mut out = String::with_capacity(url.len());
    out.push_str(&url[..path_start]);
    let mut rest = path;
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        rest = &rest[i..];
        if rest.starts_with("{{") {
            // A variable placeholder: copy it through untouched.
            let end = rest.find("}}").map_or(2, |e| e + 2);
            out.push_str(&rest[..end]);
            rest = &rest[end..];
            continue;
        }
        let Some(close) = rest.find('}') else { break };
        let name = &rest[1..close];
        if name.is_empty() || name.contains('{') {
            out.push('{');
            rest = &rest[1..];
            continue;
        }
        let param = params
            .iter()
            .find(|p| p.enabled && !p.name.is_empty() && p.name == name);
        match param {
            Some(p) => {
                let value = r.text(&p.value);
                if matches!(value.as_str(), "." | "..") {
                    r.dot_segments.insert(format!("{{{name}}}"));
                }
                out.push_str(&encode_segment(&value));
            }
            None => {
                r.unresolved.insert(format!("{{{name}}}"));
                out.push_str(&rest[..=close]);
            }
        }
        rest = &rest[close + 1..];
    }
    out.push_str(rest);
    out.push_str(&url[path_end..]);
    out
}

/// Percent-encode everything except RFC 3986 unreserved characters.
/// A value that is exactly `.` or `..` has its dots encoded so it can never become a
/// dot-segment that URL normalization would collapse.
fn encode_segment(s: &str) -> String {
    if matches!(s, "." | "..") {
        return s.replace('.', "%2E");
    }
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn resolve_auth(service: &ServiceFile, r: &mut Resolver) -> ResolvedAuth {
    let Some(cfg) = service.auth.as_ref() else {
        return ResolvedAuth::None;
    };
    match service.effective_auth_kind() {
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

fn resolve_preflight(cfg: &Preflight, r: &mut Resolver, scope: &str) -> Result<ResolvedPreflight> {
    let method = normalize_method(&cfg.method)?;
    let url = build_url(&r.text(&cfg.url), &[])?;
    let mut headers = r.pairs(&cfg.headers);
    let body = if cfg.body_type.eq_ignore_ascii_case("urlencoded") {
        if !has_header(&headers, "content-type") {
            headers.push((
                "Content-Type".into(),
                "application/x-www-form-urlencoded".into(),
            ));
        }
        ResolvedBody::Form(r.pairs(&cfg.body_params))
    } else {
        ResolvedBody::Text {
            content: r.text(&cfg.body),
            content_type: None,
        }
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

fn resolve_body(body: &Body, r: &mut Resolver, root: &Path) -> Result<ResolvedBody> {
    Ok(match body {
        Body::None => ResolvedBody::Empty,
        Body::Raw {
            content,
            content_type,
        } => ResolvedBody::Text {
            content: r.text(content),
            content_type: content_type
                .as_deref()
                .map(str::trim)
                .filter(|ct| !ct.is_empty())
                .map(|ct| r.text(ct)),
        },
        Body::Json { content } => ResolvedBody::Text {
            content: r.text(content),
            content_type: Some("application/json".into()),
        },
        Body::Form { fields } => ResolvedBody::Form(r.pairs(fields)),
        Body::Multipart { fields } => {
            let mut parts = vec![];
            for f in fields.iter().filter(|f| f.enabled && !f.name.is_empty()) {
                let name = r.text(&f.name);
                let value = r.text(&f.value);
                parts.push(if f.is_file() {
                    let (file_name, data) = read_workspace_file(root, &value)?;
                    ResolvedPart::File {
                        name,
                        file_name,
                        data,
                    }
                } else {
                    ResolvedPart::Text { name, value }
                });
            }
            ResolvedBody::Multipart(parts)
        }
        Body::Binary { path, content_type } => {
            let path = r.text(path);
            let (_, data) = read_workspace_file(root, &path)?;
            ResolvedBody::Bytes {
                data,
                content_type: content_type
                    .as_deref()
                    .map(str::trim)
                    .filter(|ct| !ct.is_empty())
                    .map(|ct| r.text(ct))
                    .unwrap_or_else(|| "application/octet-stream".into()),
            }
        }
    })
}

/// Read a file a request body refers to. Relative paths are resolved against the workspace
/// root, and the file (after following symlinks) must be inside the workspace: endpoints are
/// shared through git, so a shared request must not be able to upload arbitrary local files.
fn read_workspace_file(root: &Path, path: &str) -> Result<(String, Vec<u8>)> {
    let path = path.trim();
    if path.is_empty() {
        return Err(Error::InvalidBody("no file selected".into()));
    }
    let candidate = {
        let p = Path::new(path);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            root.join(p)
        }
    };
    let root = root.canonicalize().map_err(|e| Error::io(root, e))?;
    let file = candidate
        .canonicalize()
        .map_err(|e| Error::InvalidBody(format!("cannot read {path}: {e}")))?;
    if !file.starts_with(&root) {
        return Err(Error::InvalidBody(format!(
            "{path} is outside the workspace; files sent in a request must be inside the workspace folder"
        )));
    }
    let meta = fs::metadata(&file).map_err(|e| Error::io(&file, e))?;
    if !meta.is_file() {
        return Err(Error::InvalidBody(format!("{path} is not a file")));
    }
    if meta.len() > MAX_BODY_FILE_BYTES {
        return Err(Error::InvalidBody(format!(
            "{path} is {} bytes; the limit is {MAX_BODY_FILE_BYTES}",
            meta.len()
        )));
    }
    let data = fs::read(&file).map_err(|e| Error::io(&file, e))?;
    let name = file
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".into());
    Ok((name, data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::MemorySecretStore;

    fn nv(n: &str, v: &str) -> NameValue {
        NameValue::new(n, v)
    }

    /// Run path-param substitution; returns the URL and the sorted unresolved list.
    fn subst(url: &str, params: &[NameValue], vars: &[(&str, &str)]) -> (String, Vec<String>) {
        let vars: Variables = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        let secrets = MemorySecretStore::default();
        secrets.set("tok", "s/3 cret");
        let mut r = Resolver {
            vars: &vars,
            secrets: &secrets,
            unresolved: BTreeSet::new(),
            missing: BTreeSet::new(),
            dot_segments: BTreeSet::new(),
            secret_values: BTreeSet::new(),
        };
        let out = substitute_path_params(url, params, &mut r);
        (out, r.unresolved.into_iter().collect())
    }

    #[test]
    fn path_param_is_substituted() {
        let (u, un) = subst("http://h.io/users/{id}/posts", &[nv("id", "42")], &[]);
        assert_eq!(u, "http://h.io/users/42/posts");
        assert!(un.is_empty());
    }

    #[test]
    fn path_param_value_is_percent_encoded_as_a_segment() {
        let (u, _) = subst("http://h.io/f/{p}", &[nv("p", "a/b c")], &[]);
        assert_eq!(u, "http://h.io/f/a%2Fb%20c");
        let (u, _) = subst("http://h.io/f/{p}", &[nv("p", "x?y#z%")], &[]);
        assert_eq!(u, "http://h.io/f/x%3Fy%23z%25");
    }

    #[test]
    fn dot_segment_values_are_encoded() {
        let (u, _) = subst("http://h.io/users/{id}", &[nv("id", "..")], &[]);
        assert_eq!(u, "http://h.io/users/%2E%2E");
        let (u, _) = subst("http://h.io/users/{id}", &[nv("id", ".")], &[]);
        assert_eq!(u, "http://h.io/users/%2E");
        let (u, _) = subst("http://h.io/users/{id}", &[nv("id", "a.b")], &[]);
        assert_eq!(u, "http://h.io/users/a.b");
        let (u, _) = subst("http://h.io/users/{id}", &[nv("id", "...")], &[]);
        assert_eq!(u, "http://h.io/users/...");
    }

    #[test]
    fn path_param_value_is_interpolated_with_variables_and_secrets() {
        let (u, _) = subst(
            "http://h.io/{a}/{b}",
            &[nv("a", "{{ID}}"), nv("b", "{{secret.tok}}")],
            &[("ID", "7")],
        );
        assert_eq!(u, "http://h.io/7/s%2F3%20cret");
    }

    #[test]
    fn disabled_path_param_is_not_used() {
        let mut p = nv("id", "42");
        p.enabled = false;
        let (u, un) = subst("http://h.io/u/{id}", &[p], &[]);
        assert_eq!(u, "http://h.io/u/{id}");
        assert_eq!(un, vec!["{id}"]);
    }

    #[test]
    fn missing_path_param_stays_verbatim_and_is_reported() {
        let (u, un) = subst("http://h.io/{a}/{b}", &[nv("a", "1")], &[]);
        assert_eq!(u, "http://h.io/1/{b}");
        assert_eq!(un, vec!["{b}"]);
    }

    #[test]
    fn double_brace_variables_are_untouched() {
        let (u, un) = subst("http://h.io/{{NOPE}}/{id}", &[nv("id", "1")], &[]);
        assert_eq!(u, "http://h.io/{{NOPE}}/1");
        assert!(un.is_empty(), "{un:?}");
    }

    #[test]
    fn query_and_host_braces_are_untouched() {
        let (u, un) = subst(
            "http://h.io/p/{id}?x={x}#f={f}",
            &[nv("id", "1"), nv("x", "no"), nv("f", "no")],
            &[],
        );
        assert_eq!(u, "http://h.io/p/1?x={x}#f={f}");
        assert!(un.is_empty());
    }

    #[test]
    fn substituted_values_are_not_rescanned() {
        let (u, un) = subst("http://h.io/{a}", &[nv("a", "{a}")], &[]);
        assert_eq!(u, "http://h.io/%7Ba%7D");
        assert!(un.is_empty());
    }

    #[test]
    fn files_must_stay_inside_the_workspace() {
        let outer = tempfile::tempdir().unwrap();
        let root = outer.path().join("ws");
        fs::create_dir_all(root.join("data")).unwrap();
        fs::write(root.join("data/a.txt"), b"inside").unwrap();
        fs::write(outer.path().join("secret.txt"), b"outside").unwrap();

        let (name, data) = read_workspace_file(&root, "data/a.txt").unwrap();
        assert_eq!(
            (name.as_str(), data.as_slice()),
            ("a.txt", b"inside".as_slice())
        );

        let abs_inside = root.join("data/a.txt");
        assert!(read_workspace_file(&root, abs_inside.to_str().unwrap()).is_ok());

        for bad in [
            "../secret.txt".to_string(),
            outer.path().join("secret.txt").display().to_string(),
            "data/missing.txt".to_string(),
            "data".to_string(),
            " ".to_string(),
        ] {
            assert!(
                matches!(read_workspace_file(&root, &bad), Err(Error::InvalidBody(_))),
                "{bad:?} should be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_out_of_the_workspace_are_rejected() {
        let outer = tempfile::tempdir().unwrap();
        let root = outer.path().join("ws");
        fs::create_dir_all(&root).unwrap();
        fs::write(outer.path().join("secret.txt"), b"outside").unwrap();
        std::os::unix::fs::symlink(outer.path().join("secret.txt"), root.join("link.txt")).unwrap();
        assert!(matches!(
            read_workspace_file(&root, "link.txt"),
            Err(Error::InvalidBody(_))
        ));
    }
}
