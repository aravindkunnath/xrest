use tauri::AppHandle;

#[tauri::command]
pub fn get_git_status(
    _app: AppHandle,
    directory: String,
) -> Result<crate::core::types::GitStatus, String> {
    crate::core::git::get_git_status(&directory)
}

#[tauri::command]
pub fn git_init(
    _app: AppHandle,
    directory: String,
    remote_url: Option<String>,
) -> Result<(), String> {
    crate::core::git::init_git(&directory, remote_url)
}

#[tauri::command]
pub fn git_pull(_app: AppHandle, directory: String) -> Result<(), String> {
    crate::core::git::pull_changes(&directory)
}

#[tauri::command]
pub fn git_push(_app: AppHandle, directory: String) -> Result<(), String> {
    crate::core::git::push_changes(&directory)
}

#[tauri::command]
pub fn git_commit(_app: AppHandle, directory: String, message: String) -> Result<(), String> {
    crate::core::git::commit_changes(&directory, &message)
}

#[tauri::command]
pub fn git_sync(_app: AppHandle, directory: String) -> Result<(), String> {
    crate::core::git::sync_git(&directory)
}
