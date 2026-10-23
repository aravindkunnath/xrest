//! Exercises the FFI object the way the Swift app does (through the public Rust API).

use std::sync::Arc;
use wiremock::matchers::{header, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xrest_ffi::*;

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

#[test]
fn typed_body_and_options_round_trip_through_ffi() {
    let dir = tempfile::tempdir().unwrap();
    let ws =
        XRestWorkspace::create_workspace(dir.path().display().to_string(), "svc".into()).unwrap();
    assert_eq!(ws.service().format_version, 2);
    let mut s = spec("http://h.io/x");
    let mut file = nv("f", "data/a.png");
    file.kind = "file".into();
    s.body = RequestBody::Multipart {
        fields: vec![nv("a", "1"), file],
    };
    s.path_params = vec![nv("id", "7")];
    s.options = RequestOptions {
        timeout_ms: Some(2000),
        follow_redirects: None,
        verify_tls: Some(false),
        proxy: None,
    };
    let created = ws.create_scratch("upload".into(), s).unwrap();
    let back = ws.get_scratch(created.id).unwrap().spec;
    match back.body {
        RequestBody::Multipart { fields } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[1].kind, "file");
            assert_eq!(fields[1].value, "data/a.png");
        }
        other => panic!("unexpected body {other:?}"),
    }
    assert_eq!(back.path_params.len(), 1);
    assert_eq!(back.path_params[0].name, "id");
    assert_eq!(back.options.timeout_ms, Some(2000));
    assert_eq!(back.options.verify_tls, Some(false));
}

#[test]
fn newer_workspace_format_maps_to_its_own_error() {
    let dir = tempfile::tempdir().unwrap();
    XRestWorkspace::create_workspace(dir.path().display().to_string(), "svc".into()).unwrap();
    let path = dir.path().join("service.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap()
        .replace("formatVersion: 2", "formatVersion: 99");
    std::fs::write(&path, yaml).unwrap();
    match XRestWorkspace::open_workspace(dir.path().display().to_string()) {
        Err(XRestError::UnsupportedFormat { found, supported }) => {
            assert_eq!((found, supported), (99, 2))
        }
        Err(e) => panic!("unexpected error {e:?}"),
        Ok(_) => panic!("a newer format must not open"),
    }
}

struct MapProvider(std::collections::HashMap<String, String>);

impl SecretProvider for MapProvider {
    fn get(&self, key: String) -> Option<String> {
        self.0.get(&key).cloned()
    }
}

fn provider(pairs: &[(&str, &str)]) -> Arc<dyn SecretProvider> {
    Arc::new(MapProvider(
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    ))
}

fn new_ws(dir: &tempfile::TempDir) -> Arc<XRestWorkspace> {
    XRestWorkspace::create_workspace(dir.path().to_string_lossy().into_owned(), "Demo".into())
        .unwrap()
}

#[tokio::test]
async fn secret_provider_wins_over_memory_store_and_falls_back() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    Mock::given(path("/ping"))
        .and(header("x-a", "from-provider"))
        .and(header("x-b", "from-memory"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().unwrap();
    let ws = new_ws(&dir);
    ws.set_secret("a".into(), "from-memory-loser".into());
    ws.set_secret("b".into(), "from-memory".into());
    ws.set_secret_provider(provider(&[("a", "from-provider")]));

    let mut s = spec(&format!("{}/ping", server.uri()));
    s.headers.push(nv("X-A", "{{secret.a}}"));
    s.headers.push(nv("X-B", "{{secret.b}}"));
    let r = ws.send(s, false).await.unwrap();
    assert_eq!(r.status, 200);
}

#[tokio::test]
async fn missing_secrets_lists_exactly_the_missing_keys_before_any_request() {
    let dir = tempfile::tempdir().unwrap();
    let ws = new_ws(&dir);
    ws.set_secret_provider(provider(&[("have", "1")]));
    ws.set_secret("mem".into(), "2".into());
    let mut s = spec("http://127.0.0.1:9/never");
    s.headers.push(nv("X-1", "{{secret.have}}"));
    s.headers.push(nv("X-2", "{{secret.mem}}"));
    s.headers.push(nv("X-3", "{{secret.gone1}}"));
    s.headers.push(nv("X-4", "{{secret.gone2}}"));
    match ws.send(s, false).await {
        Err(XRestError::MissingSecrets { mut keys }) => {
            keys.sort();
            assert_eq!(keys, vec!["gone1", "gone2"]);
        }
        other => panic!("expected MissingSecrets, got {other:?}"),
    }
}

#[test]
fn secret_keys_in_use_collects_every_reference_once() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    drop(new_ws(&dir));
    let svc = std::fs::read_to_string(root.join("service.yaml")).unwrap();
    let patched = svc.replacen("bearerToken: ''", "bearerToken: '{{secret.svc}}'", 1);
    assert_ne!(svc, patched, "fixture patch must apply");
    std::fs::write(root.join("service.yaml"), patched).unwrap();
    let envs = std::fs::read_to_string(root.join("environments.yaml")).unwrap();
    let patched = envs.replacen(
        "  - name: BASE_URL\n",
        "  - name: API\n    value: ''\n    enabled: true\n    secretKey: env-key\n    type: secret\n  - name: BASE_URL\n",
        1,
    );
    assert_ne!(envs, patched, "fixture patch must apply");
    let envs = patched;
    std::fs::write(root.join("environments.yaml"), envs).unwrap();

    let ws = XRestWorkspace::open_workspace(root.to_string_lossy().into_owned()).unwrap();
    let mut e = spec("https://x/{{secret.url}}");
    e.headers.push(nv("A", "{{secret.hdr}} {{secret.svc}}"));
    ws.create_endpoint("e".into(), e).unwrap();
    let mut s = spec("https://y");
    s.params.push(nv("q", "{{secret.scratch}}"));
    s.params.push(nv("r", "{{secret.hdr}}"));
    ws.create_scratch("s".into(), s).unwrap();

    assert_eq!(
        ws.secret_keys_in_use(),
        vec!["env-key", "hdr", "scratch", "svc", "url"]
    );
}

fn env_cfg(name: &str, vars: Vec<NameValue>) -> EnvironmentConfig {
    EnvironmentConfig {
        name: name.into(),
        is_unsafe: false,
        variables: vars,
    }
}

#[test]
fn save_environments_and_service_settings_through_ffi() {
    let dir = tempfile::tempdir().unwrap();
    let ws = new_ws(&dir);
    assert!(matches!(
        ws.save_environments(vec![env_cfg("A", vec![]), env_cfg("A", vec![])]),
        Err(XRestError::InvalidInput { .. })
    ));
    ws.save_environments(vec![env_cfg("QA", vec![nv("HOST", "h")])])
        .unwrap();
    assert_eq!(ws.selected_environment().as_deref(), Some("QA"));
    assert_eq!(ws.variable_names(), vec!["HOST"]);

    let info = ws.service();
    assert_eq!(info.auth_kind, "none");
    let settings = ServiceSettings {
        name: "Renamed".into(),
        is_authenticated: true,
        auth_type: "bearer".into(),
        basic_user: String::new(),
        basic_pass: String::new(),
        bearer_token: "{{secret.t}}".into(),
        api_key_name: String::new(),
        api_key_value: String::new(),
        api_key_location: String::new(),
        preflight: None,
    };
    ws.update_service_settings(settings.clone()).unwrap();
    let info = ws.service();
    assert_eq!(info.name, "Renamed");
    assert_eq!(info.id, ws.service().id);
    assert!(info.is_authenticated);
    assert_eq!(info.auth_kind, "bearer");
    assert_eq!(info.bearer_token, "{{secret.t}}");
    assert!(info.preflight.is_none());

    let mut bad = settings;
    bad.auth_type = "kerberos".into();
    assert!(matches!(
        ws.update_service_settings(bad),
        Err(XRestError::InvalidInput { .. })
    ));
}

#[test]
fn preview_uses_the_provider_chain_and_never_returns_values() {
    let dir = tempfile::tempdir().unwrap();
    let ws = new_ws(&dir);
    ws.save_environments(vec![env_cfg("DEV", vec![nv("HOST", "api.test")])])
        .unwrap();
    ws.set_secret("mem".into(), "mem-value".into());
    ws.set_secret_provider(provider(&[("kc", "kc-value")]));
    let p = ws.preview("{{HOST}} {{secret.kc}} {{secret.mem}} {{secret.nope}} {{X}}".into());
    assert_eq!(p.resolved, "api.test •••••• •••••• {{secret.nope}} {{X}}");
    assert_eq!(p.unresolved, vec!["X", "secret.nope"]);
    assert!(!p.resolved.contains("kc-value") && !p.resolved.contains("mem-value"));
}
