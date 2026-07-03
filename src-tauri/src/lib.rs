#![allow(deprecated)]
use tauri::Manager;
mod commands;
#[cfg(test)]
mod tests;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                if let Some(window) = app.get_webview_window("main") {
                    let ns_window = window.ns_window().unwrap() as cocoa::base::id;
                    unsafe {
                        use cocoa::appkit::{NSWindow, NSWindowTitleVisibility};
                        ns_window.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
                        ns_window.setTitlebarAppearsTransparent_(cocoa::base::YES);
                    }
                }
            }

            // Init history database
            {
                use xrest_core::traits::PathProvider;
                let paths = xrest_infra::paths::TauriPathProvider::new(app.handle())?;
                let db_path = paths.history_db_path()?;
                let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
                let history_repo = xrest_infra::history::SqliteHistoryRepository::new(conn);
                use xrest_core::traits::HistoryRepository;
                history_repo.init().map_err(|e| Box::new(std::io::Error::other(e)))?;

                // Load token cache
                let token_store = std::sync::Arc::new(xrest_core::auth::cache::MemoryTokenStore::new());
                if let Ok(cache_path) = paths.token_cache_path() {
                    use xrest_core::auth::cache::TokenStore;
                    let _ = token_store.load_from_file(&cache_path, &xrest_infra::fs::RealFileSystem);
                }
                app.manage(token_store);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::config::get_services,
            commands::config::save_services,
            commands::config::get_collections,
            commands::config::save_collections,
            commands::config::get_tab_state,
            commands::config::save_tab_state,
            commands::config::get_settings,
            commands::config::save_settings,
            commands::request::send_request,
            commands::window::close_splashscreen,
            commands::import::import_service,
            commands::git::git_init,
            commands::git::get_git_status,
            commands::git::git_sync,
            commands::git::git_pull,
            commands::git::git_push,
            commands::git::git_commit,
            commands::history::get_history,
            commands::history::clear_history,
            commands::import::import_swagger,
            commands::import::import_curl,
            commands::secrets::get_secrets,
            commands::secrets::add_secret,
            commands::secrets::delete_secret,
            commands::secrets::get_secret,
            commands::request::test_preflight_config,
            commands::request::read_dotenv_variables
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
