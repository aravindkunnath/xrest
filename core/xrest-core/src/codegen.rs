//! Code export: render a resolved request as a cURL command or as client code.
//!
//! Resolution (see `resolve::resolve_for_export`) does no I/O, so file fields appear as paths
//! and the generated code reads them when it runs.

use std::collections::BTreeSet;

use url::Url;

use crate::model::RequestOptions;

/// Target of [`crate::Workspace::export_code`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeLanguage {
    Curl,
    /// Node.js, `axios`.
    Axios,
    /// `java.net.http.HttpClient`.
    Java,
    /// Kotlin, OkHttp.
    Kotlin,
}

#[derive(Debug, Clone)]
pub(crate) struct ExportRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: ExportBody,
    pub options: RequestOptions,
}

#[derive(Debug, Clone)]
pub(crate) enum ExportBody {
    Empty,
    Text(String),
    Form(Vec<(String, String)>),
    Multipart(Vec<ExportPart>),
    /// The bytes of the file at this path.
    File(String),
}

#[derive(Debug, Clone)]
pub(crate) enum ExportPart {
    Text { name: String, value: String },
    File { name: String, path: String },
}

pub(crate) fn generate(req: &ExportRequest, lang: CodeLanguage) -> String {
    match lang {
        CodeLanguage::Curl => curl(req),
        CodeLanguage::Axios => axios(req),
        CodeLanguage::Java => java(req),
        CodeLanguage::Kotlin => kotlin(req),
    }
}

// ---- helpers --------------------------------------------------------------------------------

fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Shell word that is only quoted when it needs to be (plain letters stay readable).
fn sh_arg(s: &str) -> String {
    if !s.is_empty() && s.bytes().all(|b| b.is_ascii_alphabetic()) {
        s.to_string()
    } else {
        sh_quote(s)
    }
}

/// Text for a `//` comment: control characters (newlines included) become spaces so the value
/// cannot end the comment and inject code.
fn comment_text(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}') {
                ' '
            } else {
                c
            }
        })
        .collect()
}

fn js_str(s: &str) -> String {
    let mut out = String::from("'");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' | '\u{2029}' | '\0'..='\u{1f}' => {
                out.push_str(&format!("\\u{:04x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// A double-quoted Java/Kotlin string literal (`dollar`: escape `$` for Kotlin templates).
fn dq_str(s: &str, dollar: bool) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '$' if dollar => out.push_str("\\$"),
            '\0'..='\u{1f}' | '\u{2028}' | '\u{2029}' => {
                out.push_str(&format!("\\u{:04x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn java_str(s: &str) -> String {
    dq_str(s, false)
}

fn kt_str(s: &str) -> String {
    dq_str(s, true)
}

fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn secs(ms: u64) -> String {
    if ms % 1000 == 0 {
        (ms / 1000).to_string()
    } else {
        (ms as f64 / 1000.0).to_string()
    }
}

fn content_type(req: &ExportRequest) -> Option<&str> {
    req.headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-type"))
        .map(|(_, v)| v.as_str())
}

struct Proxy {
    scheme: String,
    host: String,
    port: u16,
    auth: Option<(String, String)>,
}

fn parse_proxy(p: &str) -> Option<Proxy> {
    let u = Url::parse(p).ok()?;
    let host = u.host_str()?.to_string();
    let port = u.port_or_known_default()?;
    let auth = (!u.username().is_empty()).then(|| {
        (
            u.username().to_string(),
            u.password().unwrap_or_default().to_string(),
        )
    });
    Some(Proxy {
        scheme: u.scheme().to_string(),
        host,
        port,
        auth,
    })
}

// ---- curl -----------------------------------------------------------------------------------

fn curl(req: &ExportRequest) -> String {
    let mut first = String::from("curl");
    if req.method != "GET" || !matches!(req.body, ExportBody::Empty) {
        if req.method != "GET" {
            first.push_str(&format!(" -X {}", sh_arg(&req.method)));
        } else {
            first.push_str(" -X GET");
        }
    }
    first.push(' ');
    first.push_str(&sh_quote(&req.url));
    let mut parts = vec![first];
    for (k, v) in &req.headers {
        let default_form = matches!(req.body, ExportBody::Form(_))
            && k.eq_ignore_ascii_case("content-type")
            && v.eq_ignore_ascii_case("application/x-www-form-urlencoded");
        if !default_form {
            parts.push(format!("-H {}", sh_quote(&format!("{k}: {v}"))));
        }
    }
    let o = &req.options;
    if o.verify_tls == Some(false) {
        parts.push("-k".into());
    }
    if o.follow_redirects == Some(true) {
        parts.push("-L".into());
    }
    if let Some(ms) = o.timeout_ms.filter(|ms| *ms > 0) {
        parts.push(format!("-m {}", secs(ms)));
    }
    if let Some(p) = &o.proxy {
        parts.push(format!("-x {}", sh_quote(p)));
    }
    match &req.body {
        ExportBody::Empty => {}
        ExportBody::Text(c) => parts.push(format!("--data-raw {}", sh_quote(c))),
        ExportBody::Form(fields) => {
            for (k, v) in fields {
                parts.push(format!(
                    "--data-urlencode {}",
                    sh_quote(&format!("{k}={v}"))
                ));
            }
        }
        ExportBody::Multipart(ps) => {
            for p in ps {
                parts.push(match p {
                    ExportPart::File { name, path } => {
                        format!("-F {}", sh_quote(&format!("{name}=@{path}")))
                    }
                    ExportPart::Text { name, value } => {
                        // `-F` would read `@file`/`<file` and parse `;type=`.
                        let flag = if value.starts_with(['@', '<']) || value.contains(';') {
                            "--form-string"
                        } else {
                            "-F"
                        };
                        format!("{flag} {}", sh_quote(&format!("{name}={value}")))
                    }
                });
            }
        }
        ExportBody::File(path) => {
            parts.push(format!("--data-binary {}", sh_quote(&format!("@{path}"))))
        }
    }
    parts.join(" \\\n  ")
}

// ---- axios ----------------------------------------------------------------------------------

fn axios(req: &ExportRequest) -> String {
    let o = &req.options;
    let mut requires = vec!["const axios = require('axios');".to_string()];
    let mut pre = String::new();
    let mut cfg = format!(
        "  method: {},\n  url: {},\n",
        js_str(&req.method.to_ascii_lowercase()),
        js_str(&req.url)
    );
    let multipart = matches!(req.body, ExportBody::Multipart(_));
    if !req.headers.is_empty() || multipart {
        cfg.push_str("  headers: {\n");
        if multipart {
            cfg.push_str("    ...form.getHeaders(),\n");
        }
        for (k, v) in &req.headers {
            cfg.push_str(&format!("    {}: {},\n", js_str(k), js_str(v)));
        }
        cfg.push_str("  },\n");
    }
    match &req.body {
        ExportBody::Empty => {}
        ExportBody::Text(c) => cfg.push_str(&format!("  data: {},\n", js_str(c))),
        ExportBody::Form(fields) => {
            let rows: Vec<String> = fields
                .iter()
                .map(|(k, v)| format!("[{}, {}]", js_str(k), js_str(v)))
                .collect();
            cfg.push_str(&format!(
                "  data: new URLSearchParams([{}]),\n",
                rows.join(", ")
            ));
        }
        ExportBody::Multipart(parts) => {
            requires.push("const FormData = require('form-data');".into());
            pre.push_str("const form = new FormData();\n");
            for p in parts {
                match p {
                    ExportPart::Text { name, value } => pre.push_str(&format!(
                        "form.append({}, {});\n",
                        js_str(name),
                        js_str(value)
                    )),
                    ExportPart::File { name, path } => {
                        if !requires.iter().any(|r| r.contains("'fs'")) {
                            requires.push("const fs = require('fs');".into());
                        }
                        pre.push_str(&format!(
                            "form.append({}, fs.createReadStream({}));\n",
                            js_str(name),
                            js_str(path)
                        ))
                    }
                }
            }
            pre.push('\n');
            cfg.push_str("  data: form,\n");
        }
        ExportBody::File(path) => {
            requires.push("const fs = require('fs');".into());
            cfg.push_str(&format!("  data: fs.readFileSync({}),\n", js_str(path)));
        }
    }
    if let Some(ms) = o.timeout_ms.filter(|ms| *ms > 0) {
        cfg.push_str(&format!("  timeout: {ms},\n"));
    }
    if o.follow_redirects == Some(false) {
        cfg.push_str("  maxRedirects: 0,\n");
    }
    if o.verify_tls == Some(false) {
        requires.push("const https = require('https');".into());
        cfg.push_str("  httpsAgent: new https.Agent({ rejectUnauthorized: false }),\n");
    }
    if let Some(p) = &o.proxy {
        if p.is_empty() {
            cfg.push_str("  proxy: false,\n");
        } else if let Some(px) = parse_proxy(p) {
            cfg.push_str(&format!(
                "  proxy: {{\n    protocol: {},\n    host: {},\n    port: {},\n",
                js_str(&px.scheme),
                js_str(&px.host),
                px.port
            ));
            if let Some((u, pw)) = px.auth {
                cfg.push_str(&format!(
                    "    auth: {{ username: {}, password: {} }},\n",
                    js_str(&u),
                    js_str(&pw)
                ));
            }
            cfg.push_str("  },\n");
        } else {
            cfg.push_str(&format!(
                "  // proxy {} could not be parsed; configure it by hand.\n",
                comment_text(p)
            ));
        }
    }
    format!(
        "{}\n\n{pre}axios({{\n{cfg}}})\n  .then((response) => console.log(response.data))\n  .catch((error) => console.error(error));\n",
        requires.join("\n")
    )
}

// ---- java -----------------------------------------------------------------------------------

fn java(req: &ExportRequest) -> String {
    let o = &req.options;
    let mut imports: BTreeSet<&str> = [
        "java.net.URI",
        "java.net.http.HttpClient",
        "java.net.http.HttpRequest",
        "java.net.http.HttpResponse",
    ]
    .into();
    let mut pre = String::new();
    let mut headers = req.headers.clone();

    // client
    let mut client_opts = vec![];
    match o.follow_redirects {
        Some(true) => client_opts.push("HttpClient.Redirect.ALWAYS".to_string()),
        Some(false) => client_opts.push("HttpClient.Redirect.NEVER".to_string()),
        None => {}
    }
    let mut proxy_comment = None;
    match o.proxy.as_deref() {
        Some("") => client_opts.push("PROXY_NONE".into()),
        Some(p) => match parse_proxy(p) {
            Some(px) => {
                imports.insert("java.net.InetSocketAddress");
                imports.insert("java.net.ProxySelector");
                client_opts.push(format!(
                    "PROXY:.proxy(ProxySelector.of(new InetSocketAddress({}, {})))",
                    java_str(&px.host),
                    px.port
                ));
            }
            None => proxy_comment = Some(p.to_string()),
        },
        None => {}
    }
    let mut client_lines: Vec<String> = vec![];
    for opt in &client_opts {
        if opt == "PROXY_NONE" {
            client_lines.push(".proxy(HttpClient.Builder.NO_PROXY)".into());
        } else if let Some(l) = opt.strip_prefix("PROXY:") {
            client_lines.push(l.to_string());
        } else {
            client_lines.push(format!(".followRedirects({opt})"));
        }
    }
    let mut notes = String::new();
    if o.verify_tls == Some(false) {
        notes.push_str("        // XRest skips TLS verification for this request. HttpClient has no switch for it:\n        // configure an SSLContext that trusts all certificates to match (not recommended).\n");
    }
    if let Some(p) = proxy_comment {
        notes.push_str(&format!(
            "        // Proxy {} could not be parsed; configure it by hand.\n",
            comment_text(&p)
        ));
    }
    let client = if client_lines.is_empty() {
        "HttpClient client = HttpClient.newHttpClient();\n".to_string()
    } else {
        format!(
            "HttpClient client = HttpClient.newBuilder()\n            {}\n            .build();\n",
            client_lines.join("\n            ")
        )
    };

    // body
    let publisher = match &req.body {
        ExportBody::Empty => {
            (req.method != "GET").then(|| "HttpRequest.BodyPublishers.noBody()".to_string())
        }
        ExportBody::Text(c) => Some(format!(
            "HttpRequest.BodyPublishers.ofString({})",
            java_str(c)
        )),
        ExportBody::Form(fields) => {
            imports.insert("java.net.URLEncoder");
            imports.insert("java.nio.charset.StandardCharsets");
            let rows: Vec<String> = fields
                .iter()
                .map(|(k, v)| {
                    format!(
                        "URLEncoder.encode({}, StandardCharsets.UTF_8) + \"=\" + URLEncoder.encode({}, StandardCharsets.UTF_8)",
                        java_str(k),
                        java_str(v)
                    )
                })
                .collect();
            pre.push_str(&format!(
                "        String form = String.join(\"&\"{});\n",
                rows.iter()
                    .map(|r| format!(",\n            {r}"))
                    .collect::<String>()
            ));
            Some("HttpRequest.BodyPublishers.ofString(form)".to_string())
        }
        ExportBody::Multipart(parts) => {
            imports.insert("java.io.ByteArrayOutputStream");
            imports.insert("java.nio.charset.StandardCharsets");
            pre.push_str(
                "        String boundary = \"----XRestBoundary\" + System.currentTimeMillis();\n",
            );
            pre.push_str("        ByteArrayOutputStream out = new ByteArrayOutputStream();\n");
            for p in parts {
                match p {
                    ExportPart::Text { name, value } => {
                        let disp = format!("Content-Disposition: form-data; name=\"{name}\"");
                        pre.push_str(&format!(
                            "        out.write((\"--\" + boundary + \"\\r\\n\" + {} + \"\\r\\n\\r\\n\" + {} + \"\\r\\n\").getBytes(StandardCharsets.UTF_8));\n",
                            java_str(&disp),
                            java_str(value)
                        ));
                    }
                    ExportPart::File { name, path } => {
                        imports.insert("java.nio.file.Files");
                        imports.insert("java.nio.file.Path");
                        let disp = format!(
                            "Content-Disposition: form-data; name=\"{name}\"; filename=\"{}\"",
                            file_name(path)
                        );
                        pre.push_str(&format!(
                            "        out.write((\"--\" + boundary + \"\\r\\n\" + {} + \"\\r\\nContent-Type: application/octet-stream\\r\\n\\r\\n\").getBytes(StandardCharsets.UTF_8));\n",
                            java_str(&disp)
                        ));
                        pre.push_str(&format!(
                            "        out.write(Files.readAllBytes(Path.of({})));\n        out.write(\"\\r\\n\".getBytes(StandardCharsets.UTF_8));\n",
                            java_str(path)
                        ));
                    }
                }
            }
            pre.push_str("        out.write((\"--\" + boundary + \"--\\r\\n\").getBytes(StandardCharsets.UTF_8));\n");
            Some("HttpRequest.BodyPublishers.ofByteArray(out.toByteArray())".to_string())
        }
        ExportBody::File(path) => {
            imports.insert("java.nio.file.Path");
            Some(format!(
                "HttpRequest.BodyPublishers.ofFile(Path.of({}))",
                java_str(path)
            ))
        }
    };

    let mut b = String::from("HttpRequest request = HttpRequest.newBuilder()\n");
    b.push_str(&format!(
        "            .uri(URI.create({}))\n",
        java_str(&req.url)
    ));
    if let Some(ms) = o.timeout_ms.filter(|ms| *ms > 0) {
        imports.insert("java.time.Duration");
        b.push_str(&format!("            .timeout(Duration.ofMillis({ms}))\n"));
    }
    for (k, v) in headers.drain(..) {
        b.push_str(&format!(
            "            .header({}, {})\n",
            java_str(&k),
            java_str(&v)
        ));
    }
    if matches!(req.body, ExportBody::Multipart(_)) {
        b.push_str("            .header(\"Content-Type\", \"multipart/form-data; boundary=\" + boundary)\n");
    }
    match publisher {
        Some(p) => b.push_str(&format!(
            "            .method({}, {p})\n",
            java_str(&req.method)
        )),
        None => b.push_str("            .GET()\n"),
    }
    b.push_str("            .build();\n");

    let imports: String = imports.iter().map(|i| format!("import {i};\n")).collect();
    format!(
        "{imports}\npublic class Main {{\n    public static void main(String[] args) throws Exception {{\n{notes}        {client}{pre}        {b}        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());\n        System.out.println(response.body());\n    }}\n}}\n"
    )
}

// ---- kotlin ---------------------------------------------------------------------------------

fn kotlin(req: &ExportRequest) -> String {
    let o = &req.options;
    let mut imports: BTreeSet<&str> = ["okhttp3.OkHttpClient", "okhttp3.Request"].into();
    let ct = content_type(req).map(str::to_string);
    let has_body = !matches!(req.body, ExportBody::Empty);

    // client
    let mut opts = vec![];
    if let Some(ms) = o.timeout_ms.filter(|ms| *ms > 0) {
        imports.insert("java.time.Duration");
        opts.push(format!(".callTimeout(Duration.ofMillis({ms}))"));
    }
    if let Some(f) = o.follow_redirects {
        opts.push(format!(".followRedirects({f})"));
        opts.push(format!(".followSslRedirects({f})"));
    }
    let mut notes = String::new();
    match o.proxy.as_deref() {
        Some("") => {
            imports.insert("java.net.Proxy");
            opts.push(".proxy(Proxy.NO_PROXY)".into());
        }
        Some(p) => match parse_proxy(p) {
            Some(px) => {
                imports.insert("java.net.InetSocketAddress");
                imports.insert("java.net.Proxy");
                opts.push(format!(
                    ".proxy(Proxy(Proxy.Type.HTTP, InetSocketAddress({}, {})))",
                    kt_str(&px.host),
                    px.port
                ));
            }
            None => notes.push_str(&format!(
                "    // Proxy {} could not be parsed; configure it by hand.\n",
                comment_text(p)
            )),
        },
        None => {}
    }
    if o.verify_tls == Some(false) {
        notes.push_str("    // XRest skips TLS verification for this request. OkHttp has no switch for it:\n    // build an OkHttpClient with an SSLSocketFactory and HostnameVerifier that trust everything (not recommended).\n");
    }
    let client = if opts.is_empty() {
        "val client = OkHttpClient()\n".to_string()
    } else {
        format!(
            "val client = OkHttpClient.Builder()\n        {}\n        .build()\n",
            opts.join("\n        ")
        )
    };

    // body
    let mut body = String::new();
    let media = ct
        .as_deref()
        .map(|c| format!("{}.toMediaType()", kt_str(c)));
    if media.is_some()
        && !matches!(
            req.body,
            ExportBody::Empty | ExportBody::Multipart(_) | ExportBody::Form(_)
        )
    {
        imports.insert("okhttp3.MediaType.Companion.toMediaType");
    }
    match &req.body {
        ExportBody::Empty => {}
        ExportBody::Text(c) => {
            imports.insert("okhttp3.RequestBody.Companion.toRequestBody");
            body = format!(
                "val body = {}.toRequestBody({})\n",
                kt_str(c),
                media.clone().unwrap_or_else(|| "null".into())
            );
        }
        ExportBody::Form(fields) => {
            imports.insert("okhttp3.FormBody");
            let adds: String = fields
                .iter()
                .map(|(k, v)| format!("        .add({}, {})\n", kt_str(k), kt_str(v)))
                .collect();
            body = format!("val body = FormBody.Builder()\n{adds}        .build()\n");
        }
        ExportBody::Multipart(parts) => {
            imports.insert("okhttp3.MultipartBody");
            let mut adds = String::new();
            for p in parts {
                match p {
                    ExportPart::Text { name, value } => adds.push_str(&format!(
                        "        .addFormDataPart({}, {})\n",
                        kt_str(name),
                        kt_str(value)
                    )),
                    ExportPart::File { name, path } => {
                        imports.insert("java.io.File");
                        imports.insert("okhttp3.RequestBody.Companion.asRequestBody");
                        adds.push_str(&format!(
                            "        .addFormDataPart({}, {}, File({}).asRequestBody())\n",
                            kt_str(name),
                            kt_str(file_name(path)),
                            kt_str(path)
                        ));
                    }
                }
            }
            body = format!(
                "val body = MultipartBody.Builder()\n        .setType(MultipartBody.FORM)\n{adds}        .build()\n"
            );
        }
        ExportBody::File(path) => {
            imports.insert("java.io.File");
            imports.insert("okhttp3.RequestBody.Companion.asRequestBody");
            body = format!(
                "val body = File({}).asRequestBody({})\n",
                kt_str(path),
                media.clone().unwrap_or_else(|| "null".into())
            );
        }
    }

    let mut b = String::from("val request = Request.Builder()\n");
    b.push_str(&format!("        .url({})\n", kt_str(&req.url)));
    for (k, v) in &req.headers {
        if has_body && k.eq_ignore_ascii_case("content-type") {
            continue; // carried by the body's media type
        }
        b.push_str(&format!(
            "        .addHeader({}, {})\n",
            kt_str(k),
            kt_str(v)
        ));
    }
    if has_body {
        b.push_str(&format!("        .method({}, body)\n", kt_str(&req.method)));
    } else if req.method == "GET" {
        b.push_str("        .get()\n");
    } else if matches!(req.method.as_str(), "POST" | "PUT" | "PATCH") {
        imports.insert("okhttp3.RequestBody.Companion.toRequestBody");
        b.push_str(&format!(
            "        .method({}, ByteArray(0).toRequestBody())\n",
            kt_str(&req.method)
        ));
    } else {
        b.push_str(&format!("        .method({}, null)\n", kt_str(&req.method)));
    }
    b.push_str("        .build()\n");

    let imports: String = imports.iter().map(|i| format!("import {i}\n")).collect();
    let body = if body.is_empty() {
        String::new()
    } else {
        format!("    {body}")
    };
    format!(
        "{imports}\nfun main() {{\n{notes}    {client}{body}    {b}    client.newCall(request).execute().use {{ response ->\n        println(response.body?.string())\n    }}\n}}\n"
    )
}
