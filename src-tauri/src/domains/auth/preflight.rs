use crate::domains::service::endpoint::PreflightConfig;
use crate::io::HttpClient;
use crate::types::{Header, PreflightTestResult};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn execute_preflight(
    http: &dyn HttpClient,
    service_id: &str,
    config: &PreflightConfig,
    variables: &HashMap<String, String>,
    cache_path: Option<&std::path::PathBuf>,
) -> Result<String, String> {
    let result = test_preflight(http, service_id, config, variables, cache_path).await;
    if result.success {
        Ok(result.token.unwrap_or_default())
    } else {
        Err(result.error.unwrap_or_else(|| "Unknown error".to_string()))
    }
}

pub async fn test_preflight(
    http: &dyn HttpClient,
    service_id: &str,
    config: &PreflightConfig,
    variables: &HashMap<String, String>,
    cache_path: Option<&std::path::PathBuf>,
) -> PreflightTestResult {
    let resolved_url = resolve_variables(&config.url, variables);
    let mut resolved_body = resolve_variables(&config.body, variables);

    if config.body_type == "application/x-www-form-urlencoded" && !config.body_params.is_empty() {
        let mut params = Vec::new();
        for p in &config.body_params {
            let name = resolve_variables(&p.name, variables);
            let value = resolve_variables(&p.value, variables);
            if !name.is_empty() {
                params.push(format!(
                    "{}={}",
                    urlencoding::encode(&name),
                    urlencoding::encode(&value)
                ));
            }
        }
        if !params.is_empty() {
            resolved_body = params.join("&");
        }
    }
    let mut resolved_headers = Vec::new();
    let mut request_headers_vec = Vec::new();
    for h in &config.headers {
        let name = resolve_variables(&h.name, variables);
        let value = resolve_variables(&h.value, variables);
        resolved_headers.push((name.clone(), value.clone()));
        request_headers_vec.push(Header {
            name,
            value,
            enabled: true,
            secret_key: None,
        });
    }

    let cache_key = super::cache::generate_key(
        service_id,
        &resolved_url,
        &config.method,
        &resolved_body,
        &resolved_headers,
    );

    // Check cache
    if config.cache_token {
        if let Some(cached) = super::cache::get_cached_token(&cache_key) {
            if super::cache::is_token_valid(&cached) {
                return PreflightTestResult {
                    success: true,
                    token: Some(cached.token),
                    error: None,
                    request_url: resolved_url,
                    request_method: config.method.clone(),
                    request_headers: request_headers_vec,
                    request_body: resolved_body,
                    response_status: 200,
                    response_body: "Token served from cache".to_string(),
                    response_headers: vec![],
                    time_elapsed: 0,
                };
            }
        }
    }

    if !resolved_body.is_empty()
        && config.method.to_uppercase() != "GET"
        && config.method.to_uppercase() != "HEAD"
    {
        resolved_headers.push(("Content-Type".to_string(), config.body_type.clone()));
        request_headers_vec.push(Header {
            name: "Content-Type".to_string(),
            value: config.body_type.clone(),
            enabled: true,
            secret_key: None,
        });
    }

    let response_result = http
        .send_request(
            &config.method,
            &resolved_url,
            resolved_headers,
            if resolved_body.is_empty() {
                None
            } else {
                Some(resolved_body.clone())
            },
            vec![],
        )
        .await;

    match response_result {
        Ok(response) => {
            let response_headers_vec: Vec<Header> = response
                .headers
                .iter()
                .map(|h| Header {
                    name: h.name.clone(),
                    value: h.value.clone(),
                    enabled: true,
                    secret_key: None,
                })
                .collect();

            let response_json_result: Result<serde_json::Value, _> =
                serde_json::from_str(&response.body);

            let token_result = match response_json_result {
                Ok(response_json) => {
                    let token_opt = response_json
                        .get(&config.token_key)
                        .and_then(|v| v.as_str())
                        .map(|t| t.to_string());

                    match token_opt {
                        Some(token) => Ok((token, response_json)),
                        None => Err(format!(
                            "Token key '{}' not found in preflight response",
                            config.token_key
                        )),
                    }
                }
                Err(e) => Err(format!("Preflight response is not valid JSON: {}", e)),
            };

            match token_result {
                Ok((token, response_json)) => {
                    // Update cache
                    if config.cache_token {
                        let expires_in_seconds = if config.cache_duration == "derived" {
                            let duration_value = response_json
                                .get(&config.cache_duration_key)
                                .and_then(|v| v.as_u64())
                                .unwrap_or(3600); // Default if key missing to avoid testing failure? Or fail? Logic in execute was strict

                            match config.cache_duration_unit.as_str() {
                                "seconds" => duration_value,
                                "minutes" => duration_value * 60,
                                "hours" => duration_value * 3600,
                                "days" => duration_value * 86400,
                                _ => duration_value,
                            }
                        } else {
                            config.cache_duration.parse::<u64>().unwrap_or(3600)
                        };

                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
                        super::cache::set_cached_token(
                            cache_key,
                            token.clone(),
                            now + expires_in_seconds,
                        );

                        if let Some(path) = cache_path {
                            let _ = super::cache::save_cache_to_file(path);
                        }
                    }

                    PreflightTestResult {
                        success: true,
                        token: Some(token),
                        error: None,
                        request_url: resolved_url,
                        request_method: config.method.clone(),
                        request_headers: request_headers_vec,
                        request_body: resolved_body,
                        response_status: response.status,
                        response_body: response.body,
                        response_headers: response_headers_vec,
                        time_elapsed: response.time_elapsed,
                    }
                }
                Err(e) => PreflightTestResult {
                    success: false,
                    token: None,
                    error: Some(e),
                    request_url: resolved_url,
                    request_method: config.method.clone(),
                    request_headers: request_headers_vec,
                    request_body: resolved_body,
                    response_status: response.status,
                    response_body: response.body,
                    response_headers: response_headers_vec,
                    time_elapsed: response.time_elapsed,
                },
            }
        }
        Err(e) => PreflightTestResult {
            success: false,
            token: None,
            error: Some(e),
            request_url: resolved_url,
            request_method: config.method.clone(),
            request_headers: request_headers_vec,
            request_body: resolved_body,
            response_status: 0,
            response_body: "".to_string(),
            response_headers: vec![],
            time_elapsed: 0,
        },
    }
}

fn resolve_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let re = regex::Regex::new(r"\{\{([^}]+)\}\}").expect("Invalid regex");
    let mut result = text.to_string();
    loop {
        let before = result.clone();
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let var_name = caps[1].trim();
                variables
                    .get(var_name)
                    .cloned()
                    .unwrap_or_else(|| caps[0].to_string())
            })
            .to_string();
        if result == before {
            break;
        }
    }
    result
}
