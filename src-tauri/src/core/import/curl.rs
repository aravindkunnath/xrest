use crate::core::types::{Endpoint, EndpointMetadata, NameValue, PreflightConfig};
use curl_parser::ParsedRequest;
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

pub fn curl_to_endpoint(
    service_id: String,
    curl_command: &str,
    authenticated: bool,
    auth_type: Option<String>,
) -> Result<Endpoint, String> {
    let parsed = ParsedRequest::load(curl_command, serde_json::Value::Null)
        .map_err(|e| format!("Failed to parse cURL: {}", e))?;

    let endpoint_id = format!("e-{}", uuid::Uuid::new_v4());

    // Extract endpoint name from URL
    let url_str = parsed.url.to_string();
    let endpoint_name = if let Ok(u) = Url::parse(&url_str) {
        let path = u.path().trim_start_matches('/').replace('/', " ");
        if path.is_empty() {
            "New Endpoint".to_string()
        } else {
            path
        }
    } else {
        "New Endpoint".to_string()
    };

    let mut headers = Vec::new();
    for (name, value) in &parsed.headers {
        headers.push(NameValue {
            name: name.to_string(),
            value: value.to_str().unwrap_or("").to_string(),
            enabled: true,
            secret_key: None,
            r#type: "plain".to_string(),
        });
    }

    let body = parsed.body.join("");

    Ok(Endpoint {
        id: endpoint_id,
        service_id,
        name: endpoint_name,
        method: parsed.method.to_string(),
        url: url_str,
        authenticated,
        auth_type: auth_type.unwrap_or_else(|| "none".to_string()),
        metadata: EndpointMetadata {
            version: "1.0".to_string(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        },
        params: Vec::new(),
        headers,
        body,
        preflight: PreflightConfig {
            enabled: false,
            method: "GET".to_string(),
            url: "".to_string(),
            body: "".to_string(),
            body_type: "application/json".to_string(),
            body_params: vec![],
            headers: vec![],
            cache_token: true,
            cache_duration: "derived".to_string(),
            cache_duration_key: "expires_in".to_string(),
            cache_duration_unit: "seconds".to_string(),
            token_key: "access_token".to_string(),
            token_header: Some("Authorization".to_string()),
            ..Default::default()
        },
        last_version: 0,
        versions: vec![],
    })
}
