use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub is_git: bool,
    pub remote_url: Option<String>,
    pub branch: Option<String>,
    pub has_uncommitted_changes: bool,
    pub has_unpushed_commits: bool,
    pub last_sync: Option<u64>,
}
