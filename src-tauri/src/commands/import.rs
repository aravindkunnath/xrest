use crate::core::import::curl::curl_to_endpoint;
use crate::core::import::swagger::{create_service_from_spec, parse_spec_content};
use crate::core::service::service::ServiceDomain;
use crate::core::settings::SettingsDomain;
use crate::core::traits::{FileSystem, PathProvider};
use crate::infra::git::Git2Repository;
use crate::core::types::{Service, ServiceStub};
use crate::infra::fs::RealFileSystem;
use crate::infra::paths::TauriPathProvider;
use tauri::AppHandle;

#[tauri::command]
pub fn import_service(app: AppHandle, directory: String) -> Result<Service, String> {
    let paths = TauriPathProvider::new(&app)?;
    let settings_path = paths.settings_path()?;
    let service_domain = ServiceDomain::new(&RealFileSystem);
    let settings_domain = SettingsDomain::new(&RealFileSystem);

    let mut service = service_domain.load_service(&directory)?;
    let mut settings = settings_domain.load_settings(&settings_path)?;

    if settings.services.iter().any(|s| s.directory == directory) {
        return Err("This directory is already imported as a service.".to_string());
    }

    service.directory = directory.clone();
    settings.services.push(ServiceStub {
        id: service.id.clone(),
        name: service.name.clone(),
        directory: directory.clone(),
    });

    let service_name = service.name.clone();
    service_domain.save_service(&mut service, Some(format!("Import service: {}", service_name)), Some(&Git2Repository))?;

    Ok(service)
}

#[tauri::command]
pub async fn import_swagger(
    app: AppHandle,
    name: String,
    directory: String,
    url: Option<String>,
    file: Option<String>,
) -> Result<Service, String> {
    let content = if let Some(u) = url {
        reqwest::get(u)
            .await
            .map_err(|e| format!("Failed to fetch Swagger URL: {}", e))?
            .text()
            .await
            .map_err(|e| format!("Failed to read Swagger response: {}", e))?
    } else if let Some(f) = file {
        RealFileSystem
            .read_to_string(std::path::Path::new(&f))
            .map_err(|e| format!("Failed to read Swagger file: {}", e))?
    } else {
        return Err("No Swagger source provided".to_string());
    };

    let (base_url, endpoints) = parse_spec_content(&content, "temp")?;
    let mut service = create_service_from_spec(name, directory.clone(), base_url, endpoints);

    // Re-assign endpoint service_ids to match the generated service id
    for ep in &mut service.endpoints {
        ep.service_id = service.id.clone();
    }

    let paths = TauriPathProvider::new(&app)?;
    let settings_path = paths.settings_path()?;
    let service_domain = ServiceDomain::new(&RealFileSystem);
    let settings_domain = SettingsDomain::new(&RealFileSystem);

    let service_name = service.name.clone();
    service_domain.save_service(
        &mut service,
        Some(format!("Import service from Swagger: {}", service_name)),
        Some(&Git2Repository),
    )?;

    let mut settings = settings_domain.load_settings(&settings_path)?;
    settings.services.push(ServiceStub {
        id: service.id.clone(),
        name: service.name.clone(),
        directory: directory.clone(),
    });
    settings_domain.save_settings(&settings_path, &settings)?;

    Ok(service)
}

#[tauri::command]
pub async fn import_curl(
    app: AppHandle,
    service_id: String,
    curl_command: String,
) -> Result<Service, String> {
    let paths = TauriPathProvider::new(&app)?;
    let settings_path = paths.settings_path()?;
    let settings_domain = SettingsDomain::new(&RealFileSystem);
    let settings = settings_domain.load_settings(&settings_path)?;

    let service_stub = settings
        .services
        .iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| format!("Service not found: {}", service_id))?;

    let service_domain = ServiceDomain::new(&RealFileSystem);
    let mut service = service_domain.load_service(&service_stub.directory)?;

    let endpoint = curl_to_endpoint(
        service_id.clone(),
        &curl_command,
        service.is_authenticated,
        service.auth_type.as_ref().map(|at| at.to_string()),
    )?;

    let endpoint_name = endpoint.name.clone();
    service.endpoints.push(endpoint);
    service_domain.save_service(
        &mut service,
        Some(format!("Import endpoint from cURL: {}", endpoint_name)),
        Some(&Git2Repository),
    )?;

    Ok(service)
}

