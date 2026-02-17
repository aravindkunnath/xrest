use crate::core::service::service::ServiceDomain;
use crate::core::settings::SettingsDomain;
use crate::core::types::{Service, ServiceStub, UserSettings};
use crate::infra::fs::RealFileSystem;
use crate::infra::git::Git2Repository;
use crate::infra::paths::TauriPathProvider;
use crate::core::traits::PathProvider;
use tauri::AppHandle;

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Result<UserSettings, String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = SettingsDomain::new(&RealFileSystem);
    domain.load_settings(&paths.settings_path()?)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: UserSettings) -> Result<(), String> {
    let paths = TauriPathProvider::new(&app)?;
    let path = paths.settings_path()?;
    let domain = SettingsDomain::new(&RealFileSystem);
    // Load existing settings first to preserve other fields (like services)
    let mut current_settings = domain.load_settings(&path)?;
    current_settings.theme = settings.theme;
    domain.save_settings(&path, &current_settings)
}

#[tauri::command]
pub fn get_services(app: AppHandle) -> Result<Vec<Service>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let settings_domain = SettingsDomain::new(&RealFileSystem);
    let settings = settings_domain.load_settings(&paths.settings_path()?)?;

    let service_domain = ServiceDomain::new(&RealFileSystem);
    let mut services = Vec::new();
    let mut errors = Vec::new();

    for stub in settings.services {
        match service_domain.load_service(&stub.directory) {
            Ok(service) => {
                services.push(service);
            }
            Err(e) => {
                let err_msg = format!("Failed to load service {}: {}", stub.name, e);
                println!("{}", err_msg);
                errors.push(err_msg);
            }
        }
    }

    if !errors.is_empty() && services.is_empty() {
        return Err(errors.join("\n"));
    }

    Ok(services)
}

#[tauri::command]
pub fn save_services(
    app: AppHandle,
    mut services: Vec<Service>,
    commit_message: Option<String>,
) -> Result<Vec<Service>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let settings_path = paths.settings_path()?;
    let settings_domain = SettingsDomain::new(&RealFileSystem);
    let mut settings = settings_domain.load_settings(&settings_path)?;
    let service_domain = ServiceDomain::new(&RealFileSystem);
    let mut stubs = Vec::new();

    for service in &mut services {
        service_domain.save_service(service, commit_message.clone(), Some(&Git2Repository))?;
        stubs.push(ServiceStub {
            id: service.id.clone(),
            name: service.name.clone(),
            directory: service.directory.clone(),
        });
    }

    settings.services = stubs;
    settings_domain.save_settings(&settings_path, &settings)?;
    Ok(services)
}

#[tauri::command]
pub fn get_collections(app: AppHandle) -> Result<Vec<Service>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = ServiceDomain::new(&RealFileSystem);
    domain.load_collections(&paths.collections_path()?)
}

#[tauri::command]
pub fn save_collections(app: AppHandle, collections: Vec<Service>) -> Result<Vec<Service>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = ServiceDomain::new(&RealFileSystem);
    domain.save_collections(&paths.collections_path()?, &collections)?;
    Ok(collections)
}

#[tauri::command]
pub fn get_tab_state(app: AppHandle) -> Result<Option<crate::core::types::TabState>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = SettingsDomain::new(&RealFileSystem);
    domain.load_tab_state(&paths.tab_state_path()?)
}

#[tauri::command]
pub fn save_tab_state(app: AppHandle, state: crate::core::types::TabState) -> Result<(), String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = SettingsDomain::new(&RealFileSystem);
    domain.save_tab_state(&paths.tab_state_path()?, &state)
}
