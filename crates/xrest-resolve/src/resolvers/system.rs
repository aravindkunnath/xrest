use async_trait::async_trait;
use crate::resolver::{VariableResolver, ResolveError};

/// Resolves variables from system environment.
pub struct SystemEnvResolver;

#[async_trait]
impl VariableResolver for SystemEnvResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        Ok(std::env::var(content).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_env_resolver() -> Result<(), ResolveError> {
        let key = "TEST_SYSTEM_VAR";
        let value = "system_value";
        unsafe {
            std::env::set_var(key, value);
        }
        let resolver = SystemEnvResolver;
        assert_eq!(resolver.resolve(key).await?, Some(value.to_string()));
        assert_eq!(resolver.resolve("DEFINITELY_NOT_SET").await?, None);
        Ok(())
    }
}
