use std::fs;
use std::path::Path;

use xrest_core::*;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for e in fs::read_dir(src).unwrap() {
        let e = e.unwrap();
        let to = dst.join(e.file_name());
        if e.path().is_dir() {
            copy_dir(&e.path(), &to);
        } else {
            fs::copy(e.path(), to).unwrap();
        }
    }
}

fn spec(url: &str) -> RequestSpec {
    RequestSpec {
        method: "GET".into(),
        url: url.into(),
        ..Default::default()
    }
}

#[test]
fn create_then_open_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::create(dir.path(), "My API").unwrap();
    assert_eq!(ws.service().name, "My API");
    let names: Vec<_> = ws
        .environments()
        .iter()
        .map(|e| (e.name.as_str(), e.is_unsafe))
        .collect();
    assert_eq!(
        names,
        vec![("DEV", false), ("STAGE", false), ("PROD", true)]
    );
    assert!(dir.path().join("endpoints").is_dir());

    let ignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(ignore.contains("/scratchpad/") && ignore.contains("/.xrest/"));

    assert!(matches!(
        Workspace::create(dir.path(), "again"),
        Err(Error::AlreadyExists(_))
    ));
    let reopened = Workspace::open(dir.path()).unwrap();
    assert_eq!(reopened.service().id, ws.service().id);
}

#[test]
fn open_rejects_non_workspace() {
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        Workspace::open(dir.path()),
        Err(Error::NotAWorkspace(_))
    ));
}

#[test]
fn loads_legacy_go_format() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("legacy-service"), dir.path());
    let ws = Workspace::open(dir.path()).unwrap();

    assert_eq!(ws.service().name, "Petstore");
    assert_eq!(ws.service().effective_auth_kind(), AuthKind::Bearer);
    assert_eq!(
        ws.endpoints().len(),
        1,
        "broken file skipped, good one kept"
    );
    assert_eq!(ws.warnings().len(), 1);
    assert!(ws.warnings()[0].contains("e-broken.yaml"));

    let ep = ws.endpoint("e-1").unwrap();
    assert_eq!(ep.params[0].name, "limit");
    assert_eq!(ep.last_version, 2);

    // Legacy selectedEnvironment is honoured until the user picks one.
    assert_eq!(ws.selected_environment().unwrap().name, "STAGE");
    // `secretKey: ""` is "no secret"; a real key is exposed.
    assert!(ws.environments()[1].variables[0].secret().is_none());
    assert_eq!(
        ws.environments()[0].variables[1].secret(),
        Some("dev-token")
    );
}

#[test]
fn saving_legacy_endpoint_drops_inline_versions_and_stubs() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("legacy-service"), dir.path());
    let mut ws = Workspace::open(dir.path()).unwrap();
    let mut s = ws.endpoint("e-1").unwrap().spec();
    s.url = "{{BASE_URL}}/v2/pets".into();
    ws.update_endpoint("e-1", None, s).unwrap();
    let text = fs::read_to_string(dir.path().join("endpoints/e-1.yaml")).unwrap();
    assert!(!text.contains("versions:"));
    assert!(text.contains("serviceId: s-1700000000000"));
    assert!(text.contains("lastUpdated:"));
    // Version history lands in .xrest/, which must be gitignored even for pre-existing workspaces.
    let ignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(ignore.contains("/.xrest/"));
    assert!(dir.path().join(".xrest/versions/e-1.json").is_file());
}

#[test]
fn selected_environment_is_persisted_outside_service_yaml() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "x").unwrap();
    assert_eq!(ws.selected_environment().unwrap().name, "DEV");
    assert!(!ws.send_policy().requires_confirmation);
    ws.select_environment("PROD").unwrap();
    assert!(ws.send_policy().requires_confirmation);
    assert!(matches!(
        ws.select_environment("NOPE"),
        Err(Error::UnknownEnvironment(_))
    ));

    let again = Workspace::open(dir.path()).unwrap();
    assert_eq!(again.selected_environment().unwrap().name, "PROD");
    let service_yaml = fs::read_to_string(dir.path().join("service.yaml")).unwrap();
    assert!(!service_yaml.contains("selectedEnvironment"));
    assert!(dir.path().join(".xrest/state.yaml").is_file());
}

#[test]
fn endpoint_crud_and_versioning() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "x").unwrap();
    let ep = ws
        .create_endpoint("Users", spec("{{BASE_URL}}/users"))
        .unwrap();
    assert_eq!(ep.last_version, 1);
    assert!(ep.id.starts_with("e-"));
    assert_eq!(ep.service_id, ws.service().id);
    assert!(dir
        .path()
        .join(format!("endpoints/{}.yaml", ep.id))
        .is_file());

    // Unchanged save: no new version.
    let same = ws.update_endpoint(&ep.id, None, ep.spec()).unwrap();
    assert_eq!(same.last_version, 1);
    // Rename only: no new version either.
    let renamed = ws
        .update_endpoint(&ep.id, Some("People"), ep.spec())
        .unwrap();
    assert_eq!(renamed.last_version, 1);
    assert_eq!(renamed.name, "People");

    let mut s = ep.spec();
    s.method = "POST".into();
    s.body = Body::from_text("{}");
    let v2 = ws.update_endpoint(&ep.id, None, s).unwrap();
    assert_eq!(v2.last_version, 2);

    let versions = ws.endpoint_versions(&ep.id, 10).unwrap();
    assert_eq!(
        versions.iter().map(|v| v.version).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert_eq!(versions[0].config.method, "POST");
    assert_eq!(versions[1].config.method, "GET");

    // Persisted.
    let again = Workspace::open(dir.path()).unwrap();
    assert_eq!(again.endpoint(&ep.id).unwrap().name, "People");
    assert_eq!(again.endpoint_versions(&ep.id, 10).unwrap().len(), 2);

    ws.delete_endpoint(&ep.id).unwrap();
    assert!(ws.endpoint(&ep.id).is_err());
    assert!(!dir
        .path()
        .join(format!("endpoints/{}.yaml", ep.id))
        .exists());
    assert!(matches!(
        ws.update_endpoint("nope", None, spec("x")),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn version_history_is_capped() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "x").unwrap();
    ws.set_max_versions(5);
    let ep = ws.create_endpoint("E", spec("http://a.io/0")).unwrap();
    for i in 1..10 {
        ws.update_endpoint(&ep.id, None, spec(&format!("http://a.io/{i}")))
            .unwrap();
    }
    let v = ws.endpoint_versions(&ep.id, 100).unwrap();
    assert_eq!(v.len(), 5);
    assert_eq!(v[0].version, 10);
    assert_eq!(ws.endpoint(&ep.id).unwrap().last_version, 10);
}

#[test]
fn scratchpad_lives_in_gitignored_dir_and_is_unversioned() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "x").unwrap();
    // Simulate a user who removed the entries.
    fs::write(dir.path().join(".gitignore"), "node_modules\n").unwrap();

    let s = ws.create_scratch("try it", spec("http://a.io/")).unwrap();
    assert!(dir
        .path()
        .join(format!("scratchpad/{}.yaml", s.id))
        .is_file());
    let ignore = fs::read_to_string(dir.path().join(".gitignore")).unwrap();
    assert!(ignore.starts_with("node_modules\n"));
    assert!(ignore.contains("/scratchpad/"));

    let mut changed = s.spec.clone();
    changed.url = "http://a.io/2".into();
    let u = ws.update_scratch(&s.id, Some("renamed"), changed).unwrap();
    assert_eq!(
        (u.name.as_str(), u.spec.url.as_str()),
        ("renamed", "http://a.io/2")
    );

    let again = Workspace::open(dir.path()).unwrap();
    assert_eq!(again.scratch_requests().len(), 1);
    assert_eq!(again.scratch(&s.id).unwrap().name, "renamed");
    assert!(again.endpoints().is_empty());

    ws.delete_scratch(&s.id).unwrap();
    assert!(ws.scratch_requests().is_empty());
}

#[test]
fn promote_scratch_creates_versioned_endpoint_and_removes_scratch() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "x").unwrap();
    let mut sp = spec("{{BASE_URL}}/things");
    sp.method = "POST".into();
    sp.body = Body::from_text(r#"{"a":1}"#);
    sp.headers.push(NameValue::new("X-Trace", "1"));
    let s = ws.create_scratch("draft", sp.clone()).unwrap();

    let ep = ws.promote_scratch(&s.id, Some("Create thing")).unwrap();
    assert_eq!(ep.name, "Create thing");
    assert_eq!(ep.spec(), sp);
    assert_eq!(ep.last_version, 1);
    assert_eq!(ep.service_id, ws.service().id);
    assert!(ws.scratch(&s.id).is_err());
    assert!(!dir
        .path()
        .join(format!("scratchpad/{}.yaml", s.id))
        .exists());
    assert!(dir
        .path()
        .join(format!("endpoints/{}.yaml", ep.id))
        .is_file());
    assert_eq!(ws.endpoint_versions(&ep.id, 5).unwrap().len(), 1);

    // Name falls back to the scratch name.
    let s2 = ws
        .create_scratch("fallback name", spec("http://a.io/"))
        .unwrap();
    assert_eq!(
        ws.promote_scratch(&s2.id, None).unwrap().name,
        "fallback name"
    );
    assert!(matches!(
        ws.promote_scratch("missing", None),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn curl_import_into_scratch_and_endpoint() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "x").unwrap();
    let s = ws
        .import_curl_as_scratch(r#"curl -X POST https://h.io/api/items?x=1 -H 'A: b' -d '{"n":1}'"#)
        .unwrap();
    assert_eq!(s.name, "api items");
    assert_eq!(s.spec.method, "POST");
    assert_eq!(s.spec.params, vec![NameValue::new("x", "1")]);
    assert_eq!(
        s.spec.body,
        Body::Json {
            content: r#"{"n":1}"#.into()
        }
    );
    let e = ws
        .import_curl_as_endpoint("curl https://h.io/ping")
        .unwrap();
    assert_eq!(e.name, "ping");
    assert!(matches!(
        ws.import_curl_as_scratch("nope"),
        Err(Error::Curl(_))
    ));
}

#[test]
fn rejects_unsafe_ids() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "x").unwrap();
    assert!(matches!(
        ws.delete_endpoint("../x"),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        ws.update_scratch("../x", None, spec("a")),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn deferred_modules_report_not_implemented() {
    assert!(matches!(
        xrest_core::swagger::import_spec("{}", "s"),
        Err(Error::NotImplemented(_))
    ));
    assert!(matches!(
        xrest_core::git::status(Path::new(".")),
        Err(Error::NotImplemented(_))
    ));
}

// ---- format version -----------------------------------------------------------------------

#[test]
fn new_workspaces_are_written_at_the_current_format() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::create(dir.path(), "svc").unwrap();
    assert_eq!(ws.format_version(), FORMAT_VERSION);
    let yaml = fs::read_to_string(dir.path().join("service.yaml")).unwrap();
    assert!(
        yaml.contains(&format!("formatVersion: {FORMAT_VERSION}")),
        "{yaml}"
    );
}

#[test]
fn legacy_workspace_opens_unchanged_and_upgrades_on_first_save() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("legacy-service"), dir.path());
    let before = fs::read_to_string(dir.path().join("service.yaml")).unwrap();

    let mut ws = Workspace::open(dir.path()).unwrap();
    assert_eq!(ws.format_version(), 0);
    assert_eq!(ws.selected_environment().unwrap().name, "STAGE");
    // Opening and reading never rewrites tracked files.
    assert_eq!(
        before,
        fs::read_to_string(dir.path().join("service.yaml")).unwrap()
    );

    ws.create_scratch("probe", spec("http://h.io/x")).unwrap();
    assert_eq!(ws.format_version(), FORMAT_VERSION);
    let after = fs::read_to_string(dir.path().join("service.yaml")).unwrap();
    assert!(
        after.contains(&format!("formatVersion: {FORMAT_VERSION}")),
        "{after}"
    );
    assert!(!after.contains("selectedEnvironment"), "{after}");

    // The legacy environment choice survived the move into .xrest/state.yaml.
    let reopened = Workspace::open(dir.path()).unwrap();
    assert_eq!(reopened.format_version(), FORMAT_VERSION);
    assert_eq!(reopened.selected_environment().unwrap().name, "STAGE");
}

#[test]
fn workspace_from_a_newer_xrest_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    Workspace::create(dir.path(), "svc").unwrap();
    let path = dir.path().join("service.yaml");
    let yaml = fs::read_to_string(&path).unwrap().replace(
        &format!("formatVersion: {FORMAT_VERSION}"),
        &format!("formatVersion: {}", FORMAT_VERSION + 1),
    );
    fs::write(&path, yaml).unwrap();
    match Workspace::open(dir.path()) {
        Err(Error::UnsupportedFormat {
            found, supported, ..
        }) => assert_eq!((found, supported), (FORMAT_VERSION + 1, FORMAT_VERSION)),
        other => panic!("expected UnsupportedFormat, got {:?}", other.err()),
    }
}

#[test]
fn typed_body_and_options_persist_through_endpoint_versions() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    let mut s = spec("http://h.io/upload");
    s.body = Body::Multipart {
        fields: vec![NameValue::new("a", "1"), NameValue::file("f", "data/x.bin")],
    };
    s.options = RequestOptions {
        timeout_ms: Some(1500),
        follow_redirects: Some(false),
        ..Default::default()
    };
    let ep = ws.create_endpoint("Upload", s.clone()).unwrap();

    let mut s2 = s.clone();
    s2.body = Body::Json {
        content: "{}".into(),
    };
    ws.update_endpoint(&ep.id, None, s2.clone()).unwrap();

    let reopened = Workspace::open(dir.path()).unwrap();
    assert_eq!(reopened.endpoint(&ep.id).unwrap().spec(), s2);
    let versions = reopened.endpoint_versions(&ep.id, 10).unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[1].config.spec(), s);
}

#[test]
fn format_1_workspace_opens_and_is_stamped_2_on_first_save() {
    let dir = tempfile::tempdir().unwrap();
    Workspace::create(dir.path(), "svc").unwrap();
    let path = dir.path().join("service.yaml");
    let yaml = fs::read_to_string(&path)
        .unwrap()
        .replace("formatVersion: 2", "formatVersion: 1");
    fs::write(&path, yaml).unwrap();

    let mut ws = Workspace::open(dir.path()).unwrap();
    assert_eq!(ws.format_version(), 1);
    let mut s = spec("http://h.io/u/{id}");
    s.path_params = vec![NameValue::new("id", "7")];
    let ep = ws.create_endpoint("by id", s.clone()).unwrap();
    assert_eq!(ws.format_version(), 2);
    let after = fs::read_to_string(&path).unwrap();
    assert!(after.contains("formatVersion: 2"), "{after}");

    let reopened = Workspace::open(dir.path()).unwrap();
    let got = reopened.endpoint(&ep.id).unwrap();
    assert_eq!(got.spec().path_params, s.path_params);
}

#[test]
fn format_3_workspace_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    Workspace::create(dir.path(), "svc").unwrap();
    let path = dir.path().join("service.yaml");
    let yaml = fs::read_to_string(&path)
        .unwrap()
        .replace("formatVersion: 2", "formatVersion: 3");
    fs::write(&path, yaml).unwrap();
    assert!(matches!(
        Workspace::open(dir.path()),
        Err(Error::UnsupportedFormat {
            found: 3,
            supported: 2,
            ..
        })
    ));
}

// ---- environments and service settings (write APIs) -----------------------------------------

fn env(name: &str, vars: Vec<NameValue>) -> Environment {
    Environment {
        name: name.into(),
        is_unsafe: false,
        variables: vars,
    }
}

#[test]
fn save_environments_rejects_bad_names_and_leaves_the_file_alone() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    let path = dir.path().join("environments.yaml");
    let before = fs::read_to_string(&path).unwrap();
    for bad in [
        vec![env("", vec![])],
        vec![env("   ", vec![])],
        vec![env("A", vec![]), env(" A ", vec![])],
        vec![env("A", vec![]), env("A", vec![])],
    ] {
        assert!(
            matches!(ws.save_environments(bad), Err(Error::InvalidInput(_))),
            "expected InvalidInput"
        );
    }
    assert_eq!(before, fs::read_to_string(&path).unwrap());
    assert_eq!(ws.environments().len(), 3);
}

#[test]
fn save_environments_round_trips_trims_names_and_stamps_format() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("legacy-service"), dir.path());
    let mut ws = Workspace::open(dir.path()).unwrap();
    assert_eq!(ws.format_version(), 0);
    let mut secret = NameValue::new("API", "");
    secret.secret_key = Some("k".into());
    let mut prod = env("  PROD ", vec![NameValue::new("A", "1"), secret.clone()]);
    prod.is_unsafe = true;
    ws.save_environments(vec![prod, env("STAGE", vec![])])
        .unwrap();
    assert_eq!(ws.format_version(), FORMAT_VERSION);

    let again = Workspace::open(dir.path()).unwrap();
    assert_eq!(again.format_version(), FORMAT_VERSION);
    let names: Vec<_> = again.environments().iter().map(|e| &e.name[..]).collect();
    assert_eq!(names, vec!["PROD", "STAGE"]);
    assert!(again.environments()[0].is_unsafe);
    assert_eq!(again.environments()[0].variables[1], secret);
}

#[test]
fn save_environments_keeps_selection_or_falls_back_to_first() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    ws.select_environment("STAGE").unwrap();
    ws.save_environments(vec![env("X", vec![]), env("STAGE", vec![])])
        .unwrap();
    assert_eq!(ws.selected_environment().unwrap().name, "STAGE");
    assert_eq!(
        Workspace::open(dir.path())
            .unwrap()
            .selected_environment()
            .unwrap()
            .name,
        "STAGE"
    );

    ws.save_environments(vec![env("X", vec![]), env("Y", vec![])])
        .unwrap();
    assert_eq!(ws.selected_environment().unwrap().name, "X");
    // The stale choice must not come back if its name is re-added later.
    ws.save_environments(vec![env("Y", vec![]), env("STAGE", vec![])])
        .unwrap();
    assert_eq!(ws.selected_environment().unwrap().name, "Y");
    let again = Workspace::open(dir.path()).unwrap();
    assert_eq!(again.selected_environment().unwrap().name, "Y");

    ws.save_environments(vec![]).unwrap();
    assert!(ws.selected_environment().is_none());
}

fn bearer_settings() -> ServiceSettings {
    ServiceSettings {
        name: "Renamed".into(),
        is_authenticated: true,
        auth_type: AuthKind::Apikey,
        basic_user: "u".into(),
        basic_pass: "p".into(),
        bearer_token: "{{secret.t}}".into(),
        api_key_name: "X-Key".into(),
        api_key_value: "v".into(),
        api_key_location: "query".into(),
        preflight: Some(Preflight {
            enabled: true,
            method: "POST".into(),
            url: "https://auth/token".into(),
            token_key: "tok".into(),
            ..Default::default()
        }),
    }
}

#[test]
fn update_service_settings_round_trips_and_leaves_endpoints_untouched() {
    let dir = tempfile::tempdir().unwrap();
    copy_dir(&fixture("legacy-service"), dir.path());
    let mut ws = Workspace::open(dir.path()).unwrap();
    let id = ws.service().id.clone();
    let git = ws.service().git_url.clone();
    let ep = ws.create_endpoint("probe", spec("http://h.io/x")).unwrap();
    let ep_file = dir.path().join(format!("endpoints/{}.yaml", ep.id));
    let ep_before = fs::read_to_string(&ep_file).unwrap();
    let n_endpoints = ws.endpoints().len();

    ws.update_service_settings(bearer_settings()).unwrap();
    assert_eq!(ws.service_settings(), bearer_settings());

    let again = Workspace::open(dir.path()).unwrap();
    assert_eq!(again.service_settings(), bearer_settings());
    assert_eq!(again.service().id, id);
    assert_eq!(again.service().git_url, git);
    assert_eq!(again.format_version(), FORMAT_VERSION);
    assert_eq!(again.service().effective_auth_kind(), AuthKind::Apikey);
    assert_eq!(again.endpoints().len(), n_endpoints);
    assert_eq!(ep_before, fs::read_to_string(&ep_file).unwrap());

    // Switching auth off and dropping the preflight also persists.
    let mut off = bearer_settings();
    off.auth_type = AuthKind::None;
    off.is_authenticated = false;
    off.preflight = None;
    let mut ws = again;
    ws.update_service_settings(off.clone()).unwrap();
    assert_eq!(Workspace::open(dir.path()).unwrap().service_settings(), off);
}

#[test]
fn update_service_settings_rejects_empty_name() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    let mut s = ws.service_settings();
    s.name = "  ".into();
    assert!(matches!(
        ws.update_service_settings(s),
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(ws.service().name, "svc");
}

// ---- interpolation preview ------------------------------------------------------------------

fn preview_ws(dir: &tempfile::TempDir) -> (Workspace, std::sync::Arc<MemorySecretStore>) {
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    let store = std::sync::Arc::new(MemorySecretStore::default());
    ws.set_secret_store(store.clone());
    let mut backed = NameValue::new("TOKEN", "");
    backed.secret_key = Some("env-tok".into());
    let mut off = NameValue::new("OFF", "no");
    off.enabled = false;
    ws.save_environments(vec![env(
        "DEV",
        vec![
            NameValue::new("HOST", "api.test"),
            NameValue::new("PATH_", "{{HOST}}/v1"),
            backed,
            off,
            NameValue::new("HOST", "dup"),
        ],
    )])
    .unwrap();
    (ws, store)
}

#[test]
fn preview_resolves_variables_and_masks_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let (ws, store) = preview_ws(&dir);
    store.set("direct", "hunter2-direct");
    store.set("env-tok", "hunter2-env");
    let p = ws.preview("https://{{PATH_}}?a={{secret.direct}}&b={{ TOKEN }}");
    assert_eq!(p.resolved, "https://dup/v1?a=••••••&b=••••••");
    assert!(p.unresolved.is_empty(), "{:?}", p.unresolved);
    assert!(!p.resolved.contains("hunter2"));
}

#[test]
fn preview_reports_unresolved_variables_and_missing_secrets() {
    let dir = tempfile::tempdir().unwrap();
    let (ws, _store) = preview_ws(&dir);
    let p = ws.preview("{{NOPE}} {{OFF}} {{secret.gone}} {{TOKEN}} {{NOPE}} {{secret.gone}}");
    assert_eq!(
        p.resolved,
        "{{NOPE}} {{OFF}} {{secret.gone}} {{secret.env-tok}} {{NOPE}} {{secret.gone}}"
    );
    assert_eq!(
        p.unresolved,
        vec!["NOPE", "OFF", "secret.env-tok", "secret.gone"]
    );
}

#[test]
fn preview_without_environment_leaves_variables_unresolved() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    ws.save_environments(vec![]).unwrap();
    let p = ws.preview("{{A}}/x");
    assert_eq!(p.resolved, "{{A}}/x");
    assert_eq!(p.unresolved, vec!["A"]);
    assert!(ws.variable_names().is_empty());
}

#[test]
fn variable_names_lists_enabled_names_of_the_selected_environment() {
    let dir = tempfile::tempdir().unwrap();
    let (mut ws, _) = preview_ws(&dir);
    assert_eq!(ws.variable_names(), vec!["HOST", "PATH_", "TOKEN"]);
    let mut e2 = env(
        "Z",
        vec![NameValue::new("ONLY_Z", "1"), NameValue::new("", "x")],
    );
    e2.is_unsafe = false;
    let mut envs = ws.environments().to_vec();
    envs.push(e2);
    ws.save_environments(envs).unwrap();
    ws.select_environment("Z").unwrap();
    assert_eq!(ws.variable_names(), vec!["ONLY_Z"]);
}
