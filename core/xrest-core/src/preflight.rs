//! Preflight auth: token extraction and the token cache.
//!
//! Ported from `internal/adapters/preflight.go` and `token_cache.go`. Differences: cache keys are
//! scoped by service and environment (so a DEV token is never replayed against PROD), the
//! configured cache unit (`seconds|minutes|hours|days`) is honored, and the `Bearer ` prefix is only
//! added when the token header is `Authorization`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

/// Result of testing a preflight request without using the token cache.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreflightTestResult {
    pub method: String,
    pub url: String,
    pub request_headers: Vec<(String, String)>,
    pub request_body: String,
    pub status: Option<u16>,
    pub response_headers: Vec<(String, String)>,
    pub response_body: String,
    pub token: Option<String>,
    pub error: Option<String>,
    pub time_ms: u64,
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

/// Longest a token is ever cached, whatever the server claims.
const MAX_TOKEN_TTL: Duration = Duration::from_secs(30 * 86_400);

/// A positive, finite number of seconds as a duration, clamped to [`MAX_TOKEN_TTL`].
fn secs_to_duration(secs: f64) -> Option<Duration> {
    (secs.is_finite() && secs > 0.0)
        .then(|| Duration::from_secs_f64(secs.min(MAX_TOKEN_TTL.as_secs_f64())))
}

fn json_number(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

/// TTL from the value at `cacheDurationKey`: a relative lifetime in `unit_secs`, or, for the
/// `epoch`, `epoch_ms` and `iso8601` units, an absolute expiry time. A past or unparsable expiry
/// yields `None` (the caller falls back to the default TTL).
fn derived_ttl(unit: &str, value: Option<&serde_json::Value>, unit_secs: f64) -> Option<Duration> {
    let value = value?;
    let expires_at = match unit.trim().to_ascii_lowercase().as_str() {
        "epoch" => json_number(value),
        "epoch_ms" => json_number(value).map(|ms| ms / 1000.0),
        "iso8601" | "rfc3339" => value.as_str().and_then(parse_rfc3339),
        _ => return secs_to_duration(json_number(value)? * unit_secs),
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs_f64();
    secs_to_duration(expires_at? - now)
}

/// Parse an RFC 3339 timestamp (`2030-01-01T00:00:00[.fff]Z|+hh:mm`) to Unix seconds.
pub(crate) fn parse_rfc3339(s: &str) -> Option<f64> {
    let s = s.trim();
    let (date, rest) = s.split_once(['T', 't'])?;
    let mut d = date.split('-');
    let (y, m, day): (i64, i64, i64) = (
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
        d.next()?.parse().ok()?,
    );
    if d.next().is_some()
        || !(1970..=9999).contains(&y)
        || !(1..=12).contains(&m)
        || !(1..=31).contains(&day)
    {
        return None;
    }
    let tz_at = rest.find(['Z', 'z', '+', '-'])?;
    let (time, tz) = rest.split_at(tz_at);
    let mut t = time.split(':');
    let (hh, mm): (i64, i64) = (t.next()?.parse().ok()?, t.next()?.parse().ok()?);
    let ss: f64 = t.next()?.parse().ok()?;
    if t.next().is_some() || hh > 23 || mm > 59 || !(0.0..61.0).contains(&ss) {
        return None;
    }
    let offset = match tz {
        "Z" | "z" => 0,
        _ => {
            let sign = if tz.starts_with('-') { -1 } else { 1 };
            let (oh, om) = tz[1..].split_once(':')?;
            let (oh, om) = (oh.parse::<i64>().ok()?, om.parse::<i64>().ok()?);
            if !(0..=23).contains(&oh) || !(0..=59).contains(&om) {
                return None;
            }
            sign * (oh * 3600 + om * 60)
        }
    };
    // days-from-civil (Howard Hinnant)
    let y = y - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some((days * 86_400 + hh * 3600 + mm * 60 - offset) as f64 + ss)
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
            let value = value_at_path(&json, &cfg.cache_duration_key);
            if let Some(d) = derived_ttl(&cfg.cache_duration_unit, value, unit) {
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
    fn hostile_expiry_values_never_panic_and_clamp_to_30_days() {
        let max = Duration::from_secs(30 * 86_400);
        let cases: [(&str, &str); 9] = [
            ("epoch", r#"{"expires_in": 1e30}"#),
            ("epoch_ms", r#"{"expires_in": 1e30}"#),
            ("seconds", r#"{"expires_in": 1e30}"#),
            ("days", r#"{"expires_in": 1e300}"#),
            ("iso8601", r#"{"expires_in": "99999999-01-01T00:00:00Z"}"#),
            ("iso8601", r#"{"expires_in": "9999-12-31T23:59:59Z"}"#),
            ("epoch", r#"{"expires_in": "NaN"}"#),
            ("epoch", r#"{"expires_in": "inf"}"#),
            ("seconds", r#"{"expires_in": "-inf"}"#),
        ];
        for (unit, body) in cases {
            let mut c = cfg();
            c.cache_duration_unit = unit.into();
            let ttl = cache_ttl(&c, body.as_bytes());
            assert!(ttl.is_some_and(|t| t <= max), "{unit} {body}: {ttl:?}");
        }
        let mut c = cfg();
        c.cache_duration = "1e30".into();
        assert_eq!(cache_ttl(&c, b"{}"), Some(max));
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

    /// Civil-from-days inverse of the parser's days-from-civil, for building test input.
    fn format_rfc3339(secs: u64) -> String {
        let days = (secs / 86400) as i64;
        let rem = secs % 86400;
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + i64::from(m <= 2);
        format!(
            "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
            rem / 3600,
            rem % 3600 / 60,
            rem % 60
        )
    }

    fn epoch_cfg(unit: &str) -> Preflight {
        let mut c = cfg();
        c.cache_duration_unit = unit.into();
        c.cache_duration_key = "exp".into();
        c
    }

    fn now_secs() -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64()
    }

    fn near(d: Option<Duration>, secs: u64) -> bool {
        d.is_some_and(|d| d.as_secs_f64() > secs as f64 - 5.0 && d.as_secs_f64() <= secs as f64)
    }

    #[test]
    fn ttl_absolute_epoch_units() {
        let at = now_secs() as u64 + 600;
        let body = format!(r#"{{"exp": {at}}}"#);
        assert!(near(cache_ttl(&epoch_cfg("epoch"), body.as_bytes()), 600));
        let body = format!(r#"{{"exp": "{at}"}}"#);
        assert!(near(cache_ttl(&epoch_cfg("epoch"), body.as_bytes()), 600));
        let body = format!(r#"{{"exp": {}}}"#, at * 1000);
        assert!(near(
            cache_ttl(&epoch_cfg("epoch_ms"), body.as_bytes()),
            600
        ));
        let body = format!(r#"{{"exp": "{}"}}"#, at * 1000);
        assert!(near(
            cache_ttl(&epoch_cfg("EPOCH_MS"), body.as_bytes()),
            600
        ));
    }

    #[test]
    fn ttl_absolute_iso8601() {
        let at = now_secs() as u64 + 600;
        let iso = format_rfc3339(at);
        let body = format!(r#"{{"exp": "{iso}"}}"#);
        assert!(near(cache_ttl(&epoch_cfg("iso8601"), body.as_bytes()), 600));
        // offsets and fractional seconds
        assert_eq!(
            parse_rfc3339("2030-01-01T01:00:00.500+01:00"),
            Some(1_893_456_000.5)
        );
        assert_eq!(parse_rfc3339("1970-01-01T00:00:10Z"), Some(10.0));
        assert_eq!(parse_rfc3339("not a date"), None);
    }

    #[test]
    fn ttl_absolute_past_or_garbage_falls_back_to_default() {
        for (unit, body) in [
            ("epoch", r#"{"exp": 1000}"#),
            ("epoch_ms", r#"{"exp": 1000000}"#),
            ("iso8601", r#"{"exp": "2001-01-01T00:00:00Z"}"#),
            ("epoch", r#"{"exp": "soon"}"#),
            ("epoch_ms", r#"{"exp": null}"#),
            ("iso8601", r#"{"exp": "garbage"}"#),
            ("iso8601", r#"{"exp": 12}"#),
            ("epoch", r#"{}"#),
        ] {
            assert_eq!(
                cache_ttl(&epoch_cfg(unit), body.as_bytes()),
                Some(DEFAULT_TOKEN_TTL),
                "{unit} {body}"
            );
        }
    }
}
