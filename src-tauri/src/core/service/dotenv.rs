use crate::core::traits::FileSystem;
use std::collections::HashMap;
use std::path::PathBuf;

/// Parses .env file content into a HashMap.
/// - Skips blank lines and `#` comment lines
/// - Strips optional `export ` prefix
/// - Splits on first `=` only (value may contain `=`)
/// - Strips surrounding single/double quotes from value
/// - Strips unquoted inline comments (` #` or `\t#`)
pub fn parse_dotenv(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip blank lines and comment lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Strip optional `export ` prefix
        let line_without_export = trimmed
            .strip_prefix("export ")
            .unwrap_or(trimmed)
            .trim_start();

        // Split on first `=` only
        let Some(eq_pos) = line_without_export.find('=') else {
            continue;
        };

        let key = line_without_export[..eq_pos].trim();
        if key.is_empty() {
            continue;
        }

        let raw_value = &line_without_export[eq_pos + 1..];

        let value = parse_value(raw_value);

        map.insert(key.to_string(), value);
    }

    map
}

fn parse_value(raw: &str) -> String {
    // Check for surrounding double quotes
    if raw.starts_with('"') && raw.ends_with('"') && raw.len() >= 2 {
        // Strip quotes, preserve everything inside (including #)
        return raw[1..raw.len() - 1].to_string();
    }

    // Check for surrounding single quotes
    if raw.starts_with('\'') && raw.ends_with('\'') && raw.len() >= 2 {
        // Strip quotes, preserve everything inside (including #)
        return raw[1..raw.len() - 1].to_string();
    }

    // Unquoted value: strip inline comments (` #` or `\t#`)
    let value = strip_inline_comment(raw);
    value.trim().to_string()
}

fn strip_inline_comment(s: &str) -> &str {
    // Look for ` #` or `\t#` as inline comment delimiter
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if (bytes[i] == b' ' || bytes[i] == b'\t') && i + 1 < len && bytes[i + 1] == b'#' {
            return &s[..i];
        }
        i += 1;
    }
    s
}

/// Loads and parses `{env_name.to_lowercase()}.env` from `service_dir`.
/// Returns Ok(empty map) if file does not exist — graceful degradation.
pub fn load_dotenv_vars(
    service_dir: &str,
    env_name: &str,
    fs: &dyn FileSystem,
) -> Result<HashMap<String, String>, String> {
    let filename = format!("{}.env", env_name.to_lowercase());
    let path = PathBuf::from(service_dir).join(&filename);

    if !fs.exists(&path) {
        return Ok(HashMap::new());
    }

    let content = fs.read_to_string(&path)?;
    Ok(parse_dotenv(&content))
}
