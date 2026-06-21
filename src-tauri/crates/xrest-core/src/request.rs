use crate::service::service::ServiceDomain;
use crate::settings::SettingsDomain;
use crate::traits::{FileSystem, HttpClient, SecretStore};
use crate::types::{HistoryEntry, PreflightConfig, QResponse, RequestTab};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct RequestService<'a> {
    pub http: &'a dyn HttpClient,
    pub secret_store: &'a dyn SecretStore,
    pub fs: Option<&'a dyn FileSystem>,
    pub cache_path: Option<std::path::PathBuf>,
}

impl<'a> RequestService<'a> {
    pub fn new(
        http: &'a dyn HttpClient,
        secret_store: &'a dyn SecretStore,
        cache_path: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            http,
            secret_store,
            fs: None,
            cache_path,
        }
    }

    pub fn with_fs(mut self, fs: &'a dyn FileSystem) -> Self {
        self.fs = Some(fs);
        self
    }

    pub async fn send_request(&self, mut tab: RequestTab) -> Result<QResponse, String> {
        // Resolve variables in URL, body, and headers
        let default_vars = HashMap::new();
        let vars = tab.variables.as_ref().unwrap_or(&default_vars);
        tab.url = self.resolve_variables(&tab.url, vars);
        tab.body.content = self.resolve_variables(&tab.body.content, vars);

        for header in &mut tab.headers {
            header.name = self.resolve_variables(&header.name, vars);
            header.value = self.resolve_variables(&header.value, vars);
        }

        // Handle preflight if needed
        let mut token = None;
        let service_id_str = tab.service_id.as_deref().unwrap_or("");

        if tab.preflight.enabled && !tab.preflight.url.is_empty() {
            token = Some(
                self.execute_preflight(service_id_str, &tab.preflight, vars)
                    .await?,
            );
        } else if !service_id_str.is_empty() {
            // Even if preflight is disabled for this tab, check if we have a cached token for this service
            if let Some(cached) = crate::auth::cache::get_cached_token(service_id_str) {
                if crate::auth::cache::is_token_valid(&cached) {
                    token = Some(cached.token);
                }
            }
        }

        if let Some(token_val) = token {
            let token_header = tab
                .preflight
                .token_header
                .as_ref()
                .filter(|h| !h.is_empty())
                .cloned()
                .unwrap_or_else(|| "Authorization".to_string());
            if token_header.to_lowercase() == "authorization" {
                tab.auth.bearer_token = token_val;
                tab.auth.r#type = "bearer".to_string();
            } else {
                tab.headers.push(crate::types::Header {
                    name: token_header,
                    value: token_val,
                    enabled: true,
                    secret_key: None,
                    r#type: "plain".to_string(),
                });
                tab.auth.r#type = "none".to_string();
            }
        }

        let mut headers = Vec::new();
        for h in &tab.headers {
            headers.push((h.name.clone(), h.value.clone()));
        }

        // Add auth headers
        match tab.auth.r#type.as_str() {
            "bearer" => {
                if !tab.auth.bearer_token.is_empty() {
                    headers.push((
                        "Authorization".to_string(),
                        format!("Bearer {}", tab.auth.bearer_token),
                    ));
                }
            }
            "basic" => {
                if !tab.auth.basic_user.is_empty() {
                    let auth = format!("{}:{}", tab.auth.basic_user, tab.auth.basic_pass);
                    use base64::{engine::general_purpose, Engine as _};
                    let encoded = general_purpose::STANDARD.encode(auth);
                    headers.push(("Authorization".to_string(), format!("Basic {}", encoded)));
                }
            }
            "apikey" => {
                if !tab.auth.api_key_name.is_empty() && tab.auth.api_key_location == "header" {
                    headers.push((
                        tab.auth.api_key_name.clone(),
                        tab.auth.api_key_value.clone(),
                    ));
                }
            }
            _ => {}
        }

        let mut query = Vec::new();
        for p in &tab.params {
            query.push((p.name.clone(), p.value.clone()));
        }

        // Add apikey to query if location is query
        if tab.auth.r#type == "apikey"
            && tab.auth.api_key_location == "query"
            && !tab.auth.api_key_name.is_empty()
        {
            query.push((
                tab.auth.api_key_name.clone(),
                tab.auth.api_key_value.clone(),
            ));
        }

        if !tab.body.content.is_empty()
            && tab.method.to_uppercase() != "GET"
            && tab.method.to_uppercase() != "HEAD"
        {
            headers.push(("Content-Type".to_string(), tab.body.r#type.clone()));
        }

        self.http
            .send_request(
                &tab.method,
                &tab.url,
                headers,
                if tab.body.content.is_empty() {
                    None
                } else {
                    Some(tab.body.content.clone())
                },
                query,
            )
            .await
    }

    async fn execute_preflight(
        &self,
        service_id: &str,
        config: &PreflightConfig,
        variables: &HashMap<String, String>,
    ) -> Result<String, String> {
        crate::auth::preflight::execute_preflight(
            self.http,
            service_id,
            config,
            variables,
            self.cache_path.as_ref(),
            self.fs,
        )
        .await
    }

    fn resolve_variables(&self, text: &str, variables: &HashMap<String, String>) -> String {
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").expect("Invalid regex");
        let mut result = text.to_string();

        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 10;

        loop {
            let before = result.clone();
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let var_name = caps[1].trim();

                    if let Some(key) = var_name.strip_prefix("secret:").or_else(|| var_name.strip_prefix("secret.")) {
                        let key = key.trim();
                        match self.secret_store.get(key) {
                            Ok(val) => val,
                            Err(e) => {
                                println!("Failed to resolve secret {}: {}", key, e);
                                caps[0].to_string()
                            }
                        }
                    } else {
                        variables
                            .get(var_name)
                            .cloned()
                            .unwrap_or_else(|| caps[0].to_string())
                    }
                })
                .to_string();

            iterations += 1;
            if result == before || iterations >= MAX_ITERATIONS {
                break;
            }
        }
        result
    }
}

/// Executes a request with full service context resolution (auth/preflight inheritance)
/// and produces a history entry alongside the response.
pub async fn send_request_with_context(
    http: &dyn HttpClient,
    fs: &dyn FileSystem,
    secret_store: &dyn SecretStore,
    settings_path: &PathBuf,
    cache_path: Option<PathBuf>,
    mut tab: RequestTab,
) -> Result<(QResponse, HistoryEntry), String> {
    // Load service config and inherit auth/preflight if not overridden
    if let Some(sid) = &tab.service_id {
        let settings_domain = SettingsDomain::new(fs);
        let service_domain = ServiceDomain::new(fs);

        if let Ok(settings) = settings_domain.load_settings(settings_path) {
            if let Some(stub) = settings.services.iter().find(|s| s.id == *sid) {
                if let Ok(service) = service_domain.load_service(&stub.directory) {
                    if tab.auth.r#type == "none" {
                        tab.auth = service.auth;
                    }
                    if !tab.preflight.enabled {
                        tab.preflight = service.preflight;
                    }

                    // 1. Identify selected environment config
                    let env_name = service.selected_environment.as_deref().unwrap_or("");

                    if !env_name.is_empty() {
                        // 2. Load the .env file for this environment
                        let dotenv_map = crate::service::dotenv::load_dotenv_vars(
                            &stub.directory,
                            env_name,
                            fs,
                        ).unwrap_or_default();

                        // 3. Update the tab's variable pool with environment-level variables
                        if let Some(env_config) = service.environments.iter().find(|e| e.name == env_name) {
                            for var in &env_config.variables {
                                if var.enabled {
                                    let tab_vars = tab.variables.get_or_insert_with(HashMap::new);
                                    // Tab variables take precedence
                                    if !tab_vars.contains_key(&var.name) {
                                        let mut final_val = var.value.clone();
                                        match var.r#type.as_str() {
                                            "env" => {
                                                let key = final_val.strip_prefix("env:").unwrap_or(&final_val).trim();
                                                if let Some(resolved) = dotenv_map.get(key) {
                                                    final_val = resolved.clone();
                                                }
                                            },
                                            "secret" => {
                                                let key = final_val.trim();
                                                if let Ok(resolved) = secret_store.get(key) {
                                                    final_val = resolved;
                                                }
                                            },
                                            _ => {
                                                // Backward compatibility for "env:" prefix
                                                if let Some(key) = final_val.strip_prefix("env:") {
                                                    if let Some(resolved) = dotenv_map.get(key.trim()) {
                                                        final_val = resolved.clone();
                                                    }
                                                }
                                            }
                                        }
                                        tab_vars.insert(var.name.clone(), final_val);
                                    }
                                }
                            }
                        }

                        // 4. Resolve any "env:" prefixes in tab-provided variables
                        if let Some(tab_vars) = tab.variables.as_mut() {
                            for val in tab_vars.values_mut() {
                                if let Some(key) = val.strip_prefix("env:") {
                                    if let Some(resolved) = dotenv_map.get(key.trim()) {
                                        *val = resolved.clone();
                                    }
                                }
                            }
                        }

                        // 5. Inject all .env variables as env:KEY and env.KEY for direct interpolation
                        let tab_vars = tab.variables.get_or_insert_with(HashMap::new);
                        for (k, v) in &dotenv_map {
                            tab_vars.entry(format!("env:{}", k)).or_insert(v.clone());
                            tab_vars.entry(format!("env.{}", k)).or_insert(v.clone());
                        }
                    }
                }
            }
        }
    }

    let request_service = RequestService::new(http, secret_store, cache_path.clone()).with_fs(fs);

    // Resolve variables in URL and headers using the combined variable set
    let vars = tab.variables.clone().unwrap_or_default();
    tab.url = request_service.resolve_variables(&tab.url, &vars);
    for header in tab.headers.iter_mut() {
        header.name = request_service.resolve_variables(&header.name, &vars);
        header.value = request_service.resolve_variables(&header.value, &vars);
    }

    let req_method = tab.method.clone();
    let req_url = tab.url.clone();
    let endpoint_id = tab.endpoint_id.clone();
    let service_id = tab.service_id.clone();
    let headers_clone = tab.headers.clone();
    let body_clone = tab.body.content.clone();

    let request_service = RequestService::new(http, secret_store, cache_path).with_fs(fs);
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

    Ok((response, history_entry))
}
