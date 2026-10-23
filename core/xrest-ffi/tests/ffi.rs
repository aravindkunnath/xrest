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
fn restore_diff_and_max_versions_across_ffi() {
    let dir = tempfile::tempdir().unwrap();
    let ws = new_ws(&dir);
    ws.set_max_versions(5);
    let ep = ws
        .create_endpoint("E".into(), spec("http://a.io/1"))
        .unwrap();
    for i in 2..=7 {
        ws.update_endpoint(ep.id.clone(), "E".into(), spec(&format!("http://a.io/{i}")))
            .unwrap();
    }
    let v = ws.endpoint_versions(ep.id.clone(), 100).unwrap();
    assert_eq!(v.len(), 5);
    assert_eq!(v[4].version, 3);
    let restored = ws.restore_endpoint_version(ep.id.clone(), 3).unwrap();
    assert_eq!(restored.last_version, 8);
    assert_eq!(restored.spec.url, "http://a.io/3");
    let d = ws.diff_versions(ep.id.clone(), 7, 8).unwrap();
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].field, "url");
    assert_eq!(d[0].change, "changed");
    assert_eq!(d[0].before.as_deref(), Some("http://a.io/7"));
    assert_eq!(d[0].after.as_deref(), Some("http://a.io/3"));
    assert!(ws.diff_versions(ep.id.clone(), 8, 8).unwrap().is_empty());
    assert!(matches!(
        ws.restore_endpoint_version(ep.id.clone(), 99),
        Err(XRestError::NotFound { .. })
    ));
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
    fn contains(&self, key: String) -> bool {
        self.0.contains_key(&key)
    }
}

/// Existence only: `get` would be a bug in any preview path.
struct NoGetProvider;

impl SecretProvider for NoGetProvider {
    fn get(&self, key: String) -> Option<String> {
        panic!("get({key}) called by a preview");
    }
    fn contains(&self, key: String) -> bool {
        key == "kc"
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

#[test]
fn previews_use_contains_never_get_and_preview_request_url_masks() {
    let dir = tempfile::tempdir().unwrap();
    let ws = new_ws(&dir);
    ws.save_environments(vec![env_cfg("DEV", vec![nv("HOST", "https://api.test")])])
        .unwrap();
    ws.set_secret("mem".into(), "mem-value".into());
    ws.set_secret_provider(Arc::new(NoGetProvider));
    let p = ws.preview("{{secret.kc}} {{secret.mem}} {{secret.nope}}".into());
    assert_eq!(p.resolved, "•••••• •••••• {{secret.nope}}");

    let mut spec = spec("{{HOST}}/u/{id}");
    spec.path_params = vec![nv("id", "42")];
    spec.params = vec![nv("k", "{{secret.kc}}"), nv("limit", "5")];
    let p = ws.preview_request_url(spec);
    assert_eq!(p.resolved, "https://api.test/u/42?k=••••••&limit=5");
    assert!(p.unresolved.is_empty());
}

#[tokio::test]
async fn git_through_ffi() {
    for (k, v) in [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_CONFIG_COUNT", "1"),
        ("GIT_CONFIG_KEY_0", "init.defaultBranch"),
        ("GIT_CONFIG_VALUE_0", "main"),
        ("GIT_AUTHOR_NAME", "Test"),
        ("GIT_AUTHOR_EMAIL", "test@example.com"),
        ("GIT_COMMITTER_NAME", "Test"),
        ("GIT_COMMITTER_EMAIL", "test@example.com"),
    ] {
        std::env::set_var(k, v);
    }
    let remote = tempfile::tempdir().unwrap();
    let out = std::process::Command::new("git")
        .args(["init", "--bare", "-q"])
        .arg(remote.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let dir = tempfile::tempdir().unwrap();
    let ws = XRestWorkspace::create_workspace(dir.path().to_str().unwrap().into(), "svc".into())
        .unwrap();

    assert!(!ws.git_status().await.unwrap().is_repo);
    assert_eq!(ws.git_mode(), GitMode::Manual);
    assert!(matches!(ws.git_pull().await, Err(XRestError::Git { .. })));

    ws.git_init(Some(remote.path().to_str().unwrap().into()))
        .await
        .unwrap();
    let st = ws.git_status().await.unwrap();
    assert!(st.is_repo && st.branch == "main");
    assert!(st.uncommitted.iter().any(|f| f.path == "service.yaml"));
    assert!(ws.git_commit("Initial".into()).await.unwrap().is_some());
    assert_eq!(ws.git_commit("again".into()).await.unwrap(), None);
    ws.git_sync().await.unwrap();
    let st = ws.git_status().await.unwrap();
    assert!(st.last_sync.is_some());
    assert_eq!((st.ahead, st.behind), (0, 0));

    ws.set_git_mode(GitMode::Auto).unwrap();
    assert_eq!(ws.git_mode(), GitMode::Auto);
    ws.create_endpoint("A".into(), spec("http://x")).unwrap();
    assert!(ws.git_status().await.unwrap().uncommitted.is_empty());
    ws.reload().unwrap();
    assert_eq!(ws.list_endpoints().len(), 1);
}

#[tokio::test]
async fn openapi_import_and_fetch_through_ffi() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    let spec = r#"{"openapi":"3.0.0","info":{"title":"Pets"},"servers":[{"url":"https://x.test/v1"}],
        "paths":{"/pets":{"get":{"summary":"List"},"post":{"summary":"Add"}}}}"#;
    Mock::given(path("/spec.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string(spec))
        .mount(&server)
        .await;
    let text = fetch_text(format!("{}/spec.json", server.uri()))
        .await
        .unwrap();
    assert!(matches!(
        fetch_text("file:///etc/hosts".into()).await,
        Err(XRestError::InvalidInput { .. })
    ));

    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("svc").to_string_lossy().into_owned();
    let ws =
        XRestWorkspace::create_workspace_from_openapi(root, "Pets".into(), text.clone()).unwrap();
    assert_eq!(ws.list_endpoints().len(), 2);
    assert_eq!(ws.environments()[0].variables[0].value, "https://x.test/v1");
    let sum = ws.merge_openapi(text).unwrap();
    assert_eq!((sum.added, sum.skipped), (0, 2));
    assert!(matches!(
        ws.merge_openapi("junk".into()),
        Err(XRestError::InvalidInput { .. })
    ));
}
