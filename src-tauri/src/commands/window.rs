use tauri::AppHandle;

#[tauri::command]
pub fn close_splashscreen(app: AppHandle) {
    use tauri::Manager;
    if let Some(splashscreen) = app.get_webview_window("splashscreen") {
        let _ = splashscreen.close();
    }
    if let Some(main) = app.get_webview_window("main") {
        let _ = main.show();
        let _ = main.set_focus();
    }
}
