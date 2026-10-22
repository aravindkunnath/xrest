//! Request execution: turns a fully resolved request into an HTTP exchange via `reqwest`.
//!
//! [`PreparedSend`] is produced synchronously by [`crate::Workspace::prepare_send`] (variables
//! interpolated, secrets resolved, unsafe policy checked) and is self-contained and `Send`, so the
//! workspace does not need to be locked while the network call is in flight.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use url::Url;

use crate::error::{Error, Result};
use crate::model::{HttpResponse, Preflight, RequestOptions};
use crate::preflight::{cache_ttl, extract_token, TokenCache};

/// App-wide transport defaults; a request's [`crate::RequestOptions`] override them per send.
#[derive(Debug, Clone)]
pub struct HttpOptions {
    pub timeout: Duration,
    pub follow_redirects: bool,
    pub insecure_skip_verify: bool,
    pub use_system_proxy: bool,
    /// Explicit proxy URL; `Some("")` forces a direct connection. Overrides `use_system_proxy`.
    pub proxy: Option<String>,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            follow_redirects: true,
            insecure_skip_verify: false,
            use_system_proxy: true,
            proxy: None,
        }
    }
}

impl HttpOptions {
    /// These defaults with a request's options applied (`proxy` already interpolated).
    pub fn merged(&self, o: &RequestOptions) -> HttpOptions {
        HttpOptions {
            timeout: o
                .timeout_ms
                .filter(|ms| *ms > 0)
                .map(Duration::from_millis)
                .unwrap_or(self.timeout),
            follow_redirects: o.follow_redirects.unwrap_or(self.follow_redirects),
            insecure_skip_verify: o
                .verify_tls
                .map(|v| !v)
                .unwrap_or(self.insecure_skip_verify),
            use_system_proxy: self.use_system_proxy,
            proxy: o.proxy.clone().or_else(|| self.proxy.clone()),
        }
    }
}

/// A request with every placeholder resolved and every referenced file read.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedRequest {
    pub method: String,
    pub url: Url,
    pub headers: Vec<(String, String)>,
    pub body: ResolvedBody,
}

#[derive(Debug, Clone)]
pub(crate) enum ResolvedBody {
    Empty,
    /// `content_type: None` means "sniff" (main request only), as the legacy app did.
    Text {
        content: String,
        content_type: Option<String>,
    },
    Bytes {
        data: Vec<u8>,
        content_type: String,
    },
    Form(Vec<(String, String)>),
    Multipart(Vec<ResolvedPart>),
}

#[derive(Debug, Clone)]
pub(crate) enum ResolvedPart {
    Text {
        name: String,
        value: String,
    },
    File {
        name: String,
        file_name: String,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum ResolvedAuth {
    None,
    Bearer(String),
    Basic {
        user: String,
        pass: String,
    },
    ApiKey {
        name: String,
        value: String,
        in_query: bool,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedPreflight {
    pub request: ResolvedRequest,
    pub config: Preflight,
    pub cache_key: String,
}

/// A request ready to go on the wire.
pub struct PreparedSend {
    pub(crate) environment: Option<String>,
    pub(crate) request: ResolvedRequest,
    pub(crate) auth: ResolvedAuth,
    pub(crate) preflight: Option<ResolvedPreflight>,
    pub(crate) unresolved: Vec<String>,
    pub(crate) tokens: Arc<TokenCache>,
    pub(crate) http: HttpOptions,
}

impl PreparedSend {
    pub fn environment(&self) -> Option<&str> {
        self.environment.as_deref()
    }

    pub fn method(&self) -> &str {
        &self.request.method
    }

    /// The URL as it will be requested (before auth query params are added).
    pub fn url(&self) -> &str {
        self.request.url.as_str()
    }

    pub fn unresolved_variables(&self) -> &[String] {
        &self.unresolved
    }

    pub async fn execute(self) -> Result<HttpResponse> {
        let client = build_client(&self.http)?;
        let mut request = self.request;

        if let Some(pf) = &self.preflight {
            let token = preflight_token(&client, pf, &self.tokens).await?;
            let header = pf
                .config
                .token_header
                .as_deref()
                .filter(|h| !h.is_empty())
                .unwrap_or("Authorization")
                .to_string();
            let value = if header.eq_ignore_ascii_case("authorization") {
                format!("Bearer {token}")
            } else {
                token
            };
            set_header(&mut request.headers, &header, value);
        }

        match &self.auth {
            ResolvedAuth::None => {}
            ResolvedAuth::Bearer(t) => {
                set_header(&mut request.headers, "Authorization", format!("Bearer {t}"))
            }
            ResolvedAuth::Basic { user, pass } => {
                let enc =
                    base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
                set_header(
                    &mut request.headers,
                    "Authorization",
                    format!("Basic {enc}"),
                );
            }
            ResolvedAuth::ApiKey {
                name,
                value,
                in_query,
            } => {
                if *in_query {
                    set_query(&mut request.url, &[(name.clone(), value.clone())]);
                } else {
                    set_header(&mut request.headers, name, value.clone());
                }
            }
        }

        default_content_type(&mut request);

        let raw = send_raw(&client, &request).await?;
        let content_type = raw
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let (body, body_is_base64) = decode_body(&raw.bytes, &content_type);
        let pretty_body = if body_is_base64 {
            None
        } else {
            pretty_json(&body, &content_type)
        };
        Ok(HttpResponse {
            status: raw.status,
            status_text: raw.status_text,
            headers: raw.headers,
            content_type,
            body,
            body_is_base64,
            pretty_body,
            size_bytes: raw.bytes.len() as u64,
            time_ms: raw.elapsed.as_millis() as u64,
            final_url: raw.final_url,
            environment: self.environment,
            unresolved_variables: self.unresolved,
        })
    }
}

/// Set `Content-Type` from the body when the user did not set one.
fn default_content_type(req: &mut ResolvedRequest) {
    if matches!(req.body, ResolvedBody::Multipart(_)) {
        // reqwest writes the boundary parameter; a user-set Content-Type would break it.
        req.headers
            .retain(|(k, _)| !k.eq_ignore_ascii_case("content-type"));
        return;
    }
    if has_header(&req.headers, "content-type") {
        return;
    }
    let ct = match &req.body {
        ResolvedBody::Empty | ResolvedBody::Multipart(_) => return,
        ResolvedBody::Text { content, .. } if content.is_empty() => return,
        ResolvedBody::Text {
            content_type: Some(ct),
            ..
        } => ct.clone(),
        ResolvedBody::Text { content, .. } => {
            let t = content.trim_start();
            if t.starts_with('{') || t.starts_with('[') {
                "application/json".into()
            } else {
                "text/plain; charset=utf-8".into()
            }
        }
        ResolvedBody::Bytes { content_type, .. } => content_type.clone(),
        ResolvedBody::Form(_) => "application/x-www-form-urlencoded".into(),
    };
    set_header(&mut req.headers, "Content-Type", ct);
}

fn encode_form(pairs: &[(String, String)]) -> String {
    let mut ser = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in pairs {
        ser.append_pair(k, v);
    }
    ser.finish()
}

fn multipart_form(parts: &[ResolvedPart]) -> reqwest::multipart::Form {
    parts
        .iter()
        .fold(reqwest::multipart::Form::new(), |form, part| match part {
            ResolvedPart::Text { name, value } => form.text(name.clone(), value.clone()),
            ResolvedPart::File {
                name,
                file_name,
                data,
            } => form.part(
                name.clone(),
                reqwest::multipart::Part::bytes(data.clone()).file_name(file_name.clone()),
            ),
        })
}

pub(crate) fn has_header(headers: &[(String, String)], name: &str) -> bool {
    headers.iter().any(|(k, _)| k.eq_ignore_ascii_case(name))
}

pub(crate) fn set_header(headers: &mut Vec<(String, String)>, name: &str, value: String) {
    headers.retain(|(k, _)| !k.eq_ignore_ascii_case(name));
    headers.push((name.to_string(), value));
}

/// Parse `url` (http/https only) and merge `params` into its query: a param replaces any pair of
/// the same name already in the URL; params may repeat.
pub(crate) fn build_url(raw: &str, params: &[(String, String)]) -> Result<Url> {
    let raw = raw.trim();
    let mut url = Url::parse(raw).map_err(|e| Error::InvalidUrl {
        url: raw.to_string(),
        message: e.to_string(),
    })?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(Error::InvalidUrl {
            url: raw.to_string(),
            message: "only http and https URLs are supported".into(),
        });
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(Error::InvalidUrl {
            url: raw.to_string(),
            message: "missing host".into(),
        });
    }
    set_query(&mut url, params);
    Ok(url)
}

fn set_query(url: &mut Url, params: &[(String, String)]) {
    if params.is_empty() {
        return;
    }
    let names: HashSet<&str> = params.iter().map(|(k, _)| k.as_str()).collect();
    let kept: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(k, _)| !names.contains(k.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    url.set_query(None);
    let mut q = url.query_pairs_mut();
    for (k, v) in kept.iter().chain(params.iter()) {
        q.append_pair(k, v);
    }
}

fn build_client(opts: &HttpOptions) -> Result<reqwest::Client> {
    let mut b = reqwest::Client::builder()
        .timeout(opts.timeout)
        .user_agent(concat!("XRest/", env!("CARGO_PKG_VERSION")))
        .redirect(if opts.follow_redirects {
            reqwest::redirect::Policy::limited(10)
        } else {
            reqwest::redirect::Policy::none()
        });
    if opts.insecure_skip_verify {
        b = b.danger_accept_invalid_certs(true);
    }
    match opts.proxy.as_deref().map(str::trim) {
        Some("") => b = b.no_proxy(),
        Some(p) => {
            let proxy = reqwest::Proxy::all(p).map_err(|e| Error::InvalidUrl {
                url: p.to_string(),
                message: format!("invalid proxy: {e}"),
            })?;
            b = b.proxy(proxy);
        }
        None if !opts.use_system_proxy => b = b.no_proxy(),
        None => {}
    }
    b.build().map_err(|e| Error::Http(e.to_string()))
}

struct RawResponse {
    status: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    bytes: Vec<u8>,
    final_url: String,
    elapsed: Duration,
}

async fn send_raw(client: &reqwest::Client, req: &ResolvedRequest) -> Result<RawResponse> {
    let method = reqwest::Method::from_bytes(req.method.as_bytes())
        .map_err(|_| Error::Http(format!("invalid HTTP method {:?}", req.method)))?;
    let mut builder = client.request(method, req.url.clone());
    for (k, v) in &req.headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    builder = match &req.body {
        ResolvedBody::Empty => builder,
        ResolvedBody::Text { content, .. } if content.is_empty() => builder,
        ResolvedBody::Text { content, .. } => builder.body(content.clone()),
        ResolvedBody::Bytes { data, .. } => builder.body(data.clone()),
        ResolvedBody::Form(pairs) => builder.body(encode_form(pairs)),
        ResolvedBody::Multipart(parts) => builder.multipart(multipart_form(parts)),
    };
    let start = Instant::now();
    let resp = builder
        .send()
        .await
        .map_err(|e| Error::Http(describe(&e)))?;
    let status = resp.status();
    let final_url = resp.url().to_string();
    let headers = resp
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.to_string(),
                String::from_utf8_lossy(v.as_bytes()).into_owned(),
            )
        })
        .collect();
    let bytes = resp
        .bytes()
        .await
        .map_err(|e| Error::Http(describe(&e)))?
        .to_vec();
    Ok(RawResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or("").to_string(),
        headers,
        bytes,
        final_url,
        elapsed: start.elapsed(),
    })
}

/// `reqwest::Error` Display hides the cause chain; include it.
fn describe(e: &reqwest::Error) -> String {
    use std::error::Error as _;
    let mut msg = e.to_string();
    let mut src = e.source();
    while let Some(s) = src {
        msg.push_str(": ");
        msg.push_str(&s.to_string());
        src = s.source();
    }
    msg
}

fn decode_body(bytes: &[u8], content_type: &str) -> (String, bool) {
    let ct = content_type.to_ascii_lowercase();
    let binary_type = [
        "image/",
        "audio/",
        "video/",
        "application/octet-stream",
        "application/pdf",
    ]
    .iter()
    .any(|p| ct.starts_with(p));
    if binary_type {
        return (
            base64::engine::general_purpose::STANDARD.encode(bytes),
            true,
        );
    }
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_string(), false),
        Err(_) => (
            base64::engine::general_purpose::STANDARD.encode(bytes),
            true,
        ),
    }
}

/// Pretty-print `body` if it is JSON (by content type or by shape).
fn pretty_json(body: &str, content_type: &str) -> Option<String> {
    let trimmed = body.trim_start();
    let looks_json = content_type.to_ascii_lowercase().contains("json")
        || trimmed.starts_with('{')
        || trimmed.starts_with('[');
    if !looks_json {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(body).ok()?;
    serde_json::to_string_pretty(&value).ok()
}

async fn preflight_token(
    client: &reqwest::Client,
    pf: &ResolvedPreflight,
    cache: &TokenCache,
) -> Result<String> {
    if pf.config.cache_token {
        if let Some(t) = cache.get(&pf.cache_key) {
            return Ok(t);
        }
    }
    let resp = send_raw(client, &pf.request)
        .await
        .map_err(|e| Error::Preflight(e.to_string()))?;
    if !(200..300).contains(&resp.status) {
        let body = String::from_utf8_lossy(&resp.bytes);
        let snippet: String = body.chars().take(300).collect();
        return Err(Error::Preflight(format!(
            "status {}: {snippet}",
            resp.status
        )));
    }
    let token = extract_token(&resp.bytes, &pf.config.token_key)?;
    if let Some(ttl) = cache_ttl(&pf.config, &resp.bytes) {
        cache.put(pf.cache_key.clone(), token.clone(), ttl);
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(k: &str, v: &str) -> (String, String) {
        (k.into(), v.into())
    }

    #[test]
    fn url_params_replace_same_name_and_repeat() {
        let u = build_url(
            "http://h.io/x?a=1&b=2",
            &[p("a", "9"), p("c", "x y"), p("c", "z")],
        )
        .unwrap();
        assert_eq!(u.as_str(), "http://h.io/x?b=2&a=9&c=x+y&c=z");
    }

    #[test]
    fn url_without_params_untouched() {
        let u = build_url("http://h.io/x?a=%20b", &[]).unwrap();
        assert_eq!(u.as_str(), "http://h.io/x?a=%20b");
    }

    #[test]
    fn url_validation() {
        assert!(build_url("example.com/x", &[]).is_err());
        assert!(build_url("ftp://example.com", &[]).is_err());
        assert!(build_url("{{BASE}}/x", &[]).is_err());
        assert!(build_url("https://example.com", &[]).is_ok());
    }

    #[test]
    fn header_set_is_case_insensitive() {
        let mut h = vec![p("authorization", "old"), p("X", "1")];
        set_header(&mut h, "Authorization", "new".into());
        assert_eq!(h, vec![p("X", "1"), p("Authorization", "new")]);
    }

    #[test]
    fn pretty_json_keeps_order_and_big_numbers() {
        let out = pretty_json(
            r#"{"z":1,"a":[12345678901234567890,1.50]}"#,
            "application/json",
        )
        .unwrap();
        assert_eq!(
            out,
            "{\n  \"z\": 1,\n  \"a\": [\n    12345678901234567890,\n    1.50\n  ]\n}"
        );
        assert!(pretty_json("plain text", "text/plain").is_none());
        assert!(pretty_json("{broken", "application/json").is_none());
        assert!(pretty_json("[1]", "text/plain").is_some());
    }

    #[test]
    fn body_decoding() {
        assert_eq!(decode_body(b"hi", "text/plain"), ("hi".into(), false));
        assert_eq!(decode_body(b"hi", "image/png"), ("aGk=".into(), true));
        assert_eq!(
            decode_body(&[0xff, 0xfe], "text/plain"),
            ("//4=".into(), true)
        );
    }
}
