//! OpenAPI import into workspaces.
//!
//! Imports save every endpoint without per-endpoint auto-commits, then make one commit
//! (`Import OpenAPI: N endpoints`) in auto mode.

use std::path::Path;

use super::Workspace;
use crate::error::Result;
use crate::model::*;
use crate::storage;
use crate::swagger::{self, ImportSummary, ImportedEndpoint, PLACEHOLDER_BASE_URL};

/// The part of an endpoint URL that identifies the operation: after `{{BASE_URL}}`, without a
/// query string.
fn match_path(url: &str) -> &str {
    let u = url.strip_prefix("{{BASE_URL}}").unwrap_or(url);
    u.split('?').next().unwrap_or(u)
}

impl Workspace {
    /// Create a workspace in `root` from an OpenAPI/Swagger document: DEV/STAGE/PROD (PROD
    /// unsafe) each with `BASE_URL`, and one endpoint per operation at version 1. The content is
    /// validated before anything is created. A spec without servers gets
    /// `https://api.example.com` and a workspace warning. An empty `name` falls back to the
    /// spec's title.
    pub fn create_from_openapi(root: impl AsRef<Path>, name: &str, content: &str) -> Result<Self> {
        let spec = swagger::parse(content)?;
        let name = [name.trim(), spec.title.as_str()]
            .into_iter()
            .find(|n| !n.is_empty())
            .unwrap_or("Imported API");
        let mut ws = Self::create(root, name)?;
        let (base, placeholder) = match &spec.base_url {
            Some(b) => (b.clone(), false),
            None => (PLACEHOLDER_BASE_URL.to_string(), true),
        };
        let envs = ws
            .environments
            .iter()
            .map(|e| Environment {
                variables: vec![NameValue::new("BASE_URL", base.clone())],
                ..e.clone()
            })
            .collect();
        ws.save_environments(envs)?;
        if placeholder {
            ws.warnings.push(format!(
                "The API description declares no server; BASE_URL is the placeholder {PLACEHOLDER_BASE_URL}. Set the real URL in the environments."
            ));
        }
        ws.add_imported(spec.endpoints)?;
        Ok(ws)
    }

    /// Add the operations of `content` that the workspace does not have yet. Operations are
    /// matched by method + path (URL after `{{BASE_URL}}`); existing endpoints are never
    /// modified. In auto mode all additions are one commit.
    pub fn merge_openapi(&mut self, content: &str) -> Result<ImportSummary> {
        let spec = swagger::parse(content)?;
        let mut seen: std::collections::HashSet<(String, String)> = self
            .endpoints
            .iter()
            .map(|e| (e.method.to_uppercase(), match_path(&e.url).to_string()))
            .collect();
        let (new, old): (Vec<_>, Vec<_>) = spec
            .endpoints
            .into_iter()
            .partition(|e| seen.insert((e.method.to_uppercase(), e.path.clone())));
        let added = self.add_imported(new)?;
        Ok(ImportSummary {
            added,
            skipped: old.len() as u32,
        })
    }

    fn add_imported(&mut self, eps: Vec<ImportedEndpoint>) -> Result<u32> {
        let mut n = 0u32;
        for e in eps {
            self.create_endpoint_quiet(&e.name, e.spec)?;
            n += 1;
        }
        if n > 0 {
            let msg = format!(
                "Import OpenAPI: {n} endpoint{}",
                if n == 1 { "" } else { "s" }
            );
            self.auto_commit(&[storage::ENDPOINTS_DIR], &msg);
        }
        Ok(n)
    }
}
