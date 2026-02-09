use crate::resolver::{ResolveError, VariableResolver};
use async_trait::async_trait;
use google_cloud_secretmanager_v1::client::SecretManagerService;
use std::sync::Arc;

#[async_trait]
pub trait GcpBackend: Send + Sync {
    async fn get_secret(&self, name: &str) -> Result<Option<String>, ResolveError>;
}

/// Real implementation using the `google-cloud-secretmanager` crate.
pub struct RealGcpBackend {
    service: SecretManagerService,
}

impl RealGcpBackend {
    pub async fn new() -> Self {
        let service = SecretManagerService::builder().build().await.unwrap();
        Self { service }
    }
}

#[async_trait]
impl GcpBackend for RealGcpBackend {
    async fn get_secret(&self, name: &str) -> Result<Option<String>, ResolveError> {
        let response = self
            .service
            .access_secret_version()
            .set_name(name.to_string())
            .send()
            .await
            .map_err(|e| {
                ResolveError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("GCP Secret Manager error: {}", e),
                ))
            })?;

        let payload = response
            .payload
            .ok_or_else(|| ResolveError::Error("Empty payload from GCP".to_string()))?;

        let value = String::from_utf8(payload.data.to_vec())
            .map_err(|e| ResolveError::Error(format!("UTF-8 decoding error: {}", e)))?;

        Ok(Some(value))
    }
}

/// Resolves secrets from GCP Secret Manager.
pub struct GcpResolver {
    backend: Arc<dyn GcpBackend>,
}

impl GcpResolver {
    pub fn new(backend: Arc<dyn GcpBackend>) -> Self {
        Self { backend }
    }
}

#[async_trait]
impl VariableResolver for GcpResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        if let Some(path) = content.strip_prefix("gcp:") {
            return self.backend.get_secret(path).await;
        }
        Ok(None)
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub struct MockGcpBackend {
    pub secrets: std::collections::HashMap<String, String>,
}

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl GcpBackend for MockGcpBackend {
    async fn get_secret(&self, name: &str) -> Result<Option<String>, ResolveError> {
        Ok(self.secrets.get(name).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_gcp_resolver_with_mock() -> Result<(), ResolveError> {
        let mut secrets = HashMap::new();
        secrets.insert(
            "projects/my-project/secrets/my-secret/versions/latest".to_string(),
            "super-secret".to_string(),
        );

        let backend = Arc::new(MockGcpBackend { secrets });
        let resolver = GcpResolver::new(backend);

        assert_eq!(
            resolver
                .resolve("gcp:projects/my-project/secrets/my-secret/versions/latest")
                .await?,
            Some("super-secret".to_string())
        );
        assert_eq!(resolver.resolve("no-prefix").await?, None);
        Ok(())
    }
}
