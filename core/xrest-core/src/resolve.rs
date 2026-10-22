//! Resolution: turn a [`RequestSpec`] into a [`PreparedSend`] for one environment.
//!
//! Pure with respect to the workspace: everything it needs arrives in a [`SendContext`]. It
//! interpolates `{{VAR}}`s, resolves `{{secret.KEY}}`s, applies service auth and preflight,
//! reads files referenced by the body, and enforces the unsafe-environment policy first.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::exec::{
    build_url, has_header, HttpOptions, PreparedSend, ResolvedAuth, ResolvedBody, ResolvedPart,
    ResolvedPreflight, ResolvedRequest,
};
use crate::interpolate::{interpolate, resolve_secrets, unresolved, Variables};
use crate::model::{AuthKind, Body, Environment, NameValue, Preflight, RequestSpec, ServiceFile};
use crate::policy::SendPolicy;
use crate::preflight::TokenCache;
use crate::secrets::SecretStore;

/// Largest file a request body may read (binary bodies and multipart file fields).
pub const MAX_BODY_FILE_BYTES: u64 = 64 * 1024 * 1024;

/// What resolution needs from a workspace.
pub(crate) struct SendContext<'a> {
    pub root: &'a Path,
    pub service: &'a ServiceFile,
    pub environment: Option<&'a Environment>,
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
    SendPolicy::for_environment(env).check(confirmed)?;

    let vars = variables(env);
    let mut r = Resolver {
        vars: &vars,
        secrets: ctx.secrets,
        unresolved: BTreeSet::new(),
        missing: BTreeSet::new(),
    };

    let params = r.pairs(&spec.params);
    let headers = r.pairs(&spec.headers);
    let url = build_url(&r.text(&spec.url), &params)?;
    let method = normalize_method(&spec.method);
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
        tokens: ctx.tokens,
        http,
    })
}

/// The environment's enabled variables; secret-backed ones become `{{secret.KEY}}` references.
fn variables(env: Option<&Environment>) -> Variables {
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

    /// Enabled, named rows with both sides resolved.
    fn pairs(&mut self, rows: &[NameValue]) -> Vec<(String, String)> {
        rows.iter()
            .filter(|h| h.enabled && !h.name.is_empty())
            .map(|h| (self.text(&h.name), self.text(&h.value)))
            .collect()
    }
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
    let method = normalize_method(&cfg.method);
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
