use crate::io::MockHttpClient;
use crate::services::RequestService;
use crate::types::{AuthConfig, BodyConfig, PreflightConfig, QResponse, RequestTab};
use mockall::predicate;
use std::collections::HashMap;

#[tokio::test]
async fn test_preflight_token_caching() {
    let mut mock_http = MockHttpClient::new();

    // 1. Initial preflight request
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("POST"),
            predicate::eq("https://auth.example.com/token"),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .times(1) // SHOUlD ONLY BE CALLED ONCE
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"access_token": "secret_token_123"}"#.to_string(),
                    error: None,
                    time_elapsed: 10,
                    size: 40,
                })
            })
        });

    // 2. Main request
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("GET"),
            predicate::eq("https://api.example.com/data"),
            predicate::function(|headers: &Vec<(String, String)>| {
                headers
                    .iter()
                    .any(|(n, v)| n == "Authorization" && v == "Bearer secret_token_123")
            }),
            predicate::always(),
            predicate::always(),
        )
        .times(2) // SHOULD BE CALLED TWICE (once per send_request call)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: "data".to_string(),
                    error: None,
                    time_elapsed: 10,
                    size: 4,
                })
            })
        });

    let service = RequestService::new(&mock_http, None);

    let preflight = PreflightConfig {
        enabled: true,
        method: "POST".to_string(),
        url: "https://auth.example.com/token".to_string(),
        body: "".to_string(),
        body_type: "application/json".to_string(),
        body_params: vec![],
        headers: vec![],
        cache_token: true,
        cache_duration: "3600".to_string(),
        cache_duration_key: "".to_string(),
        cache_duration_unit: "seconds".to_string(),
        token_key: "access_token".to_string(),
        token_header: Some("Authorization".to_string()),
    };

    let tab = RequestTab {
        id: "tab1".to_string(),
        endpoint_id: None,
        title: "Test".to_string(),
        method: "GET".to_string(),
        url: "https://api.example.com/data".to_string(),
        params: vec![],
        headers: vec![],
        body: BodyConfig {
            r#type: "none".to_string(),
            content: "".to_string(),
        },
        auth: AuthConfig {
            r#type: "none".to_string(),
            active: true,
            bearer_token: "".to_string(),
            basic_user: "".to_string(),
            basic_pass: "".to_string(),
            api_key_name: "".to_string(),
            api_key_value: "".to_string(),
            api_key_location: "header".to_string(),
        },
        active_sub_tab: None,
        service_id: Some("test-service".to_string()),
        preflight,
        variables: None,
        is_edited: false,
    };

    // First request - should trigger preflight
    let result1 = service.send_request(tab.clone()).await;
    assert!(result1.is_ok());

    // Second request - should USE CACHE
    let result2 = service.send_request(tab).await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_preflight_token_sharing_between_endpoints() {
    let mut mock_http = MockHttpClient::new();

    // 1. Preflight for Endpoint A
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("POST"),
            predicate::eq("https://auth.example.com/token"),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .times(1)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"access_token": "shared_token"}"#.to_string(),
                    error: None,
                    time_elapsed: 10,
                    size: 40,
                })
            })
        });

    // 2. Main requests for A and B
    mock_http
        .expect_send_request()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::function(|headers: &Vec<(String, String)>| {
                headers
                    .iter()
                    .any(|(n, v)| n == "Authorization" && v == "Bearer shared_token")
            }),
            predicate::always(),
            predicate::always(),
        )
        .times(2)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: "ok".to_string(),
                    error: None,
                    time_elapsed: 1,
                    size: 2,
                })
            })
        });

    let service = RequestService::new(&mock_http, None);

    let preflight = PreflightConfig {
        enabled: true,
        method: "POST".to_string(),
        url: "https://auth.example.com/token".to_string(),
        body: "".to_string(),
        body_type: "application/json".to_string(),
        body_params: vec![],
        headers: vec![],
        cache_token: true,
        cache_duration: "3600".to_string(),
        cache_duration_key: "".to_string(),
        cache_duration_unit: "seconds".to_string(),
        token_key: "access_token".to_string(),
        token_header: Some("Authorization".to_string()),
    };

    let mut tab_a = create_mock_tab("GET", "https://api.a.com", None);
    tab_a.service_id = Some("service-1".to_string());
    tab_a.preflight = preflight.clone();

    let mut tab_b = create_mock_tab("GET", "https://api.b.com", None);
    tab_b.service_id = Some("service-1".to_string());
    tab_b.preflight = preflight; // Same preflight config

    // Send A - triggers preflight
    service.send_request(tab_a).await.unwrap();

    // Send B - should hit cache
    service.send_request(tab_b).await.unwrap();
}

#[tokio::test]
async fn test_preflight_token_implicit_sharing() {
    let mut mock_http = MockHttpClient::new();

    // 1. Preflight for Endpoint A
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("POST"),
            predicate::eq("https://auth.example.com/token"),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .times(1)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"access_token": "implicit_shared_token"}"#.to_string(),
                    error: None,
                    time_elapsed: 10,
                    size: 40,
                })
            })
        });

    // 2. Main requests for A and B
    mock_http
        .expect_send_request()
        .with(
            predicate::always(),
            predicate::always(),
            predicate::function(|headers: &Vec<(String, String)>| {
                headers
                    .iter()
                    .any(|(n, v)| n == "Authorization" && v == "Bearer implicit_shared_token")
            }),
            predicate::always(),
            predicate::always(),
        )
        .times(2)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: "ok".to_string(),
                    error: None,
                    time_elapsed: 1,
                    size: 2,
                })
            })
        });

    let service = RequestService::new(&mock_http, None);

    let preflight_on = PreflightConfig {
        enabled: true,
        method: "POST".to_string(),
        url: "https://auth.example.com/token".to_string(),
        body: "".to_string(),
        body_type: "application/json".to_string(),
        body_params: vec![],
        headers: vec![],
        cache_token: true,
        cache_duration: "3600".to_string(),
        cache_duration_key: "".to_string(),
        cache_duration_unit: "seconds".to_string(),
        token_key: "access_token".to_string(),
        token_header: Some("Authorization".to_string()),
    };

    let preflight_off = PreflightConfig {
        enabled: false,
        ..preflight_on.clone()
    };

    let mut tab_a = create_mock_tab("GET", "https://api.a.com", None);
    tab_a.service_id = Some("service-implicit".to_string());
    tab_a.preflight = preflight_on;

    let mut tab_b = create_mock_tab("GET", "https://api.b.com", None);
    tab_b.service_id = Some("service-implicit".to_string());
    tab_b.preflight = preflight_off; // Preflight DISABLED

    // Send A - triggers preflight and caches token
    service.send_request(tab_a).await.unwrap();

    // Send B - SHOULD hit cache even if preflight is disabled, because it's in the same service
    // and we want "automated" auth.
    service
        .send_request(tab_b)
        .await
        .expect("Request B should have the cached token injected");
}

#[tokio::test]
async fn test_preflight_token_caching_no_service_id() {
    let mut mock_http = MockHttpClient::new();

    // Expect two DIFFERENT preflight requests because they have different URLs
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("POST"),
            predicate::eq("https://auth.a.com/token"),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .times(1)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"access_token": "token_a"}"#.to_string(),
                    error: None,
                    time_elapsed: 1,
                    size: 1,
                })
            })
        });

    mock_http
        .expect_send_request()
        .with(
            predicate::eq("POST"),
            predicate::eq("https://auth.b.com/token"),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .times(1)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"access_token": "token_b"}"#.to_string(),
                    error: None,
                    time_elapsed: 1,
                    size: 1,
                })
            })
        });

    // Main requests - should only match api.a.com and api.b.com
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("GET"),
            predicate::function(|url: &str| url.contains("api.")),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .times(2)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: "ok".to_string(),
                    error: None,
                    time_elapsed: 1,
                    size: 1,
                })
            })
        });

    let service = RequestService::new(&mock_http, None);

    let mut tab_a = create_mock_tab("GET", "https://api.a.com", None);
    tab_a.service_id = None;
    tab_a.preflight.enabled = true;
    tab_a.preflight.method = "POST".to_string();
    tab_a.preflight.url = "https://auth.a.com/token".to_string();

    let mut tab_b = create_mock_tab("GET", "https://api.b.com", None);
    tab_b.service_id = None;
    tab_b.preflight.enabled = true;
    tab_b.preflight.method = "POST".to_string();
    tab_b.preflight.url = "https://auth.b.com/token".to_string();

    service.send_request(tab_a).await.unwrap();
    service.send_request(tab_b).await.unwrap();

    // If they collided, one would have overwritten the other, or failed.
    // But since they have different URLs, they should have different cache keys.
}

fn create_mock_tab(
    method: &str,
    url: &str,
    variables: Option<HashMap<String, String>>,
) -> RequestTab {
    RequestTab {
        id: "test-id".to_string(),
        endpoint_id: None,
        title: "Test Tab".to_string(),
        method: method.to_string(),
        url: url.to_string(),
        params: vec![],
        headers: vec![],
        body: BodyConfig {
            r#type: "none".to_string(),
            content: "".to_string(),
        },
        auth: AuthConfig {
            r#type: "none".to_string(),
            active: true,
            bearer_token: "".to_string(),
            basic_user: "".to_string(),
            basic_pass: "".to_string(),
            api_key_name: "".to_string(),
            api_key_value: "".to_string(),
            api_key_location: "header".to_string(),
        },
        active_sub_tab: None,
        service_id: None,
        preflight: PreflightConfig {
            enabled: false,
            method: "GET".to_string(),
            url: "".to_string(),
            body: "".to_string(),
            body_type: "application/json".to_string(),
            body_params: vec![],
            headers: vec![],
            cache_token: true,
            cache_duration: "3600".to_string(),
            cache_duration_key: "".to_string(),
            cache_duration_unit: "seconds".to_string(),
            token_key: "access_token".to_string(),
            token_header: None,
        },
        variables,
        is_edited: false,
    }
}
