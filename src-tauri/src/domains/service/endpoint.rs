use crate::types::{Header, Param};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PreflightConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub body: String,
    #[serde(default = "default_body_type")]
    pub body_type: String,
    #[serde(default)]
    pub body_params: Vec<Param>,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(default)]
    pub cache_token: bool,
    #[serde(default)]
    pub cache_duration: String,
    #[serde(default)]
    pub cache_duration_key: String,
    #[serde(default = "default_duration_unit")]
    pub cache_duration_unit: String,
    #[serde(default)]
    pub token_key: String,
    #[serde(default)]
    pub token_header: Option<String>,
}

fn default_method() -> String {
    "POST".to_string()
}

fn default_body_type() -> String {
    "application/json".to_string()
}

fn default_duration_unit() -> String {
    "seconds".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointMetadata {
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub last_updated: u64,
}

fn default_version() -> String {
    "1".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RequestConfig {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(default)]
    pub body: String,
    #[serde(default = "default_preflight_config")]
    pub preflight: PreflightConfig,
}

fn default_auth_type() -> String {
    "none".to_string()
}

fn default_preflight_config() -> PreflightConfig {
    PreflightConfig {
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
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EndpointVersion {
    pub version: i32,
    pub config: RequestConfig,
    pub last_updated: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub id: String,
    #[serde(default)]
    pub service_id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub authenticated: bool,
    #[serde(default = "default_auth_type")]
    pub auth_type: String,
    #[serde(default = "default_metadata")]
    pub metadata: EndpointMetadata,
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(default)]
    pub headers: Vec<Header>,
    #[serde(default)]
    pub body: String,
    #[serde(default = "default_preflight_config")]
    pub preflight: PreflightConfig,
    #[serde(default)]
    pub last_version: i32,
    #[serde(default)]
    pub versions: Vec<EndpointVersion>,
}

fn default_metadata() -> EndpointMetadata {
    EndpointMetadata {
        version: "1".to_string(),
        last_updated: 0,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EndpointStub {
    pub id: String,
    pub name: String,
    pub url: String,
}
