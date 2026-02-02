use crate::io::{MockFileSystem, MockHttpClient};
use crate::services::RequestService;
use crate::types::{PreflightConfig, QResponse, RequestTab};
use mockall::predicate;

#[tokio::test]
async fn test_send_request_with_preflight() {
    let mut mock_http = MockHttpClient::new();

    // Expect preflight request
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("POST"),
            predicate::eq("https://auth.example.com/token"),
            predicate::always(), // headers
            predicate::always(), // body
            predicate::always(), // query
        )
        .times(1)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 200,
                    status_text: "OK".to_string(),
                    headers: vec![],
                    body: r#"{"access_token": "secret_token"}"#.to_string(),
                    error: None,
                    time_elapsed: 10,
                    size: 30,
                })
            })
        });

    // Expect main request with injected token
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("GET"),
            predicate::eq("https://api.example.com/data"),
            predicate::function(|headers: &Vec<(String, String)>| {
                headers
                    .iter()
                    .any(|(n, v)| n == "Authorization" && v == "Bearer secret_token")
            }),
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
                    body: "data".to_string(),
                    error: None,
                    time_elapsed: 20,
                    size: 4,
                })
            })
        });

    let service = RequestService::new(&mock_http, None);
    let tab = RequestTab {
        id: "tab1".to_string(),
        endpoint_id: Some("endpoint1".to_string()),
        title: "Test Tab".to_string(),
        method: "GET".to_string(),
        url: "https://api.example.com/data".to_string(),
        params: vec![],
        headers: vec![],
        body: crate::types::BodyConfig {
            r#type: "none".to_string(),
            content: "".to_string(),
        },
        auth: crate::types::AuthConfig {
            r#type: "none".to_string(),
            active: true,
            bearer_token: "".to_string(),
            basic_user: "".to_string(),
            basic_pass: "".to_string(),
            api_key_name: "".to_string(),
            api_key_value: "".to_string(),
            api_key_location: "header".to_string(),
        },
        active_sub_tab: Some("headers".to_string()),
        service_id: Some("service1".to_string()),
        preflight: PreflightConfig {
            enabled: true,
            method: "POST".to_string(),
            url: "https://auth.example.com/token".to_string(),
            body: "".to_string(),
            body_type: "application/json".to_string(),
            body_params: vec![],
            headers: vec![],
            cache_token: false,
            cache_duration: "3600".to_string(),
            cache_duration_key: "".to_string(),
            cache_duration_unit: "seconds".to_string(),
            token_key: "access_token".to_string(),
            token_header: Some("Authorization".to_string()),
        },
        variables: None,
        is_edited: false,
    };

    let result = service.send_request(tab).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.body, "data");
}

use std::collections::HashMap;

#[tokio::test]
async fn test_variable_resolution() {
    let mut mock_http = MockHttpClient::new();

    // Expect main request with resolved URL and body
    mock_http
        .expect_send_request()
        .with(
            predicate::eq("POST"),
            predicate::eq("https://api.example.com/items/123"),
            predicate::always(),
            predicate::function(|body: &Option<String>| {
                body.as_ref()
                    .map(|b| b.contains("item-123"))
                    .unwrap_or(false)
            }),
            predicate::always(),
        )
        .times(1)
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 201,
                    status_text: "Created".to_string(),
                    headers: vec![],
                    body: "{}".to_string(),
                    error: None,
                    time_elapsed: 10,
                    size: 2,
                })
            })
        });

    let service = RequestService::new(&mock_http, None);
    let mut variables = HashMap::new();
    variables.insert(
        "BASE_URL".to_string(),
        "https://api.example.com".to_string(),
    );
    variables.insert("ITEM_ID".to_string(), "123".to_string());

    let tab = RequestTab {
        id: "tab1".to_string(),
        endpoint_id: None,
        title: "Test Tab".to_string(),
        method: "POST".to_string(),
        url: "{{BASE_URL}}/items/{{ITEM_ID}}".to_string(),
        params: vec![],
        headers: vec![],
        body: crate::types::BodyConfig {
            r#type: "application/json".to_string(),
            content: "{\"id\": \"item-{{ITEM_ID}}\"}".to_string(),
        },
        auth: crate::types::AuthConfig {
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
        variables: Some(variables),
        is_edited: false,
    };

    let result = service.send_request(tab).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_http_error_handling() {
    let mut mock_http = MockHttpClient::new();

    // Simulate a network failure
    mock_http
        .expect_send_request()
        .returning(|_, _, _, _, _| Box::pin(async { Err("Network unreachable".to_string()) }));

    let service = RequestService::new(&mock_http, None);
    let tab = create_mock_tab("GET", "https://api.example.com", None);

    let result = service.send_request(tab).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Network unreachable");
}

#[tokio::test]
async fn test_status_codes_and_headers() {
    let mut mock_http = MockHttpClient::new();

    mock_http
        .expect_send_request()
        .with(
            predicate::eq("GET"),
            predicate::eq("https://api.example.com/not-found"),
            predicate::always(),
            predicate::always(),
            predicate::always(),
        )
        .returning(|_, _, _, _, _| {
            Box::pin(async {
                Ok(QResponse {
                    status: 404,
                    status_text: "Not Found".to_string(),
                    headers: vec![crate::types::Header {
                        name: "Content-Type".to_string(),
                        value: "application/json".to_string(),
                        enabled: true,
                        secret_key: None,
                    }],
                    body: "{\"error\": \"not found\"}".to_string(),
                    error: None,
                    time_elapsed: 5,
                    size: 22,
                })
            })
        });

    let service = RequestService::new(&mock_http, None);
    let tab = create_mock_tab("GET", "https://api.example.com/not-found", None);

    let result = service.send_request(tab).await;
    assert!(result.is_ok());
    let resp = result.unwrap();
    assert_eq!(resp.status, 404);
    assert_eq!(resp.status_text, "Not Found");
    assert!(resp
        .headers
        .iter()
        .any(|h| h.name == "Content-Type" && h.value == "application/json"));
}

// Helper function to create a basic mock tab for testing
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
        body: crate::types::BodyConfig {
            r#type: "none".to_string(),
            content: "".to_string(),
        },
        auth: crate::types::AuthConfig {
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
