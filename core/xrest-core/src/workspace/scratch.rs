//! The scratchpad (`scratchpad/<id>.yaml`, gitignored), promotion and cURL import.

use super::{now_secs, Workspace};
use crate::curl;
use crate::error::{Error, Result};
use crate::model::*;
use crate::storage;

impl Workspace {
    pub fn scratch_requests(&self) -> &[ScratchRequest] {
        &self.scratch
    }

    pub fn scratch(&self, id: &str) -> Result<&ScratchRequest> {
        self.scratch
            .iter()
            .find(|s| s.id == id)
            .ok_or_else(|| Error::NotFound {
                what: "scratch request",
                id: id.to_string(),
            })
    }

    fn sort_scratch(&mut self) {
        self.scratch.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    pub fn create_scratch(&mut self, name: &str, spec: RequestSpec) -> Result<ScratchRequest> {
        normalize_method(&spec.method)?;
        self.ensure_current_format()?;
        let now = now_secs();
        let s = ScratchRequest {
            id: format!("r-{}", uuid::Uuid::new_v4()),
            name: name.to_string(),
            spec,
            created_at: now,
            updated_at: now,
        };
        storage::save_scratch(&self.root, &s)?;
        self.scratch.push(s.clone());
        self.sort_scratch();
        Ok(s)
    }

    pub fn update_scratch(
        &mut self,
        id: &str,
        name: Option<&str>,
        spec: RequestSpec,
    ) -> Result<ScratchRequest> {
        normalize_method(&spec.method)?;
        let mut s = self.scratch(id)?.clone();
        let unchanged = s.spec == spec && name.is_none_or(|n| n == s.name);
        if unchanged {
            return Ok(s);
        }
        self.ensure_current_format()?;
        if let Some(n) = name {
            s.name = n.to_string();
        }
        s.spec = spec;
        s.updated_at = now_secs();
        storage::save_scratch(&self.root, &s)?;
        if let Some(slot) = self.scratch.iter_mut().find(|x| x.id == id) {
            *slot = s.clone();
        }
        self.sort_scratch();
        Ok(s)
    }

    pub fn delete_scratch(&mut self, id: &str) -> Result<()> {
        self.scratch(id)?;
        storage::remove_file_if_exists(&storage::scratch_path(&self.root, id)?)?;
        self.scratch.retain(|s| s.id != id);
        Ok(())
    }

    /// Turn a scratch request into a versioned endpoint (version 1) and remove the scratch file.
    /// `name` overrides the scratch request's name.
    pub fn promote_scratch(&mut self, id: &str, name: Option<&str>) -> Result<Endpoint> {
        let s = self.scratch(id)?.clone();
        let name = name
            .filter(|n| !n.trim().is_empty())
            .map(str::to_string)
            .or_else(|| (!s.name.trim().is_empty()).then(|| s.name.clone()))
            .unwrap_or_else(|| curl::endpoint_name(&s.spec.url));
        let ep = self.create_endpoint(&name, s.spec)?;
        // The endpoint exists on disk now; only then drop the scratch file.
        self.delete_scratch(id)?;
        Ok(ep)
    }

    fn spec_from_curl(&self, parsed: curl::ParsedCurl) -> RequestSpec {
        RequestSpec {
            method: parsed.method,
            url: parsed.url,
            authenticated: self.service.is_authenticated,
            params: parsed.params,
            path_params: vec![],
            headers: parsed.headers,
            body: parsed.body,
            options: parsed.options,
            preflight: None,
        }
    }

    pub fn import_curl_as_scratch(&mut self, command: &str) -> Result<ScratchRequest> {
        let parsed = curl::parse(command)?;
        let name = parsed.name.clone();
        let spec = self.spec_from_curl(parsed);
        self.create_scratch(&name, spec)
    }

    pub fn import_curl_as_endpoint(&mut self, command: &str) -> Result<Endpoint> {
        let parsed = curl::parse(command)?;
        let name = parsed.name.clone();
        let spec = self.spec_from_curl(parsed);
        self.create_endpoint(&name, spec)
    }
}
