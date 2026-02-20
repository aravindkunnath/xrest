use crate::core::service::service::ServiceDomain;
use crate::core::settings::SettingsDomain;
use crate::core::traits::PathProvider;
use crate::core::types::{Service, UserSettings};
use crate::infra::fs::RealFileSystem;
use crate::infra::git::Git2Repository;
use crate::infra::paths::TauriPathProvider;
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
    let domain = SettingsDomain::new(&RealFileSystem);
    domain.update_theme(&paths.settings_path()?, settings.theme)
}

#[tauri::command]
pub fn get_services(app: AppHandle) -> Result<Vec<Service>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let settings_domain = SettingsDomain::new(&RealFileSystem);
    let service_domain = ServiceDomain::new(&RealFileSystem);
    settings_domain.load_all_services(&paths.settings_path()?, &service_domain)
}

#[tauri::command]
pub fn save_services(
    app: AppHandle,
    mut services: Vec<Service>,
    commit_message: Option<String>,
) -> Result<Vec<Service>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let domain = SettingsDomain::new(&RealFileSystem);
    domain.save_all_services(
        &paths.settings_path()?,
        &mut services,
        commit_message,
        Some(&Git2Repository),
    )?;
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
