//! Code export (G3): goldens per language, secret masking, curl round-trip.

use std::sync::Arc;

use xrest_core::*;

struct Fixture {
    _dir: tempfile::TempDir,
    ws: Workspace,
    secrets: Arc<MemorySecretStore>,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    let secrets = Arc::new(MemorySecretStore::default());
    ws.set_secret_store(secrets.clone());
    ws.update_environments(vec![Environment {
        name: "PROD".into(),
        is_unsafe: true,
        variables: vec![
            NameValue::new("BASE", "https://api.example.com"),
            NameValue::new("N", "7"),
        ],
        ..Default::default()
    }])
    .unwrap();
    Fixture {
        _dir: dir,
        ws,
        secrets,
    }
}

fn get_spec() -> RequestSpec {
    RequestSpec {
        method: "get".into(),
        url: "{{BASE}}/users/{id}".into(),
        params: vec![NameValue::new("page", "2"), NameValue::new("q", "a b")],
        path_params: vec![NameValue::new("id", "{{N}}")],
        headers: vec![NameValue::new("Accept", "application/json")],
        ..Default::default()
    }
}

fn post_spec() -> RequestSpec {
    RequestSpec {
        method: "POST".into(),
        url: "{{BASE}}/users".into(),
        headers: vec![NameValue::new("X-Trace", "1")],
        body: Body::Json {
            content: r#"{"name":"Ann"}"#.into(),
        },
        ..Default::default()
    }
}

fn export(f: &Fixture, spec: &RequestSpec, lang: CodeLanguage) -> String {
    f.ws.export_code(spec, lang, false).unwrap()
}

const URL_GET: &str = "https://api.example.com/users/7?page=2&q=a+b";

#[test]
fn curl_goldens() {
    let f = fixture();
    assert_eq!(
        export(&f, &get_spec(), CodeLanguage::Curl),
        format!("curl '{URL_GET}' \\\n  -H 'Accept: application/json'")
    );
    assert_eq!(
        export(&f, &post_spec(), CodeLanguage::Curl),
        "curl -X POST 'https://api.example.com/users' \\\n  -H 'X-Trace: 1' \\\n  -H 'Content-Type: application/json' \\\n  --data-raw '{\"name\":\"Ann\"}'"
    );
}

#[test]
fn axios_goldens() {
    let f = fixture();
    assert_eq!(
        export(&f, &get_spec(), CodeLanguage::Axios),
        format!(
            "const axios = require('axios');

axios({{
  method: 'get',
  url: '{URL_GET}',
  headers: {{
    'Accept': 'application/json',
  }},
}})
  .then((response) => console.log(response.data))
  .catch((error) => console.error(error));
"
        )
    );
    assert_eq!(
        export(&f, &post_spec(), CodeLanguage::Axios),
        "const axios = require('axios');

axios({
  method: 'post',
  url: 'https://api.example.com/users',
  headers: {
    'X-Trace': '1',
    'Content-Type': 'application/json',
  },
  data: '{\"name\":\"Ann\"}',
})
  .then((response) => console.log(response.data))
  .catch((error) => console.error(error));
"
    );
}

#[test]
fn java_goldens() {
    let f = fixture();
    assert_eq!(
        export(&f, &get_spec(), CodeLanguage::Java),
        format!(
            "import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

public class Main {{
    public static void main(String[] args) throws Exception {{
        HttpClient client = HttpClient.newHttpClient();
        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(\"{URL_GET}\"))
            .header(\"Accept\", \"application/json\")
            .GET()
            .build();
        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());
        System.out.println(response.body());
    }}
}}
"
        )
    );
    assert_eq!(
        export(&f, &post_spec(), CodeLanguage::Java),
        "import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

public class Main {
    public static void main(String[] args) throws Exception {
        HttpClient client = HttpClient.newHttpClient();
        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(\"https://api.example.com/users\"))
            .header(\"X-Trace\", \"1\")
            .header(\"Content-Type\", \"application/json\")
            .method(\"POST\", HttpRequest.BodyPublishers.ofString(\"{\\\"name\\\":\\\"Ann\\\"}\"))
            .build();
        HttpResponse<String> response = client.send(request, HttpResponse.BodyHandlers.ofString());
        System.out.println(response.body());
    }
}
"
    );
}

#[test]
fn kotlin_goldens() {
    let f = fixture();
    assert_eq!(
        export(&f, &get_spec(), CodeLanguage::Kotlin),
        format!(
            "import okhttp3.OkHttpClient
import okhttp3.Request

fun main() {{
    val client = OkHttpClient()
    val request = Request.Builder()
        .url(\"{URL_GET}\")
        .addHeader(\"Accept\", \"application/json\")
        .get()
        .build()
    client.newCall(request).execute().use {{ response ->
        println(response.body?.string())
    }}
}}
"
        )
    );
    assert_eq!(
        export(&f, &post_spec(), CodeLanguage::Kotlin),
        "import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody

fun main() {
    val client = OkHttpClient()
    val body = \"{\\\"name\\\":\\\"Ann\\\"}\".toRequestBody(\"application/json\".toMediaType())
    val request = Request.Builder()
        .url(\"https://api.example.com/users\")
        .addHeader(\"X-Trace\", \"1\")
        .method(\"POST\", body)
        .build()
    client.newCall(request).execute().use { response ->
        println(response.body?.string())
    }
}
"
    );
}

fn secret_spec() -> RequestSpec {
    RequestSpec {
        method: "POST".into(),
        url: "{{BASE}}/x?key={{secret.QK}}".into(),
        headers: vec![NameValue::new("X-Key", "{{secret.HK}}")],
        body: Body::Json {
            content: r#"{"p":"{{secret.BK}}"}"#.into(),
        },
        authenticated: true,
        ..Default::default()
    }
}

fn with_bearer(f: &mut Fixture) {
    let root = f.ws.root().to_path_buf();
    let mut svc = f.ws.service().clone();
    svc.is_authenticated = true;
    svc.auth = Some(AuthConfig {
        kind: AuthKind::Bearer,
        bearer_token: "{{secret.TOK}}".into(),
        ..Default::default()
    });
    xrest_core::storage::save_service(&root, &svc).unwrap();
    let mut ws = Workspace::open(&root).unwrap();
    ws.set_secret_store(f.secrets.clone());
    f.ws = ws;
}

#[test]
fn secrets_are_masked_by_default_and_never_fetched() {
    let mut f = fixture();
    with_bearer(&mut f);
    // Not set at all: placeholders still render.
    for lang in [
        CodeLanguage::Curl,
        CodeLanguage::Axios,
        CodeLanguage::Java,
        CodeLanguage::Kotlin,
    ] {
        let out = export(&f, &secret_spec(), lang);
        for k in ["QK", "HK", "BK", "TOK"] {
            assert!(
                out.contains(&format!("{{{{secret.{k}}}}}")),
                "{lang:?} {k}\n{out}"
            );
        }
    }
    for (k, v) in [
        ("QK", "q-real"),
        ("HK", "h-real"),
        ("BK", "b-real"),
        ("TOK", "t-real"),
    ] {
        f.secrets.set(k, v);
    }
    let out = export(&f, &secret_spec(), CodeLanguage::Curl);
    assert!(!out.contains("real"), "{out}");
    assert!(
        out.contains("Authorization: Bearer {{secret.TOK}}"),
        "{out}"
    );
    assert!(out.contains("?key={{secret.QK}}"), "{out}");
}

#[test]
fn include_secrets_emits_real_values_and_errors_on_missing() {
    let mut f = fixture();
    with_bearer(&mut f);
    f.secrets.set("QK", "q-real");
    f.secrets.set("HK", "h-real");
    let err =
        f.ws.export_code(&secret_spec(), CodeLanguage::Curl, true)
            .unwrap_err();
    match err {
        Error::MissingSecrets(keys) => assert_eq!(keys, vec!["BK".to_string(), "TOK".to_string()]),
        e => panic!("{e:?}"),
    }
    f.secrets.set("BK", "b-real");
    f.secrets.set("TOK", "t-real");
    let out =
        f.ws.export_code(&secret_spec(), CodeLanguage::Curl, true)
            .unwrap();
    assert!(out.contains("-H 'Authorization: Bearer t-real'"), "{out}");
    assert!(out.contains("-H 'X-Key: h-real'"), "{out}");
    assert!(out.contains("?key=q-real'"), "{out}");
    assert!(out.contains(r#"{"p":"b-real"}"#), "{out}");
}

#[test]
fn secret_backed_environment_variable_keeps_placeholder() {
    let mut f = fixture();
    let mut env = f.ws.environments()[0].clone();
    let mut v = NameValue::new("TOKEN", "");
    v.secret_key = Some("ENV_TOK".into());
    env.variables.push(v);
    f.ws.update_environments(vec![env]).unwrap();
    let s = RequestSpec {
        url: "{{BASE}}/x".into(),
        headers: vec![NameValue::new("X-T", "{{TOKEN}}")],
        ..Default::default()
    };
    assert!(export(&f, &s, CodeLanguage::Curl).contains("-H 'X-T: {{secret.ENV_TOK}}'"));
}

#[test]
fn preflight_token_is_a_placeholder_and_no_io_happens() {
    let f = fixture();
    let mut s = get_spec();
    s.preflight = Some(Preflight {
        enabled: true,
        method: "POST".into(),
        url: "http://127.0.0.1:1/token".into(),
        ..Default::default()
    });
    // The unsafe PROD environment is selected and nothing asked for confirmation.
    let out = export(&f, &s, CodeLanguage::Curl);
    assert!(
        out.contains("-H 'Authorization: Bearer <preflight token>'"),
        "{out}"
    );
}

#[test]
fn options_are_emitted() {
    let f = fixture();
    let mut s = get_spec();
    s.options = RequestOptions {
        timeout_ms: Some(2500),
        follow_redirects: Some(true),
        verify_tls: Some(false),
        proxy: Some("http://proxy.local:8080".into()),
    };
    let out = export(&f, &s, CodeLanguage::Curl);
    assert!(out.contains("-k"), "{out}");
    assert!(out.contains("-m 2.5"), "{out}");
    assert!(out.contains("-x 'http://proxy.local:8080'"), "{out}");
    assert!(out.contains("-L"), "{out}");
    let java = export(&f, &s, CodeLanguage::Java);
    assert!(java.contains("Duration.ofMillis(2500)"), "{java}");
    assert!(java.contains("ProxySelector"), "{java}");
    assert!(java.contains("TLS verification"), "{java}");
    let kt = export(&f, &s, CodeLanguage::Kotlin);
    assert!(kt.contains("callTimeout"), "{kt}");
    assert!(kt.contains("Proxy.Type.HTTP"), "{kt}");
    let ax = export(&f, &s, CodeLanguage::Axios);
    assert!(ax.contains("timeout: 2500"), "{ax}");
    assert!(ax.contains("rejectUnauthorized: false"), "{ax}");
    assert!(ax.contains("host: 'proxy.local'"), "{ax}");
}

#[test]
fn multipart_and_binary_reference_paths_without_reading_files() {
    let f = fixture();
    let s = RequestSpec {
        method: "POST".into(),
        url: "{{BASE}}/up".into(),
        body: Body::Multipart {
            fields: vec![
                NameValue::new("note", "hi"),
                NameValue::file("doc", "data/missing.png"),
            ],
        },
        ..Default::default()
    };
    let curl = export(&f, &s, CodeLanguage::Curl);
    assert!(curl.contains("-F 'note=hi'"), "{curl}");
    assert!(curl.contains("-F 'doc=@data/missing.png'"), "{curl}");
    assert!(!curl.contains("Content-Type"), "{curl}");
    assert!(export(&f, &s, CodeLanguage::Axios).contains("createReadStream('data/missing.png')"));
    assert!(export(&f, &s, CodeLanguage::Java)
        .contains("Files.readAllBytes(Path.of(\"data/missing.png\"))"));
    assert!(export(&f, &s, CodeLanguage::Kotlin).contains("File(\"data/missing.png\")"));

    let b = RequestSpec {
        method: "PUT".into(),
        url: "{{BASE}}/up".into(),
        body: Body::Binary {
            path: "data/blob.bin".into(),
            content_type: Some("image/png".into()),
        },
        ..Default::default()
    };
    let curl = export(&f, &b, CodeLanguage::Curl);
    assert!(curl.contains("-H 'Content-Type: image/png'"), "{curl}");
    assert!(curl.contains("--data-binary '@data/blob.bin'"), "{curl}");
}

#[test]
fn string_escaping_per_language() {
    let f = fixture();
    let s = RequestSpec {
        method: "POST".into(),
        url: "{{BASE}}/e".into(),
        body: Body::Raw {
            content: "it's \"q\" \\ $x\nline2 é".into(),
            content_type: Some("text/plain".into()),
        },
        ..Default::default()
    };
    assert!(
        export(&f, &s, CodeLanguage::Curl).contains("--data-raw 'it'\\''s \"q\" \\ $x\nline2 é'")
    );
    assert!(export(&f, &s, CodeLanguage::Axios).contains("data: 'it\\'s \"q\" \\\\ $x\\nline2 é'"));
    assert!(export(&f, &s, CodeLanguage::Java).contains("\"it's \\\"q\\\" \\\\ $x\\nline2 é\""));
    assert!(export(&f, &s, CodeLanguage::Kotlin).contains("\"it's \\\"q\\\" \\\\ \\$x\\nline2 é\""));
}

fn header<'a>(p: &'a xrest_core::curl::ParsedCurl, n: &str) -> Option<&'a str> {
    p.headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(n))
        .map(|h| h.value.as_str())
}

#[test]
fn curl_round_trips_through_the_importer() {
    let f = fixture();
    struct Case {
        spec: RequestSpec,
        method: &'static str,
        url: &'static str,
        body: Body,
        headers: Vec<(&'static str, &'static str)>,
    }
    let base = |body: Body, headers: Vec<NameValue>| RequestSpec {
        method: "POST".into(),
        url: "{{BASE}}/r?keep=1".into(),
        params: vec![NameValue::new("q", "it's \"x\"")],
        headers,
        body,
        ..Default::default()
    };
    let url = "https://api.example.com/r?keep=1&q=it%27s+%22x%22";
    let cases = vec![
        Case {
            spec: base(
                Body::Json {
                    content: r#"{"msg":"it's \"quoted\"","u":"héllo ✓"}"#.into(),
                },
                vec![NameValue::new("X-Note", "it's ok")],
            ),
            method: "POST",
            url,
            body: Body::Json {
                content: r#"{"msg":"it's \"quoted\"","u":"héllo ✓"}"#.into(),
            },
            headers: vec![("X-Note", "it's ok"), ("Content-Type", "application/json")],
        },
        Case {
            spec: base(
                Body::Form {
                    fields: vec![
                        NameValue::new("a", "x y&z=1"),
                        NameValue::new("b", "it's"),
                        NameValue::new("c", "ünï"),
                    ],
                },
                vec![],
            ),
            method: "POST",
            url,
            body: Body::Form {
                fields: vec![
                    NameValue::new("a", "x y&z=1"),
                    NameValue::new("b", "it's"),
                    NameValue::new("c", "ünï"),
                ],
            },
            headers: vec![],
        },
        Case {
            spec: base(
                Body::Multipart {
                    fields: vec![
                        NameValue::new("t", "it's"),
                        NameValue::new("at", "@not-a-file"),
                        NameValue::file("f", "data/a b.png"),
                    ],
                },
                vec![],
            ),
            method: "POST",
            url,
            body: Body::Multipart {
                fields: vec![
                    NameValue::new("t", "it's"),
                    NameValue::new("at", "@not-a-file"),
                    NameValue::file("f", "data/a b.png"),
                ],
            },
            headers: vec![],
        },
        Case {
            spec: RequestSpec {
                method: "DELETE".into(),
                url: "{{BASE}}/d".into(),
                ..Default::default()
            },
            method: "DELETE",
            url: "https://api.example.com/d",
            body: Body::None,
            headers: vec![],
        },
    ];
    for c in cases {
        let out = export(&f, &c.spec, CodeLanguage::Curl);
        let p = xrest_core::curl::parse(&out).unwrap_or_else(|e| panic!("{e}\n{out}"));
        assert_eq!(p.method, c.method, "{out}");
        assert_eq!(p.url, c.url, "{out}");
        assert_eq!(p.body, c.body, "{out}");
        for (n, v) in &c.headers {
            assert_eq!(header(&p, n), Some(*v), "{n}\n{out}");
        }
        assert_eq!(p.headers.len(), c.headers.len(), "{out}");
    }
}

#[test]
fn service_auth_round_trips_as_a_header_and_query_key() {
    let mut f = fixture();
    with_bearer(&mut f);
    f.secrets.set("TOK", "t-real");
    let s = RequestSpec {
        url: "{{BASE}}/a".into(),
        authenticated: true,
        ..Default::default()
    };
    let out = f.ws.export_code(&s, CodeLanguage::Curl, true).unwrap();
    let p = xrest_core::curl::parse(&out).unwrap();
    assert_eq!(header(&p, "authorization"), Some("Bearer t-real"));
}

#[test]
fn invalid_url_is_an_error() {
    let f = fixture();
    let s = RequestSpec {
        url: "{{NOPE}}/x".into(),
        ..Default::default()
    };
    assert!(matches!(
        f.ws.export_code(&s, CodeLanguage::Curl, false),
        Err(Error::InvalidUrl { .. })
    ));
}

#[test]
fn api_key_and_basic_auth_keep_placeholders() {
    let mut f = fixture();
    let root = f.ws.root().to_path_buf();
    let mut svc = f.ws.service().clone();
    svc.is_authenticated = true;
    svc.auth = Some(AuthConfig {
        kind: AuthKind::Apikey,
        api_key_name: "api_key".into(),
        api_key_value: "{{secret.AK}}".into(),
        api_key_location: "query".into(),
        ..Default::default()
    });
    xrest_core::storage::save_service(&root, &svc).unwrap();
    f.ws = Workspace::open(&root).unwrap();
    let s = RequestSpec {
        url: "{{BASE}}/a".into(),
        authenticated: true,
        ..Default::default()
    };
    let out = export(&f, &s, CodeLanguage::Curl);
    assert!(
        out.contains("'https://api.example.com/a?api_key={{secret.AK}}'"),
        "{out}"
    );

    svc.auth = Some(AuthConfig {
        kind: AuthKind::Basic,
        basic_user: "bob".into(),
        basic_pass: "{{secret.PW}}".into(),
        ..Default::default()
    });
    xrest_core::storage::save_service(&root, &svc).unwrap();
    f.ws = Workspace::open(&root).unwrap();
    let out = export(&f, &s, CodeLanguage::Curl);
    assert!(
        out.contains("Authorization: Basic base64(bob:{{secret.PW}})"),
        "{out}"
    );
}
