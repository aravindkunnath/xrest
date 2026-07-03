use xrest_core::request::send_request_with_context;
use xrest_core::traits::{HistoryRepository, PathProvider};
use xrest_core::types::{PreflightConfig, QResponse, RequestTab};
use xrest_infra::fs::RealFileSystem;
use xrest_infra::history::SqliteHistoryRepository;
use xrest_infra::http::RealHttpClient;
use xrest_infra::keyring::KeyringSecretStore;
use xrest_infra::paths::TauriPathProvider;
use rusqlite::Connection;
use tauri::AppHandle;
use xrest_core::settings::SettingsDomain;
use tauri::Manager;

#[tauri::command]
pub async fn send_request(app: AppHandle, tab: RequestTab) -> Result<QResponse, String> {
    let paths = TauriPathProvider::new(&app)?;
    let settings_path = paths.settings_path()?;
    let cache_path = paths.token_cache_path().ok();
    let db_path = paths.history_db_path()?;

    let token_store_state = app.try_state::<std::sync::Arc<xrest_core::auth::cache::MemoryTokenStore>>();
    let token_store = token_store_state.as_ref().map(|s| &***s as &dyn xrest_core::auth::cache::TokenStore);

    let (response, history_entry) = send_request_with_context(
        &RealHttpClient,
        &RealFileSystem,
        &KeyringSecretStore,
        &settings_path,
        cache_path,
        token_store,
        tab,
    )
    .await?;

    // Persist history (non-Send repo, so handled here after the await)
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
pub async fn read_dotenv_variables(
    app: AppHandle,
    service_id: String,
    env_name: String,
) -> Result<std::collections::HashMap<String, String>, String> {
    let paths = TauriPathProvider::new(&app)?;
    let settings_path = paths.settings_path()?;
    let fs = RealFileSystem;
    let settings_domain = SettingsDomain::new(&fs);
    let settings = settings_domain.load_settings(&settings_path)?;

    let stub = settings
        .services
        .iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| format!("Service not found: {}", service_id))?;

    xrest_core::service::dotenv::load_dotenv_vars(&stub.directory, &env_name, &fs)
}

#[tauri::command]
pub async fn test_preflight_config(
    app: AppHandle,
    service_id: String,
    config: PreflightConfig,
    variables: std::collections::HashMap<String, String>,
) -> Result<xrest_core::types::PreflightTestResult, String> {
    let paths = TauriPathProvider::new(&app)?;
    let cache_path = paths.token_cache_path().ok();

    let token_store_state = app.try_state::<std::sync::Arc<xrest_core::auth::cache::MemoryTokenStore>>();
    let token_store = token_store_state.as_ref().map(|s| &***s as &dyn xrest_core::auth::cache::TokenStore);

    Ok(xrest_core::auth::preflight::test_preflight(
        &RealHttpClient,
        &service_id,
        &config,
        &variables,
        token_store,
        cache_path.as_ref(),
        Some(&RealFileSystem as &dyn xrest_core::traits::FileSystem),
    )
    .await)
}
