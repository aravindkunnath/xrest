use async_trait::async_trait;
use crate::resolver::{VariableResolver, ResolveError};

/// Resolves secrets from Azure.
pub struct AzureResolver;

#[async_trait]
impl VariableResolver for AzureResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        if let Some(path) = content.strip_prefix("az:") {
            // Placeholder for Azure logic
            return Ok(Some(format!("mock-azure-value-for-{}", path)));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_azure_resolver() -> Result<(), ResolveError> {
        let resolver = AzureResolver;
        assert_eq!(
            resolver.resolve("az:secret-name").await?,
            Some("mock-azure-value-for-secret-name".to_string())
        );
        assert_eq!(resolver.resolve("no-prefix").await?, None);
        Ok(())
    }
}
