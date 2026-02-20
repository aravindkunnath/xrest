use crate::core::import::curl::curl_to_endpoint;
use crate::core::import::swagger::{create_service_from_spec, parse_spec_content};
use crate::core::service::service::ServiceDomain;
use crate::core::settings::SettingsDomain;
use crate::core::traits::{FileSystem, GitRepository};
use crate::core::types::{Service, ServiceStub};
use std::path::PathBuf;

pub struct ImportDomain<'a> {
    service_domain: ServiceDomain<'a>,
    settings_domain: SettingsDomain<'a>,
}

impl<'a> ImportDomain<'a> {
    pub fn new(fs: &'a dyn FileSystem) -> Self {
        Self {
            service_domain: ServiceDomain::new(fs),
            settings_domain: SettingsDomain::new(fs),
        }
    }

    pub fn import_from_directory(
        &self,
        settings_path: &PathBuf,
        directory: String,
        git: Option<&dyn GitRepository>,
    ) -> Result<Service, String> {
        let mut service = self.service_domain.load_service(&directory)?;
        let mut settings = self.settings_domain.load_settings(settings_path)?;

        if settings.services.iter().any(|s| s.directory == directory) {
            return Err("This directory is already imported as a service.".to_string());
        }

        service.directory = directory.clone();
        settings.services.push(ServiceStub {
            id: service.id.clone(),
            name: service.name.clone(),
            directory,
        });
        self.settings_domain
            .save_settings(settings_path, &settings)?;

        let service_name = service.name.clone();
        self.service_domain.save_service(
            &mut service,
            Some(format!("Import service: {}", service_name)),
            git,
        )?;

        Ok(service)
    }

    pub fn import_from_swagger(
        &self,
        settings_path: &PathBuf,
        name: String,
        directory: String,
        content: &str,
        git: Option<&dyn GitRepository>,
    ) -> Result<Service, String> {
        let (base_url, endpoints) = parse_spec_content(content, "temp")?;
        let mut service = create_service_from_spec(name, directory.clone(), base_url, endpoints);

        for ep in &mut service.endpoints {
            ep.service_id = service.id.clone();
        }

        let service_name = service.name.clone();
        self.service_domain.save_service(
            &mut service,
            Some(format!("Import service from Swagger: {}", service_name)),
            git,
        )?;

        let mut settings = self.settings_domain.load_settings(settings_path)?;
        settings.services.push(ServiceStub {
            id: service.id.clone(),
            name: service.name.clone(),
            directory,
        });
        self.settings_domain
            .save_settings(settings_path, &settings)?;

        Ok(service)
    }

    pub fn import_curl_endpoint(
        &self,
        settings_path: &PathBuf,
        service_id: String,
        curl_command: &str,
        git: Option<&dyn GitRepository>,
    ) -> Result<Service, String> {
        let settings = self.settings_domain.load_settings(settings_path)?;

        let service_stub = settings
            .services
            .iter()
            .find(|s| s.id == service_id)
            .ok_or_else(|| format!("Service not found: {}", service_id))?;

        let mut service = self.service_domain.load_service(&service_stub.directory)?;

        let endpoint = curl_to_endpoint(
            service_id,
            curl_command,
            service.is_authenticated,
            service.auth_type.as_ref().map(|at| at.to_string()),
        )?;

        let endpoint_name = endpoint.name.clone();
        service.endpoints.push(endpoint);
        self.service_domain.save_service(
            &mut service,
            Some(format!("Import endpoint from cURL: {}", endpoint_name)),
            git,
        )?;

        Ok(service)
    }
}
