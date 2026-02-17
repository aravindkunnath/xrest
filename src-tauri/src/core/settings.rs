use crate::core::service::service::ServiceStub;
use crate::core::traits::FileSystem;
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
}
