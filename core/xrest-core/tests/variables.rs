//! Variable layers and workspace safety rules, end to end through `Workspace`.

use std::fs;
use std::sync::Arc;

use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use xrest_core::*;

fn nv(name: &str, value: &str) -> NameValue {
    NameValue::new(name, value)
}

fn env(name: &str, is_unsafe: bool, vars: Vec<NameValue>) -> Environment {
    Environment {
        name: name.into(),
        is_unsafe,
        variables: vars,
        ..Default::default()
    }
}

fn spec(method: &str, url: &str) -> RequestSpec {
    RequestSpec {
        method: method.into(),
        url: url.into(),
        ..Default::default()
    }
}

fn no_proxy(ws: &mut Workspace) {
    ws.set_http_options(HttpOptions {
        use_system_proxy: false,
        ..Default::default()
    });
}

/// A workspace whose DEV and PROD both point at `server`.
fn workspace(dir: &tempfile::TempDir, server: &MockServer) -> Workspace {
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    no_proxy(&mut ws);
    ws.save_environments(vec![
        env(
            "DEV",
            false,
            vec![nv("BASE_URL", &server.uri()), nv("USER", "dev-user")],
        ),
        env("PROD", true, vec![nv("BASE_URL", &server.uri())]),
    ])
    .unwrap();
    ws
}

// ---- layers ---------------------------------------------------------------------------------

#[tokio::test]
async fn every_layer_reaches_the_request_with_the_right_precedence() {
    let dir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let mut ws = workspace(&dir, &server);

    let globals = Arc::new(GlobalVariables::new(vec![
        nv("EMAIL", "me@example.com"),
        nv("REGION", "global-region"),
    ]));
    ws.set_global_variables(globals.clone());
    ws.save_workspace_variables(vec![nv("REGION", "us-east-1"), nv("API_VERSION", "v2")])
        .unwrap();
    ws.save_local_overrides("DEV", vec![nv("USER", "my-user")])
        .unwrap();

    Mock::given(path("/v2/users/my-user"))
        .and(header("x-region", "us-east-1"))
        .and(header("x-email", "me@example.com"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/{{API_VERSION}}/users/{{USER}}");
    s.headers = vec![nv("X-Region", "{{REGION}}"), nv("X-Email", "{{EMAIL}}")];
    let resp = ws.prepare_send(&s, false).unwrap().execute().await.unwrap();
    assert_eq!(resp.status, 200);
    assert!(resp.unresolved_variables.is_empty());

    // Globals are live: a change reaches the next send without reattaching.
    globals.set(vec![nv("EMAIL", "other@example.com")]);
    assert_eq!(ws.preview("{{EMAIL}}").resolved, "other@example.com");
}

#[test]
fn overrides_are_per_environment_local_and_persist() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    ws.save_local_overrides("DEV", vec![nv("USER", "7"), nv("  ", "dropped")])
        .unwrap();
    assert_eq!(ws.local_overrides("DEV"), &[nv("USER", "7")]);
    assert!(ws.local_overrides("PROD").is_empty());
    assert!(matches!(
        ws.save_local_overrides("NOPE", vec![]),
        Err(Error::UnknownEnvironment(_))
    ));

    // Stored under .xrest/, which is gitignored; never in a tracked file.
    let stored = fs::read_to_string(dir.path().join(".xrest/overrides.yaml")).unwrap();
    assert!(
        stored.contains("DEV") && stored.contains("USER"),
        "{stored}"
    );
    let ignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(ignore.contains("/.xrest/"));

    let reopened = Workspace::open(dir.path()).unwrap();
    assert_eq!(reopened.local_overrides("DEV"), &[nv("USER", "7")]);

    // An empty list removes the entry.
    let mut ws = reopened;
    ws.save_local_overrides("DEV", vec![]).unwrap();
    assert!(ws.local_overrides("DEV").is_empty());
    let stored = fs::read_to_string(dir.path().join(".xrest/overrides.yaml")).unwrap();
    assert!(!stored.contains("DEV"), "{stored}");
}

#[test]
fn workspace_variables_persist_in_variables_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    ws.save_workspace_variables(vec![nv(" API_VERSION ", "v2"), nv("", "x")])
        .unwrap();
    assert_eq!(ws.workspace_variables(), &[nv("API_VERSION", "v2")]);
    let file = fs::read_to_string(dir.path().join("variables.yaml")).unwrap();
    assert!(file.contains("API_VERSION"), "{file}");
    let reopened = Workspace::open(dir.path()).unwrap();
    assert_eq!(reopened.workspace_variables(), &[nv("API_VERSION", "v2")]);
}

#[test]
fn report_explains_where_each_value_comes_from() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    ws.save_environments(vec![
        env("DEV", false, vec![nv("USER", "42")]),
        env("PROD", true, vec![nv("REGION", "eu-west-1")]),
    ])
    .unwrap();
    ws.save_workspace_variables(vec![nv("REGION", "us-east-1")])
        .unwrap();
    ws.save_local_overrides("DEV", vec![nv("USER", "7")])
        .unwrap();

    let dev = ws.variable_report(Some("DEV")).unwrap();
    let user = dev.iter().find(|r| r.name == "USER").unwrap();
    assert_eq!(user.winner, Some(VariableLayer::Local));
    assert_eq!(user.values[0].value, "7");
    assert_eq!(user.values[1].layer, VariableLayer::Environment);
    let region = dev.iter().find(|r| r.name == "REGION").unwrap();
    assert_eq!(region.winner, Some(VariableLayer::Workspace));

    let prod = ws.variable_report(Some("PROD")).unwrap();
    let region = prod.iter().find(|r| r.name == "REGION").unwrap();
    assert_eq!(region.winner, Some(VariableLayer::Environment));
    assert_eq!(region.values.len(), 2);
    assert!(!prod.iter().any(|r| r.name == "USER"));

    // `None` means the selected environment.
    assert_eq!(ws.variable_report(None).unwrap(), dev);
    assert!(ws.variable_report(Some("NOPE")).is_err());
}

#[test]
fn secret_keys_in_layers_are_reported_as_in_use() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    let secret = |name: &str, key: &str| NameValue {
        secret_key: Some(key.into()),
        ..nv(name, "")
    };
    ws.set_global_variables(Arc::new(GlobalVariables::new(vec![secret(
        "G",
        "global-key",
    )])));
    ws.save_workspace_variables(vec![secret("W", "workspace-key")])
        .unwrap();
    ws.save_local_overrides("DEV", vec![secret("L", "local-key")])
        .unwrap();
    let keys = ws.secret_keys_in_use();
    for k in ["global-key", "workspace-key", "local-key"] {
        assert!(keys.contains(&k.to_string()), "{k} in {keys:?}");
    }
}

#[tokio::test]
async fn builtins_resolve_once_per_send_and_never_count_as_unresolved() {
    let dir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let ws = workspace(&dir, &server);
    Mock::given(method("POST"))
        .and(|r: &wiremock::Request| {
            let id = r
                .headers
                .get("x-id")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            let body = String::from_utf8_lossy(&r.body);
            uuid::Uuid::parse_str(id).is_ok() && body == format!("{{\"id\":\"{id}\"}}")
        })
        .respond_with(ResponseTemplate::new(201))
        .expect(1)
        .mount(&server)
        .await;
    let mut s = spec("POST", "{{BASE_URL}}/items");
    s.headers = vec![nv("X-Id", "{{$uuid}}")];
    s.body = Body::Json {
        content: r#"{"id":"{{$uuid}}"}"#.into(),
    };
    let resp = ws.prepare_send(&s, false).unwrap().execute().await.unwrap();
    assert_eq!(resp.status, 201);
    assert!(resp.unresolved_variables.is_empty());

    // A preview leaves built-ins as written and does not flag them.
    let p = ws.preview("{{$uuid}} {{$timestamp}}");
    assert_eq!(p.resolved, "{{$uuid}} {{$timestamp}}");
    assert!(p.unresolved.is_empty());
}

// ---- safety rules ---------------------------------------------------------------------------

#[tokio::test]
async fn changes_only_scope_sends_reads_without_confirmation_but_not_writes() {
    let dir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let mut ws = workspace(&dir, &server);
    let mut envs = ws.environments().to_vec();
    envs[1].confirm = ConfirmScope::Changes;
    envs[1].min_countdown_seconds = 5;
    ws.save_environments(envs).unwrap();
    ws.select_environment("PROD").unwrap();

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(204))
        .expect(1)
        .mount(&server)
        .await;

    let read = spec("GET", "{{BASE_URL}}/x");
    assert!(!ws.send_policy_for("GET").requires_confirmation);
    ws.prepare_send(&read, false)
        .unwrap()
        .execute()
        .await
        .unwrap();

    let write = spec("DELETE", "{{BASE_URL}}/x");
    let policy = ws.send_policy_for("DELETE");
    assert!(policy.requires_confirmation);
    assert_eq!(policy.min_countdown_seconds, 5);
    assert!(matches!(
        ws.prepare_send(&write, false),
        Err(Error::ConfirmationRequired { .. })
    ));
    ws.prepare_send(&write, true)
        .unwrap()
        .execute()
        .await
        .unwrap();

    // The environment is still unsafe as a whole.
    assert!(ws.send_policy().requires_confirmation);
}

#[test]
fn environment_rules_are_validated_and_persisted() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    // New workspaces get a color per environment.
    let colors: Vec<_> = ws
        .environments()
        .iter()
        .map(|e| e.color.clone().unwrap_or_default())
        .collect();
    assert_eq!(colors, vec!["green", "amber", "red"]);

    let mut envs = ws.environments().to_vec();
    envs[2].confirm = ConfirmScope::Changes;
    envs[2].min_countdown_seconds = 999;
    envs[2].color = Some(" Purple ".into());
    ws.save_environments(envs.clone()).unwrap();
    let prod = &ws.environments()[2];
    assert_eq!(prod.min_countdown_seconds, MAX_MIN_COUNTDOWN_SECONDS);
    assert_eq!(prod.color.as_deref(), Some("purple"));

    let yaml = fs::read_to_string(dir.path().join("environments.yaml")).unwrap();
    assert!(yaml.contains("confirm: changes"), "{yaml}");
    assert!(yaml.contains("minCountdownSeconds: 30"), "{yaml}");
    let reopened = Workspace::open(dir.path()).unwrap();
    assert_eq!(reopened.environments()[2].confirm, ConfirmScope::Changes);

    envs[0].color = Some("chartreuse".into());
    assert!(matches!(
        ws.save_environments(envs),
        Err(Error::InvalidInput(_))
    ));
}

#[test]
fn old_environment_files_default_to_confirm_every_request() {
    let dir = tempfile::tempdir().unwrap();
    Workspace::create(dir.path(), "svc").unwrap();
    fs::write(
        dir.path().join("environments.yaml"),
        "- name: PROD\n  isUnsafe: true\n  variables: []\n",
    )
    .unwrap();
    let ws = Workspace::open(dir.path()).unwrap();
    let prod = &ws.environments()[0];
    assert_eq!(prod.confirm, ConfirmScope::Every);
    assert_eq!(prod.min_countdown_seconds, 0);
    assert_eq!(prod.color, None);
    assert!(ws.send_policy_for("GET").requires_confirmation);
}

#[test]
fn appearance_is_validated_and_stored_in_service_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    assert!(ws.appearance().is_empty());
    ws.set_appearance(WorkspaceAppearance {
        color: Some("Blue".into()),
        icon: Some("server".into()),
    })
    .unwrap();
    let yaml = fs::read_to_string(dir.path().join("service.yaml")).unwrap();
    assert!(
        yaml.contains("color: blue") && yaml.contains("icon: server"),
        "{yaml}"
    );
    assert_eq!(
        Workspace::open(dir.path())
            .unwrap()
            .appearance()
            .icon
            .as_deref(),
        Some("server")
    );
    for bad in [
        WorkspaceAppearance {
            color: Some("neon".into()),
            icon: None,
        },
        WorkspaceAppearance {
            color: None,
            icon: Some("rocket".into()),
        },
    ] {
        assert!(matches!(
            ws.set_appearance(bad),
            Err(Error::InvalidInput(_))
        ));
    }
    // Clearing works.
    ws.set_appearance(WorkspaceAppearance::default()).unwrap();
    let yaml = fs::read_to_string(dir.path().join("service.yaml")).unwrap();
    assert!(!yaml.contains("appearance"), "{yaml}");
}

#[test]
fn format_2_workspace_upgrades_when_variables_are_first_saved() {
    let dir = tempfile::tempdir().unwrap();
    Workspace::create(dir.path(), "svc").unwrap();
    let path = dir.path().join("service.yaml");
    let yaml = fs::read_to_string(&path).unwrap().replace(
        &format!("formatVersion: {FORMAT_VERSION}"),
        "formatVersion: 2",
    );
    fs::write(&path, yaml).unwrap();
    let mut ws = Workspace::open(dir.path()).unwrap();
    assert_eq!(ws.format_version(), 2);
    // Local overrides are per user and untracked: they do not upgrade the shared format.
    ws.save_local_overrides("DEV", vec![nv("A", "1")]).unwrap();
    assert_eq!(ws.format_version(), 2);
    ws.save_workspace_variables(vec![nv("A", "1")]).unwrap();
    assert_eq!(ws.format_version(), FORMAT_VERSION);
}
