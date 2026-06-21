use crate::types::{GitStatus, HistoryEntry, QResponse};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

#[async_trait]
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
pub trait FileSystem: Send + Sync {
    fn read_to_string(&self, path: &Path) -> Result<String, String>;
    fn write(&self, path: &Path, content: &str) -> Result<(), String>;
    fn exists(&self, path: &Path) -> bool;
    fn create_dir_all(&self, path: &Path) -> Result<(), String>;
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, String>;
}

#[async_trait]
#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
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

pub trait GitRepository: Send + Sync {
    fn is_repo(&self, directory: &str) -> bool;
    fn init(&self, directory: &str, remote_url: Option<String>) -> Result<(), String>;
    fn status(&self, directory: &str) -> Result<GitStatus, String>;
    fn commit(&self, directory: &str, message: &str) -> Result<(), String>;
    fn pull(&self, directory: &str) -> Result<(), String>;
    fn push(&self, directory: &str) -> Result<(), String>;
    fn sync(&self, directory: &str) -> Result<(), String>;
}

pub trait HistoryRepository {
    fn init(&self) -> Result<(), String>;
    fn save(&self, entry: HistoryEntry) -> Result<(), String>;
    fn get_history(&self, limit: usize, offset: usize) -> Result<Vec<HistoryEntry>, String>;
    fn clear(&self) -> Result<(), String>;
}

#[cfg_attr(any(test, feature = "mocks"), mockall::automock)]
pub trait SecretStore: Send + Sync {
    fn get(&self, key: &str) -> Result<String, String>;
    fn set(&self, key: &str, value: &str) -> Result<(), String>;
    fn delete(&self, key: &str) -> Result<(), String>;
}
