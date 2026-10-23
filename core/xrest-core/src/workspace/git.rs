//! Git: status, init, commit, pull/push/sync, the per-user commit mode and auto-commit on save.
//!
//! The git commands themselves live in [`crate::git`] and only need the workspace root, so
//! async callers can run them without holding the workspace; afterwards they call
//! [`Workspace::finish_git_sync`] to record the sync time and reload from disk.

use super::{now_secs, Workspace};
use crate::error::Result;
use crate::git::{self, GitStatus};
use crate::model::{GitMode, WorkspaceState};
use crate::storage;

impl Workspace {
    pub fn git_mode(&self) -> GitMode {
        self.state.git_mode
    }

    pub fn set_git_mode(&mut self, mode: GitMode) -> Result<()> {
        if self.state.git_mode == mode {
            return Ok(());
        }
        let next = WorkspaceState {
            git_mode: mode,
            ..self.state.clone()
        };
        storage::save_state(&self.root, &next)?;
        self.state = next;
        Ok(())
    }

    /// Re-read everything from disk (after a pull, or an external edit). Version history,
    /// secrets, HTTP options and the token cache are kept. On failure nothing changes.
    pub fn reload(&mut self) -> Result<()> {
        let loaded = storage::load(&self.root)?;
        self.service = loaded.service;
        self.environments = loaded.environments;
        self.endpoints = loaded.endpoints;
        self.scratch = loaded.scratch;
        self.state = loaded.state;
        self.warnings = loaded.warnings;
        self.tokens.clear();
        Ok(())
    }

    /// The recorded time of the last successful pull/push/sync.
    pub fn git_status_last_sync(&self) -> Option<u64> {
        self.state.last_sync
    }

    pub fn git_status(&self) -> Result<GitStatus> {
        let mut s = git::status(&self.root)?;
        s.last_sync = self.state.last_sync;
        Ok(s)
    }

    pub fn git_init(&mut self, remote: Option<String>) -> Result<()> {
        git::init(&self.root, remote.as_deref())
    }

    pub fn git_commit(&mut self, message: &str) -> Result<Option<String>> {
        git::commit(&self.root, message)
    }

    /// Pull, then reload from disk and stamp `last_sync`.
    pub fn git_pull(&mut self) -> Result<()> {
        git::pull(&self.root)?;
        self.finish_git_sync()
    }

    /// Push, then stamp `last_sync`.
    pub fn git_push(&mut self) -> Result<()> {
        git::push(&self.root)?;
        self.finish_git_sync()
    }

    /// Pull then push.
    pub fn git_sync(&mut self) -> Result<()> {
        git::pull(&self.root)?;
        git::push(&self.root)?;
        self.finish_git_sync()
    }

    /// Call after a successful pull/push/sync that ran outside the workspace: reloads from disk
    /// (new endpoints appear) and records the sync time in `.xrest/state.yaml`.
    pub fn finish_git_sync(&mut self) -> Result<()> {
        self.reload()?;
        let next = WorkspaceState {
            last_sync: Some(now_secs()),
            ..self.state.clone()
        };
        storage::save_state(&self.root, &next)?;
        self.state = next;
        Ok(())
    }

    /// In auto mode, commit exactly `paths` (relative to the root). A failure never fails the
    /// save; it is recorded as a warning.
    pub(super) fn auto_commit(&mut self, paths: &[&str], message: &str) {
        if self.state.git_mode != GitMode::Auto || !git::is_repo(&self.root) {
            return;
        }
        if let Err(e) = git::try_commit_paths(&self.root, paths, message) {
            self.warnings.push(format!("auto-commit failed: {e}"));
        }
    }

    pub(super) fn auto_commit_endpoint(&mut self, id: &str, message: &str) {
        let path = format!("{}/{id}.yaml", storage::ENDPOINTS_DIR);
        self.auto_commit(&[&path], message);
    }
}
