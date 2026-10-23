//! Field-level diff between two request configs (endpoint version comparison).

use crate::model::{Body, NameValue, Preflight, RequestConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Removed,
    Changed,
}

impl ChangeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ChangeKind::Added => "added",
            ChangeKind::Removed => "removed",
            ChangeKind::Changed => "changed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    pub field: String,
    pub change: ChangeKind,
    pub before: Option<String>,
    pub after: Option<String>,
}

type Rows<'a> = &'a [NameValue];

fn push(out: &mut Vec<FieldChange>, field: &str, before: Option<String>, after: Option<String>) {
    let change = match (&before, &after) {
        (None, None) => return,
        (Some(b), Some(a)) if b == a => return,
        (None, Some(_)) => ChangeKind::Added,
        (Some(_), None) => ChangeKind::Removed,
        _ => ChangeKind::Changed,
    };
    out.push(FieldChange {
        field: field.to_string(),
        change,
        before,
        after,
    });
}

/// Empty text counts as absent.
fn text(s: &str) -> Option<String> {
    (!s.is_empty()).then(|| s.to_string())
}

fn row_repr(r: &NameValue) -> String {
    let mut s = r.value.clone();
    if let Some(k) = r.secret() {
        s = format!("{s} (secret {k})");
    }
    if r.is_file() {
        s = format!("{s} (file)");
    }
    if !r.enabled {
        s = format!("{s} (disabled)");
    }
    s
}

/// Rows matched by name (duplicates pair up in order): a's rows first, then rows only in b.
fn diff_rows(out: &mut Vec<FieldChange>, prefix: &str, a: Rows, b: Rows) {
    let nth = |rows: Rows, i: usize| -> usize {
        rows[..i].iter().filter(|r| r.name == rows[i].name).count()
    };
    let find = |rows: Rows, name: &str, n: usize| -> Option<usize> {
        rows.iter()
            .enumerate()
            .filter(|(_, r)| r.name == name)
            .nth(n)
            .map(|(i, _)| i)
    };
    for (i, r) in a.iter().enumerate() {
        let after = find(b, &r.name, nth(a, i)).map(|j| row_repr(&b[j]));
        push(
            out,
            &format!("{prefix}{}", r.name),
            Some(row_repr(r)),
            after,
        );
    }
    for (j, r) in b.iter().enumerate() {
        if find(a, &r.name, nth(b, j)).is_none() {
            push(out, &format!("{prefix}{}", r.name), None, Some(row_repr(r)));
        }
    }
}

fn rows_text(rows: Rows) -> String {
    rows.iter()
        .map(|r| format!("{}={}", r.name, row_repr(r)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// (type label, content text, content type)
fn body_parts(b: &Body) -> (&'static str, String, Option<String>) {
    match b {
        Body::None => ("none", String::new(), None),
        Body::Raw {
            content,
            content_type,
        } => ("raw", content.clone(), content_type.clone()),
        Body::Json { content } => ("json", content.clone(), None),
        Body::Form { fields } => ("form", rows_text(fields), None),
        Body::Multipart { fields } => ("multipart", rows_text(fields), None),
        Body::Binary { path, content_type } => ("binary", path.clone(), content_type.clone()),
    }
}

fn diff_preflight(out: &mut Vec<FieldChange>, a: Option<&Preflight>, b: Option<&Preflight>) {
    if a.is_none() && b.is_none() {
        return;
    }
    let d = Preflight::default();
    let (pa, pb) = (a.unwrap_or(&d), b.unwrap_or(&d));
    // An absent preflight reads as all-false; don't report false defaults as additions.
    let boolean = |out: &mut Vec<FieldChange>, f: &str, x: bool, y: bool| {
        let side = |present: bool, v: bool| {
            (present && (v || (a.is_some() && b.is_some()))).then(|| v.to_string())
        };
        push(out, f, side(a.is_some(), x), side(b.is_some(), y));
    };
    let t = |o: Option<&Preflight>, v: &str| -> Option<String> { o.and_then(|_| text(v)) };
    boolean(out, "preflight.enabled", pa.enabled, pb.enabled);
    push(out, "preflight.method", t(a, &pa.method), t(b, &pb.method));
    push(out, "preflight.url", t(a, &pa.url), t(b, &pb.url));
    push(out, "preflight.body", t(a, &pa.body), t(b, &pb.body));
    push(
        out,
        "preflight.bodyType",
        t(a, &pa.body_type),
        t(b, &pb.body_type),
    );
    diff_rows(
        out,
        "preflight.bodyParam:",
        &pa.body_params,
        &pb.body_params,
    );
    diff_rows(out, "preflight.header:", &pa.headers, &pb.headers);
    boolean(out, "preflight.cacheToken", pa.cache_token, pb.cache_token);
    push(
        out,
        "preflight.cacheDuration",
        t(a, &pa.cache_duration),
        t(b, &pb.cache_duration),
    );
    push(
        out,
        "preflight.cacheDurationKey",
        t(a, &pa.cache_duration_key),
        t(b, &pb.cache_duration_key),
    );
    push(
        out,
        "preflight.cacheDurationUnit",
        t(a, &pa.cache_duration_unit),
        t(b, &pb.cache_duration_unit),
    );
    push(
        out,
        "preflight.tokenKey",
        t(a, &pa.token_key),
        t(b, &pb.token_key),
    );
    push(
        out,
        "preflight.tokenHeader",
        a.and_then(|p| p.token_header.clone()),
        b.and_then(|p| p.token_header.clone()),
    );
}

/// Field-level changes going from `a` to `b`, in a stable order: method, url, authenticated,
/// params, path params, headers, body, options, preflight.
pub fn diff_configs(a: &RequestConfig, b: &RequestConfig) -> Vec<FieldChange> {
    let mut out = Vec::new();
    push(
        &mut out,
        "method",
        Some(a.method.clone()),
        Some(b.method.clone()),
    );
    push(&mut out, "url", Some(a.url.clone()), Some(b.url.clone()));
    push(
        &mut out,
        "authenticated",
        Some(a.authenticated.to_string()),
        Some(b.authenticated.to_string()),
    );
    diff_rows(&mut out, "param:", &a.params, &b.params);
    diff_rows(&mut out, "pathParam:", &a.path_params, &b.path_params);
    diff_rows(&mut out, "header:", &a.headers, &b.headers);

    let (ta, ca, cta) = body_parts(&a.body);
    let (tb, cb, ctb) = body_parts(&b.body);
    push(&mut out, "body.type", Some(ta.into()), Some(tb.into()));
    push(&mut out, "body.content", text(&ca), text(&cb));
    push(&mut out, "body.contentType", cta, ctb);

    let (oa, ob) = (&a.options, &b.options);
    push(
        &mut out,
        "options.timeoutMs",
        oa.timeout_ms.map(|v| v.to_string()),
        ob.timeout_ms.map(|v| v.to_string()),
    );
    push(
        &mut out,
        "options.followRedirects",
        oa.follow_redirects.map(|v| v.to_string()),
        ob.follow_redirects.map(|v| v.to_string()),
    );
    push(
        &mut out,
        "options.verifyTls",
        oa.verify_tls.map(|v| v.to_string()),
        ob.verify_tls.map(|v| v.to_string()),
    );
    push(
        &mut out,
        "options.proxy",
        oa.proxy.clone(),
        ob.proxy.clone(),
    );

    diff_preflight(&mut out, a.preflight.as_ref(), b.preflight.as_ref());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn base() -> RequestConfig {
        RequestConfig {
            method: "GET".into(),
            url: "http://a.io/x".into(),
            authenticated: false,
            auth_type: "none".into(),
            params: vec![],
            path_params: vec![],
            headers: vec![],
            body: Body::None,
            options: Default::default(),
            preflight: None,
        }
    }

    fn ch(field: &str, change: ChangeKind, b: Option<&str>, a: Option<&str>) -> FieldChange {
        FieldChange {
            field: field.into(),
            change,
            before: b.map(Into::into),
            after: a.map(Into::into),
        }
    }

    #[test]
    fn identical_is_empty() {
        let mut c = base();
        c.headers = vec![NameValue::new("A", "1")];
        c.body = Body::Json {
            content: "{}".into(),
        };
        assert!(diff_configs(&c, &c.clone()).is_empty());
    }

    #[test]
    fn scalars() {
        let a = base();
        let mut b = base();
        b.method = "POST".into();
        b.url = "http://a.io/y".into();
        b.authenticated = true;
        assert_eq!(
            diff_configs(&a, &b),
            vec![
                ch("method", ChangeKind::Changed, Some("GET"), Some("POST")),
                ch(
                    "url",
                    ChangeKind::Changed,
                    Some("http://a.io/x"),
                    Some("http://a.io/y")
                ),
                ch(
                    "authenticated",
                    ChangeKind::Changed,
                    Some("false"),
                    Some("true")
                ),
            ]
        );
    }

    #[test]
    fn named_rows_added_removed_value_and_enabled() {
        let mut a = base();
        a.params = vec![NameValue::new("keep", "1"), NameValue::new("gone", "x")];
        a.headers = vec![NameValue::new("H", "1")];
        a.path_params = vec![NameValue::new("id", "1")];
        let mut b = base();
        let mut off = NameValue::new("keep", "1");
        off.enabled = false;
        b.params = vec![off, NameValue::new("new", "y")];
        b.headers = vec![NameValue::new("H", "2")];
        b.path_params = vec![NameValue::new("id", "2")];
        assert_eq!(
            diff_configs(&a, &b),
            vec![
                ch(
                    "param:keep",
                    ChangeKind::Changed,
                    Some("1"),
                    Some("1 (disabled)")
                ),
                ch("param:gone", ChangeKind::Removed, Some("x"), None),
                ch("param:new", ChangeKind::Added, None, Some("y")),
                ch("pathParam:id", ChangeKind::Changed, Some("1"), Some("2")),
                ch("header:H", ChangeKind::Changed, Some("1"), Some("2")),
            ]
        );
    }

    #[test]
    fn body_type_and_content() {
        let a = base();
        let mut b = base();
        b.body = Body::Json {
            content: "{\"a\":1}".into(),
        };
        assert_eq!(
            diff_configs(&a, &b),
            vec![
                ch("body.type", ChangeKind::Changed, Some("none"), Some("json")),
                ch("body.content", ChangeKind::Added, None, Some("{\"a\":1}")),
            ]
        );
        let mut c = base();
        c.body = Body::Json {
            content: "{\"a\":2}".into(),
        };
        assert_eq!(
            diff_configs(&b, &c),
            vec![ch(
                "body.content",
                ChangeKind::Changed,
                Some("{\"a\":1}"),
                Some("{\"a\":2}")
            )]
        );
        let mut f = base();
        f.body = Body::Form {
            fields: vec![NameValue::new("q", "1")],
        };
        let mut g = base();
        g.body = Body::Form {
            fields: vec![NameValue::new("q", "2")],
        };
        assert_eq!(
            diff_configs(&f, &g),
            vec![ch(
                "body.content",
                ChangeKind::Changed,
                Some("q=1"),
                Some("q=2")
            )]
        );
    }

    #[test]
    fn options_keys() {
        let a = base();
        let mut b = base();
        b.options = RequestOptions {
            timeout_ms: Some(5000),
            follow_redirects: Some(false),
            verify_tls: None,
            proxy: Some("http://p".into()),
        };
        assert_eq!(
            diff_configs(&a, &b),
            vec![
                ch("options.timeoutMs", ChangeKind::Added, None, Some("5000")),
                ch(
                    "options.followRedirects",
                    ChangeKind::Added,
                    None,
                    Some("false")
                ),
                ch("options.proxy", ChangeKind::Added, None, Some("http://p")),
            ]
        );
        assert_eq!(
            diff_configs(&b, &a)
                .iter()
                .map(|c| c.change)
                .collect::<Vec<_>>(),
            vec![ChangeKind::Removed; 3]
        );
    }

    #[test]
    fn preflight_fields() {
        let mut a = base();
        a.preflight = Some(Preflight {
            enabled: true,
            method: "POST".into(),
            url: "http://auth".into(),
            token_key: "access_token".into(),
            headers: vec![NameValue::new("X", "1")],
            ..Default::default()
        });
        let mut b = a.clone();
        let p = b.preflight.as_mut().unwrap();
        p.url = "http://auth2".into();
        p.cache_token = true;
        p.headers = vec![NameValue::new("X", "2")];
        assert_eq!(
            diff_configs(&a, &b),
            vec![
                ch(
                    "preflight.url",
                    ChangeKind::Changed,
                    Some("http://auth"),
                    Some("http://auth2")
                ),
                ch(
                    "preflight.header:X",
                    ChangeKind::Changed,
                    Some("1"),
                    Some("2")
                ),
                ch(
                    "preflight.cacheToken",
                    ChangeKind::Changed,
                    Some("false"),
                    Some("true")
                ),
            ]
        );
        // Whole preflight added: every populated field shows up as added.
        let none = base();
        let d = diff_configs(&none, &a);
        assert!(d.iter().all(|c| c.change == ChangeKind::Added));
        assert!(d.iter().any(|c| c.field == "preflight.url"));
        assert!(d.iter().any(|c| c.field == "preflight.header:X"));
    }
}
