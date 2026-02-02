use crate::types::QResponse;
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

pub struct RealHttpClient;

#[async_trait]
impl HttpClient for RealHttpClient {
    async fn send_request(
        &self,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<String>,
        query: Vec<(String, String)>,
    ) -> Result<QResponse, String> {
        let client = reqwest::Client::new();
        let mut builder = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            "HEAD" => client.head(url),
            _ => return Err(format!("Unsupported method: {}", method)),
        };

        for (name, value) in headers {
            builder = builder.header(name, value);
        }

        if !query.is_empty() {
            builder = builder.query(&query);
        }

        if let Some(b) = body {
            builder = builder.body(b);
        }

        let start = std::time::Instant::now();
        let response = builder.send().await.map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;

        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        let mut res_headers = Vec::new();
        for (name, value) in response.headers() {
            res_headers.push(crate::types::Header {
                name: name.to_string(),
                value: value.to_str().unwrap_or_default().to_string(),
                enabled: true,
                secret_key: None,
            });
        }

        let body_content = response.text().await.map_err(|e| e.to_string())?;
        let _size = body_content.len() as u64;

        Ok(QResponse {
            status,
            status_text,
            headers: res_headers,
            body: body_content,
            error: None,
            time_elapsed: elapsed,
            size: _size,
        })
    }
}
