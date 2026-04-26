use crate::core::auth::preflight::test_preflight;
use crate::core::service::endpoint::PreflightConfig;
use crate::core::traits::MockHttpClient;
use crate::core::types::QResponse;
use mockall::predicate;
use std::collections::HashMap;

#[tokio::test]
async fn test_preflight_telemetry_success() {
    let mut mock_http = MockHttpClient::new();

    mock_http
        .expect_send_request()
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"access_token": "test_token", "expires_in": 3600}"#.to_string(),
                    error: None,
                    time_elapsed: 50,
                    size: 100,
                })
            })
        });

    let config = PreflightConfig {
        enabled: true,
        method: "POST".to_string(),
        url: "https://auth.com/token".to_string(),
        body: "".to_string(),
        body_type: "application/json".to_string(),
        body_params: vec![],
        headers: vec![],
        cache_token: true,
        cache_duration_mode: "derived".to_string(),
        cache_duration_seconds: 60,
        cache_duration: "".to_string(),
        cache_duration_key: "expires_in".to_string(),
        cache_duration_unit: "seconds".to_string(),
        token_key: "access_token".to_string(),
        token_header: None,
    };

    let result = test_preflight(&mock_http, "test-service", &config, &HashMap::new(), None, None).await;

    assert!(result.success);
    assert_eq!(result.token, Some("test_token".to_string()));
    assert_eq!(result.extraction_path, Some("access_token".to_string()));
    assert_eq!(result.cache_status, "miss"); // First run is a miss
    assert!(result.cache_status_detail.is_some());
    assert_eq!(result.response_status, 200);
}

#[tokio::test]
async fn test_preflight_telemetry_extraction_error() {
    let mut mock_http = MockHttpClient::new();

    mock_http
        .expect_send_request()
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"wrong_key": "test_token"}"#.to_string(),
                    error: None,
                    time_elapsed: 10,
                    size: 10,
                })
            })
        });

    let config = PreflightConfig {
        enabled: true,
        method: "POST".to_string(),
        url: "https://auth.com/token".to_string(),
        token_key: "access_token".to_string(),
        ..Default::default()
    };

    // Use a manual default to avoid Default trait issues if not implemented perfectly
    let mut config = config;
    config.cache_token = true;
    config.cache_duration_mode = "manual".to_string();
    config.cache_duration_seconds = 300;

    let result = test_preflight(&mock_http, "test-service", &config, &HashMap::new(), None, None).await;

    assert!(!result.success);
    assert_eq!(result.cache_status, "error");
    assert_eq!(result.extraction_path, Some("access_token".to_string()));
    assert!(result.error.unwrap().contains("Token key 'access_token' not found"));
}

#[tokio::test]
async fn test_preflight_telemetry_manual_duration() {
    let mut mock_http = MockHttpClient::new();

    mock_http
        .expect_send_request()
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"token": "abc"}"#.to_string(),
                    error: None,
                    time_elapsed: 5,
                    size: 5,
                })
            })
        });

    let config = PreflightConfig {
        enabled: true,
        method: "POST".to_string(),
        url: "https://auth.com/token".to_string(),
        cache_token: true,
        cache_duration_mode: "manual".to_string(),
        cache_duration_seconds: 1234,
        token_key: "token".to_string(),
        ..Default::default()
    };

    let result = test_preflight(&mock_http, "test-service-manual", &config, &HashMap::new(), None, None).await;

    assert!(result.success);
    assert_eq!(result.cache_status, "miss");
    // We can't easily verify the actual cache duration stored in the global cache without more plumbing,
    // but we've verified the code paths are hit.
}
