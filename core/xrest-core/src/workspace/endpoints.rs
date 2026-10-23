//! Versioned endpoints (`endpoints/<id>.yaml`, history in `.xrest/versions/`).

use super::{now_secs, Workspace};
use crate::diff::{diff_configs, FieldChange};
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
        let ep = self.create_endpoint_quiet(name, spec)?;
        self.auto_commit_endpoint(&ep.id, &format!("Create endpoint: {}", ep.name));
        Ok(ep)
    }

    /// [`Self::create_endpoint`] without the auto-commit, for batch operations that commit once.
    pub(super) fn create_endpoint_quiet(
        &mut self,
        name: &str,
        spec: RequestSpec,
    ) -> Result<Endpoint> {
        normalize_method(&spec.method)?;
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
        normalize_method(&spec.method)?;
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
        self.auto_commit_endpoint(id, &format!("Update endpoint: {}", ep.name));
        Ok(ep)
    }

    pub fn delete_endpoint(&mut self, id: &str) -> Result<()> {
        let name = self.endpoint(id)?.name.clone();
        storage::remove_file_if_exists(&storage::endpoint_path(&self.root, id)?)?;
        self.versions.clear(id)?;
        self.endpoints.retain(|e| e.id != id);
        self.auto_commit_endpoint(id, &format!("Delete endpoint: {name}"));
        Ok(())
    }

    /// Version history, newest first.
    pub fn endpoint_versions(&self, id: &str, limit: usize) -> Result<Vec<EndpointVersion>> {
        self.endpoint(id)?;
        self.versions.list(id, limit)
    }

    fn version_config(&self, id: &str, version: i32) -> Result<RequestConfig> {
        self.versions
            .list(id, usize::MAX)?
            .into_iter()
            .find(|v| v.version == version)
            .map(|v| v.config)
            .ok_or_else(|| Error::NotFound {
                what: "endpoint version",
                id: format!("{id}@{version}"),
            })
    }

    /// Make a past version's request the current one. Goes through [`Self::update_endpoint`], so
    /// it is recorded as a new version (`MAX+1`); history is never rewritten. Restoring the
    /// config that is already current changes nothing.
    pub fn restore_endpoint_version(&mut self, id: &str, version: i32) -> Result<Endpoint> {
        self.endpoint(id)?;
        let config = self.version_config(id, version)?;
        self.update_endpoint(id, None, config.spec())
    }

    /// Changes going from version `from` to version `to`.
    pub fn diff_versions(&self, id: &str, from: i32, to: i32) -> Result<Vec<FieldChange>> {
        self.endpoint(id)?;
        let a = self.version_config(id, from)?;
        let b = self.version_config(id, to)?;
        Ok(diff_configs(&a, &b))
    }
}
