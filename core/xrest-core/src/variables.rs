//! Variable layers: where a `{{NAME}}` gets its value.
//!
//! Most specific first:
//!
//! | Layer | Stored in | Shared? |
//! | --- | --- | --- |
//! | [`VariableLayer::Local`] | `.xrest/overrides.yaml`, per environment | no (gitignored) |
//! | [`VariableLayer::Environment`] | `environments.yaml` | yes |
//! | [`VariableLayer::Workspace`] | `variables.yaml` (every environment) | yes |
//! | [`VariableLayer::Global`] | the app's global store (every workspace) | no |
//! | [`VariableLayer::BuiltIn`] | computed per send (`{{$uuid}}` …) | — |
//!
//! A disabled or unnamed row does not count, so the next layer down shows through. A row with a
//! `secretKey` resolves to `{{secret.KEY}}`, looked up in the secret store later; secret values
//! never live in these files.

use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::interpolate::Variables;
use crate::model::{Environment, NameValue};

/// One source of variable values. Ordered most specific first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VariableLayer {
    Local,
    Environment,
    Workspace,
    Global,
    BuiltIn,
}

/// The layers that are lists of rows, for one environment. Built-ins are added on resolve.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VariableLayers {
    pub global: Vec<NameValue>,
    pub workspace: Vec<NameValue>,
    /// The user's overrides for the environment being resolved.
    pub local: Vec<NameValue>,
}

/// A built-in dynamic variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinVariable {
    pub name: &'static str,
    pub description: &'static str,
}

/// Every built-in. Each gets one fresh value per send, so the same name repeated in a request
/// (say a header and the body) has the same value.
pub const BUILTIN_VARIABLES: &[BuiltinVariable] = &[
    BuiltinVariable {
        name: "$uuid",
        description: "A random UUID (v4)",
    },
    BuiltinVariable {
        name: "$timestamp",
        description: "Current Unix time in seconds",
    },
    BuiltinVariable {
        name: "$isoTimestamp",
        description: "Current time in UTC, ISO 8601 (2026-10-29T14:03:00Z)",
    },
    BuiltinVariable {
        name: "$randomInt",
        description: "A random integer from 0 to 1000",
    },
];

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_VARIABLES.iter().any(|b| b.name == name)
}

/// Fresh values for every built-in.
pub fn builtin_values() -> Variables {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let random = uuid::Uuid::new_v4();
    let bytes = random.as_bytes();
    let n = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) % 1001;
    let mut vars = Variables::new();
    vars.insert("$uuid".into(), uuid::Uuid::new_v4().to_string());
    vars.insert("$timestamp".into(), now.to_string());
    vars.insert("$isoTimestamp".into(), iso8601_utc(now));
    vars.insert("$randomInt".into(), n.to_string());
    vars
}

/// What a row contributes to the variable map, if anything.
fn row_value(v: &NameValue) -> Option<String> {
    if !v.enabled || v.name.trim().is_empty() {
        return None;
    }
    Some(match v.secret() {
        Some(key) => format!("{{{{secret.{key}}}}}"),
        None => v.value.clone(),
    })
}

impl VariableLayers {
    fn ordered<'a>(
        &'a self,
        env: Option<&'a Environment>,
    ) -> [(VariableLayer, &'a [NameValue]); 4] {
        [
            (VariableLayer::Local, self.local.as_slice()),
            (
                VariableLayer::Environment,
                env.map(|e| e.variables.as_slice()).unwrap_or_default(),
            ),
            (VariableLayer::Workspace, self.workspace.as_slice()),
            (VariableLayer::Global, self.global.as_slice()),
        ]
    }

    /// The variable map for `env` without built-ins (for display).
    pub fn resolve_static(&self, env: Option<&Environment>) -> Variables {
        let mut vars = Variables::new();
        // Least specific first, so more specific layers overwrite.
        for (_, rows) in self.ordered(env).iter().rev() {
            for v in rows.iter() {
                if let Some(value) = row_value(v) {
                    vars.insert(v.name.trim().to_string(), value);
                }
            }
        }
        vars
    }

    /// The variable map for one send in `env`: built-ins, then every layer.
    pub fn resolve(&self, env: Option<&Environment>) -> Variables {
        let mut vars = builtin_values();
        vars.extend(self.resolve_static(env));
        vars
    }

    /// Every enabled variable name in any layer (for autocomplete), most specific layer first,
    /// each once, then the built-ins.
    pub fn names(&self, env: Option<&Environment>) -> Vec<String> {
        let mut out: Vec<String> = vec![];
        for (_, rows) in self.ordered(env) {
            for v in rows.iter().filter(|v| row_value(v).is_some()) {
                let name = v.name.trim().to_string();
                if !out.contains(&name) {
                    out.push(name);
                }
            }
        }
        for b in BUILTIN_VARIABLES {
            if !out.iter().any(|n| n == b.name) {
                out.push(b.name.to_string());
            }
        }
        out
    }

    /// For each name defined in any layer: every layer's row for it, most specific first, and
    /// which layer wins. Names are ordered by first appearance: environment, workspace, global,
    /// then names only overridden locally.
    pub fn report(&self, env: Option<&Environment>) -> Vec<VariableReport> {
        let layers = self.ordered(env);
        let mut names: Vec<String> = vec![];
        for idx in [1, 2, 3, 0] {
            for v in layers[idx].1 {
                let name = v.name.trim();
                if !name.is_empty() && !names.iter().any(|n| n == name) {
                    names.push(name.to_string());
                }
            }
        }
        names
            .into_iter()
            .map(|name| {
                let mut values = vec![];
                for (layer, rows) in &layers {
                    // The last row with the name is the one that counts, as in `resolve`.
                    if let Some(v) = rows.iter().rev().find(|v| v.name.trim() == name) {
                        values.push(LayerValue {
                            layer: *layer,
                            value: v.value.clone(),
                            secret_key: v.secret().map(str::to_string),
                            enabled: v.enabled,
                        });
                    }
                }
                let winner = values.iter().find(|v| v.enabled).map(|v| v.layer);
                VariableReport {
                    name,
                    values,
                    winner,
                }
            })
            .collect()
    }
}

/// One layer's row for a variable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerValue {
    pub layer: VariableLayer,
    /// The stored value; empty for a secret-backed row.
    pub value: String,
    pub secret_key: Option<String>,
    pub enabled: bool,
}

/// Where a variable's value comes from in one environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableReport {
    pub name: String,
    /// Every layer that has a row for the name, most specific first.
    pub values: Vec<LayerValue>,
    /// The layer whose value is used, or `None` if every row is disabled.
    pub winner: Option<VariableLayer>,
}

/// Global variables: one list shared by every open workspace, persisted by the app wherever it
/// keeps user data. Thread-safe; workspaces read a snapshot per send.
#[derive(Debug, Default)]
pub struct GlobalVariables {
    rows: RwLock<Vec<NameValue>>,
}

impl GlobalVariables {
    pub fn new(rows: Vec<NameValue>) -> Self {
        Self {
            rows: RwLock::new(rows),
        }
    }

    pub fn get(&self) -> Vec<NameValue> {
        self.rows.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn set(&self, rows: Vec<NameValue>) {
        *self.rows.write().unwrap_or_else(|e| e.into_inner()) = rows;
    }
}

/// Unix seconds as `YYYY-MM-DDTHH:MM:SSZ` (proleptic Gregorian, UTC).
pub fn iso8601_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Days since 1970-01-01 to (year, month, day). Howard Hinnant's algorithm.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(rows: Vec<NameValue>) -> Environment {
        Environment {
            name: "DEV".into(),
            variables: rows,
            ..Default::default()
        }
    }

    fn off(name: &str, value: &str) -> NameValue {
        NameValue {
            enabled: false,
            ..NameValue::new(name, value)
        }
    }

    fn secret(name: &str, key: &str) -> NameValue {
        NameValue {
            secret_key: Some(key.into()),
            ..NameValue::new(name, "")
        }
    }

    #[test]
    fn most_specific_layer_wins() {
        let layers = VariableLayers {
            global: vec![NameValue::new("A", "g"), NameValue::new("ONLY_G", "g")],
            workspace: vec![NameValue::new("A", "w"), NameValue::new("B", "w")],
            local: vec![NameValue::new("C", "l")],
        };
        let e = env(vec![NameValue::new("B", "e"), NameValue::new("C", "e")]);
        let v = layers.resolve_static(Some(&e));
        assert_eq!(v["A"], "w");
        assert_eq!(v["B"], "e");
        assert_eq!(v["C"], "l");
        assert_eq!(v["ONLY_G"], "g");
        // Without an environment only the other layers apply.
        assert_eq!(layers.resolve_static(None)["B"], "w");
    }

    #[test]
    fn disabled_rows_let_the_next_layer_show_through() {
        let layers = VariableLayers {
            workspace: vec![NameValue::new("A", "w")],
            local: vec![off("A", "l")],
            ..Default::default()
        };
        let e = env(vec![off("A", "e")]);
        assert_eq!(layers.resolve_static(Some(&e))["A"], "w");
        let report = layers.report(Some(&e));
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].winner, Some(VariableLayer::Workspace));
        let order: Vec<_> = report[0].values.iter().map(|v| v.layer).collect();
        assert_eq!(
            order,
            vec![
                VariableLayer::Local,
                VariableLayer::Environment,
                VariableLayer::Workspace
            ]
        );
    }

    #[test]
    fn secrets_become_references_and_report_their_key() {
        let layers = VariableLayers {
            global: vec![secret("TOKEN", "my-token")],
            ..Default::default()
        };
        assert_eq!(layers.resolve_static(None)["TOKEN"], "{{secret.my-token}}");
        let r = &layers.report(None)[0];
        assert_eq!(r.values[0].secret_key.as_deref(), Some("my-token"));
        assert_eq!(r.winner, Some(VariableLayer::Global));
    }

    #[test]
    fn report_orders_names_by_layer_then_local_only() {
        let layers = VariableLayers {
            global: vec![NameValue::new("G", "")],
            workspace: vec![NameValue::new("W", "")],
            local: vec![NameValue::new("L", ""), NameValue::new("E", "")],
        };
        let e = env(vec![NameValue::new("E", "")]);
        let names: Vec<_> = layers
            .report(Some(&e))
            .into_iter()
            .map(|r| r.name)
            .collect();
        assert_eq!(names, vec!["E", "W", "G", "L"]);
    }

    #[test]
    fn names_are_unique_and_end_with_builtins() {
        let layers = VariableLayers {
            workspace: vec![NameValue::new("A", ""), off("HIDDEN", "")],
            local: vec![NameValue::new("A", "")],
            ..Default::default()
        };
        let names = layers.names(None);
        assert_eq!(names[0], "A");
        assert!(!names.contains(&"HIDDEN".to_string()));
        assert_eq!(names.iter().filter(|n| *n == "A").count(), 1);
        assert_eq!(names.last().map(String::as_str), Some("$randomInt"));
    }

    #[test]
    fn builtins_are_filled_and_user_rows_win() {
        let layers = VariableLayers {
            workspace: vec![NameValue::new("$uuid", "fixed")],
            ..Default::default()
        };
        let v = layers.resolve(None);
        assert_eq!(v["$uuid"], "fixed");
        let fresh = VariableLayers::default().resolve(None);
        assert!(uuid::Uuid::parse_str(&fresh["$uuid"]).is_ok());
        assert!(fresh["$timestamp"].parse::<u64>().unwrap() > 1_700_000_000);
        assert!(fresh["$randomInt"].parse::<u32>().unwrap() <= 1000);
        assert!(fresh["$isoTimestamp"].ends_with('Z'));
    }

    #[test]
    fn iso8601_known_values() {
        assert_eq!(iso8601_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(iso8601_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(iso8601_utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }

    #[test]
    fn global_store_is_shared() {
        let g = GlobalVariables::default();
        g.set(vec![NameValue::new("A", "1")]);
        assert_eq!(g.get(), vec![NameValue::new("A", "1")]);
    }
}
