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
    s.body = "{}".into();
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
    sp.body = r#"{"a":1}"#.into();
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
    assert_eq!(s.spec.body, r#"{"n":1}"#);
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
