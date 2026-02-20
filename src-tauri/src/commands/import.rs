use crate::core::import::service::ImportDomain;
use crate::core::traits::{FileSystem, PathProvider};
use crate::core::types::Service;
use crate::infra::fs::RealFileSystem;
use crate::infra::git::Git2Repository;
use crate::infra::paths::TauriPathProvider;
use tauri::AppHandle;

#[tauri::command]
pub fn import_service(app: AppHandle, directory: String) -> Result<Service, String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = ImportDomain::new(&RealFileSystem);
    domain.import_from_directory(&paths.settings_path()?, directory, Some(&Git2Repository))
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

    let paths = TauriPathProvider::new(&app)?;
    let domain = ImportDomain::new(&RealFileSystem);
    domain.import_from_swagger(
        &paths.settings_path()?,
        name,
        directory,
        &content,
        Some(&Git2Repository),
    )
}

#[tauri::command]
pub async fn import_curl(
    app: AppHandle,
    service_id: String,
    curl_command: String,
) -> Result<Service, String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = ImportDomain::new(&RealFileSystem);
    domain.import_curl_endpoint(
        &paths.settings_path()?,
        service_id,
        &curl_command,
        Some(&Git2Repository),
    )
}
