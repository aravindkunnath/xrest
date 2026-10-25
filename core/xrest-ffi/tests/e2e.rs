//! End-to-end: one service lifecycle through the FFI surface, against a local mock server.
//! This is what the Claude Stop hook runs (`.claude/hooks/e2e.sh`).

use std::sync::Arc;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xrest_ffi::*;

fn nv(name: &str, value: &str) -> NameValue {
    NameValue {
        name: name.into(),
        value: value.into(),
        enabled: true,
        secret_key: None,
        kind: "plain".into(),
    }
}

fn spec(url: &str) -> RequestSpec {
    RequestSpec {
        method: "GET".into(),
        url: url.into(),
        authenticated: false,
        params: vec![],
        path_params: vec![],
        headers: vec![],
        body: RequestBody::Empty,
        options: RequestOptions::default(),
        preflight: None,
    }
}

fn env(name: &str, is_unsafe: bool, base_url: &str) -> EnvironmentConfig {
    EnvironmentConfig {
        name: name.into(),
        is_unsafe,
        variables: vec![nv("BASE_URL", base_url)],
    }
}

fn path_of(dir: &tempfile::TempDir) -> String {
    dir.path().to_string_lossy().into_owned()
}

#[tokio::test]
async fn unsafe_guard_secrets_history_and_reopen() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    Mock::given(path("/ping"))
        .and(header("x-key", "s3cr3t"))
        .respond_with(ResponseTemplate::new(200).set_body_string("pong"))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let ws = XRestWorkspace::create_workspace(path_of(&dir), "Demo".into()).unwrap();
    ws.save_environments(vec![
        env("DEV", false, &server.uri()),
        env("PROD", true, &server.uri()),
    ])
    .unwrap();
    let hist = XRestHistory::open(
        dir.path()
            .join("hist/h.sqlite")
            .to_string_lossy()
            .into_owned(),
    )
    .unwrap();
    ws.set_history(hist.clone());

    let mut s = spec("{{BASE_URL}}/ping");
    s.headers.push(nv("X-Key", "{{secret.k}}"));

    // A secret with no value fails before any request.
    match ws.send(s.clone(), false).await {
        Err(XRestError::MissingSecrets { keys }) => assert_eq!(keys, vec!["k"]),
        other => panic!("expected MissingSecrets, got {other:?}"),
    }
    ws.set_secret("k".into(), "s3cr3t".into());

    // DEV sends freely.
    assert!(!ws.send_policy().requires_confirmation);
    assert_eq!(ws.send(s.clone(), false).await.unwrap().status, 200);

    // PROD refuses an unconfirmed send, makes no request, then allows a confirmed one.
    ws.select_environment("PROD".into()).unwrap();
    assert!(ws.send_policy().requires_confirmation);
    let before = server.received_requests().await.unwrap().len();
    match ws.send(s.clone(), false).await {
        Err(XRestError::ConfirmationRequired { environment }) => assert_eq!(environment, "PROD"),
        other => panic!("expected ConfirmationRequired, got {other:?}"),
    }
    assert_eq!(server.received_requests().await.unwrap().len(), before);
    assert_eq!(ws.send(s.clone(), true).await.unwrap().status, 200);

    // History records both sends and never stores the secret value.
    let all = HistoryFilter {
        service_id: None,
        endpoint_id: None,
        method: None,
        status_class: None,
    };
    assert_eq!(hist.count(all.clone()).unwrap(), 2);
    let entry = hist
        .get(hist.list(all, 1, 0).unwrap()[0].id)
        .unwrap()
        .unwrap();
    assert!(entry.request_headers.iter().all(|h| h.value != "s3cr3t"));

    // Endpoints and their versions survive reopening the workspace from disk.
    let ep = ws.create_endpoint("Ping".into(), s.clone()).unwrap();
    let mut edited = s.clone();
    edited.method = "POST".into();
    ws.update_endpoint(ep.id.clone(), "Ping".into(), edited)
        .unwrap();
    let reopened = XRestWorkspace::open_workspace(path_of(&dir)).unwrap();
    assert_eq!(reopened.list_endpoints().len(), 1);
    assert_eq!(reopened.selected_environment().as_deref(), Some("PROD"));
    let versions = reopened.endpoint_versions(ep.id.clone(), 10).unwrap();
    assert_eq!(
        versions.iter().map(|v| v.version).collect::<Vec<_>>(),
        vec![2, 1]
    );
    reopened.restore_endpoint_version(ep.id.clone(), 1).unwrap();
    assert_eq!(reopened.get_endpoint(ep.id).unwrap().spec.method, "GET");
}

#[tokio::test]
async fn preflight_token_is_cached_and_scoped_per_environment() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(
            r#"{"access_token":"pf-token","expires_in":3600}"#,
            "application/json",
        ))
        .expect(3) // DEV once (second send cached), PROD once, DEV again after clear
        .mount(&server)
        .await;
    Mock::given(path("/secure"))
        .and(header("authorization", "Bearer pf-token"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let ws = XRestWorkspace::create_workspace(path_of(&dir), "Demo".into()).unwrap();
    ws.save_environments(vec![
        env("DEV", false, &server.uri()),
        env("PROD", true, &server.uri()),
    ])
    .unwrap();
    let mut settings = {
        let svc = ws.service();
        ServiceSettings {
            name: svc.name,
            is_authenticated: true,
            auth_type: "none".into(),
            basic_user: String::new(),
            basic_pass: String::new(),
            bearer_token: String::new(),
            api_key_name: String::new(),
            api_key_value: String::new(),
            api_key_location: String::new(),
            preflight: None,
        }
    };
    settings.preflight = Some(Preflight {
        enabled: true,
        method: "POST".into(),
        url: "{{BASE_URL}}/oauth/token".into(),
        body: "{}".into(),
        body_type: "raw".into(),
        body_params: vec![],
        headers: vec![],
        cache_token: true,
        cache_duration: "derived".into(),
        cache_duration_key: "expires_in".into(),
        cache_duration_unit: "seconds".into(),
        token_key: "access_token".into(),
        token_header: None,
    });
    ws.update_service_settings(settings).unwrap();

    let mut s = spec("{{BASE_URL}}/secure");
    s.authenticated = true;
    let ws: Arc<XRestWorkspace> = ws;

    assert_eq!(ws.send(s.clone(), false).await.unwrap().status, 200); // DEV: fetch #1
    assert_eq!(ws.send(s.clone(), false).await.unwrap().status, 200); // cached
    ws.select_environment("PROD".into()).unwrap();
    assert_eq!(ws.send(s.clone(), true).await.unwrap().status, 200); // PROD: fetch #2
    ws.select_environment("DEV".into()).unwrap();
    ws.clear_token_cache();
    assert_eq!(ws.send(s, false).await.unwrap().status, 200); // DEV: fetch #3
}
