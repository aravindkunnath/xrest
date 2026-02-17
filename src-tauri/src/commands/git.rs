use crate::core::traits::GitRepository;
use crate::infra::git::Git2Repository;
use tauri::AppHandle;

#[tauri::command]
pub fn get_git_status(
    _app: AppHandle,
    directory: String,
) -> Result<crate::core::types::GitStatus, String> {
    let git = Git2Repository;
    git.status(&directory)
}

#[tauri::command]
pub fn git_init(
    _app: AppHandle,
    directory: String,
    remote_url: Option<String>,
) -> Result<(), String> {
    let git = Git2Repository;
    git.init(&directory, remote_url)
}

#[tauri::command]
pub fn git_pull(_app: AppHandle, directory: String) -> Result<(), String> {
    let git = Git2Repository;
    git.pull(&directory)
}

#[tauri::command]
pub fn git_push(_app: AppHandle, directory: String) -> Result<(), String> {
    let git = Git2Repository;
    git.push(&directory)
}

#[tauri::command]
pub fn git_commit(_app: AppHandle, directory: String, message: String) -> Result<(), String> {
    let git = Git2Repository;
    git.commit(&directory, &message)
}

#[tauri::command]
pub fn git_sync(_app: AppHandle, directory: String) -> Result<(), String> {
    let git = Git2Repository;
    git.sync(&directory)
}
