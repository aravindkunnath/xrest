use crate::core::service::service::{ServiceDomain, ServiceStub};
use crate::core::traits::{FileSystem, GitRepository};
use crate::core::types::Service;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub theme: String, // "light", "dark", "system"
    #[serde(default)]
    pub services: Vec<ServiceStub>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            services: Vec::new(),
        }
    }
}

pub struct SettingsDomain<'a> {
    fs: &'a dyn FileSystem,
}

impl<'a> SettingsDomain<'a> {
    pub fn new(fs: &'a dyn FileSystem) -> Self {
        Self { fs }
    }

    pub fn load_settings(&self, path: &PathBuf) -> Result<UserSettings, String> {
        if !self.fs.exists(path) {
            let settings = UserSettings::default();
            self.save_settings(path, &settings)?;
            return Ok(settings);
        }
        let content = self.fs.read_to_string(path).map_err(|e| e.to_string())?;
        match serde_yaml::from_str::<UserSettings>(&content) {
            Ok(settings) => Ok(settings),
            Err(e) => {
                println!(
                    "Failed to parse settings.yaml: {}. Falling back to default.",
                    e
                );
                Ok(UserSettings::default())
            }
        }
    }

    pub fn save_settings(&self, path: &PathBuf, settings: &UserSettings) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !self.fs.exists(parent) {
                self.fs.create_dir_all(parent)?;
            }
        }
        let content = serde_yaml::to_string(settings).map_err(|e| e.to_string())?;
        self.fs.write(path, &content)?;
        Ok(())
    }

    pub fn load_tab_state(&self, path: &PathBuf) -> Result<Option<crate::core::types::TabState>, String> {
        if !self.fs.exists(path) {
            return Ok(None);
        }
        let content = self.fs.read_to_string(path).map_err(|e| e.to_string())?;
        let state: crate::core::types::TabState =
            serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
        Ok(Some(state))
    }

    pub fn save_tab_state(
        &self,
        path: &PathBuf,
        state: &crate::core::types::TabState,
    ) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            if !self.fs.exists(parent) {
                self.fs.create_dir_all(parent)?;
            }
        }
        let content = serde_yaml::to_string(state).map_err(|e| e.to_string())?;
        self.fs.write(path, &content)?;
        Ok(())
    }

    pub fn update_theme(&self, path: &PathBuf, theme: String) -> Result<(), String> {
        let mut settings = self.load_settings(path)?;
        settings.theme = theme;
        self.save_settings(path, &settings)
    }

    pub fn load_all_services(
        &self,
        settings_path: &PathBuf,
        service_domain: &ServiceDomain,
    ) -> Result<Vec<Service>, String> {
        let settings = self.load_settings(settings_path)?;
        let mut services = Vec::new();
        let mut errors = Vec::new();

        for stub in settings.services {
            match service_domain.load_service(&stub.directory) {
                Ok(service) => services.push(service),
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

    pub fn save_all_services(
        &self,
        settings_path: &PathBuf,
        services: &mut Vec<Service>,
        commit_message: Option<String>,
        git: Option<&dyn GitRepository>,
    ) -> Result<(), String> {
        let mut settings = self.load_settings(settings_path)?;
        let mut stubs = Vec::new();

        for service in services.iter_mut() {
            let service_domain = ServiceDomain::new(self.fs);
            service_domain.save_service(service, commit_message.clone(), git)?;
            stubs.push(ServiceStub {
                id: service.id.clone(),
                name: service.name.clone(),
                directory: service.directory.clone(),
            });
        }

        settings.services = stubs;
        self.save_settings(settings_path, &settings)
    }
}
