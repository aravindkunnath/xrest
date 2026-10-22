//! cURL command import, ported from `internal/import/curl.go`.
//!
//! Differences from the Go version: any whitespace (including the newlines of `\`-continued
//! commands) separates tokens, and an unknown flag never swallows a following `http(s)://` URL.
//! As in Go, `-u/--user`, `-b/--cookie` and `-A/--user-agent` are ignored: credentials must not
//! silently end up in files that are committed to git.
//! TODO(curl-auth): offer to map `-u user:pass` to service-level basic auth.

use crate::error::{Error, Result};
use crate::model::NameValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCurl {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<NameValue>,
    pub body: String,
    pub params: Vec<NameValue>,
}

fn is_http_method(s: &str) -> bool {
    matches!(
        s,
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "CONNECT" | "TRACE"
    )
}

/// Split a shell-like command into tokens. Quote characters are kept (see [`unquote`]).
fn tokenize(cmd: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut cur = String::new();
    let (mut in_single, mut in_double, mut escape) = (false, false, false);
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
            cur.push(ch);
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            cur.push(ch);
            continue;
        }
        if ch.is_whitespace() && !in_single && !in_double {
            if !cur.is_empty() {
                tokens.push(std::mem::take(&mut cur));
            }
            continue;
        }
        cur.push(ch);
    }
    if !cur.is_empty() {
        tokens.push(cur);
    }
    tokens
}

fn unquote(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 2
        && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\''))
    {
        return &s[1..s.len() - 1];
    }
    s
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
    let mut body = String::new();

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
                    let h = unquote(n);
                    if let Some(c) = h.find(':').filter(|c| *c > 0) {
                        headers.push(NameValue::new(h[..c].trim(), h[c + 1..].trim()));
                    }
                    i += 1;
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" | "--data-urlencode" => {
                if let Some(n) = next {
                    body = unquote(n).to_string();
                    if method == "GET" {
                        method = "POST".into();
                    }
                    i += 1;
                }
            }
            "--url" => {
                if let Some(n) = next {
                    if url.is_empty() {
                        url = unquote(n).to_string();
                    }
                    i += 1;
                }
            }
            // Flags with an argument that does not affect the request definition.
            "-u" | "--user" | "-b" | "--cookie" | "-A" | "--user-agent" | "-o" | "--output"
            | "-w" | "--write-out" | "--connect-timeout" | "--max-time" | "--retry"
            | "--retry-delay" => {
                if next.is_some() {
                    i += 1;
                }
            }
            // Known boolean flags.
            "-k" | "--insecure" | "-L" | "--location" | "-i" | "--include" | "-s" | "--silent"
            | "-S" | "--show-error" | "-v" | "--verbose" | "--compressed" => {}
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
                    let raw = unquote(tok);
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
    Ok(ParsedCurl {
        name: endpoint_name(&url),
        params: query_params(&url),
        method,
        url,
        headers,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(p.body, r#"{"key":"value"}"#);
        assert_eq!(
            p.headers,
            vec![NameValue::new("Content-Type", "application/json")]
        );
    }

    #[test]
    fn data_implies_post() {
        let p = parse(r#"curl -d "key=value" http://api.example.com/submit"#).unwrap();
        assert_eq!(p.method, "POST");
        assert_eq!(p.body, "key=value");
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
        let p = parse("curl -k -L -s -S --compressed --connect-timeout 10 --max-time 30 http://api.example.com")
            .unwrap();
        assert_eq!(p.method, "GET");
        assert_eq!(p.url, "http://api.example.com");
        assert_eq!(p.name, "root");
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
