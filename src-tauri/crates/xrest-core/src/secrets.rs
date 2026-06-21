use crate::traits::{FileSystem, SecretStore};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretKey {
    pub key: String,
}

pub struct SecretsDomain<'a> {
    fs: &'a dyn FileSystem,
    store: &'a dyn SecretStore,
}

impl<'a> SecretsDomain<'a> {
    pub fn new(fs: &'a dyn FileSystem, store: &'a dyn SecretStore) -> Self {
        Self { fs, store }
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
        self.store.set(key, value)?;

        let mut keys = self.load_keys(secrets_path)?;
        if !keys.contains(&key.to_string()) {
            keys.push(key.to_string());
            self.save_keys(secrets_path, &keys)?;
        }

        Ok(())
    }

    pub fn delete_secret(&self, secrets_path: &Path, key: &str) -> Result<(), String> {
        self.store.delete(key)?;

        let mut keys = self.load_keys(secrets_path)?;
        keys.retain(|k| k != key);
        self.save_keys(secrets_path, &keys)?;

        Ok(())
    }

    pub fn get_secret(&self, key: &str) -> Result<String, String> {
        self.store.get(key)
    }
}
