use async_trait::async_trait;
use std::collections::HashMap;
use crate::resolver::{VariableResolver, ResolveError};

/// Resolves variables from a local `.env` file map.
pub struct EnvFileResolver {
    pub(crate) vars: HashMap<String, String>,
}

impl EnvFileResolver {
    pub fn new(vars: HashMap<String, String>) -> Self {
        Self { vars }
    }
}

#[async_trait]
impl VariableResolver for EnvFileResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        Ok(self.vars.get(content).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_file_resolver() -> Result<(), ResolveError> {
        let mut env_vars = HashMap::new();
        env_vars.insert("LOCAL_VAR".to_string(), "local_value".to_string());
        let resolver = EnvFileResolver::new(env_vars);
        assert_eq!(resolver.resolve("LOCAL_VAR").await?, Some("local_value".to_string()));
        assert_eq!(resolver.resolve("NON_EXISTENT").await?, None);
        Ok(())
    }
}
