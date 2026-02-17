use crate::core::traits::FileSystem;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::path::Path;

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

    fn load_keys(&self, secrets_path: &Path) -> Result<Vec<String>, String> {
        if !self.fs.exists(secrets_path) {
            return Ok(Vec::new());
        }
        let content = self.fs.read_to_string(secrets_path).map_err(|e| e.to_string())?;
        let keys: Vec<String> = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
        Ok(keys)
    }

    fn save_keys(&self, secrets_path: &Path, keys: &[String]) -> Result<(), String> {
        if let Some(parent) = secrets_path.parent() {
            if !self.fs.exists(parent) {
                self.fs.create_dir_all(parent)?;
            }
        }
        let content = serde_yaml::to_string(keys).map_err(|e| e.to_string())?;
        self.fs.write(secrets_path, &content)?;
        Ok(())
    }

    pub fn list_secrets(&self, secrets_path: &Path) -> Result<Vec<String>, String> {
        self.load_keys(secrets_path)
    }

    pub fn add_secret(
        &self,
        secrets_path: &Path,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        entry
            .set_password(value)
            .map_err(|e| format!("Failed to set secret in keyring: {}", e))?;

        let mut keys = self.load_keys(secrets_path)?;
        if !keys.contains(&key.to_string()) {
            keys.push(key.to_string());
            self.save_keys(secrets_path, &keys)?;
        }

        Ok(())
    }

    pub fn delete_secret(&self, secrets_path: &Path, key: &str) -> Result<(), String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        let _ = entry.delete_credential();

        let mut keys = self.load_keys(secrets_path)?;
        keys.retain(|k| k != key);
        self.save_keys(secrets_path, &keys)?;

        Ok(())
    }

    pub fn get_secret(key: &str) -> Result<String, String> {
        let entry = Entry::new("xrest-secrets", key)
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        entry
            .get_password()
            .map_err(|e| format!("Failed to get secret from keyring: {}", e))
    }
}
