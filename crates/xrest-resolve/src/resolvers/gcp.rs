use async_trait::async_trait;
use crate::resolver::{VariableResolver, ResolveError};

/// Resolves secrets from GCP Secret Manager.
pub struct GcpResolver;

#[async_trait]
impl VariableResolver for GcpResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        if let Some(path) = content.strip_prefix("gcp:") {
            // Placeholder for GCP SDK logic
            return Ok(Some(format!("mock-gcp-value-for-{}", path)));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gcp_resolver() -> Result<(), ResolveError> {
        let resolver = GcpResolver;
        assert_eq!(
            resolver.resolve("gcp:secret-path").await?,
            Some("mock-gcp-value-for-secret-path".to_string())
        );
        assert_eq!(resolver.resolve("no-prefix").await?, None);
        Ok(())
    }
}
