use crate::core::traits::HttpClient;
use crate::core::types::{PreflightConfig, QResponse, RequestTab};
use std::collections::HashMap;

pub struct RequestService<'a> {
    pub http: &'a dyn HttpClient,
    pub cache_path: Option<std::path::PathBuf>,
}

impl<'a> RequestService<'a> {
    pub fn new(http: &'a dyn HttpClient, cache_path: Option<std::path::PathBuf>) -> Self {
        Self { http, cache_path }
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
            if let Some(cached) = crate::core::auth::cache::get_cached_token(service_id_str) {
                if crate::core::auth::cache::is_token_valid(&cached) {
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
                tab.headers.push(crate::core::types::Header {
                    name: token_header,
                    value: token_val,
                    enabled: true,
                    secret_key: None,
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
                if !tab.auth.api_key_name.is_empty() {
                    if tab.auth.api_key_location == "header" {
                        headers.push((
                            tab.auth.api_key_name.clone(),
                            tab.auth.api_key_value.clone(),
                        ));
                    }
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
        crate::core::auth::preflight::execute_preflight(
            self.http,
            service_id,
            config,
            variables,
            self.cache_path.as_ref(),
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

                    if var_name.starts_with("secret.") {
                        let key = &var_name[7..];
                        match crate::core::secrets::SecretsDomain::get_secret(key) {
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
