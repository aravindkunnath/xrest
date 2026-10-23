//! Versioned endpoints (`endpoints/<id>.yaml`, history in `.xrest/versions/`).

use super::{now_secs, Workspace};
use crate::error::{Error, Result};
use crate::model::*;
use crate::storage;

impl Workspace {
    pub fn endpoints(&self) -> &[Endpoint] {
        &self.endpoints
    }

    pub fn endpoint(&self, id: &str) -> Result<&Endpoint> {
        self.endpoints
            .iter()
            .find(|e| e.id == id)
            .ok_or_else(|| Error::NotFound {
                what: "endpoint",
                id: id.to_string(),
            })
    }

    fn sort_endpoints(&mut self) {
        self.endpoints.sort_by(|a, b| {
            a.name
                .to_lowercase()
                .cmp(&b.name.to_lowercase())
                .then_with(|| a.id.cmp(&b.id))
        });
    }

    pub fn create_endpoint(&mut self, name: &str, spec: RequestSpec) -> Result<Endpoint> {
        self.ensure_current_format()?;
        let now = now_secs();
        let mut ep = Endpoint {
            id: format!("e-{}", uuid::Uuid::new_v4()),
            service_id: self.service.id.clone(),
            name: name.to_string(),
            method: String::new(),
            url: String::new(),
            authenticated: false,
            auth_type: self.service.effective_auth_kind().as_str().to_string(),
            metadata: EndpointMetadata {
                version: "1.0".into(),
                last_updated: now,
            },
            params: vec![],
            path_params: vec![],
            headers: vec![],
            body: Body::None,
            options: RequestOptions::default(),
            preflight: None,
            last_version: 0,
        };
        ep.apply_spec(spec);
        let v = self.versions.add(
            &ep.id,
            RequestConfig::from_endpoint(&ep),
            now,
            self.max_versions,
        )?;
        ep.last_version = v.version;
        storage::save_endpoint(&self.root, &ep)?;
        self.endpoints.push(ep.clone());
        self.sort_endpoints();
        Ok(ep)
    }

    /// Update an endpoint. A new version snapshot is recorded when the request config changed.
    pub fn update_endpoint(
        &mut self,
        id: &str,
        name: Option<&str>,
        spec: RequestSpec,
    ) -> Result<Endpoint> {
        let mut ep = self.endpoint(id)?.clone();
        let before = RequestConfig::from_endpoint(&ep);
        let renamed = name.is_some_and(|n| n != ep.name);
        if let Some(n) = name {
            ep.name = n.to_string();
        }
        ep.apply_spec(spec);
        let after = RequestConfig::from_endpoint(&ep);
        if before == after && !renamed {
            return Ok(ep);
        }
        self.ensure_current_format()?;
        let now = now_secs();
        ep.metadata.last_updated = now;
        if before != after {
            storage::ensure_gitignore(&self.root)?;
            let v = self.versions.add(id, after, now, self.max_versions)?;
            ep.last_version = v.version;
        }
        storage::save_endpoint(&self.root, &ep)?;
        if let Some(slot) = self.endpoints.iter_mut().find(|e| e.id == id) {
            *slot = ep.clone();
        }
        self.sort_endpoints();
        Ok(ep)
    }

    pub fn delete_endpoint(&mut self, id: &str) -> Result<()> {
        self.endpoint(id)?;
        storage::remove_file_if_exists(&storage::endpoint_path(&self.root, id)?)?;
        self.versions.clear(id)?;
        self.endpoints.retain(|e| e.id != id);
        Ok(())
    }

    /// Version history, newest first.
    pub fn endpoint_versions(&self, id: &str, limit: usize) -> Result<Vec<EndpointVersion>> {
        self.endpoint(id)?;
        self.versions.list(id, limit)
    }
}
