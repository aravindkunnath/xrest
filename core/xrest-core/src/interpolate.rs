//! `{{VAR}}` interpolation, ported from the legacy frontend (`resolveVariables`).
//!
//! * `{{ name }}` is replaced by the variable's value; whitespace inside the braces is trimmed.
//! * Unknown names are left untouched (and reported by [`unresolved`]).
//! * Replacement is recursive (variables may reference variables) up to [`MAX_DEPTH`] levels.
//! * `{{secret.KEY}}` placeholders are never touched here; see [`resolve_secrets`].

use std::collections::{BTreeSet, HashMap};

pub const MAX_DEPTH: usize = 10;

pub type Variables = HashMap<String, String>;

/// Find the next `{{ ... }}` at or after `from`. Returns (start, end_exclusive, trimmed name).
/// Mirrors the regex `\{\{(.*?)\}\}` where `.` does not match line breaks.
fn next_placeholder(text: &str, mut from: usize) -> Option<(usize, usize, &str)> {
    while let Some(rel) = text[from..].find("{{") {
        let start = from + rel;
        let inner_start = start + 2;
        let rel_end = text[inner_start..].find("}}")?;
        let inner = &text[inner_start..inner_start + rel_end];
        if !inner.contains(['\n', '\r']) {
            return Some((start, inner_start + rel_end + 2, inner.trim()));
        }
        // A line break inside the braces: the regex would retry from the next character.
        from = start + 1;
    }
    None
}

fn replace_once(text: &str, vars: &Variables) -> String {
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    while let Some((start, end, name)) = next_placeholder(text, cursor) {
        out.push_str(&text[cursor..start]);
        match vars.get(name) {
            Some(v) => out.push_str(v),
            None => out.push_str(&text[start..end]),
        }
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    out
}

/// Interpolate `text` with `vars`.
pub fn interpolate(text: &str, vars: &Variables) -> String {
    let mut current = text.to_string();
    for _ in 0..MAX_DEPTH {
        let replaced = replace_once(&current, vars);
        let changed = replaced != current;
        current = replaced;
        if !(changed && current.contains("{{")) {
            break;
        }
    }
    current
}

/// Names of all `{{name}}` placeholders in `text` (including `secret.*`), in order.
pub fn placeholders(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cursor = 0;
    while let Some((_, end, name)) = next_placeholder(text, cursor) {
        out.push(name.to_string());
        cursor = end;
    }
    out
}

/// Non-secret placeholders in `text` that still have no value (call after [`interpolate`]).
pub fn unresolved(text: &str) -> Vec<String> {
    placeholders(text)
        .into_iter()
        .filter(|n| !n.starts_with("secret."))
        .collect()
}

/// Replace `{{secret.KEY}}` using `lookup`. Returns the new text and the keys that were missing
/// (their placeholders are kept verbatim).
pub fn resolve_secrets(
    text: &str,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> (String, Vec<String>) {
    let mut out = String::with_capacity(text.len());
    let mut missing = BTreeSet::new();
    let mut cursor = 0;
    while let Some((start, end, name)) = next_placeholder(text, cursor) {
        out.push_str(&text[cursor..start]);
        match name.strip_prefix("secret.") {
            Some(key) => match lookup(key) {
                Some(v) => out.push_str(&v),
                None => {
                    missing.insert(key.to_string());
                    out.push_str(&text[start..end]);
                }
            },
            None => out.push_str(&text[start..end]),
        }
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    (out, missing.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, &str)]) -> Variables {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn replaces_and_trims() {
        let v = vars(&[("HOST", "example.com")]);
        assert_eq!(
            interpolate("https://{{ HOST }}/x", &v),
            "https://example.com/x"
        );
        assert_eq!(
            interpolate("{{HOST}}{{HOST}}", &v),
            "example.comexample.com"
        );
    }

    #[test]
    fn unknown_left_verbatim() {
        let v = vars(&[]);
        assert_eq!(interpolate("a{{NOPE}}b", &v), "a{{NOPE}}b");
        assert_eq!(unresolved("a{{NOPE}}b{{ secret.X }}"), vec!["NOPE"]);
    }

    #[test]
    fn recursive() {
        let v = vars(&[("A", "{{B}}/a"), ("B", "{{C}}/b"), ("C", "root")]);
        assert_eq!(interpolate("{{A}}", &v), "root/b/a");
    }

    #[test]
    fn self_reference_terminates() {
        let v = vars(&[("A", "x{{A}}")]);
        let out = interpolate("{{A}}", &v);
        assert!(out.starts_with("xxxxxxxxxx"));
        assert!(out.ends_with("{{A}}"));
    }

    #[test]
    fn newline_inside_braces_is_not_a_placeholder() {
        let v = vars(&[("A", "1")]);
        assert_eq!(interpolate("{{\nA}} {{A}}", &v), "{{\nA}} 1");
    }

    #[test]
    fn unterminated_is_left_alone() {
        let v = vars(&[("A", "1")]);
        assert_eq!(interpolate("{{A", &v), "{{A");
        assert_eq!(interpolate("}}{{A}}", &v), "}}1");
    }

    #[test]
    fn secrets() {
        let lookup = |k: &str| (k == "tok").then(|| "s3cret".to_string());
        let (out, missing) =
            resolve_secrets("Bearer {{secret.tok}} {{secret.nope}} {{X}}", &lookup);
        assert_eq!(out, "Bearer s3cret {{secret.nope}} {{X}}");
        assert_eq!(missing, vec!["nope"]);
    }

    #[test]
    fn utf8_safe() {
        let v = vars(&[("É", "é")]);
        assert_eq!(interpolate("ü{{É}}ß", &v), "üéß");
    }
}
