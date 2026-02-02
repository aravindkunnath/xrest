use crate::types::Variable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentConfig {
    pub name: String,
    #[serde(default)]
    pub is_unsafe: bool,
    pub variables: Vec<Variable>,
}
