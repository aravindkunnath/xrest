use crate::core::secrets::SecretsDomain;
use crate::core::traits::PathProvider;
use crate::infra::fs::RealFileSystem;
use crate::infra::keyring::KeyringSecretStore;
use crate::infra::paths::TauriPathProvider;

#[tauri::command]
pub async fn get_secrets(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    println!("DEBUG: get_secrets called");
    tokio::task::spawn_blocking(move || {
        let paths = TauriPathProvider::new(&app)?;
        let domain = SecretsDomain::new(&RealFileSystem, &KeyringSecretStore);
        domain.list_secrets(&paths.secrets_path()?)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn add_secret(
    app: tauri::AppHandle,
    key: String,
    value: String,
) -> Result<Vec<String>, String> {
    println!("DEBUG: add_secret called for key: {}", key);
    tokio::task::spawn_blocking(move || {
        let paths = TauriPathProvider::new(&app)?;
        let secrets_path = paths.secrets_path()?;
        let domain = SecretsDomain::new(&RealFileSystem, &KeyringSecretStore);
        domain.add_secret(&secrets_path, &key, &value)?;
        domain.list_secrets(&secrets_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn delete_secret(app: tauri::AppHandle, key: String) -> Result<Vec<String>, String> {
    println!("DEBUG: delete_secret called for key: {}", key);
    tokio::task::spawn_blocking(move || {
        let paths = TauriPathProvider::new(&app)?;
        let secrets_path = paths.secrets_path()?;
        let domain = SecretsDomain::new(&RealFileSystem, &KeyringSecretStore);
        domain.delete_secret(&secrets_path, &key)?;
        domain.list_secrets(&secrets_path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_secret(key: String) -> Result<String, String> {
    println!("DEBUG: get_secret called for key: {}", key);
    tokio::task::spawn_blocking(move || {
        let store = KeyringSecretStore;
        crate::core::traits::SecretStore::get(&store, &key)
    })
    .await
    .map_err(|e| e.to_string())?
}
