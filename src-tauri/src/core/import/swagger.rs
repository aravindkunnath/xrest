use crate::core::types::{
    AuthConfig, AuthType, Endpoint, EndpointMetadata, EnvironmentConfig, NameValue,
    PreflightConfig, Service,
};
use openapiv3::OpenAPI;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn parse_spec_content(
    content: &str,
    service_id: &str,
) -> Result<(String, Vec<Endpoint>), String> {
    let value: serde_json::Value = if content.trim().starts_with('{') {
        serde_json::from_str(content).map_err(|e| format!("Failed to parse JSON: {}", e))?
    } else {
        serde_yaml::from_str(content).map_err(|e| format!("Failed to parse YAML: {}", e))?
    };

    let mut endpoints = Vec::new();
    let base_url;

    if value.get("openapi").is_some() {
        let openapi: OpenAPI =
            serde_json::from_value(value).map_err(|e| format!("Invalid OpenAPI 3: {}", e))?;

        base_url = openapi
            .servers
            .first()
            .map(|s| s.url.clone())
            .unwrap_or_else(|| "https://api.example.com".to_string());

        for (path, path_item) in openapi.paths.iter() {
            if let Some(item) = path_item.as_item() {
                let methods = [
                    ("GET", &item.get),
                    ("POST", &item.post),
                    ("PUT", &item.put),
                    ("DELETE", &item.delete),
                    ("PATCH", &item.patch),
                ];

                for (method, op_opt) in methods {
                    if let Some(op) = op_opt {
                        let endpoint_id = format!("e-{}", uuid::Uuid::new_v4());
                        let endpoint_name = op
                            .summary
                            .clone()
                            .or_else(|| op.operation_id.clone())
                            .unwrap_or_else(|| format!("{} {}", method, path));

                        let mut params = Vec::new();
                        let mut headers = Vec::new();

                        for param_ref in &op.parameters {
                            if let Some(p) = param_ref.as_item() {
                                match p {
                                    openapiv3::Parameter::Query { parameter_data, .. } => {
                                        params.push(NameValue {
                                            name: parameter_data.name.clone(),
                                            value: "".to_string(),
                                            enabled: true,
                                            secret_key: None,
                                        });
                                    }
                                    openapiv3::Parameter::Header { parameter_data, .. } => {
                                        headers.push(NameValue {
                                            name: parameter_data.name.clone(),
                                            value: "".to_string(),
                                            enabled: true,
                                            secret_key: None,
                                        });
                                    }
                                    _ => continue,
                                }
                            }
                        }

                        endpoints.push(create_endpoint(
                            endpoint_id,
                            service_id.to_string(),
                            endpoint_name,
                            method.to_string(),
                            path.clone(),
                            params,
                            headers,
                        ));
                    }
                }
            }
        }
    } else {
        let host = value
            .get("host")
            .and_then(|v| v.as_str())
            .unwrap_or("api.example.com");
        let base_path = value.get("basePath").and_then(|v| v.as_str()).unwrap_or("");
        let scheme = value
            .get("schemes")
            .and_then(|v| v.as_array())
            .and_then(|a| a.first())
            .and_then(|v| v.as_str())
            .unwrap_or("https");

        base_url = format!("{}://{}{}", scheme, host, base_path);

        if let Some(paths) = value.get("paths").and_then(|v| v.as_object()) {
            for (path, path_value) in paths {
                if let Some(methods_obj) = path_value.as_object() {
                    for (method, op_value) in methods_obj {
                        let method_upper = method.to_uppercase();
                        if !["GET", "POST", "PUT", "DELETE", "PATCH"]
                            .contains(&method_upper.as_str())
                        {
                            continue;
                        }

                        let endpoint_id = format!("e-{}", uuid::Uuid::new_v4());
                        let endpoint_name = op_value
                            .get("summary")
                            .and_then(|v| v.as_str())
                            .or_else(|| op_value.get("operationId").and_then(|v| v.as_str()))
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("{} {}", method_upper, path));

                        let mut params = Vec::new();
                        let mut headers = Vec::new();

                        if let Some(parameters) =
                            op_value.get("parameters").and_then(|v| v.as_array())
                        {
                            for p in parameters {
                                let p_name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                let p_in = p.get("in").and_then(|v| v.as_str()).unwrap_or("");

                                if p_in == "query" {
                                    params.push(NameValue {
                                        name: p_name.to_string(),
                                        value: "".to_string(),
                                        enabled: true,
                                        secret_key: None,
                                    });
                                } else if p_in == "header" {
                                    headers.push(NameValue {
                                        name: p_name.to_string(),
                                        value: "".to_string(),
                                        enabled: true,
                                        secret_key: None,
                                    });
                                }
                            }
                        }

                        endpoints.push(create_endpoint(
                            endpoint_id,
                            service_id.to_string(),
                            endpoint_name,
                            method_upper,
                            path.clone(),
                            params,
                            headers,
                        ));
                    }
                }
            }
        }
    }

    Ok((base_url, endpoints))
}

/// Create a new Service from a parsed spec with default environments
pub fn create_service_from_spec(
    name: String,
    directory: String,
    base_url: String,
    endpoints: Vec<Endpoint>,
) -> Service {
    let service_id = format!(
        "s-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    Service {
        id: service_id,
        name,
        environments: vec![
            EnvironmentConfig {
                name: "DEV".to_string(),
                is_unsafe: false,
                variables: vec![NameValue {
                    name: "BASE_URL".to_string(),
                    value: base_url.clone(),
                    enabled: true,
                    secret_key: None,
                }],
            },
            EnvironmentConfig {
                name: "STAGE".to_string(),
                is_unsafe: false,
                variables: vec![NameValue {
                    name: "BASE_URL".to_string(),
                    value: base_url.clone(),
                    enabled: true,
                    secret_key: None,
                }],
            },
            EnvironmentConfig {
                name: "PROD".to_string(),
                is_unsafe: true,
                variables: vec![NameValue {
                    name: "BASE_URL".to_string(),
                    value: base_url,
                    enabled: true,
                    secret_key: None,
                }],
            },
        ],
        is_authenticated: false,
        auth_type: Some(AuthType::None),
        auth: AuthConfig {
            r#type: "none".to_string(),
            active: true,
            basic_user: "".to_string(),
            basic_pass: "".to_string(),
            bearer_token: "".to_string(),
            api_key_name: "".to_string(),
            api_key_value: "".to_string(),
            api_key_location: "header".to_string(),
        },
        preflight: PreflightConfig {
            enabled: false,
            method: "POST".to_string(),
            url: "".to_string(),
            body: "".to_string(),
            body_type: "application/json".to_string(),
            body_params: vec![],
            headers: vec![],
            cache_token: true,
            cache_duration: "".to_string(),
            cache_duration_key: "".to_string(),
            cache_duration_unit: "seconds".to_string(),
            token_key: "".to_string(),
            token_header: None,
        },
        endpoints,
        directory,
        selected_environment: Some("DEV".to_string()),
        git_url: None,
    }
}

fn create_endpoint(
    id: String,
    service_id: String,
    name: String,
    method: String,
    url: String,
    params: Vec<NameValue>,
    headers: Vec<NameValue>,
) -> Endpoint {
    Endpoint {
        id,
        service_id,
        name,
        method,
        url,
        authenticated: false,
        auth_type: "none".to_string(),
        metadata: EndpointMetadata {
            version: "1.0".to_string(),
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        },
        params,
        headers,
        body: "".to_string(),
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
        },
        last_version: 0,
        versions: vec![],
    }
}
