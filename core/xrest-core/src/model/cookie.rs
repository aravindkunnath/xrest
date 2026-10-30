//! `Set-Cookie` response cookies.

/// One cookie set by a response's `Set-Cookie` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseCookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<String>,
    pub max_age: Option<i64>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<String>,
}

/// Parse one `Set-Cookie` header value. Returns `None` for lines without `=` or with an
/// empty name.
pub fn parse_set_cookie(header_value: &str) -> Option<ResponseCookie> {
    let mut parts = header_value.split(';');
    let (name, value) = parts.next()?.split_once('=')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut cookie = ResponseCookie {
        name: name.to_string(),
        value: value.trim().to_string(),
        domain: None,
        path: None,
        expires: None,
        max_age: None,
        secure: false,
        http_only: false,
        same_site: None,
    };
    for part in parts {
        let part = part.trim();
        let (key, val) = match part.split_once('=') {
            Some((k, v)) => (k.trim(), Some(v.trim().to_string())),
            None => (part, None),
        };
        match key.to_ascii_lowercase().as_str() {
            "domain" => cookie.domain = val,
            "path" => cookie.path = val,
            "expires" => cookie.expires = val,
            "max-age" => cookie.max_age = val.and_then(|v| v.parse().ok()),
            "secure" => cookie.secure = true,
            "httponly" => cookie.http_only = true,
            "samesite" => cookie.same_site = val,
            _ => {}
        }
    }
    Some(cookie)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_attributes() {
        let c = parse_set_cookie(
            "sid=abc123; Domain=example.com; Path=/; Expires=Wed, 21 Oct 2026 07:28:00 GMT; Max-Age=3600; Secure; HttpOnly; SameSite=Lax",
        )
        .unwrap();
        assert_eq!(c.name, "sid");
        assert_eq!(c.value, "abc123");
        assert_eq!(c.domain.as_deref(), Some("example.com"));
        assert_eq!(c.path.as_deref(), Some("/"));
        assert_eq!(c.expires.as_deref(), Some("Wed, 21 Oct 2026 07:28:00 GMT"));
        assert_eq!(c.max_age, Some(3600));
        assert!(c.secure && c.http_only);
        assert_eq!(c.same_site.as_deref(), Some("Lax"));
    }

    #[test]
    fn value_keeps_equals_signs() {
        assert_eq!(parse_set_cookie("t=a=b==").unwrap().value, "a=b==");
    }

    #[test]
    fn attribute_names_are_case_insensitive() {
        let c = parse_set_cookie("a=1; HTTPONLY; secure; max-age=5").unwrap();
        assert!(c.http_only && c.secure);
        assert_eq!(c.max_age, Some(5));
    }

    #[test]
    fn malformed_lines_are_skipped() {
        for line in ["", "novalue", "=x"] {
            assert!(parse_set_cookie(line).is_none(), "{line:?}");
        }
    }

    #[test]
    fn bad_max_age_is_none() {
        assert_eq!(parse_set_cookie("a=1; Max-Age=soon").unwrap().max_age, None);
    }
}
