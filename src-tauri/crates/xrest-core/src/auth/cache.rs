#[cfg(feature = "desktop")]
use std::collections::HashMap;
#[cfg(feature = "desktop")]
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Represents a cached token with expiration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedToken {
    pub token: String,
    pub expires_at: u64, // Unix timestamp in seconds
}

impl CachedToken {
    /// Check if a cached token is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now < self.expires_at
    }
}

pub struct TokenKey;

impl TokenKey {
    /// Helper to generate a unique cache key based on service ID and preflight details
    pub fn generate(
        service_id: &str,
        url: &str,
        method: &str,
        body: &str,
        headers: &[(String, String)],
    ) -> String {
        // If we have a service ID, we want to share the token across all endpoints in that service.
        if !service_id.is_empty() {
            return service_id.to_string();
        }

        // For the scratchpad (no service ID), we must use the preflight details to avoid collisions
        // between different APIs the user might be testing.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        method.hash(&mut hasher);
        body.hash(&mut hasher);
        headers.hash(&mut hasher);
        let hash = hasher.finish();

        format!("scratchpad:{:x}", hash)
    }
}

/// Port defining how tokens are stored and retrieved
pub trait TokenStore: Send + Sync {
    fn get(&self, key: &str) -> Option<CachedToken>;
    fn set(&self, key: String, token: String, expires_at: u64);
    fn clear(&self, key: Option<&str>);
    fn load_from_file(&self, path: &std::path::Path, fs: &dyn crate::traits::FileSystem) -> Result<(), String>;
    fn save_to_file(&self, path: &std::path::Path, fs: &dyn crate::traits::FileSystem) -> Result<(), String>;
}

/// In-memory implementation of TokenStore, with optional file persistence
#[cfg(feature = "desktop")]
pub struct MemoryTokenStore {
    inner: Arc<Mutex<HashMap<String, CachedToken>>>,
}

#[cfg(feature = "desktop")]
impl MemoryTokenStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[cfg(feature = "desktop")]
impl Default for MemoryTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "desktop")]
impl TokenStore for MemoryTokenStore {
    fn get(&self, key: &str) -> Option<CachedToken> {
        let cache = self.inner.lock().unwrap();
        cache.get(key).cloned()
    }

    fn set(&self, key: String, token: String, expires_at: u64) {
        let mut cache = self.inner.lock().unwrap();
        cache.insert(key, CachedToken { token, expires_at });
    }

    fn clear(&self, key: Option<&str>) {
        let mut cache = self.inner.lock().unwrap();
        if let Some(k) = key {
            cache.remove(k);
        } else {
            cache.clear();
        }
    }

    fn load_from_file(&self, path: &std::path::Path, fs: &dyn crate::traits::FileSystem) -> Result<(), String> {
        if !fs.exists(path) {
            return Ok(());
        }
        let content = fs.read_to_string(path)?;
        let loaded: HashMap<String, CachedToken> = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;

        let mut cache = self.inner.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        for (key, token) in loaded {
            if token.expires_at > now {
                cache.insert(key, token);
            }
        }
        Ok(())
    }

    fn save_to_file(&self, path: &std::path::Path, fs: &dyn crate::traits::FileSystem) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !fs.exists(parent) {
                fs.create_dir_all(parent)?;
            }
        }
        let cache = self.inner.lock().unwrap();
        let content = serde_yaml::to_string(&*cache).map_err(|e| e.to_string())?;
        fs.write(path, &content)?;
        Ok(())
    }
}
