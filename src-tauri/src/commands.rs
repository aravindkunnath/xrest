use crate::io::{RealFileSystem, RealHttpClient};
use crate::services::{ConfigService, RequestService};
use crate::types::{
    AuthConfig, AuthType, Endpoint, EndpointMetadata, EnvironmentConfig, HistoryEntry, NameValue,
    PreflightConfig, QResponse, RequestTab, Service, UserSettings,
};
use openapiv3::OpenAPI;

use curl_parser::ParsedRequest;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use url::Url;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<UserSettings, String> {
    let service = ConfigService::new(&RealFileSystem);
    service.load_settings(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: UserSettings) -> Result<(), String> {
    let service = ConfigService::new(&RealFileSystem);
    // Load existing settings first to preserve other fields (like services)
    let mut current_settings = service.load_settings(&app)?;
    current_settings.theme = settings.theme;
    service.save_settings(&app, &current_settings)
}

#[tauri::command]
pub fn get_services(app: AppHandle) -> Result<Vec<Service>, String> {
    let config_service = ConfigService::new(&RealFileSystem);
    let settings = config_service.load_settings(&app)?;

    let mut services = Vec::new();
    let mut errors = Vec::new();

    for stub in settings.services {
        match config_service.load_service(&stub.directory) {
            Ok(service) => {
                services.push(service);
            }
            Err(e) => {
                let err_msg = format!("Failed to load service {}: {}", stub.name, e);
                println!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    }

    if !errors.is_empty() && services.is_empty() {
        return Err(errors.join("\n"));
    }

    Ok(services)
}

#[tauri::command]
pub fn save_services(
    app: AppHandle,
    mut services: Vec<Service>,
    commit_message: Option<String>,
) -> Result<Vec<Service>, String> {
    let config_service = ConfigService::new(&RealFileSystem);
    let mut settings = config_service.load_settings(&app)?;
    let mut stubs = Vec::new();

    for service in &mut services {
        config_service.save_service(service, commit_message.clone())?;
        stubs.push(crate::types::ServiceStub {
            id: service.id.clone(),
            name: service.name.clone(),
            directory: service.directory.clone(),
        });
    }

    settings.services = stubs;
    config_service.save_settings(&app, &settings)?;
    Ok(services)
}

#[tauri::command]
pub fn get_collections(app: AppHandle) -> Result<Vec<Service>, String> {
    let service = ConfigService::new(&RealFileSystem);
    service.load_collections(&app)
}

#[tauri::command]
pub fn save_collections(app: AppHandle, collections: Vec<Service>) -> Result<Vec<Service>, String> {
    let service = ConfigService::new(&RealFileSystem);
    service.save_collections(&app, &collections)?;
    Ok(collections)
}

#[tauri::command]
pub fn get_tab_state(app: AppHandle) -> Result<Option<crate::types::TabState>, String> {
    let service = ConfigService::new(&RealFileSystem);
    service.load_tab_state(&app)
}

#[tauri::command]
pub fn save_tab_state(app: AppHandle, state: crate::types::TabState) -> Result<(), String> {
    let service = ConfigService::new(&RealFileSystem);
    service.save_tab_state(&app, &state)
}

#[tauri::command]
pub async fn send_request(app: AppHandle, mut tab: RequestTab) -> Result<QResponse, String> {
    if let Some(sid) = &tab.service_id {
        let app_handle = app.clone();
        let sid_clone = sid.clone();

        let service_config = tokio::task::spawn_blocking(move || {
            let config = ConfigService::new(&RealFileSystem);
            let settings = config.load_settings(&app_handle).ok()?;
            let stub = settings.services.iter().find(|s| s.id == sid_clone)?;
            config.load_service(&stub.directory).ok()
        })
        .await
        .map_err(|e| e.to_string())?;

        if let Some(service) = service_config {
            if tab.auth.r#type == "none" {
                tab.auth = service.auth;
            }
            if !tab.preflight.enabled {
                tab.preflight = service.preflight;
            }
        }
    }

    let cache_path = crate::domains::auth::get_token_cache_path(&app).ok();
    let request_service = RequestService::new(&RealHttpClient, cache_path);
    let req_method = tab.method.clone();
    let req_url = tab.url.clone();
    let endpoint_id = tab.endpoint_id.clone();
    let service_id = tab.service_id.clone();
    let headers_clone = tab.headers.clone();
    let body_clone = tab.body.content.clone();

    let response = request_service.send_request(tab).await?;

    let history_entry = HistoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        service_id,
        endpoint_id,
        method: req_method,
        url: req_url,
        request_headers: headers_clone,
        request_body: body_clone,
        response_status: response.status,
        response_status_text: response.status_text.clone(),
        response_headers: response.headers.clone(),
        response_body: response.body.clone(),
        time_elapsed: response.time_elapsed,
        size: response.size,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let app_handle = app.clone();
    tokio::spawn(async move {
        if let Err(e) = crate::history::save_history(&app_handle, history_entry) {
            eprintln!("Failed to save history: {}", e);
        }
    });

    Ok(response)
}

#[tauri::command]
pub fn close_splashscreen(app: AppHandle) {
    use tauri::Manager;
    if let Some(splashscreen) = app.get_webview_window("splashscreen") {
        let _ = splashscreen.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}
#[tauri::command]
pub fn import_service(app: AppHandle, directory: String) -> Result<Service, String> {
    let config_service = ConfigService::new(&RealFileSystem);
    let mut service = config_service.load_service(&directory)?;
    let mut settings = config_service.load_settings(&app)?;

    if settings.services.iter().any(|s| s.directory == directory) {
        return Err("This directory is already imported as a service.".to_string());
    }

    service.directory = directory.clone();
    settings.services.push(crate::types::ServiceStub {
        id: service.id.clone(),
        name: service.name.clone(),
        directory: directory.clone(),
    });

    let service_name = service.name.clone();
    config_service.save_service(&mut service, Some(format!("Import service: {}", service_name)))?;

    Ok(service)
}

#[tauri::command]
pub fn get_git_status(
    _app: AppHandle,
    directory: String,
) -> Result<crate::types::GitStatus, String> {
    crate::domains::git::get_git_status(&directory)
}

#[tauri::command]
pub fn git_init(
    _app: AppHandle,
    directory: String,
    remote_url: Option<String>,
) -> Result<(), String> {
    crate::domains::git::init_git(&directory, remote_url)
}

#[tauri::command]
pub fn git_pull(_app: AppHandle, directory: String) -> Result<(), String> {
    crate::domains::git::pull_changes(&directory)
}

#[tauri::command]
pub fn git_push(_app: AppHandle, directory: String) -> Result<(), String> {
    crate::domains::git::push_changes(&directory)
}

#[tauri::command]
pub fn git_commit(_app: AppHandle, directory: String, message: String) -> Result<(), String> {
    crate::domains::git::commit_changes(&directory, &message)
}

#[tauri::command]
pub fn git_sync(_app: AppHandle, directory: String) -> Result<(), String> {
    crate::domains::git::sync_git(&directory)
}

#[tauri::command]
pub fn get_history(
    app: AppHandle,
    limit: usize,
    offset: usize,
) -> Result<Vec<HistoryEntry>, String> {
    crate::history::get_history(&app, limit, offset)
}

#[tauri::command]
pub async fn import_swagger(
    app: AppHandle,
    name: String,
    directory: String,
    url: Option<String>,
    file: Option<String>,
) -> Result<Service, String> {
    let config_service = ConfigService::new(&RealFileSystem);
    let content = if let Some(u) = url {
        reqwest::get(u)
            .await
            .map_err(|e| format!("Failed to fetch Swagger URL: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to read Swagger response: {}", e))?
    } else if let Some(f) = file {
        config_service
            .fs
            .read_to_string(std::path::Path::new(&f))
            .map_err(|e| format!("Failed to read Swagger file: {}", e))?
    } else {
        return Err("No Swagger source provided".to_string());
    };

    let service_id = format!(
        "s-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let (base_url, endpoints) = parse_spec_content(&content, &service_id)?;

    let mut service = Service {
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
        directory: directory.clone(),
        selected_environment: Some("DEV".to_string()),
        git_url: None,
    };

    let config_service = ConfigService::new(&RealFileSystem);
    let service_name = service.name.clone();
    config_service.save_service(&mut service, Some(format!("Import service from Swagger: {}", service_name)))?;

    let mut settings = config_service.load_settings(&app)?;
    settings.services.push(crate::types::ServiceStub {
        id: service.id.clone(),
        name: service.name.clone(),
        directory: directory.clone(),
    });
    config_service.save_settings(&app, &settings)?;

    Ok(service)
}

#[tauri::command]
pub async fn import_curl(
    app: AppHandle,
    service_id: String,
    curl_command: String,
) -> Result<Service, String> {
    let config_service = ConfigService::new(&RealFileSystem);
    let settings = config_service.load_settings(&app)?;

    let service_stub = settings
        .services
        .iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| format!("Service not found: {}", service_id))?;

    let mut service = config_service.load_service(&service_stub.directory)?;

    let endpoint = curl_to_endpoint(
        service_id.clone(),
        &curl_command,
        service.is_authenticated,
        service.auth_type.as_ref().map(|at| at.to_string()),
    )?;

    let endpoint_name = endpoint.name.clone();
    service.endpoints.push(endpoint);
    config_service.save_service(&mut service, Some(format!("Import endpoint from cURL: {}", endpoint_name)))?;

    Ok(service)
}

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
        },
        last_version: 0,
        versions: vec![],
    })
}

#[tauri::command]
pub fn clear_history(app: AppHandle) -> Result<(), String> {
    crate::history::clear_history(&app)
}

pub fn parse_spec_content(
    content: &str,
    service_id: &str,
) -> Result<(String, Vec<Endpoint>), String> {
    let value: serde_json::Value = if content.trim().starts_with('{') {
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?
    } else {
        serde_yaml::from_str(&content).map_err(|e| format!("Failed to parse YAML: {}", e))?
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

                        endpoints.push(Endpoint {
                            id: endpoint_id,
                            service_id: service_id.to_string(),
                            name: endpoint_name,
                            method: method.to_string(),
                            url: path.clone(),
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
                        });
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

                        endpoints.push(Endpoint {
                            id: endpoint_id,
                            service_id: service_id.to_string(),
                            name: endpoint_name,
                            method: method_upper,
                            url: path.clone(),
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
                        });
                    }
                }
            }
        }
    }

    Ok((base_url, endpoints))
}

#[tauri::command]
pub async fn test_preflight_config(
    app: AppHandle,
    service_id: String,
    config: PreflightConfig,
    variables: std::collections::HashMap<String, String>,
) -> Result<crate::types::PreflightTestResult, String> {
    let cache_path = crate::domains::auth::get_token_cache_path(&app).ok();

    // We use the same simple resolution as the actual preflight logic for now to match behavior
    // In the future we should unify secret resolution across the board

    Ok(crate::domains::auth::preflight::test_preflight(
        &RealHttpClient,
        &service_id,
        &config,
        &variables,
        cache_path.as_ref(),
    )
    .await)
}

#[tauri::command]
pub async fn get_secrets(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    println!("DEBUG: get_secrets called");
    tokio::task::spawn_blocking(move || {
        let domain = crate::domains::secrets::SecretsDomain::new(&RealFileSystem);
        domain.list_secrets(&app)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn add_secret(
    app: tauri::AppHandle,
    key: String,
    value: String,
) -> Result<Vec<String>, String> {
    println!("DEBUG: add_secret called for key: {}", key);
    tokio::task::spawn_blocking(move || {
        let domain = crate::domains::secrets::SecretsDomain::new(&RealFileSystem);
        domain.add_secret(&app, &key, &value)?;
        domain.list_secrets(&app)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_secret(app: tauri::AppHandle, key: String) -> Result<Vec<String>, String> {
    println!("DEBUG: delete_secret called for key: {}", key);
    tokio::task::spawn_blocking(move || {
        let domain = crate::domains::secrets::SecretsDomain::new(&RealFileSystem);
        domain.delete_secret(&app, &key)?;
        domain.list_secrets(&app)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_secret(key: String) -> Result<String, String> {
    println!("DEBUG: get_secret called for key: {}", key);
    tokio::task::spawn_blocking(move || crate::domains::secrets::SecretsDomain::get_secret(&key))
        .await
        .map_err(|e| e.to_string())?
}
