use xrest_core::traits::FileSystem;
use std::path::{Path, PathBuf};

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    }

    fn write(&self, path: &Path, content: &str) -> Result<(), String> {
        std::fs::write(path, content).map_err(|e| e.to_string())
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn create_dir_all(&self, path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| e.to_string())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, String> {
        let mut paths = Vec::new();
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            paths.push(entry.map_err(|e| e.to_string())?.path());
        }
        Ok(paths)
    }
}
