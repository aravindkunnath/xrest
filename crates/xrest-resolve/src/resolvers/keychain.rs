use async_trait::async_trait;
use crate::resolver::{VariableResolver, ResolveError};
use std::sync::Arc;

#[async_trait]
pub trait KeychainBackend: Send + Sync {
    async fn get_secret(&self, key: &str) -> Result<Option<String>, ResolveError>;
}

/// Real implementation using the `keyring` crate.
pub struct OsKeychainBackend {
    service: String,
}

impl OsKeychainBackend {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
        }
    }
}

#[async_trait]
impl KeychainBackend for OsKeychainBackend {
    async fn get_secret(&self, key: &str) -> Result<Option<String>, ResolveError> {
        let entry = keyring::Entry::new(&self.service, key)
            .map_err(|e| ResolveError::Error(format!("Keychain initialization error: {}", e)))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(ResolveError::Error(format!("Keychain retrieval error: {}", e))),
        }
    }
}

/// Resolves secrets from the OS keychain.
pub struct KeychainResolver {
    backend: Arc<dyn KeychainBackend>,
}

impl KeychainResolver {
    pub fn new(backend: Arc<dyn KeychainBackend>) -> Self {
        Self { backend }
    }

    /// Creates a resolver with the default OS keychain backend.
    pub fn default() -> Self {
        Self::new(Arc::new(OsKeychainBackend::new("xrest")))
    }
}

#[async_trait]
impl VariableResolver for KeychainResolver {
    async fn resolve(&self, content: &str) -> Result<Option<String>, ResolveError> {
        if let Some(key) = content.strip_prefix("secret:") {
            return self.backend.get_secret(key).await;
        }
        Ok(None)
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub struct MockKeychainBackend {
    pub storage: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>,
}

#[cfg(any(test, feature = "test-utils"))]
impl MockKeychainBackend {
    pub fn new(data: std::collections::HashMap<String, String>) -> Self {
        Self {
            storage: std::sync::Arc::new(std::sync::Mutex::new(data)),
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
#[async_trait]
impl KeychainBackend for MockKeychainBackend {
    async fn get_secret(&self, key: &str) -> Result<Option<String>, ResolveError> {
        Ok(self.storage.lock().unwrap().get(key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_keychain_resolver_with_mock() -> Result<(), ResolveError> {
        let mut storage = HashMap::new();
        storage.insert("api_key".to_string(), "super-secret-value".to_string());
        
        let backend = Arc::new(MockKeychainBackend::new(storage));
        let resolver = KeychainResolver::new(backend);

        // Test valid secret retrieval
        assert_eq!(
            resolver.resolve("secret:api_key").await?,
            Some("super-secret-value".to_string())
        );

        // Test non-existent secret
        assert_eq!(resolver.resolve("secret:unknown").await?, None);

        // Test wrong prefix
        assert_eq!(resolver.resolve("key:api_key").await?, None);

        Ok(())
    }
}
