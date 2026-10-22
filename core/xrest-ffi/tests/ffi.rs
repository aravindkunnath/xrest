//! Exercises the FFI object the way the Swift app does (through the public Rust API).

use wiremock::matchers::{header, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xrest_ffi::*;

fn spec(url: &str) -> RequestSpec {
    RequestSpec {
        method: "GET".into(),
        url: url.into(),
        authenticated: false,
        params: vec![],
        headers: vec![],
        body: String::new(),
        preflight: None,
    }
}

fn nv(name: &str, value: &str) -> NameValue {
    NameValue {
        name: name.into(),
        value: value.into(),
        enabled: true,
        secret_key: None,
        kind: "plain".into(),
    }
}

#[tokio::test]
async fn full_flow_through_ffi_surface() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    Mock::given(path("/ping"))
        .and(header("x-secret", "s3"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"pong":true}"#))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let ws =
        XRestWorkspace::create_workspace(dir.path().to_string_lossy().into_owned(), "Demo".into())
            .unwrap();
    assert_eq!(ws.service().name, "Demo");
    assert_eq!(ws.selected_environment().as_deref(), Some("DEV"));
    assert_eq!(ws.environments().len(), 3);
    assert!(ws.environments()[2].is_unsafe);

    // Point DEV's BASE_URL at the mock by editing environments.yaml, as a user would.
    let envs = std::fs::read_to_string(dir.path().join("environments.yaml")).unwrap();
    let envs = envs.replacen("value: ''", &format!("value: {}", server.uri()), 1);
    std::fs::write(dir.path().join("environments.yaml"), envs).unwrap();
    let ws = XRestWorkspace::open_workspace(dir.path().to_string_lossy().into_owned()).unwrap();

    ws.set_secret("k".into(), "s3".into());
    let mut s = spec("{{BASE_URL}}/ping");
    s.headers.push(nv("X-Secret", "{{secret.k}}"));

    let scratch = ws.create_scratch("ping".into(), s.clone()).unwrap();
    assert_eq!(ws.list_scratch().len(), 1);

    let resp = ws.send(s.clone(), false).await.unwrap();
    assert_eq!((resp.status, resp.body.as_str()), (200, r#"{"pong":true}"#));
    assert_eq!(resp.environment.as_deref(), Some("DEV"));

    let ep = ws
        .promote_scratch(scratch.id.clone(), Some("Ping".into()))
        .unwrap();
    assert!(ws.list_scratch().is_empty());
    assert_eq!(ws.list_endpoints()[0].id, ep.id);
    assert_eq!(ep.last_version, 1);

    let mut edited = ep.spec.clone();
    edited.method = "POST".into();
    let ep2 = ws
        .update_endpoint(ep.id.clone(), "Ping".into(), edited)
        .unwrap();
    assert_eq!(ep2.last_version, 2);
    let versions = ws.endpoint_versions(ep.id.clone(), 10).unwrap();
    assert_eq!(
        versions.iter().map(|v| v.version).collect::<Vec<_>>(),
        vec![2, 1]
    );

    // Unsafe guardrail across the boundary.
    ws.select_environment("PROD".into()).unwrap();
    let policy = ws.send_policy();
    assert!(policy.requires_confirmation);
    assert_eq!(policy.environment.as_deref(), Some("PROD"));
    match ws.send(s, false).await {
        Err(XRestError::ConfirmationRequired { environment }) => assert_eq!(environment, "PROD"),
        other => panic!("{:?}", other.err()),
    }
}

#[test]
fn errors_map_to_ffi_variants() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        XRestWorkspace::open_workspace(dir.path().to_string_lossy().into_owned()),
        Err(XRestError::InvalidInput { .. })
    ));
    let ws =
        XRestWorkspace::create_workspace(dir.path().to_string_lossy().into_owned(), "x".into())
            .unwrap();
    assert!(matches!(
        ws.get_endpoint("nope".into()),
        Err(XRestError::NotFound { .. })
    ));
    assert!(matches!(
        ws.select_environment("NOPE".into()),
        Err(XRestError::InvalidInput { .. })
    ));
    assert!(matches!(
        ws.import_curl_as_scratch("not curl".into()),
        Err(XRestError::InvalidInput { .. })
    ));
}
