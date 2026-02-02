use crate::io::FileSystem;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretKey {
    pub key: String,
}

pub struct SecretsDomain<'a> {
    fs: &'a dyn FileSystem,
}

impl<'a> SecretsDomain<'a> {
    pub fn new(fs: &'a dyn FileSystem) -> Self {
        Self { fs }
    }

    fn get_secrets_path<R: Runtime>(&self, app: &AppHandle<R>) -> Result<PathBuf, String> {
        let path = app.path().app_config_dir().map_err(|e| e.to_string())?;
        Ok(path.join("secrets.yaml"))
    }

    fn load_keys<R: Runtime>(&self, app: &AppHandle<R>) -> Result<Vec<String>, String> {
        let path = self.get_secrets_path(app)?;
        if !self.fs.exists(&path) {
            return Ok(Vec::new());
        }
        let content = self.fs.read_to_string(&path).map_err(|e| e.to_string())?;
        let keys: Vec<String> = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
        Ok(keys)
    }

    fn save_keys<R: Runtime>(&self, app: &AppHandle<R>, keys: &[String]) -> Result<(), String> {
        let path = self.get_secrets_path(app)?;
        if let Some(parent) = path.parent() {
            if !self.fs.exists(parent) {
                self.fs.create_dir_all(parent)?;
            }
        }
        let content = serde_yaml::to_string(keys).map_err(|e| e.to_string())?;
        self.fs.write(&path, &content)?;
        Ok(())
    }

    pub fn list_secrets<R: Runtime>(&self, app: &AppHandle<R>) -> Result<Vec<String>, String> {
        self.load_keys(app)
    }

    pub fn add_secret<R: Runtime>(
        &self,
        app: &AppHandle<R>,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        entry
            .set_password(value)
            .map_err(|e| format!("Failed to set secret in keyring: {}", e))?;

        let mut keys = self.load_keys(app)?;
        if !keys.contains(&key.to_string()) {
            keys.push(key.to_string());
            self.save_keys(app, &keys)?;
        }

        Ok(())
    }

    pub fn delete_secret<R: Runtime>(&self, app: &AppHandle<R>, key: &str) -> Result<(), String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        let _ = entry.delete_credential();

        let mut keys = self.load_keys(app)?;
        keys.retain(|k| k != key);
        self.save_keys(app, &keys)?;

        Ok(())
    }

    pub fn get_secret(_key: &str) -> Result<String, String> {
        let entry = Entry::new("xrest-secrets", _key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        entry
            .get_password()
            .map_err(|e| format!("Failed to get secret from keyring: {}", e))
    }
}
