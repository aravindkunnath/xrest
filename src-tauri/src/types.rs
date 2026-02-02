use serde::{Deserialize, Serialize};

pub use crate::domains::auth::AuthConfig;
pub use crate::domains::auth::AuthType;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NameValue {
    pub name: String,
    pub value: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub secret_key: Option<String>,
}

fn default_true() -> bool {
    true
}

pub type Variable = NameValue;
pub type Param = NameValue;
pub type Header = NameValue;

pub use crate::domains::service::environment::EnvironmentConfig;

pub use crate::domains::service::endpoint::{Endpoint, EndpointMetadata, PreflightConfig};
pub use crate::domains::service::service::{Service, ServiceStub};

pub use crate::domains::git::GitStatus;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<Header>,
    pub body: String,
    pub error: Option<String>,
    pub time_elapsed: u64,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreflightTestResult {
    pub success: bool,
    pub token: Option<String>,
    pub error: Option<String>,
    pub request_url: String,
    pub request_method: String,
    pub request_headers: Vec<Header>,
    pub request_body: String,
    pub response_status: u16,
    pub response_body: String,
    pub response_headers: Vec<Header>,
    pub time_elapsed: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BodyConfig {
    pub r#type: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestTab {
    pub id: String,
    pub endpoint_id: Option<String>,
    pub title: String,
    pub method: String,
    pub url: String,
    pub params: Vec<Param>,
    pub headers: Vec<Header>,
    pub body: BodyConfig,
    pub auth: AuthConfig,
    #[serde(default)]
    pub active_sub_tab: Option<String>,
    #[serde(default)]
    pub service_id: Option<String>,
    pub preflight: PreflightConfig,
    #[serde(default)]
    pub variables: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    pub is_edited: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SettingsTab {
    pub id: String,
    pub title: String,
    pub service_id: String,
    pub service_data: Service,
    #[serde(default)]
    pub is_edited: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Tab {
    Request(RequestTab),
    Settings(SettingsTab),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TabState {
    pub active_tab_id: String,
    pub tabs: Vec<Tab>,
}

pub use crate::domains::settings::UserSettings;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub service_id: Option<String>,
    pub endpoint_id: Option<String>,
    pub method: String,
    pub url: String,
    pub request_headers: Vec<Header>,
    pub request_body: String,
    pub response_status: u16,
    pub response_status_text: String,
    pub response_headers: Vec<Header>,
    pub response_body: String,
    pub time_elapsed: u64,
    pub size: u64,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_serialization() {
        let endpoint = Endpoint {
            id: "e1".to_string(),
            service_id: "s1".to_string(),
            name: "Test Endpoint".to_string(),
            method: "GET".to_string(),
            url: "/test".to_string(),
            authenticated: true,
            auth_type: "bearer".to_string(),
            metadata: EndpointMetadata {
                version: "1.0".to_string(),
                last_updated: 123456789,
            },
            params: vec![NameValue {
                name: "p1".to_string(),
                value: "v1".to_string(),
                enabled: true,
                secret_key: None,
            }],
            headers: vec![NameValue {
                name: "h1".to_string(),
                value: "v1".to_string(),
                enabled: true,
                secret_key: None,
            }],
            body: "".to_string(),
            preflight: PreflightConfig {
                enabled: true,
                method: "GET".to_string(),
                url: "/preflight".to_string(),
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
        };

        let yaml = serde_yaml::to_string(&endpoint).unwrap();
        println!("{}", yaml);

        assert!(yaml.contains("id: e1"));
        assert!(yaml.contains("serviceId: s1"));
        assert!(yaml.contains("authType: bearer"));
        assert!(yaml.contains("method: GET"));
    }

    #[test]
    fn test_service_serialization() {
        let service = Service {
            id: "s1".to_string(),
            name: "Test Service".to_string(),
            environments: vec![EnvironmentConfig {
                name: "DEV".to_string(),
                is_unsafe: false,
                variables: vec![NameValue {
                    name: "BASE_URL".to_string(),
                    value: "http://localhost:3000".to_string(),
                    enabled: true,
                    secret_key: None,
                }],
            }],
            is_authenticated: true,
            auth_type: Some(AuthType::Bearer),
            auth: AuthConfig {
                r#type: "bearer".to_string(),
                active: true,
                basic_user: "".to_string(),
                basic_pass: "".to_string(),
                bearer_token: "token".to_string(),
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
            endpoints: vec![],
            directory: "/tmp".to_string(),
            selected_environment: Some("DEV".to_string()),
            git_url: None,
        };

        let yaml = serde_yaml::to_string(&service).unwrap();
        assert!(yaml.contains("id: s1"));
        assert!(yaml.contains("name: Test Service"));
        assert!(yaml.contains("authType: bearer"));

        let deserialized: Service = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.id, "s1");
        assert_eq!(deserialized.environments[0].name, "DEV");
    }
}
