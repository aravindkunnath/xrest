use git2::{IndexAddOption, PushOptions, Repository, Signature};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

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

pub fn is_git_repo(directory: &str) -> bool {
    Repository::open(directory).is_ok()
}

pub fn init_git(directory: &str, remote_url: Option<String>) -> Result<(), String> {
    let repo = Repository::init(directory).map_err(|e| e.to_string())?;
    if let Some(url) = remote_url {
        repo.remote("origin", &url).map_err(|e| e.to_string())?;
    }
    commit_changes(directory, "Initial commit from xrest")?;
    Ok(())
}

pub fn get_git_status(directory: &str) -> Result<GitStatus, String> {
    let repo = Repository::open(directory).map_err(|e| e.to_string())?;

    let is_git = true;
    let remote_url = repo
        .find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()));

    let head = repo.head().ok();
    let branch = head
        .as_ref()
        .and_then(|h| h.shorthand().map(|s| s.to_string()));

    let mut status_options = git2::StatusOptions::new();
    status_options.include_untracked(true);
    let statuses = repo
        .statuses(Some(&mut status_options))
        .map_err(|e| e.to_string())?;
    let has_uncommitted_changes = !statuses.is_empty();

    // Check for unpushed commits
    let mut has_unpushed_commits = false;
    if let (Ok(local), Ok(remote)) = (
        repo.revparse_single("HEAD"),
        repo.revparse_single("origin/main"),
    ) {
        let local_id = local.id();
        let remote_id = remote.id();
        if local_id != remote_id {
            // This is a bit simplified, but checks if local HEAD is ahead
            let (ahead, _) = repo
                .graph_ahead_behind(local_id, remote_id)
                .unwrap_or((0, 0));
            has_unpushed_commits = ahead > 0;
        }
    } else if repo.revparse_single("HEAD").is_ok() && repo.find_remote("origin").is_ok() {
        // If we have HEAD but no origin/main, we likely have unpushed commits
        has_unpushed_commits = true;
    }

    let last_sync = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(GitStatus {
        is_git,
        remote_url,
        branch,
        has_uncommitted_changes,
        has_unpushed_commits,
        last_sync: Some(last_sync),
    })
}

pub fn commit_changes(directory: &str, message: &str) -> Result<(), String> {
    let repo = Repository::open(directory).map_err(|e| e.to_string())?;
    let mut index = repo.index().map_err(|e| e.to_string())?;

    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(|e| e.to_string())?;
    index.write().map_err(|e| e.to_string())?;

    let tree_id = index.write_tree().map_err(|e| e.to_string())?;
    let tree = repo.find_tree(tree_id).map_err(|e| e.to_string())?;

    let signature = Signature::now("xrest App", "info@xrest.io").map_err(|e| e.to_string())?;

    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit().map_err(|e| e.to_string())?),
        Err(_) => None,
    };

    let parents = match &parent_commit {
        Some(c) => vec![c],
        None => vec![],
    };

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn pull_changes(directory: &str) -> Result<(), String> {
    let repo = Repository::open(directory).map_err(|e| e.to_string())?;

    // Pull (Fetch + Merge)
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
    let mut fetch_options = git2::FetchOptions::new();

    // In a real app we'd handle auth here, but for now we'll assume SSH/stored creds
    remote
        .fetch(&["main"], Some(&mut fetch_options), None)
        .map_err(|e| e.to_string())?;

    // Simplified merge
    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| e.to_string())?;
    let fetch_commit = fetch_head.peel_to_commit().map_err(|e| e.to_string())?;
    let annotated_fetch_commit = repo
        .find_annotated_commit(fetch_commit.id())
        .map_err(|e| e.to_string())?;

    let analysis = repo
        .merge_analysis(&[&annotated_fetch_commit])
        .map_err(|e| e.to_string())?;

    if analysis.0.is_fast_forward() {
        let mut head = repo.head().map_err(|e| e.to_string())?;
        head.set_target(fetch_commit.id(), "fast-forward")
            .map_err(|e| e.to_string())?;
        repo.checkout_head(None).map_err(|e| e.to_string())?;
    } else if analysis.0.is_normal() {
        // Normal merge might have conflicts
        repo.merge(&[&annotated_fetch_commit], None, None)
            .map_err(|e| e.to_string())?;

        // After merge, if there are no conflicts, we should commit
        let index = repo.index().map_err(|e| e.to_string())?;
        if index.has_conflicts() {
            return Err("Merge conflict detected. Please resolve manually.".to_string());
        }

        // Create merge commit
        let msg = format!("Merge branch 'main' of remote");
        commit_changes(directory, &msg)?;
    }

    Ok(())
}

pub fn push_changes(directory: &str) -> Result<(), String> {
    let repo = Repository::open(directory).map_err(|e| e.to_string())?;
    let mut remote = repo.find_remote("origin").map_err(|e| e.to_string())?;
    let mut push_options = PushOptions::new();

    remote
        .push(
            &["refs/heads/main:refs/heads/main"],
            Some(&mut push_options),
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn sync_git(directory: &str) -> Result<(), String> {
    // Commit any local changes first
    let _ = commit_changes(directory, "Sync point: auto-committing local changes");

    pull_changes(directory)?;
    push_changes(directory)?;

    Ok(())
}
