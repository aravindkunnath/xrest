mod commands;
pub mod domains;
pub mod history;
mod io;
mod services;
#[cfg(test)]
mod tests;
mod types;

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
            history::init_db(app.handle())?;
            // Load token cache
            if let Ok(cache_path) = domains::auth::get_token_cache_path(app.handle()) {
                let _ = domains::auth::cache::load_cache_from_file(&cache_path);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_services,
            commands::save_services,
            commands::get_collections,
            commands::save_collections,
            commands::get_tab_state,
            commands::save_tab_state,
            commands::get_settings,
            commands::save_settings,
            commands::send_request,
            commands::close_splashscreen,
            commands::import_service,
            commands::git_init,
            commands::get_git_status,
            commands::git_sync,
            commands::git_pull,
            commands::git_push,
            commands::git_commit,
            commands::get_history,
            commands::clear_history,
            commands::import_swagger,
            commands::import_curl,
            commands::get_secrets,
            commands::add_secret,
            commands::delete_secret,
            commands::get_secret
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
