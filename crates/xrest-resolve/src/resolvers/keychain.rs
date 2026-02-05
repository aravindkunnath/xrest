use async_trait::async_trait;
use crate::resolver::{VariableResolver, ResolveError};

/// Resolves secrets from the OS keychain.
pub struct KeychainResolver;

#[async_trait]
impl VariableResolver for KeychainResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        if let Some(key) = content.strip_prefix("secret:") {
            // Placeholder for actual keychain logic
            return Ok(Some(format!("mock-keychain-value-for-{}", key)));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_keychain_resolver() -> Result<(), ResolveError> {
        let resolver = KeychainResolver;
        assert_eq!(
            resolver.resolve("secret:my-token").await?,
            Some("mock-keychain-value-for-my-token".to_string())
        );
        assert_eq!(resolver.resolve("no-prefix").await?, None);
        Ok(())
    }
}
