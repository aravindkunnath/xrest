//! Preflight auth: token extraction and the token cache.
//!
//! Ported from `internal/adapters/preflight.go` and `token_cache.go`. Differences: cache keys are
//! scoped by service and environment (so a DEV token is never replayed against PROD), the
//! configured cache unit (`seconds|minutes|hours|days`) is honored, and the `Bearer ` prefix is only
//! added when the token header is `Authorization`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::model::Preflight;

pub const DEFAULT_TOKEN_TTL: Duration = Duration::from_secs(5 * 60);

struct Cached {
    token: String,
    expires_at: Instant,
}

/// In-memory token cache shared by all sends of a workspace.
#[derive(Default)]
pub struct TokenCache {
    inner: Mutex<HashMap<String, Cached>>,
}

impl TokenCache {
    pub fn get(&self, key: &str) -> Option<String> {
        let mut map = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        match map.get(key) {
            Some(c) if Instant::now() < c.expires_at => Some(c.token.clone()),
            Some(_) => {
                map.remove(key);
                None
            }
            None => None,
        }
    }

    pub fn put(&self, key: impl Into<String>, token: impl Into<String>, ttl: Duration) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).insert(
            key.into(),
            Cached {
                token: token.into(),
                expires_at: Instant::now() + ttl,
            },
        );
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}

/// Walk a dot-separated path (`data.auth.token`) into a JSON value. Empty path is the root.
pub fn value_at_path<'a>(root: &'a serde_json::Value, path: &str) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(root);
    }
    path.split('.')
        .try_fold(root, |cur, part| cur.as_object()?.get(part))
}

/// Extract the token string from a preflight response body.
pub fn extract_token(body: &[u8], token_key: &str) -> Result<String> {
    let json: serde_json::Value = serde_json::from_slice(body)
        .map_err(|e| Error::Preflight(format!("response body is not valid JSON: {e}")))?;
    let key = if token_key.is_empty() {
        "access_token"
    } else {
        token_key
    };
    match value_at_path(&json, key) {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        Some(serde_json::Value::String(_)) => {
            Err(Error::Preflight(format!("token at {key:?} is empty")))
        }
        Some(_) => Err(Error::Preflight(format!(
            "token at {key:?} is not a string"
        ))),
        None => Err(Error::Preflight(format!(
            "token key {key:?} not found in response JSON"
        ))),
    }
}

fn unit_seconds(unit: &str) -> f64 {
    match unit.trim().to_ascii_lowercase().as_str() {
        "minutes" | "minute" | "min" => 60.0,
        "hours" | "hour" | "hrs" => 3600.0,
        "days" | "day" => 86400.0,
        _ => 1.0,
    }
}

fn secs_to_duration(secs: f64) -> Option<Duration> {
    (secs.is_finite() && secs > 0.0).then(|| Duration::from_secs_f64(secs))
}

/// How long to cache the token, or `None` if caching is disabled.
pub fn cache_ttl(cfg: &Preflight, body: &[u8]) -> Option<Duration> {
    if !cfg.cache_token {
        return None;
    }
    let unit = unit_seconds(&cfg.cache_duration_unit);
    // A fixed number wins over "derived".
    if let Ok(n) = cfg.cache_duration.trim().parse::<f64>() {
        if let Some(d) = secs_to_duration(n * unit) {
            return Some(d);
        }
    }
    if !cfg.cache_duration_key.is_empty() {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
            let n = match value_at_path(&json, &cfg.cache_duration_key) {
                Some(serde_json::Value::Number(n)) => n.as_f64(),
                Some(serde_json::Value::String(s)) => s.trim().parse::<f64>().ok(),
                _ => None,
            };
            if let Some(d) = n.and_then(|n| secs_to_duration(n * unit)) {
                return Some(d);
            }
        }
    }
    Some(DEFAULT_TOKEN_TTL)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> Preflight {
        Preflight {
            enabled: true,
            cache_token: true,
            cache_duration: "derived".into(),
            cache_duration_key: "expires_in".into(),
            cache_duration_unit: "seconds".into(),
            ..Default::default()
        }
    }

    #[test]
    fn extracts_nested_token() {
        let body = br#"{"data":{"auth":{"token":"abc"}},"access_token":"top"}"#;
        assert_eq!(extract_token(body, "data.auth.token").unwrap(), "abc");
        assert_eq!(extract_token(body, "").unwrap(), "top");
    }

    #[test]
    fn token_errors() {
        assert!(extract_token(b"not json", "t").is_err());
        assert!(extract_token(br#"{"t":1}"#, "t").is_err());
        assert!(extract_token(br#"{"x":"y"}"#, "t").is_err());
        assert!(extract_token(br#"{"t":""}"#, "t").is_err());
    }

    #[test]
    fn ttl_derived_with_units() {
        let body = br#"{"expires_in": 120}"#;
        assert_eq!(cache_ttl(&cfg(), body), Some(Duration::from_secs(120)));
        let mut c = cfg();
        c.cache_duration_unit = "minutes".into();
        assert_eq!(cache_ttl(&c, body), Some(Duration::from_secs(7200)));
        let body = br#"{"expires_in": "30"}"#;
        assert_eq!(cache_ttl(&cfg(), body), Some(Duration::from_secs(30)));
    }

    #[test]
    fn ttl_fixed_default_and_disabled() {
        let mut c = cfg();
        c.cache_duration = "10".into();
        c.cache_duration_unit = "minutes".into();
        assert_eq!(cache_ttl(&c, b"{}"), Some(Duration::from_secs(600)));
        assert_eq!(cache_ttl(&cfg(), b"{}"), Some(DEFAULT_TOKEN_TTL));
        c.cache_token = false;
        assert_eq!(cache_ttl(&c, b"{}"), None);
    }

    #[test]
    fn cache_expiry() {
        let c = TokenCache::default();
        c.put("k", "t", Duration::from_millis(30));
        assert_eq!(c.get("k").as_deref(), Some("t"));
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(c.get("k"), None);
        c.put("k", "t", Duration::from_secs(60));
        c.clear();
        assert_eq!(c.get("k"), None);
    }
}
