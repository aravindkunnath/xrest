//! Execution history: SQLite store, recording on send, masking, caps.

use std::sync::Arc;
use std::time::Instant;

use wiremock::matchers::path;
use wiremock::{Mock, MockServer, ResponseTemplate};
use xrest_core::history::{
    HistoryEntry, HistoryFilter, HistoryStore, SqliteHistory, MASK, MAX_BODY_BYTES,
};
use xrest_core::*;

fn entry(method: &str, status: Option<u16>) -> HistoryEntry {
    HistoryEntry {
        id: 0,
        created_at: 1_000,
        service_id: "svc".into(),
        service_name: "Svc".into(),
        endpoint_id: Some("ep1".into()),
        environment: Some("DEV".into()),
        method: method.into(),
        url: "http://x/y".into(),
        request_headers: vec![("A".into(), "b".into())],
        request_body: "req".into(),
        status,
        status_text: "OK".into(),
        response_headers: vec![],
        response_body: "resp".into(),
        body_is_base64: false,
        truncated: false,
        time_ms: 5,
        size_bytes: 4,
        error: if status.is_none() {
            Some("boom".into())
        } else {
            None
        },
    }
}

fn store() -> (tempfile::TempDir, Arc<SqliteHistory>) {
    let dir = tempfile::tempdir().unwrap();
    let h = SqliteHistory::open(dir.path().join("h.sqlite")).unwrap();
    (dir, Arc::new(h))
}

struct Fx {
    _dir: tempfile::TempDir,
    ws: Workspace,
    server: MockServer,
    secrets: Arc<MemorySecretStore>,
    hist: Arc<SqliteHistory>,
    _hdir: tempfile::TempDir,
}

async fn fx() -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let server = MockServer::start().await;
    let mut ws = Workspace::create(dir.path(), "svc").unwrap();
    ws.set_http_options(HttpOptions {
        use_system_proxy: false,
        ..Default::default()
    });
    let secrets = Arc::new(MemorySecretStore::default());
    ws.set_secret_store(secrets.clone());
    ws.update_environments(vec![
        Environment {
            name: "DEV".into(),
            is_unsafe: false,
            variables: vec![NameValue::new("BASE_URL", server.uri())],
        },
        Environment {
            name: "PROD".into(),
            is_unsafe: true,
            variables: vec![NameValue::new("BASE_URL", server.uri())],
        },
    ])
    .unwrap();
    let (hdir, hist) = store();
    ws.set_history(hist.clone());
    Fx {
        _dir: dir,
        ws,
        server,
        secrets,
        hist,
        _hdir: hdir,
    }
}

fn spec(method: &str, url: &str) -> RequestSpec {
    RequestSpec {
        method: method.into(),
        url: url.into(),
        ..Default::default()
    }
}

async fn send(f: &mut Fx, s: &RequestSpec) -> Result<HttpResponse> {
    let prepared = f.ws.prepare_send(s, false)?;
    let sink = f.ws.history_sink(Some("ep1"));
    let (res, warning) = prepared.execute_recorded(sink).await;
    if let Some(w) = warning {
        f.ws.push_warning(w);
    }
    res
}

#[test]
fn store_roundtrip_filters_paging_clear() {
    let (_d, h) = store();
    for i in 0..5 {
        let mut e = entry(if i % 2 == 0 { "GET" } else { "POST" }, Some(200 + i * 100));
        e.created_at = 1_000 + i as i64;
        h.record(&e).unwrap();
    }
    h.record(&entry("GET", None)).unwrap();
    assert_eq!(h.count(&HistoryFilter::default()).unwrap(), 6);

    let all = h.list(&HistoryFilter::default(), 3, 0).unwrap();
    assert_eq!(all.len(), 3);
    assert!(all[0].id > all[1].id, "newest first");
    let page2 = h.list(&HistoryFilter::default(), 3, 3).unwrap();
    assert_eq!(page2.len(), 3);
    assert!(page2[0].id < all[2].id);

    let get = HistoryFilter {
        method: Some("GET".into()),
        ..Default::default()
    };
    assert_eq!(h.count(&get).unwrap(), 4);
    let class = |c| HistoryFilter {
        status_class: Some(c),
        ..Default::default()
    };
    assert_eq!(h.count(&class(2)).unwrap(), 1);
    assert_eq!(h.count(&class(3)).unwrap(), 1);
    assert_eq!(h.count(&class(4)).unwrap(), 1);
    assert_eq!(h.count(&class(5)).unwrap(), 1);
    assert_eq!(h.count(&class(0)).unwrap(), 1, "0 = transport error");
    let other = HistoryFilter {
        service_id: Some("nope".into()),
        ..Default::default()
    };
    assert_eq!(h.count(&other).unwrap(), 0);
    let ep = HistoryFilter {
        endpoint_id: Some("ep1".into()),
        ..Default::default()
    };
    assert_eq!(h.count(&ep).unwrap(), 6);

    let full = h.get(all[0].id).unwrap().unwrap();
    assert_eq!(full.response_body, "resp");
    assert_eq!(
        full.request_headers,
        vec![("A".to_string(), "b".to_string())]
    );
    assert!(h.get(99_999).unwrap().is_none());

    h.clear().unwrap();
    assert_eq!(h.count(&HistoryFilter::default()).unwrap(), 0);
}

#[test]
fn ten_thousand_entries_first_page_is_fast() {
    let (_d, h) = store();
    for _ in 0..10_000 {
        h.record(&entry("GET", Some(200))).unwrap();
    }
    let start = Instant::now();
    let page = h.list(&HistoryFilter::default(), 50, 0).unwrap();
    let elapsed = start.elapsed();
    eprintln!("first page of 50 over 10000 entries: {elapsed:?}");
    assert_eq!(page.len(), 50);
    assert!(elapsed.as_millis() < 200, "took {elapsed:?}");
}

#[test]
fn record_caps_bodies_at_one_mib() {
    let (_d, h) = store();
    let mut e = entry("GET", Some(200));
    e.response_body = "a".repeat(MAX_BODY_BYTES + 1);
    let id = h.record(&e).unwrap();
    let got = h.get(id).unwrap().unwrap();
    assert!(got.truncated);
    assert_eq!(got.response_body.len(), MAX_BODY_BYTES);

    let mut e = entry("GET", Some(200));
    e.response_body = "a".repeat(MAX_BODY_BYTES);
    let id = h.record(&e).unwrap();
    assert!(!h.get(id).unwrap().unwrap().truncated);

    // Multi-byte char straddling the cap: cut on a boundary.
    let mut e = entry("GET", Some(200));
    e.response_body = format!("{}é", "a".repeat(MAX_BODY_BYTES - 1));
    let id = h.record(&e).unwrap();
    let got = h.get(id).unwrap().unwrap();
    assert!(got.truncated);
    assert_eq!(got.response_body.len(), MAX_BODY_BYTES - 1);

    // base64: truncated on decoded bytes.
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD;
    let mut e = entry("GET", Some(200));
    e.body_is_base64 = true;
    e.response_body = b64.encode(vec![7u8; MAX_BODY_BYTES + 1]);
    let id = h.record(&e).unwrap();
    let got = h.get(id).unwrap().unwrap();
    assert!(got.truncated);
    assert_eq!(
        b64.decode(&got.response_body).unwrap().len(),
        MAX_BODY_BYTES
    );
}

#[tokio::test]
async fn send_records_masked_entry() {
    let mut f = fx().await;
    Mock::given(path("/items"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("set-cookie", "sid=abc")
                .set_body_string("echo token-SECRET-1234 done"),
        )
        .mount(&f.server)
        .await;
    f.secrets.set("api", "token-SECRET-1234");
    f.secrets.set("q", "qs-secret-9999");
    f.secrets.set("tiny", "ab");
    let mut s = spec("POST", "{{BASE_URL}}/items?key={{secret.q}}");
    s.headers
        .push(NameValue::new("Authorization", "Bearer {{secret.api}}"));
    s.headers.push(NameValue::new("Cookie", "a=b"));
    s.headers.push(NameValue::new(
        "X-Other",
        "v-{{secret.api}}-{{secret.tiny}}",
    ));
    s.body = Body::from_text(r#"{"t":"{{secret.api}}"}"#);
    send(&mut f, &s).await.unwrap();

    assert_eq!(f.hist.count(&HistoryFilter::default()).unwrap(), 1);
    let id = f.hist.list(&HistoryFilter::default(), 1, 0).unwrap()[0].id;
    let e = f.hist.get(id).unwrap().unwrap();
    assert_eq!(e.service_id, f.ws.service().id);
    assert_eq!(e.endpoint_id.as_deref(), Some("ep1"));
    assert_eq!(e.environment.as_deref(), Some("DEV"));
    assert_eq!(e.method, "POST");
    assert_eq!(e.status, Some(200));
    assert!(
        e.url.contains(&format!("key={MASK}")) || e.url.contains("key=%E2%80%A2"),
        "{}",
        e.url
    );
    let dump = format!("{e:?}");
    assert!(!dump.contains("token-SECRET-1234"), "{dump}");
    assert!(!dump.contains("qs-secret-9999"), "{dump}");
    assert!(!dump.contains("sid=abc"));
    let h = |name: &str| {
        e.request_headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.clone())
            .unwrap()
    };
    assert_eq!(h("authorization"), MASK);
    assert_eq!(h("cookie"), MASK);
    assert_eq!(
        h("x-other"),
        format!("v-{MASK}-ab"),
        "short secrets are not masked"
    );
    assert!(e
        .response_headers
        .iter()
        .any(|(k, v)| k == "set-cookie" && v == MASK));
    assert_eq!(e.request_body, format!(r#"{{"t":"{MASK}"}}"#));
    assert_eq!(e.response_body, format!("echo {MASK} done"));
}

#[tokio::test]
async fn non_2xx_and_transport_errors_are_recorded_but_policy_refusal_is_not() {
    let mut f = fx().await;
    Mock::given(path("/missing"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&f.server)
        .await;
    send(&mut f, &spec("GET", "{{BASE_URL}}/missing"))
        .await
        .unwrap();
    let r = send(&mut f, &spec("GET", "http://127.0.0.1:1/")).await;
    assert!(r.is_err());
    assert_eq!(f.hist.count(&HistoryFilter::default()).unwrap(), 2);
    let list = f.hist.list(&HistoryFilter::default(), 10, 0).unwrap();
    assert_eq!(list[0].status, None);
    assert!(list[0].error.is_some());
    assert_eq!(list[1].status, Some(404));

    f.ws.select_environment("PROD").unwrap();
    let r = send(&mut f, &spec("GET", "{{BASE_URL}}/missing")).await;
    assert!(matches!(r, Err(Error::ConfirmationRequired { .. })));
    assert_eq!(f.hist.count(&HistoryFilter::default()).unwrap(), 2);
}

#[tokio::test]
async fn preflight_token_is_masked_and_recording_failure_is_a_warning() {
    let mut f = fx().await;
    Mock::given(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"access":"pf-token-xyz"}"#))
        .mount(&f.server)
        .await;
    Mock::given(path("/data"))
        .respond_with(ResponseTemplate::new(200).set_body_string("pf-token-xyz"))
        .mount(&f.server)
        .await;
    let mut s = spec("GET", "{{BASE_URL}}/data");
    s.headers.push(NameValue::new("X-Echo", "unused"));
    s.preflight = Some(Preflight {
        enabled: true,
        method: "POST".into(),
        url: "{{BASE_URL}}/token".into(),
        token_key: "access".into(),
        token_header: Some("X-Token".into()),
        ..Default::default()
    });
    send(&mut f, &s).await.unwrap();
    let id = f.hist.list(&HistoryFilter::default(), 1, 0).unwrap()[0].id;
    let e = f.hist.get(id).unwrap().unwrap();
    assert!(!format!("{e:?}").contains("pf-token-xyz"));
    assert_eq!(e.response_body, MASK);
    assert!(e
        .request_headers
        .iter()
        .any(|(k, v)| k == "X-Token" && v == MASK));

    // A store that fails: the send still succeeds, a warning is raised.
    struct Broken;
    impl HistoryStore for Broken {
        fn record(&self, _: &HistoryEntry) -> Result<i64> {
            Err(Error::InvalidInput("disk full".into()))
        }
        fn list(
            &self,
            _: &HistoryFilter,
            _: u32,
            _: u32,
        ) -> Result<Vec<xrest_core::history::HistorySummary>> {
            Ok(vec![])
        }
        fn get(&self, _: i64) -> Result<Option<HistoryEntry>> {
            Ok(None)
        }
        fn count(&self, _: &HistoryFilter) -> Result<u64> {
            Ok(0)
        }
        fn clear(&self) -> Result<()> {
            Ok(())
        }
    }
    f.ws.set_history(Arc::new(Broken));
    let before = f.ws.warnings().len();
    let resp = send(&mut f, &spec("GET", "{{BASE_URL}}/data"))
        .await
        .unwrap();
    assert_eq!(resp.status, 200);
    assert_eq!(f.ws.warnings().len(), before + 1);
    assert!(f.ws.warnings().last().unwrap().contains("disk full"));
}
