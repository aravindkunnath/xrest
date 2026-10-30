//! OpenAPI / Swagger import: parsing, workspace creation, merge, fetch_text.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};
use xrest_core::swagger::{self, fetch_text, fetch_text_with};
use xrest_core::*;

const V2: &str = include_str!("fixtures/petstore_v2.json");
const V3: &str = include_str!("fixtures/petstore_v3.yaml");

fn find<'a>(s: &'a ImportedSpec, method: &str, p: &str) -> &'a ImportedEndpoint {
    s.endpoints
        .iter()
        .find(|e| e.method == method && e.path == p)
        .unwrap_or_else(|| panic!("no {method} {p}"))
}

fn nv(rows: &[NameValue]) -> Vec<(&str, &str)> {
    rows.iter()
        .map(|r| (r.name.as_str(), r.value.as_str()))
        .collect()
}

#[test]
fn v2_base_url_and_endpoint_set() {
    let s = swagger::parse(V2).unwrap();
    assert_eq!(s.title, "Swagger Petstore");
    assert_eq!(
        s.base_url.as_deref(),
        Some("https://petstore.swagger.io/v2")
    );
    let mut ids: Vec<String> = s
        .endpoints
        .iter()
        .map(|e| format!("{} {}", e.method, e.path))
        .collect();
    ids.sort();
    assert_eq!(
        ids,
        [
            "DELETE /pet/{petId}",
            "GET /pet/findByStatus",
            "GET /pet/{petId}",
            "GET /store/inventory",
            "POST /pet",
            "POST /pet/{petId}",
            "PUT /pet"
        ]
    );
}

#[test]
fn name_fallback_summary_then_operation_id_then_method_path() {
    let s = swagger::parse(V2).unwrap();
    assert_eq!(find(&s, "POST", "/pet").name, "Add a new pet");
    assert_eq!(find(&s, "PUT", "/pet").name, "updatePet");
    assert_eq!(
        find(&s, "GET", "/store/inventory").name,
        "GET /store/inventory"
    );
}

#[test]
fn url_keeps_path_tokens_and_adds_path_params_from_item_level_ref() {
    let s = swagger::parse(V2).unwrap();
    let get = find(&s, "GET", "/pet/{petId}");
    assert_eq!(get.spec.url, "{{BASE_URL}}/pet/{petId}");
    assert_eq!(get.spec.method, "GET");
    assert_eq!(nv(&get.spec.path_params), [("petId", "1")]);
    assert!(!get.spec.authenticated);
}

#[test]
fn query_and_header_params_with_defaults() {
    let s = swagger::parse(V2).unwrap();
    let q = find(&s, "GET", "/pet/findByStatus");
    assert_eq!(nv(&q.spec.params), [("status", "available"), ("limit", "")]);
    let d = find(&s, "DELETE", "/pet/{petId}");
    assert_eq!(nv(&d.spec.headers), [("api_key", "")]);
    assert_eq!(nv(&d.spec.path_params), [("petId", "1")]);
    assert!(d.spec.params.is_empty());
}

#[test]
fn v2_body_example_and_form_data() {
    let s = swagger::parse(V2).unwrap();
    match &find(&s, "POST", "/pet").spec.body {
        Body::Json { content } => {
            let v: serde_json::Value = serde_json::from_str(content).unwrap();
            assert_eq!(v["name"], "doggie");
            assert_eq!(v["status"], "available");
        }
        other => panic!("{other:?}"),
    }
    // schema $ref -> definitions example
    match &find(&s, "PUT", "/pet").spec.body {
        Body::Json { content } => {
            let v: serde_json::Value = serde_json::from_str(content).unwrap();
            assert_eq!(v["name"], "rex");
        }
        other => panic!("{other:?}"),
    }
    match &find(&s, "POST", "/pet/{petId}").spec.body {
        Body::Form { fields } => assert_eq!(nv(fields), [("name", ""), ("status", "sold")]),
        other => panic!("{other:?}"),
    }
    assert_eq!(find(&s, "GET", "/pet/{petId}").spec.body, Body::None);
}

#[test]
fn v3_server_variables_use_defaults() {
    let s = swagger::parse(V3).unwrap();
    assert_eq!(s.title, "Petstore v3");
    assert_eq!(s.base_url.as_deref(), Some("https://api.petstore.io/v3"));
}

#[test]
fn v3_params_refs_path_level_and_example() {
    let s = swagger::parse(V3).unwrap();
    let list = find(&s, "GET", "/pets");
    assert_eq!(list.name, "List pets");
    // path-item ref param first, then operation params
    assert_eq!(nv(&list.spec.params), [("limit", "20"), ("tag", "")]);
    assert_eq!(nv(&list.spec.headers), [("X-Request-Id", "abc-123")]);
    let get = find(&s, "GET", "/pets/{petId}");
    assert_eq!(nv(&get.spec.path_params), [("petId", "42")]);
    assert_eq!(find(&s, "OPTIONS", "/pets/{petId}").name, "petOptions");
}

#[test]
fn v3_json_example_media_and_schema_ref_and_form() {
    let s = swagger::parse(V3).unwrap();
    match &find(&s, "POST", "/pets").spec.body {
        Body::Json { content } => {
            let v: serde_json::Value = serde_json::from_str(content).unwrap();
            assert_eq!(v["name"], "Rex");
            assert_eq!(v["tags"][1], "b");
        }
        other => panic!("{other:?}"),
    }
    match &find(&s, "PUT", "/pets/{petId}").spec.body {
        Body::Json { content } => {
            let v: serde_json::Value = serde_json::from_str(content).unwrap();
            assert_eq!(v["id"], 7);
            assert_eq!(v["name"], "rex");
        }
        other => panic!("{other:?}"),
    }
    match &find(&s, "POST", "/login").spec.body {
        Body::Form { fields } => {
            assert_eq!(nv(fields), [("username", ""), ("password", "hunter2")])
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn no_servers_has_no_base_url() {
    let s = swagger::parse(r#"{"openapi":"3.0.0","info":{"title":"T"},"paths":{}}"#).unwrap();
    assert_eq!(s.base_url, None);
    assert!(s.endpoints.is_empty());
}

#[test]
fn invalid_content_is_invalid_input() {
    for bad in [
        "",
        "   ",
        "not: [valid",
        "just text",
        r#"{"info":{"title":"x"},"paths":{}}"#,
        r#"{"openapi":"4.0.0","paths":{}}"#,
        "[1,2]",
    ] {
        match swagger::parse(bad) {
            Err(Error::InvalidInput(_)) => {}
            other => panic!("{bad:?} -> {other:?}"),
        }
    }
}

// ---- workspaces --------------------------------------------------------------------------

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn create_workspace_from_openapi_sets_up_envs_and_saves_endpoints() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("svc");
    let ws = Workspace::create_from_openapi(&root, "Pets", V2).unwrap();
    assert_eq!(ws.service().name, "Pets");
    let envs = ws.environments();
    assert_eq!(
        envs.iter()
            .map(|e| (e.name.as_str(), e.is_unsafe))
            .collect::<Vec<_>>(),
        [("DEV", false), ("STAGE", false), ("PROD", true)]
    );
    for e in envs {
        let v = e.variables.iter().find(|v| v.name == "BASE_URL").unwrap();
        assert_eq!(v.value, "https://petstore.swagger.io/v2");
    }
    assert_eq!(ws.endpoints().len(), 7);
    assert!(ws.endpoints().iter().all(|e| e.last_version == 1));
    assert!(ws.warnings().is_empty());
    // everything is on disk
    let reopened = Workspace::open(&root).unwrap();
    assert_eq!(reopened.endpoints().len(), 7);
    assert_eq!(reopened.environments().len(), 3);
    assert_eq!(
        reopened
            .endpoint_versions(&reopened.endpoints()[0].id, 10)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn create_workspace_without_servers_uses_placeholder_and_warns() {
    let dir = tempfile::tempdir().unwrap();
    let spec = r#"{"openapi":"3.0.0","info":{"title":"T"},"paths":{"/a":{"get":{}}}}"#;
    let ws = Workspace::create_from_openapi(dir.path().join("w"), "W", spec).unwrap();
    for e in ws.environments() {
        assert_eq!(e.variables[0].value, "https://api.example.com");
    }
    assert_eq!(ws.warnings().len(), 1);
    assert!(ws.warnings()[0].contains("api.example.com"));
    assert_eq!(ws.endpoints().len(), 1);
}

#[test]
fn create_workspace_invalid_content_creates_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("w");
    assert!(matches!(
        Workspace::create_from_openapi(&root, "W", "nope"),
        Err(Error::InvalidInput(_))
    ));
    assert!(!root.exists());
}

#[test]
fn create_workspace_empty_name_falls_back_to_title() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::create_from_openapi(dir.path().join("w"), "  ", V3).unwrap();
    assert_eq!(ws.service().name, "Petstore v3");
}

#[test]
fn reimport_keeps_custom_and_deleted_environments() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create_from_openapi(dir.path().join("w"), "W", V3).unwrap();
    let env = |name: &str, is_unsafe: bool| Environment {
        name: name.into(),
        is_unsafe,
        variables: vec![],
        ..Default::default()
    };
    ws.save_environments(vec![env("qa-eu", false), env("perf", true)])
        .unwrap();
    ws.merge_openapi(V3).unwrap();
    let got: Vec<_> = ws
        .environments()
        .iter()
        .map(|e| (e.name.as_str(), e.is_unsafe))
        .collect();
    assert_eq!(got, [("qa-eu", false), ("perf", true)]);
}

#[test]
fn merge_adds_only_new_and_leaves_edited_endpoint_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create_from_openapi(dir.path().join("w"), "W", V3).unwrap();
    let before = ws.endpoints().len();
    // edit an imported endpoint
    let id = ws
        .endpoints()
        .iter()
        .find(|e| e.name == "List pets")
        .unwrap()
        .id
        .clone();
    let mut spec = ws.endpoint(&id).unwrap().spec();
    spec.headers.push(NameValue::new("X-Mine", "1"));
    ws.update_endpoint(&id, Some("My list"), spec).unwrap();
    let edited = ws.endpoint(&id).unwrap().clone();

    // remove one op the spec has, so merge has something to add
    let del = ws
        .endpoints()
        .iter()
        .find(|e| e.name == "Log in")
        .unwrap()
        .id
        .clone();
    ws.delete_endpoint(&del).unwrap();

    let sum = ws.merge_openapi(V3).unwrap();
    assert_eq!(
        sum,
        ImportSummary {
            added: 1,
            skipped: (before - 1) as u32
        }
    );
    assert_eq!(ws.endpoints().len(), before);
    assert_eq!(ws.endpoint(&id).unwrap(), &edited);

    // idempotent
    let sum = ws.merge_openapi(V3).unwrap();
    assert_eq!(
        sum,
        ImportSummary {
            added: 0,
            skipped: before as u32
        }
    );
}

#[test]
fn merge_matches_hand_made_endpoint_by_method_and_path() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "W").unwrap();
    let spec = RequestSpec {
        url: "{{BASE_URL}}/store/inventory".into(),
        ..Default::default()
    };
    ws.create_endpoint("mine", spec).unwrap();
    let sum = ws.merge_openapi(V2).unwrap();
    assert_eq!(
        sum,
        ImportSummary {
            added: 6,
            skipped: 1
        }
    );
    assert_eq!(ws.endpoints().len(), 7);
}

#[test]
fn merge_invalid_content_changes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "W").unwrap();
    assert!(matches!(
        ws.merge_openapi("nope"),
        Err(Error::InvalidInput(_))
    ));
    assert!(ws.endpoints().is_empty());
}

#[test]
fn merge_in_auto_mode_makes_one_commit() {
    let dir = tempfile::tempdir().unwrap();
    for (k, v) in [
        ("GIT_CONFIG_GLOBAL", "/dev/null"),
        ("GIT_CONFIG_NOSYSTEM", "1"),
        ("GIT_AUTHOR_NAME", "T"),
        ("GIT_AUTHOR_EMAIL", "t@example.com"),
        ("GIT_COMMITTER_NAME", "T"),
        ("GIT_COMMITTER_EMAIL", "t@example.com"),
    ] {
        std::env::set_var(k, v);
    }
    let mut ws = Workspace::create(dir.path(), "W").unwrap();
    ws.git_init(None).unwrap();
    ws.git_commit("init").unwrap();
    ws.set_git_mode(GitMode::Auto).unwrap();
    let base = git(dir.path(), &["rev-list", "--count", "HEAD"]);
    let sum = ws.merge_openapi(V2).unwrap();
    assert_eq!(sum.added, 7);
    assert_eq!(
        git(dir.path(), &["rev-list", "--count", "HEAD"])
            .parse::<u32>()
            .unwrap(),
        base.parse::<u32>().unwrap() + 1
    );
    assert_eq!(
        git(dir.path(), &["log", "-1", "--format=%s"]),
        "Import OpenAPI: 7 endpoints"
    );
    assert_eq!(
        git(dir.path(), &["status", "--porcelain", "--", "endpoints"]),
        ""
    );
    // nothing new -> no commit
    ws.merge_openapi(V2).unwrap();
    assert_eq!(
        git(dir.path(), &["rev-list", "--count", "HEAD"])
            .parse::<u32>()
            .unwrap(),
        base.parse::<u32>().unwrap() + 1
    );
}

#[test]
fn create_from_openapi_in_fresh_folder_makes_no_repo() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("w");
    Workspace::create_from_openapi(&root, "W", V2).unwrap();
    assert!(!root.join(".git").exists());
    assert!(fs::read_to_string(root.join(".gitignore"))
        .unwrap()
        .contains(".xrest"));
}

// ---- fetch_text --------------------------------------------------------------------------

#[tokio::test]
async fn fetch_text_returns_body() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    Mock::given(path("/spec.yaml"))
        .respond_with(ResponseTemplate::new(200).set_body_string("héllo: 1"))
        .mount(&server)
        .await;
    let got = fetch_text(&format!("{}/spec.yaml", server.uri()))
        .await
        .unwrap();
    assert_eq!(got, "héllo: 1");
}

#[tokio::test]
async fn fetch_text_rejects_non_http_schemes_and_garbage() {
    for u in [
        "file:///etc/passwd",
        "ftp://x/y",
        "javascript:alert(1)",
        "nope",
        "",
    ] {
        assert!(
            matches!(fetch_text(u).await, Err(Error::InvalidUrl { .. })),
            "{u}"
        );
    }
}

#[tokio::test]
async fn fetch_text_enforces_cap_while_streaming() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    Mock::given(path("/big"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'a'; 5000]))
        .mount(&server)
        .await;
    let url = format!("{}/big", server.uri());
    assert!(matches!(
        fetch_text_with(&url, 1000, Duration::from_secs(5)).await,
        Err(Error::InvalidInput(_))
    ));
    assert_eq!(
        fetch_text_with(&url, 5000, Duration::from_secs(5))
            .await
            .unwrap()
            .len(),
        5000
    );
}

#[tokio::test]
async fn fetch_text_errors_on_status_timeout_and_bad_utf8() {
    std::env::set_var("NO_PROXY", "127.0.0.1,localhost");
    let server = MockServer::start().await;
    Mock::given(path("/404"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;
    Mock::given(path("/slow"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(800)))
        .mount(&server)
        .await;
    Mock::given(path("/bin"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0xff, 0xfe, 0x41]))
        .mount(&server)
        .await;
    let base = server.uri();
    assert!(matches!(
        fetch_text(&format!("{base}/404")).await,
        Err(Error::Http(_))
    ));
    assert!(matches!(
        fetch_text_with(&format!("{base}/slow"), 1000, Duration::from_millis(200)).await,
        Err(Error::Http(_))
    ));
    assert!(matches!(
        fetch_text(&format!("{base}/bin")).await,
        Err(Error::Http(_))
    ));
}
