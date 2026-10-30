//! End-to-end sends against a local wiremock server.

use std::sync::Arc;

use wiremock::matchers::{body_json, body_string, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xrest_core::*;

struct Fixture {
    _dir: tempfile::TempDir,
    ws: Workspace,
    server: MockServer,
    secrets: Arc<MemorySecretStore>,
}

async fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    ws.set_http_options(HttpOptions {
        use_system_proxy: false,
        ..Default::default()
    });
    let secrets = Arc::new(MemorySecretStore::default());
    ws.set_secret_store(secrets.clone());
    let uri = server.uri();
    let envs = vec![
        Environment {
            name: "DEV".into(),
            is_unsafe: false,
            variables: vec![
                NameValue::new("BASE_URL", uri.clone()),
                NameValue::new("USER_ID", "42"),
            ],
            ..Default::default()
        },
        Environment {
            name: "PROD".into(),
            is_unsafe: true,
            variables: vec![NameValue::new("BASE_URL", uri)],
            ..Default::default()
        },
    ];
    ws.update_environments(envs).unwrap();
    Fixture {
        _dir: dir,
        ws,
        server,
        secrets,
    }
}

fn spec(method: &str, url: &str) -> RequestSpec {
    RequestSpec {
        method: method.into(),
        url: url.into(),
        ..Default::default()
    }
}

fn set_service(f: &mut Fixture, edit: impl FnOnce(&mut ServiceFile)) {
    // Rewrite service.yaml and reopen, as a user editing the file would.
    let root = f.ws.root().to_path_buf();
    let mut svc = f.ws.service().clone();
    edit(&mut svc);
    xrest_core::storage::save_service(&root, &svc).unwrap();
    let mut ws = Workspace::open(&root).unwrap();
    ws.set_http_options(HttpOptions {
        use_system_proxy: false,
        ..Default::default()
    });
    ws.set_secret_store(f.secrets.clone());
    f.ws = ws;
}

#[tokio::test]
async fn sends_with_interpolation_params_headers_and_body() {
    let f = fixture().await;
    Mock::given(method("POST"))
        .and(path("/users/42"))
        .and(query_param("page", "2"))
        .and(query_param("keep", "yes"))
        .and(header("x-user", "42"))
        .and(header("content-type", "application/json"))
        .and(body_json(serde_json::json!({"id": "42"})))
        .respond_with(
            ResponseTemplate::new(201)
                .insert_header("x-reply", "ok")
                .set_body_string(r#"{"created":true}"#),
        )
        .expect(1)
        .mount(&f.server)
        .await;

    let mut s = spec("post", "{{BASE_URL}}/users/{{USER_ID}}?page=1&keep=yes");
    s.params.push(NameValue::new("page", "2"));
    let mut disabled = NameValue::new("ignored", "x");
    disabled.enabled = false;
    s.params.push(disabled);
    s.headers.push(NameValue::new("X-User", "{{ USER_ID }}"));
    s.body = Body::from_text(r#"{"id": "{{USER_ID}}"}"#);

    let prepared = f.ws.prepare_send(&s, false).unwrap();
    assert_eq!(prepared.environment(), Some("DEV"));
    let resp = prepared.execute().await.unwrap();
    assert_eq!(resp.status, 201);
    assert_eq!(resp.status_text, "Created");
    assert_eq!(resp.body, r#"{"created":true}"#);
    assert_eq!(
        resp.pretty_body.as_deref(),
        Some("{\n  \"created\": true\n}")
    );
    assert!(!resp.body_is_base64);
    assert_eq!(resp.size_bytes, 16);
    assert!(resp
        .headers
        .iter()
        .any(|(k, v)| k == "x-reply" && v == "ok"));
    assert!(resp.unresolved_variables.is_empty());
    assert!(resp.final_url.contains("page=2"));
}

#[tokio::test]
async fn non_2xx_is_a_response_not_an_error() {
    let f = fixture().await;
    Mock::given(path("/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&f.server)
        .await;
    let resp =
        f.ws.prepare_send(&spec("GET", "{{BASE_URL}}/missing"), false)
            .unwrap()
            .execute()
            .await
            .unwrap();
    assert_eq!((resp.status, resp.body.as_str()), (404, "nope"));
}

#[tokio::test]
async fn reports_unresolved_variables_but_still_sends() {
    let f = fixture().await;
    Mock::given(path("/x"))
        .and(header("x-a", "{{NOT_DEFINED}}"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/x");
    s.headers.push(NameValue::new("X-A", "{{NOT_DEFINED}}"));
    let prepared = f.ws.prepare_send(&s, false).unwrap();
    assert_eq!(prepared.unresolved_variables(), ["NOT_DEFINED"]);
    let resp = prepared.execute().await.unwrap();
    assert_eq!(resp.unresolved_variables, vec!["NOT_DEFINED"]);
}

#[tokio::test]
async fn path_params_fill_the_url_path_and_missing_ones_are_reported() {
    let f = fixture().await;
    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/users/{id}/files/{name}/{gone}?k={k}");
    s.path_params = vec![
        NameValue::new("id", "{{USER_ID}}"),
        NameValue::new("name", "a b"),
    ];
    let prepared = f.ws.prepare_send(&s, false).unwrap();
    assert_eq!(prepared.unresolved_variables(), ["{gone}"]);
    prepared.execute().await.unwrap();
    let got = f.server.received_requests().await.unwrap();
    assert_eq!(
        got[0].url.as_str().split_once("/users").unwrap().1,
        "/42/files/a%20b/%7Bgone%7D?k={k}"
    );
}

#[tokio::test]
async fn dot_segment_path_params_are_not_collapsed() {
    let f = fixture().await;
    let mut s = spec("GET", "{{BASE_URL}}/users/{id}");
    s.path_params = vec![NameValue::new("id", "..")];
    // The URL layer treats %2E%2E as a dot-segment too, so the value cannot be sent safely.
    match f.ws.prepare_send(&s, false) {
        Err(Error::InvalidInput(m)) => assert!(m.contains("{id}"), "{m}"),
        Ok(p) => panic!("collapsed to {}", p.url()),
        Err(e) => panic!("unexpected {e:?}"),
    }
    s.path_params = vec![NameValue::new("id", "a.b")];
    assert!(f
        .ws
        .prepare_send(&s, false)
        .unwrap()
        .url()
        .ends_with("/users/a.b"));
}

#[tokio::test]
async fn unsafe_environment_requires_explicit_confirmation() {
    let mut f = fixture().await;
    Mock::given(path("/danger"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    f.ws.select_environment("PROD").unwrap();
    assert!(f.ws.send_policy().requires_confirmation);

    let s = spec("DELETE", "{{BASE_URL}}/danger");
    match f.ws.prepare_send(&s, false) {
        Err(Error::ConfirmationRequired { environment }) => assert_eq!(environment, "PROD"),
        other => panic!("expected ConfirmationRequired, got {:?}", other.err()),
    }
    // Also by-id helpers.
    let sc = f.ws.create_scratch("d", s.clone()).unwrap();
    assert!(matches!(
        f.ws.prepare_send_scratch(&sc.id, false),
        Err(Error::ConfirmationRequired { .. })
    ));
    let ep = f.ws.create_endpoint("d", s.clone()).unwrap();
    assert!(matches!(
        f.ws.prepare_send_endpoint(&ep.id, false),
        Err(Error::ConfirmationRequired { .. })
    ));

    let resp =
        f.ws.prepare_send(&s, true)
            .unwrap()
            .execute()
            .await
            .unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(resp.environment.as_deref(), Some("PROD"));
    // `expect(1)` verifies on drop that the refused attempts never hit the network.
}

#[tokio::test]
async fn scratch_requests_use_the_same_environment() {
    let mut f = fixture().await;
    Mock::given(path("/scratch"))
        .respond_with(ResponseTemplate::new(200).set_body_string("hi"))
        .mount(&f.server)
        .await;
    let sc =
        f.ws.create_scratch("s", spec("GET", "{{BASE_URL}}/scratch"))
            .unwrap();
    let resp =
        f.ws.prepare_send_scratch(&sc.id, false)
            .unwrap()
            .execute()
            .await
            .unwrap();
    assert_eq!(resp.body, "hi");
    assert_eq!(resp.environment.as_deref(), Some("DEV"));
}

#[tokio::test]
async fn service_bearer_auth_only_when_request_opts_in() {
    let mut f = fixture().await;
    set_service(&mut f, |s| {
        s.is_authenticated = true;
        s.auth_type = Some(AuthKind::Bearer);
        s.auth = Some(AuthConfig {
            kind: AuthKind::Bearer,
            bearer_token: "{{secret.api}}".into(),
            ..Default::default()
        });
    });
    f.secrets.set("api", "tok-123");
    Mock::given(path("/authed"))
        .and(header("authorization", "Bearer tok-123"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    Mock::given(path("/anon"))
        .respond_with(move |req: &wiremock::Request| {
            assert!(req.headers.get("authorization").is_none());
            ResponseTemplate::new(200)
        })
        .expect(1)
        .mount(&f.server)
        .await;

    let mut authed = spec("GET", "{{BASE_URL}}/authed");
    authed.authenticated = true;
    f.ws.prepare_send(&authed, false)
        .unwrap()
        .execute()
        .await
        .unwrap();
    f.ws.prepare_send(&spec("GET", "{{BASE_URL}}/anon"), false)
        .unwrap()
        .execute()
        .await
        .unwrap();
}

#[tokio::test]
async fn basic_and_apikey_auth() {
    let mut f = fixture().await;
    set_service(&mut f, |s| {
        s.is_authenticated = true;
        s.auth_type = Some(AuthKind::Basic);
        s.auth = Some(AuthConfig {
            kind: AuthKind::Basic,
            basic_user: "aladdin".into(),
            basic_pass: "open sesame".into(),
            ..Default::default()
        });
    });
    Mock::given(path("/basic"))
        .and(header(
            "authorization",
            "Basic YWxhZGRpbjpvcGVuIHNlc2FtZQ==",
        ))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/basic");
    s.authenticated = true;
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();

    for (location, expect_header) in [("header", true), ("query", false)] {
        set_service(&mut f, |svc| {
            svc.auth_type = Some(AuthKind::Apikey);
            svc.auth = Some(AuthConfig {
                kind: AuthKind::Apikey,
                api_key_name: "X-Key".into(),
                api_key_value: "k1".into(),
                api_key_location: location.into(),
                ..Default::default()
            });
        });
        let p = format!("/apikey-{location}");
        let mut m = Mock::given(path(p.as_str()));
        m = if expect_header {
            m.and(header("x-key", "k1"))
        } else {
            m.and(query_param("X-Key", "k1"))
        };
        m.respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&f.server)
            .await;
        let mut s = spec("GET", &format!("{{{{BASE_URL}}}}{p}"));
        s.authenticated = true;
        f.ws.prepare_send(&s, false)
            .unwrap()
            .execute()
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn missing_secret_is_an_error_before_any_request() {
    let f = fixture().await;
    let mut s = spec("GET", "{{BASE_URL}}/x");
    s.headers.push(NameValue::new("X-S", "{{secret.absent}}"));
    match f.ws.prepare_send(&s, false) {
        Err(Error::MissingSecrets(keys)) => assert_eq!(keys, vec!["absent"]),
        other => panic!("{:?}", other.err()),
    }
}

#[tokio::test]
async fn environment_variable_with_secret_key_resolves_from_store() {
    let mut f = fixture().await;
    let mut envs = f.ws.environments().to_vec();
    let mut v = NameValue::new("API_KEY", "");
    v.secret_key = Some("prod-key".into());
    envs[0].variables.push(v);
    f.ws.update_environments(envs).unwrap();
    f.secrets.set("prod-key", "shh");
    Mock::given(path("/k"))
        .and(header("x-api-key", "shh"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/k");
    s.headers.push(NameValue::new("X-Api-Key", "{{API_KEY}}"));
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();
}

fn preflight(server: &MockServer) -> Preflight {
    Preflight {
        enabled: true,
        method: "POST".into(),
        url: format!("{}/oauth/token", server.uri()),
        body: r#"{"client":"abc"}"#.into(),
        cache_token: true,
        cache_duration: "derived".into(),
        cache_duration_key: "expires_in".into(),
        cache_duration_unit: "seconds".into(),
        token_key: "data.token".into(),
        ..Default::default()
    }
}

#[tokio::test]
async fn preflight_acquires_injects_and_caches_token() {
    let mut f = fixture().await;
    let pf = preflight(&f.server);
    set_service(&mut f, |s| {
        s.is_authenticated = true;
        s.preflight = Some(pf);
    });
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string(r#"{"client":"abc"}"#))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"data": {"token": "pf-token"}, "expires_in": 3600}),
            ),
        )
        .expect(1) // second send must come from the cache
        .mount(&f.server)
        .await;
    Mock::given(path("/secure"))
        .and(header("authorization", "Bearer pf-token"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&f.server)
        .await;

    let mut s = spec("GET", "{{BASE_URL}}/secure");
    s.authenticated = true;
    for _ in 0..2 {
        let r =
            f.ws.prepare_send(&s, false)
                .unwrap()
                .execute()
                .await
                .unwrap();
        assert_eq!(r.status, 200);
    }
}

#[tokio::test]
async fn preflight_token_cache_is_scoped_per_environment_and_clearable() {
    let mut f = fixture().await;
    // Same preflight URL for both environments; tokens must not leak across them.
    let pf = preflight(&f.server);
    set_service(&mut f, |s| {
        s.is_authenticated = true;
        s.preflight = Some(pf);
    });
    Mock::given(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"token": "t"}, "expires_in": 3600})),
        )
        .expect(3)
        .mount(&f.server)
        .await;
    Mock::given(path("/secure"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/secure");
    s.authenticated = true;

    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap(); // DEV: fetch #1
    f.ws.select_environment("PROD").unwrap();
    f.ws.prepare_send(&s, true)
        .unwrap()
        .execute()
        .await
        .unwrap(); // PROD: fetch #2
    f.ws.prepare_send(&s, true)
        .unwrap()
        .execute()
        .await
        .unwrap(); // PROD: cached
    f.ws.clear_token_cache();
    f.ws.prepare_send(&s, true)
        .unwrap()
        .execute()
        .await
        .unwrap(); // fetch #3
}

#[tokio::test]
async fn endpoint_preflight_overrides_service_and_custom_header_has_no_bearer_prefix() {
    let mut f = fixture().await;
    let svc_pf = Preflight {
        url: format!("{}/svc-token", f.server.uri()),
        ..preflight(&f.server)
    };
    set_service(&mut f, |s| {
        s.is_authenticated = true;
        s.preflight = Some(svc_pf);
    });
    Mock::given(path("/svc-token"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&f.server)
        .await;
    Mock::given(path("/ep-token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"access_token": "abc"})),
        )
        .expect(1)
        .mount(&f.server)
        .await;
    Mock::given(path("/who"))
        .and(header("x-token", "abc"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/who");
    s.authenticated = true;
    s.preflight = Some(Preflight {
        enabled: true,
        method: "GET".into(),
        url: "{{BASE_URL}}/ep-token".into(),
        token_header: Some("X-Token".into()),
        ..Default::default()
    });
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();
}

#[tokio::test]
async fn preflight_form_body_and_failure_modes() {
    let f = fixture().await;
    Mock::given(path("/form-token"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(body_string("grant_type=client_credentials&id=a+b"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"access_token": "ok"})),
        )
        .expect(1)
        .mount(&f.server)
        .await;
    Mock::given(path("/denied"))
        .respond_with(ResponseTemplate::new(401).set_body_string("bad creds"))
        .mount(&f.server)
        .await;
    Mock::given(path("/garbage"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>"))
        .mount(&f.server)
        .await;
    Mock::given(path("/t"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&f.server)
        .await;

    let mut s = spec("GET", "{{BASE_URL}}/t");
    s.preflight = Some(Preflight {
        enabled: true,
        method: "POST".into(),
        url: "{{BASE_URL}}/form-token".into(),
        body_type: "urlencoded".into(),
        body_params: vec![
            NameValue::new("grant_type", "client_credentials"),
            NameValue::new("id", "a b"),
        ],
        ..Default::default()
    });
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();

    for (p, needle) in [("/denied", "401"), ("/garbage", "not valid JSON")] {
        let mut s = spec("GET", "{{BASE_URL}}/t");
        s.preflight = Some(Preflight {
            enabled: true,
            method: "GET".into(),
            url: format!("{{{{BASE_URL}}}}{p}"),
            ..Default::default()
        });
        match f.ws.prepare_send(&s, false).unwrap().execute().await {
            Err(Error::Preflight(m)) => assert!(m.contains(needle), "{m}"),
            other => panic!("{:?}", other.err()),
        }
    }
}

#[tokio::test]
async fn invalid_url_and_connection_errors() {
    let f = fixture().await;
    assert!(matches!(
        f.ws.prepare_send(&spec("GET", "not a url"), false),
        Err(Error::InvalidUrl { .. })
    ));
    assert!(matches!(
        f.ws.prepare_send(&spec("GET", "{{MISSING}}/x"), false),
        Err(Error::InvalidUrl { .. })
    ));
    // Port 1 on localhost: connection refused.
    let r =
        f.ws.prepare_send(&spec("GET", "http://127.0.0.1:1/"), false)
            .unwrap()
            .execute()
            .await;
    assert!(matches!(r, Err(Error::Http(_))), "{:?}", r.err());
}

#[tokio::test]
async fn binary_responses_are_base64() {
    let f = fixture().await;
    Mock::given(path("/img"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(vec![0x89, 0x50, 0x4e, 0x47], "image/png"),
        )
        .mount(&f.server)
        .await;
    let r =
        f.ws.prepare_send(&spec("GET", "{{BASE_URL}}/img"), false)
            .unwrap()
            .execute()
            .await
            .unwrap();
    assert!(r.body_is_base64);
    assert_eq!(r.body, "iVBORw==");
    assert_eq!(r.size_bytes, 4);
}

#[tokio::test]
async fn text_body_without_content_type_gets_default() {
    let f = fixture().await;
    Mock::given(path("/t"))
        .and(header("content-type", "text/plain; charset=utf-8"))
        .and(body_string("hello"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("PUT", "{{BASE_URL}}/t");
    s.body = Body::from_text("hello");
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();
}

// ---- typed bodies ------------------------------------------------------------------------

async fn send(f: &Fixture, s: &RequestSpec) -> Result<HttpResponse> {
    f.ws.prepare_send(s, false)?.execute().await
}

#[tokio::test]
async fn json_body_always_sends_json_content_type() {
    let f = fixture().await;
    Mock::given(path("/j"))
        .and(header("content-type", "application/json"))
        .and(body_string("not json-looking but typed as json"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("POST", "{{BASE_URL}}/j");
    s.body = Body::Json {
        content: "not json-looking but typed as json".into(),
    };
    send(&f, &s).await.unwrap();
}

#[tokio::test]
async fn raw_body_uses_its_content_type_but_a_header_wins() {
    let f = fixture().await;
    Mock::given(path("/x"))
        .and(header("content-type", "application/xml"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    Mock::given(path("/h"))
        .and(header("content-type", "text/csv"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("POST", "{{BASE_URL}}/x");
    s.body = Body::Raw {
        content: "<a/>".into(),
        content_type: Some("application/xml".into()),
    };
    send(&f, &s).await.unwrap();
    s.url = "{{BASE_URL}}/h".into();
    s.headers = vec![NameValue::new("Content-Type", "text/csv")];
    send(&f, &s).await.unwrap();
}

#[tokio::test]
async fn form_body_is_urlencoded_with_variables() {
    let f = fixture().await;
    Mock::given(path("/f"))
        .and(header("content-type", "application/x-www-form-urlencoded"))
        .and(body_string("user=42&q=a+b%26c"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut disabled = NameValue::new("skip", "me");
    disabled.enabled = false;
    let mut s = spec("POST", "{{BASE_URL}}/f");
    s.body = Body::Form {
        fields: vec![
            NameValue::new("user", "{{USER_ID}}"),
            NameValue::new("q", "a b&c"),
            disabled,
        ],
    };
    send(&f, &s).await.unwrap();
}

#[tokio::test]
async fn multipart_body_sends_text_and_file_parts() {
    let f = fixture().await;
    let data = f.ws.root().join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("a.txt"), b"FILE-CONTENTS").unwrap();
    Mock::given(path("/m"))
        .and(|r: &wiremock::Request| {
            let ct = r
                .headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let body = String::from_utf8_lossy(&r.body);
            ct.starts_with("multipart/form-data; boundary=")
                && body.contains("name=\"title\"")
                && body.contains("hello 42")
                && body.contains("filename=\"a.txt\"")
                && body.contains("FILE-CONTENTS")
        })
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("POST", "{{BASE_URL}}/m");
    // A user-set Content-Type would break the boundary, so it is dropped.
    s.headers = vec![NameValue::new("Content-Type", "multipart/form-data")];
    s.body = Body::Multipart {
        fields: vec![
            NameValue::new("title", "hello {{USER_ID}}"),
            NameValue::file("upload", "data/a.txt"),
        ],
    };
    send(&f, &s).await.unwrap();
}

#[tokio::test]
async fn binary_body_sends_file_bytes() {
    let f = fixture().await;
    std::fs::write(f.ws.root().join("blob.bin"), [0u8, 1, 2, 255]).unwrap();
    Mock::given(path("/b"))
        .and(header("content-type", "application/octet-stream"))
        .and(|r: &wiremock::Request| r.body == [0u8, 1, 2, 255])
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;
    let mut s = spec("PUT", "{{BASE_URL}}/b");
    s.body = Body::Binary {
        path: "blob.bin".into(),
        content_type: None,
    };
    send(&f, &s).await.unwrap();
}

#[tokio::test]
async fn files_outside_the_workspace_are_never_sent() {
    let f = fixture().await;
    let outside = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(outside.path(), b"private").unwrap();
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&f.server)
        .await;
    let mut s = spec("POST", "{{BASE_URL}}/b");
    s.body = Body::Binary {
        path: outside.path().display().to_string(),
        content_type: None,
    };
    assert!(matches!(
        f.ws.prepare_send(&s, false),
        Err(Error::InvalidBody(_))
    ));
    s.body = Body::Multipart {
        fields: vec![NameValue::file("f", "../../etc/passwd")],
    };
    assert!(matches!(
        f.ws.prepare_send(&s, false),
        Err(Error::InvalidBody(_))
    ));
}

// ---- request options ---------------------------------------------------------------------

#[tokio::test]
async fn per_request_timeout_overrides_the_default() {
    let f = fixture().await;
    Mock::given(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_millis(800)))
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/slow");
    s.options.timeout_ms = Some(100);
    assert!(matches!(send(&f, &s).await, Err(Error::Http(_))));
    s.options.timeout_ms = None;
    assert_eq!(send(&f, &s).await.unwrap().status, 200);
}

#[tokio::test]
async fn follow_redirects_can_be_disabled_per_request() {
    let f = fixture().await;
    Mock::given(path("/old"))
        .respond_with(
            ResponseTemplate::new(302).insert_header("location", format!("{}/new", f.server.uri())),
        )
        .mount(&f.server)
        .await;
    Mock::given(path("/new"))
        .respond_with(ResponseTemplate::new(200).set_body_string("arrived"))
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/old");
    assert_eq!(send(&f, &s).await.unwrap().body, "arrived");
    s.options.follow_redirects = Some(false);
    assert_eq!(send(&f, &s).await.unwrap().status, 302);
}

#[tokio::test]
async fn invalid_proxy_is_reported() {
    let f = fixture().await;
    let mut s = spec("GET", "{{BASE_URL}}/x");
    s.options.proxy = Some("not a url".into());
    assert!(matches!(send(&f, &s).await, Err(Error::InvalidUrl { .. })));
}

#[test]
fn options_merge_over_defaults() {
    let base = HttpOptions::default();
    let merged = base.merged(&RequestOptions {
        timeout_ms: Some(0),
        follow_redirects: Some(false),
        verify_tls: Some(false),
        proxy: Some(String::new()),
    });
    assert_eq!(merged.timeout, base.timeout, "0 means the default");
    assert!(!merged.follow_redirects);
    assert!(merged.insecure_skip_verify);
    assert_eq!(merged.proxy.as_deref(), Some(""));
    let untouched = base.merged(&RequestOptions::default());
    assert_eq!(untouched.timeout, base.timeout);
    assert!(untouched.follow_redirects && !untouched.insecure_skip_verify);
}

fn token_mock(token_json: serde_json::Value) -> Mock {
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(token_json))
}

#[tokio::test]
async fn test_preflight_returns_exchange_and_token_without_touching_cache() {
    let mut f = fixture().await;
    let mut pf = preflight(&f.server);
    pf.url = "{{BASE_URL}}/oauth/token".into();
    pf.headers.push(NameValue::new("X-User", "{{USER_ID}}"));
    let svc_pf = pf.clone();
    set_service(&mut f, |s| {
        s.is_authenticated = true;
        s.preflight = Some(svc_pf);
    });
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(header("x-user", "42"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"token": "tt"}, "expires_in": 3600})),
        )
        .expect(3) // two tests + the send that must still fetch
        .mount(&f.server)
        .await;
    Mock::given(path("/secure"))
        .and(header("authorization", "Bearer tt"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&f.server)
        .await;

    // Explicit preflight (even a disabled draft), then the service one with None.
    let mut draft = pf.clone();
    draft.enabled = false;
    for given in [Some(draft), None] {
        let r = f.ws.test_preflight(given, false).await.unwrap();
        assert_eq!(r.method, "POST");
        assert_eq!(r.url, format!("{}/oauth/token", f.server.uri()));
        assert_eq!(r.status, Some(200));
        assert_eq!(r.token.as_deref(), Some("tt"));
        assert_eq!(r.error, None);
        assert_eq!(r.request_body, r#"{"client":"abc"}"#);
        assert!(r
            .request_headers
            .iter()
            .any(|(k, v)| k == "X-User" && v == "42"));
        assert!(r.response_body.contains("tt"));
        assert!(!r.response_headers.is_empty());
    }

    // The cache was neither read nor written: a send still fetches (third hit on the mock).
    let mut s = spec("GET", "{{BASE_URL}}/secure");
    s.authenticated = true;
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();
}

#[tokio::test]
async fn test_preflight_does_not_read_a_warm_cache() {
    let mut f = fixture().await;
    let pf = preflight(&f.server);
    let svc_pf = pf.clone();
    set_service(&mut f, |s| {
        s.is_authenticated = true;
        s.preflight = Some(svc_pf);
    });
    token_mock(serde_json::json!({"data": {"token": "tt"}, "expires_in": 3600}))
        .expect(2) // the send warms the cache; the test still hits the server
        .mount(&f.server)
        .await;
    Mock::given(path("/secure"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/secure");
    s.authenticated = true;
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();
    let r = f.ws.test_preflight(Some(pf), false).await.unwrap();
    assert_eq!(r.token.as_deref(), Some("tt"));
}

#[tokio::test]
async fn test_preflight_bad_token_key_is_a_result_with_error() {
    let f = fixture().await;
    token_mock(serde_json::json!({"data": {"token": "tt"}}))
        .mount(&f.server)
        .await;
    let mut pf = preflight(&f.server);
    pf.token_key = "data.nope".into();
    let r = f.ws.test_preflight(Some(pf), false).await.unwrap();
    assert_eq!(r.status, Some(200));
    assert_eq!(r.token, None);
    assert!(
        r.error.as_deref().unwrap().contains("data.nope"),
        "{:?}",
        r.error
    );
    assert!(r.response_body.contains("tt"));
}

#[tokio::test]
async fn test_preflight_non_2xx_and_transport_failures_are_results() {
    let f = fixture().await;
    Mock::given(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(401).set_body_string("denied"))
        .mount(&f.server)
        .await;
    let r =
        f.ws.test_preflight(Some(preflight(&f.server)), false)
            .await
            .unwrap();
    assert_eq!(r.status, Some(401));
    assert_eq!(r.response_body, "denied");
    assert_eq!(r.token, None);
    assert!(r.error.is_some());

    let mut pf = preflight(&f.server);
    pf.url = "http://127.0.0.1:1/x".into();
    let r = f.ws.test_preflight(Some(pf), false).await.unwrap();
    assert_eq!(r.status, None);
    assert!(r.error.is_some());
}

#[tokio::test]
async fn test_preflight_without_any_preflight_is_an_error() {
    let f = fixture().await;
    let err = f.ws.test_preflight(None, false).await.unwrap_err();
    assert!(matches!(err, Error::Preflight(_)), "{err:?}");
}

#[tokio::test]
async fn test_preflight_unsafe_environment_needs_confirmation_before_any_io() {
    let mut f = fixture().await;
    Mock::given(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {"token": "t"}})),
        )
        .expect(1) // only the confirmed call
        .mount(&f.server)
        .await;
    f.ws.select_environment("PROD").unwrap();
    let pf = preflight(&f.server);
    let err =
        f.ws.test_preflight(Some(pf.clone()), false)
            .await
            .unwrap_err();
    assert!(
        matches!(err, Error::ConfirmationRequired { ref environment } if environment == "PROD")
    );
    assert_eq!(f.server.received_requests().await.unwrap().len(), 0);
    let r = f.ws.test_preflight(Some(pf), true).await.unwrap();
    assert_eq!(r.token.as_deref(), Some("t"));
}

#[tokio::test]
async fn update_service_settings_clears_the_token_cache() {
    let f = fixture().await;
    let mut f = f;
    let mut settings = f.ws.service_settings();
    settings.is_authenticated = true;
    settings.preflight = Some(preflight(&f.server));
    f.ws.update_service_settings(settings.clone()).unwrap();
    Mock::given(path("/oauth/token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"data": {"token": "t"}, "expires_in": 3600})),
        )
        .expect(2) // warm once; the settings update forces a second fetch
        .mount(&f.server)
        .await;
    Mock::given(path("/secure"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/secure");
    s.authenticated = true;
    for _ in 0..2 {
        f.ws.prepare_send(&s, false)
            .unwrap()
            .execute()
            .await
            .unwrap(); // second one is cached
    }
    f.ws.update_service_settings(settings).unwrap();
    f.ws.prepare_send(&s, false)
        .unwrap()
        .execute()
        .await
        .unwrap();
}

#[tokio::test]
async fn response_cookies_are_parsed_in_order() {
    let f = fixture().await;
    Mock::given(path("/login"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("Set-Cookie", "sid=abc; Path=/; HttpOnly")
                .append_header("Set-Cookie", "theme=dark; Max-Age=60"),
        )
        .mount(&f.server)
        .await;
    let resp =
        f.ws.prepare_send(&spec("GET", "{{BASE_URL}}/login"), false)
            .unwrap()
            .execute()
            .await
            .unwrap();
    assert_eq!(resp.cookies.len(), 2);
    assert_eq!(resp.cookies[0].name, "sid");
    assert!(resp.cookies[0].http_only);
    assert_eq!(resp.cookies[1].name, "theme");
    assert_eq!(resp.cookies[1].max_age, Some(60));
}
