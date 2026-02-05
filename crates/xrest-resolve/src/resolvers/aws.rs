use async_trait::async_trait;
use crate::resolver::{VariableResolver, ResolveError};

/// Resolves secrets from AWS.
pub struct AwsResolver;

#[async_trait]
impl VariableResolver for AwsResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        if let Some(path) = content.strip_prefix("aws:") {
            // Placeholder for AWS SDK logic
            return Ok(Some(format!("mock-aws-value-for-{}", path)));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_aws_resolver() -> Result<(), ResolveError> {
        let resolver = AwsResolver;
        assert_eq!(
            resolver.resolve("aws:secret-id").await?,
            Some("mock-aws-value-for-secret-id".to_string())
        );
        assert_eq!(resolver.resolve("no-prefix").await?, None);
        Ok(())
    }
}
