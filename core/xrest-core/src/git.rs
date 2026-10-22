//! Git integration.
//!
//! TODO(git): port `internal/adapters/git.go` (status, commit on save, pull/push, remote URL).
//! Decide between shelling out to `git` and `git2`/`gix`. Note `scratchpad/` and `.xrest/` must stay
//! out of commits; the default `.gitignore` written by the storage layer already covers that.

use std::path::Path;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub is_git: bool,
    pub branch: String,
    pub has_uncommitted_changes: bool,
}

pub fn status(_workspace_root: &Path) -> Result<GitStatus> {
    Err(Error::NotImplemented("git status"))
}
