use crate::core::types::QResponse;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String, String>;
    fn write(&self, path: &Path, content: &str) -> Result<(), String>;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> Result<(), String>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, String>;
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait HttpClient: Send + Sync {
    async fn send_request(
        &self,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<String>,
        query: Vec<(String, String)>,
    ) -> Result<QResponse, String>;
}

pub trait PathProvider: Send + Sync {
    fn settings_path(&self) -> Result<PathBuf, String>;
    fn tab_state_path(&self) -> Result<PathBuf, String>;
    fn secrets_path(&self) -> Result<PathBuf, String>;
    fn history_db_path(&self) -> Result<PathBuf, String>;
    fn token_cache_path(&self) -> Result<PathBuf, String>;
    fn collections_path(&self) -> Result<PathBuf, String>;
}
