use crate::core::request::RequestService;
use crate::core::service::service::ServiceDomain;
use crate::core::settings::SettingsDomain;
use crate::core::traits::{HistoryRepository, PathProvider};
use crate::core::types::{HistoryEntry, PreflightConfig, QResponse, RequestTab};
use crate::infra::fs::RealFileSystem;
use crate::infra::history::SqliteHistoryRepository;
use crate::infra::http::RealHttpClient;
use crate::infra::keyring::KeyringSecretStore;
use crate::infra::paths::TauriPathProvider;
use rusqlite::Connection;
use tauri::AppHandle;

#[tauri::command]
pub async fn send_request(app: AppHandle, mut tab: RequestTab) -> Result<QResponse, String> {
    let paths = TauriPathProvider::new(&app)?;

    if let Some(sid) = &tab.service_id {
        let settings_path = paths.settings_path()?;
        let sid_clone = sid.clone();

        let service_config = tokio::task::spawn_blocking(move || {
            let settings_domain = SettingsDomain::new(&RealFileSystem);
            let settings = settings_domain.load_settings(&settings_path).ok()?;
            let stub = settings.services.iter().find(|s| s.id == sid_clone)?;
            let service_domain = ServiceDomain::new(&RealFileSystem);
            service_domain.load_service(&stub.directory).ok()
        })
        .await
        .map_err(|e| e.to_string())?;

        if let Some(service) = service_config {
            if tab.auth.r#type == "none" {
                tab.auth = service.auth;
            }
            if !tab.preflight.enabled {
                tab.preflight = service.preflight;
            }
        }
    }

    let cache_path = paths.token_cache_path().ok();
    let request_service = RequestService::new(&RealHttpClient, &KeyringSecretStore, cache_path)
        .with_fs(&RealFileSystem);
    let req_method = tab.method.clone();
    let req_url = tab.url.clone();
    let endpoint_id = tab.endpoint_id.clone();
    let service_id = tab.service_id.clone();
    let headers_clone = tab.headers.clone();
    let body_clone = tab.body.content.clone();

    let response = request_service.send_request(tab).await?;

    let history_entry = HistoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        service_id,
        endpoint_id,
        method: req_method,
        url: req_url,
        request_headers: headers_clone,
        request_body: body_clone,
        response_status: response.status,
        response_status_text: response.status_text.clone(),
        response_headers: response.headers.clone(),
        response_body: response.body.clone(),
        time_elapsed: response.time_elapsed,
        size: response.size,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let db_path = paths.history_db_path()?;
    tokio::spawn(async move {
        if let Ok(conn) = Connection::open(db_path) {
            let repo = SqliteHistoryRepository::new(conn);
            if let Err(e) = repo.save(history_entry) {
                eprintln!("Failed to save history: {}", e);
            }
        }
    });

    Ok(response)
}

#[tauri::command]
pub async fn test_preflight_config(
    app: AppHandle,
    service_id: String,
    config: PreflightConfig,
    variables: std::collections::HashMap<String, String>,
) -> Result<crate::core::types::PreflightTestResult, String> {
    let paths = TauriPathProvider::new(&app)?;
    let cache_path = paths.token_cache_path().ok();

    Ok(crate::core::auth::preflight::test_preflight(
        &RealHttpClient,
        &service_id,
        &config,
        &variables,
        cache_path.as_ref(),
        Some(&RealFileSystem as &dyn crate::core::traits::FileSystem),
    )
    .await)
}
