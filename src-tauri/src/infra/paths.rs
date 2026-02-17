use crate::core::traits::PathProvider;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

pub struct TauriPathProvider {
    config_dir: PathBuf,
    cache_dir: PathBuf,
}

impl TauriPathProvider {
    pub fn new<R: Runtime>(app: &AppHandle<R>) -> Result<Self, String> {
        let config_dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
        let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
        Ok(Self { config_dir, cache_dir })
    }
}

impl PathProvider for TauriPathProvider {
    fn settings_path(&self) -> Result<PathBuf, String> {
        Ok(self.config_dir.join("settings.yaml"))
    }

    fn tab_state_path(&self) -> Result<PathBuf, String> {
        Ok(self.config_dir.join("tabstate.yaml"))
    }

    fn secrets_path(&self) -> Result<PathBuf, String> {
        Ok(self.config_dir.join("secrets.yaml"))
    }

    fn history_db_path(&self) -> Result<PathBuf, String> {
        if !self.config_dir.exists() {
            std::fs::create_dir_all(&self.config_dir).map_err(|e| e.to_string())?;
        }
        Ok(self.config_dir.join("history.db"))
    }

    fn token_cache_path(&self) -> Result<PathBuf, String> {
        Ok(self.cache_dir.join("token_cache.yaml"))
    }

    fn collections_path(&self) -> Result<PathBuf, String> {
        Ok(self.config_dir.join("collections.yaml"))
    }
}
