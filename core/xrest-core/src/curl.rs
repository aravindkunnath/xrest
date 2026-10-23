//! cURL command import, ported from `internal/import/curl.go`.
//!
//! Differences from the Go version: any whitespace (including the newlines of `\`-continued
//! commands) separates tokens, and an unknown flag never swallows a following `http(s)://` URL.
//! As in Go, `-u/--user`, `-b/--cookie` and `-A/--user-agent` are ignored: credentials must not
//! silently end up in files that are committed to git.
//! TODO(curl-auth): offer to map `-u user:pass` to service-level basic auth.

use crate::error::{Error, Result};
use crate::model::{Body, NameValue, RequestOptions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCurl {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<NameValue>,
    pub body: Body,
    pub options: RequestOptions,
    pub params: Vec<NameValue>,
}

/// One `-d`-style argument, kept apart until the body type is known.
enum Data {
    /// `-d`, `--data`, `--data-raw`, `--data-binary`: sent as given (already encoded).
    Raw(String),
    /// `--data-urlencode [name=]content`: curl URL-encodes `content`.
    Encode(Option<String>, String),
}

fn encode(v: &str) -> String {
    url::form_urlencoded::byte_serialize(v.as_bytes()).collect()
}

fn header_value<'a>(headers: &'a [NameValue], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str())
}

/// Pick a typed body for curl's data arguments, matching what curl would send.
fn data_body(data: Vec<Data>, headers: &[NameValue]) -> Body {
    if data.is_empty() {
        return Body::None;
    }
    let ct = header_value(headers, "content-type").map(str::to_ascii_lowercase);
    let form_ct = ct
        .as_deref()
        .is_none_or(|c| c.starts_with("application/x-www-form-urlencoded"));
    if form_ct && data.iter().all(|d| matches!(d, Data::Encode(Some(_), _))) {
        let fields = data
            .into_iter()
            .filter_map(|d| match d {
                Data::Encode(Some(n), v) => Some(NameValue::new(n, v)),
                _ => None,
            })
            .collect();
        return Body::Form { fields };
    }
    let content = data
        .into_iter()
        .map(|d| match d {
            Data::Raw(s) => s,
            Data::Encode(Some(n), v) => format!("{n}={}", encode(&v)),
            Data::Encode(None, v) => encode(&v),
        })
        .collect::<Vec<_>>()
        .join("&");
    let looks_json = {
        let t = content.trim_start();
        (t.starts_with('{') || t.starts_with('['))
            && serde_json::from_str::<serde_json::Value>(&content).is_ok()
    };
    match ct.as_deref() {
        Some(c) if c.contains("json") => Body::Json { content },
        // The explicit header decides the type; keep the text as-is.
        Some(_) => Body::Raw {
            content,
            content_type: None,
        },
        None if looks_json => Body::Json { content },
        // curl's default for -d.
        None => Body::Raw {
            content,
            content_type: Some("application/x-www-form-urlencoded".into()),
        },
    }
}

/// `-F name=value` / `-F name=@path[;type=…]`: `@` and `<` read a file.
fn form_field(arg: &str, literal: bool) -> Option<NameValue> {
    let (name, value) = arg.split_once('=')?;
    if name.is_empty() {
        return None;
    }
    if !literal {
        if let Some(path) = value.strip_prefix('@').or_else(|| value.strip_prefix('<')) {
            let path = path.split(';').next().unwrap_or(path);
            return Some(NameValue::file(name, path));
        }
    }
    Some(NameValue::new(name, value))
}

fn is_http_method(s: &str) -> bool {
    matches!(
        s,
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "CONNECT" | "TRACE"
    )
}

/// Split a shell-like command into tokens, removing quotes as a POSIX shell does (adjacent
/// quoted and unquoted pieces join; `''` is an empty token).
fn tokenize(cmd: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut cur = String::new();
    let (mut in_single, mut in_double, mut escape, mut quoted) = (false, false, false, false);
    let mut chars = cmd.chars();
    while let Some(ch) = chars.next() {
        if escape {
            cur.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' && in_double {
            escape = true;
            continue;
        }
        if ch == '\\' && !in_single && !in_double {
            if let Some(next) = chars.next() {
                // `\<newline>` is a line continuation, not a literal newline.
                if next != '\n' && next != '\r' {
                    cur.push(next);
                }
            }
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            quoted = true;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            quoted = true;
            continue;
        }
        if ch.is_whitespace() && !in_single && !in_double {
            if !cur.is_empty() || quoted {
                tokens.push(std::mem::take(&mut cur));
                quoted = false;
            }
            continue;
        }
        cur.push(ch);
    }
    if !cur.is_empty() || quoted {
        tokens.push(cur);
    }
    tokens
}

/// Human-readable endpoint name from a URL path (`/api/v2/users` -> `api v2 users`).
pub fn endpoint_name(url: &str) -> String {
    let Some(idx) = url.find("://") else {
        return "New Endpoint".into();
    };
    let rest = &url[idx + 3..];
    let Some(slash) = rest.find('/') else {
        return "root".into();
    };
    let mut path = &rest[slash + 1..];
    if let Some(q) = path.find('?') {
        path = &path[..q];
    }
    if path.is_empty() {
        return "root".into();
    }
    let spaced: String = path
        .chars()
        .map(|c| if matches!(c, '/' | '-' | '_') { ' ' } else { c })
        .collect();
    let collapsed = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        "New Endpoint".into()
    } else {
        collapsed
    }
}

/// Query parameters of a URL as raw (not decoded) pairs, as the Go importer did.
pub fn query_params(url: &str) -> Vec<NameValue> {
    let Some(q) = url.find('?') else {
        return vec![];
    };
    let mut query = &url[q + 1..];
    if let Some(f) = query.find('#') {
        query = &query[..f];
    }
    query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => NameValue::new(k, v),
            None => NameValue::new(pair, ""),
        })
        .collect()
}

pub fn parse(command: &str) -> Result<ParsedCurl> {
    let cmd = command.trim();
    let lower = cmd.to_ascii_lowercase();
    if !(lower.starts_with("curl") && cmd[4..].starts_with(char::is_whitespace)) {
        return Err(Error::Curl("command does not start with 'curl'".into()));
    }
    let tokens = tokenize(&cmd[4..]);

    let mut method = "GET".to_string();
    let mut url = String::new();
    let mut headers = vec![];
    let mut data = vec![];
    let mut form = vec![];
    let mut options = RequestOptions::default();

    let mut i = 0;
    while i < tokens.len() {
        let tok = tokens[i].as_str();
        let next = tokens.get(i + 1).map(String::as_str);
        match tok {
            "-X" | "--request" => {
                if let Some(n) = next {
                    method = n.to_ascii_uppercase();
                    i += 1;
                }
            }
            "-H" | "--header" => {
                if let Some(n) = next {
                    let h = n;
                    if let Some(c) = h.find(':').filter(|c| *c > 0) {
                        headers.push(NameValue::new(h[..c].trim(), h[c + 1..].trim()));
                    }
                    i += 1;
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-ascii" => {
                if let Some(n) = next {
                    data.push(Data::Raw(n.to_string()));
                    if method == "GET" {
                        method = "POST".into();
                    }
                    i += 1;
                }
            }
            "--data-urlencode" => {
                if let Some(n) = next {
                    let arg = n;
                    data.push(match arg.split_once('=') {
                        Some(("", v)) => Data::Encode(None, v.to_string()),
                        Some((k, v)) => Data::Encode(Some(k.to_string()), v.to_string()),
                        None => Data::Encode(None, arg.to_string()),
                    });
                    if method == "GET" {
                        method = "POST".into();
                    }
                    i += 1;
                }
            }
            "-F" | "--form" | "--form-string" => {
                if let Some(n) = next {
                    if let Some(f) = form_field(n, tok == "--form-string") {
                        form.push(f);
                    }
                    if method == "GET" {
                        method = "POST".into();
                    }
                    i += 1;
                }
            }
            "-k" | "--insecure" => options.verify_tls = Some(false),
            "-m" | "--max-time" => {
                if let Some(n) = next {
                    if let Ok(secs) = n.parse::<f64>() {
                        if secs > 0.0 {
                            options.timeout_ms = Some((secs * 1000.0).round() as u64);
                        }
                    }
                    i += 1;
                }
            }
            "-x" | "--proxy" => {
                if let Some(n) = next {
                    options.proxy = Some(n.to_string());
                    i += 1;
                }
            }
            "--url" => {
                if let Some(n) = next {
                    if url.is_empty() {
                        url = n.to_string();
                    }
                    i += 1;
                }
            }
            // Flags with an argument that does not affect the request definition.
            "-u" | "--user" | "-b" | "--cookie" | "-A" | "--user-agent" | "-o" | "--output"
            | "-w" | "--write-out" | "--connect-timeout" | "--retry" | "--retry-delay" => {
                if next.is_some() {
                    i += 1;
                }
            }
            // Known boolean flags.
            "-L" | "--location" | "-i" | "--include" | "-s" | "--silent" | "-S"
            | "--show-error" | "-v" | "--verbose" | "--compressed" => {}
            t if t.starts_with('-') => {
                // Unknown flag: skip its argument if it plausibly has one.
                if let Some(n) = next {
                    if !n.starts_with('-')
                        && !n.starts_with("http://")
                        && !n.starts_with("https://")
                        && !n.starts_with("'http")
                        && !n.starts_with("\"http")
                    {
                        i += 1;
                    }
                }
            }
            _ => {
                if url.is_empty() {
                    let raw = tok;
                    match raw.split_once(' ') {
                        Some((m, u)) if is_http_method(m) && u.starts_with("http") => {
                            method = m.to_string();
                            url = u.to_string();
                        }
                        _ => url = raw.to_string(),
                    }
                }
            }
        }
        i += 1;
    }

    if url.is_empty() {
        return Err(Error::Curl("no URL found".into()));
    }
    let body = if form.is_empty() {
        data_body(data, &headers)
    } else {
        Body::Multipart { fields: form }
    };
    Ok(ParsedCurl {
        name: endpoint_name(&url),
        params: query_params(&url),
        method,
        url,
        headers,
        body,
        options,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_escape_and_adjacent_quotes_join() {
        let p = parse(r#"curl 'http://a.b/c' -H 'X: it'\''s' -d "a"'b'c"#).unwrap();
        assert_eq!(p.headers, vec![NameValue::new("X", "it's")]);
        assert_eq!(
            p.body,
            Body::Raw {
                content: "abc".into(),
                content_type: Some("application/x-www-form-urlencoded".into())
            }
        );
        let p = parse("curl http://a.b/c -d ''").unwrap();
        assert_eq!(p.method, "POST");
    }

    #[test]
    fn simple_get() {
        let p = parse("curl http://api.example.com/users").unwrap();
        assert_eq!(p.method, "GET");
        assert_eq!(p.url, "http://api.example.com/users");
        assert_eq!(p.name, "users");
    }

    #[test]
    fn post_with_body_and_headers() {
        let p = parse(
            r#"curl -X POST 'http://api.example.com/data' -H 'Content-Type: application/json' -d '{"key":"value"}'"#,
        )
        .unwrap();
        assert_eq!(p.method, "POST");
        assert_eq!(
            p.body,
            Body::Json {
                content: r#"{"key":"value"}"#.into()
            }
        );
        assert_eq!(
            p.headers,
            vec![NameValue::new("Content-Type", "application/json")]
        );
    }

    #[test]
    fn data_implies_post() {
        let p = parse(r#"curl -d "key=value" http://api.example.com/submit"#).unwrap();
        assert_eq!(p.method, "POST");
        assert_eq!(
            p.body,
            Body::Raw {
                content: "key=value".into(),
                content_type: Some("application/x-www-form-urlencoded".into())
            }
        );
        let p = parse(r#"curl -X PUT -d 'x' http://a.b/c"#).unwrap();
        assert_eq!(p.method, "PUT");
    }

    #[test]
    fn query_params_extracted() {
        let p = parse("curl 'http://api.example.com/users?page=1&limit=10&flag'").unwrap();
        assert_eq!(
            p.params,
            vec![
                NameValue::new("page", "1"),
                NameValue::new("limit", "10"),
                NameValue::new("flag", "")
            ]
        );
    }

    #[test]
    fn ignored_flags() {
        let p = parse("curl -L -s -S --compressed --connect-timeout 10 http://api.example.com")
            .unwrap();
        assert_eq!(p.method, "GET");
        assert_eq!(p.url, "http://api.example.com");
        assert_eq!(p.name, "root");
        assert!(p.options.is_empty());
        assert_eq!(p.body, Body::None);
    }

    #[test]
    fn transport_flags_become_options() {
        let p = parse("curl -k --max-time 2.5 -x http://proxy:8080 https://h.io/x").unwrap();
        assert_eq!(p.options.verify_tls, Some(false));
        assert_eq!(p.options.timeout_ms, Some(2500));
        assert_eq!(p.options.proxy.as_deref(), Some("http://proxy:8080"));
        assert_eq!(p.url, "https://h.io/x");
    }

    #[test]
    fn json_body_is_detected_without_a_header() {
        let p = parse(r#"curl https://h.io -d '[1,2]'"#).unwrap();
        assert_eq!(
            p.body,
            Body::Json {
                content: "[1,2]".into()
            }
        );
    }

    #[test]
    fn explicit_content_type_keeps_raw_text() {
        let p = parse(r#"curl https://h.io -H 'Content-Type: text/xml' -d '<a/>'"#).unwrap();
        assert_eq!(
            p.body,
            Body::Raw {
                content: "<a/>".into(),
                content_type: None
            }
        );
    }

    #[test]
    fn data_urlencode_becomes_form_fields() {
        let p = parse(r#"curl https://h.io --data-urlencode 'q=a b' --data-urlencode 'n=1&2'"#)
            .unwrap();
        assert_eq!(p.method, "POST");
        assert_eq!(
            p.body,
            Body::Form {
                fields: vec![NameValue::new("q", "a b"), NameValue::new("n", "1&2")]
            }
        );
        // Mixed with raw data, it is encoded into one string like curl does.
        let p = parse(r#"curl https://h.io -d 'a=1' --data-urlencode 'b=x y'"#).unwrap();
        assert_eq!(
            p.body,
            Body::Raw {
                content: "a=1&b=x+y".into(),
                content_type: Some("application/x-www-form-urlencoded".into())
            }
        );
    }

    #[test]
    fn form_flags_become_multipart() {
        let p = parse(
            r#"curl https://h.io/up -F 'title=hi' -F 'file=@data/a.png;type=image/png' --form-string 'raw=@x'"#,
        )
        .unwrap();
        assert_eq!(p.method, "POST");
        assert_eq!(
            p.body,
            Body::Multipart {
                fields: vec![
                    NameValue::new("title", "hi"),
                    NameValue::file("file", "data/a.png"),
                    NameValue::new("raw", "@x"),
                ]
            }
        );
    }

    #[test]
    fn multiline_with_continuations() {
        let p =
            parse("curl -X DELETE \\\n  -H 'X-A: 1' \\\n  https://h.io/api/v2/items/1").unwrap();
        assert_eq!(p.method, "DELETE");
        assert_eq!(p.url, "https://h.io/api/v2/items/1");
        assert_eq!(p.name, "api v2 items 1");
    }

    #[test]
    fn unknown_flag_does_not_swallow_url() {
        let p = parse("curl --http1.1 https://h.io/x").unwrap();
        assert_eq!(p.url, "https://h.io/x");
    }

    #[test]
    fn inline_method_in_url_token() {
        let p = parse("curl 'POST http://h.io/x'").unwrap();
        assert_eq!(p.method, "POST");
        assert_eq!(p.url, "http://h.io/x");
    }

    #[test]
    fn errors() {
        assert!(parse("wget http://x").is_err());
        assert!(parse("").is_err());
        assert!(parse("curl -X POST").is_err());
        assert!(parse("curling http://x").is_err());
    }

    #[test]
    fn names() {
        assert_eq!(endpoint_name("http://h.io"), "root");
        assert_eq!(endpoint_name("http://h.io/"), "root");
        assert_eq!(endpoint_name("http://h.io/a-b/c_d?x=1"), "a b c d");
        assert_eq!(endpoint_name("nonsense"), "New Endpoint");
    }
}
