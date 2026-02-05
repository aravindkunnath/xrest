use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use xrest_resolve::{
    EnvFileResolver, KeychainBackend, KeychainResolver, ResolveError, Resolver, 
    ResolverStrategy, SystemEnvResolver, Variable,
};

/// A local mock backend for integration testing to avoid OS keychain dependency.
struct IntegrationMockBackend {
    storage: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl KeychainBackend for IntegrationMockBackend {
    async fn get_secret(&self, key: &str) -> Result<Option<String>, ResolveError> {
        Ok(self.storage.lock().unwrap().get(key).cloned())
    }
}

#[tokio::test]
async fn test_full_resolution_chain() -> Result<(), ResolveError> {
    let mut resolver = Resolver::new();

    // 1. Setup Local Env Map (Highest priority)
    let mut env_map = HashMap::new();
    env_map.insert("API_URL".to_string(), "https://api.example.com".to_string());
    resolver.add_strategy(ResolverStrategy::EnvFile(EnvFileResolver::new(env_map)));

    // 2. Setup System Env
    unsafe {
        std::env::set_var("USER_ROLE", "admin");
    }
    resolver.add_strategy(ResolverStrategy::SystemEnv(SystemEnvResolver));

    // 3. Setup Mock Keychain
    let mut keychain_data = HashMap::new();
    keychain_data.insert("db_password".to_string(), "super-secret-pw".to_string());
    let backend = Arc::new(IntegrationMockBackend {
        storage: Mutex::new(keychain_data),
    });
    resolver.add_strategy(ResolverStrategy::Keychain(KeychainResolver::new(backend)));

    // Test Resolution of various types
    let v1 = Variable::new("URL".into(), "{{ API_URL }}".into());
    let v2 = Variable::new("ROLE".into(), "{{ USER_ROLE }}".into());
    let v3 = Variable::new("DB".into(), "{{ secret:db_password }}".into());
    let v4 = Variable::new("LITERAL".into(), "just-plain-text".into());

    assert_eq!(resolver.resolve_variable(&v1).await?, "https://api.example.com");
    assert_eq!(resolver.resolve_variable(&v2).await?, "admin");
    assert_eq!(resolver.resolve_variable(&v3).await?, "super-secret-pw");
    assert_eq!(resolver.resolve_variable(&v4).await?, "just-plain-text");

    Ok(())
}
