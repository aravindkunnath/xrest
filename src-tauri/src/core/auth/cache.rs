use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Represents a cached token with expiration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedToken {
    pub token: String,
    pub expires_at: u64, // Unix timestamp in seconds
}

/// Global token cache: key -> CachedToken
pub type TokenCacheInner = HashMap<String, CachedToken>;
pub static TOKEN_CACHE: Lazy<Arc<Mutex<TokenCacheInner>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Get a cached token, returns None if not found or expired
pub fn get_cached_token(key: &str) -> Option<CachedToken> {
    let cache = TOKEN_CACHE.lock().unwrap();
    cache.get(key).cloned()
}

/// Set a cached token with expiration time
pub fn set_cached_token(key: String, token: String, expires_at: u64) {
    let mut cache = TOKEN_CACHE.lock().unwrap();
    cache.insert(key, CachedToken { token, expires_at });
}

/// Helper to generate a unique cache key based on service ID and preflight details
pub fn generate_key(
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

/// Check if a cached token is still valid (not expired)
pub fn is_token_valid(cached: &CachedToken) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now < cached.expires_at
}

/// Persistence: Save the cache to a file
pub fn save_cache_to_file(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
    }
    let cache = TOKEN_CACHE.lock().unwrap();
    let content = serde_yaml::to_string(&*cache).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

/// Persistence: Load the cache from a file
pub fn load_cache_from_file(path: &std::path::Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let loaded: TokenCacheInner = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;

    let mut cache = TOKEN_CACHE.lock().unwrap();
    // Clean up expired tokens while loading
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

/// Clear all cached tokens, optionally for a specific key
#[allow(dead_code)]
pub fn clear_token_cache(key: Option<String>) {
    let mut cache = TOKEN_CACHE.lock().unwrap();
    if let Some(k) = key {
        cache.remove(&k);
    } else {
        cache.clear();
    }
}
