//! Git integration by shelling out to the system `git` (no shell, args as a vector), so
//! credentials come from the user's own git/SSH setup.
//!
//! Every call runs `git -C <root> …` with `GIT_TERMINAL_PROMPT=0` (never block on a prompt) and
//! `LC_ALL=C` (parseable output). Staging only ever touches workspace content
//! ([`TRACKED_PATHS`]); `scratchpad/` and `.xrest/` are never named, so they stay out of commits
//! even if `.gitignore` was edited. Commits use the user's configured identity; if none is set,
//! git's error is returned as [`Error::Git`].

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use crate::error::{Error, Result};
use crate::storage;

/// The only paths XRest ever stages, relative to the workspace root.
pub const TRACKED_PATHS: [&str; 4] = [
    storage::SERVICE_FILE,
    storage::ENVIRONMENTS_FILE,
    storage::ENDPOINTS_DIR,
    storage::GITIGNORE_FILE,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitFileStatus {
    pub path: String,
    /// `modified`, `added`, `deleted`, `renamed`, `untracked` or `conflicted`.
    pub status: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitStatus {
    pub is_repo: bool,
    /// Current branch (empty when not a repo; `(detached)` when detached).
    pub branch: String,
    pub remote_url: Option<String>,
    pub uncommitted: Vec<GitFileStatus>,
    /// Commits not on the upstream / on the upstream but not here, as of the last fetch.
    pub ahead: u32,
    pub behind: u32,
    /// Unix time of the last successful pull/push/sync. Filled in by the workspace.
    pub last_sync: Option<u64>,
}

/// Serialises the git commands that change the repository (commit, pull, push, init), so an
/// auto-commit on save cannot race a manual action (two `git` processes fighting over
/// `index.lock`). One gate per repository root, held only while a command runs.
fn gate(root: &Path) -> Arc<Mutex<()>> {
    static GATES: Mutex<Option<HashMap<PathBuf, Arc<Mutex<()>>>>> = Mutex::new(None);
    let mut gates = GATES.lock().unwrap_or_else(|e| e.into_inner());
    gates
        .get_or_insert_with(HashMap::new)
        .entry(root.to_path_buf())
        .or_default()
        .clone()
}

fn lock(gate: &Mutex<()>) -> MutexGuard<'_, ()> {
    gate.lock().unwrap_or_else(|e| e.into_inner())
}

fn base_command(root: &Path, args: &[&str]) -> Command {
    let mut c = Command::new("git");
    c.arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("LC_ALL", "C")
        .stdin(Stdio::null());
    c
}

/// `git status` must not take optional locks (it would refresh the index and could collide with
/// a concurrent commit).
fn status_command(root: &Path, args: &[&str]) -> Command {
    let mut c = base_command(root, args);
    c.env("GIT_OPTIONAL_LOCKS", "0");
    c
}

fn run(root: &Path, args: &[&str]) -> Result<Output> {
    exec(base_command(root, args))
}

fn exec(mut cmd: Command) -> Result<Output> {
    cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Error::Git("git not found".into())
        } else {
            Error::Git(format!("could not run git: {e}"))
        }
    })
}

fn message(out: &Output) -> String {
    let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if err.is_empty() {
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    } else {
        err
    }
}

/// Run git and require success; returns trimmed stdout.
fn ok(root: &Path, args: &[&str]) -> Result<String> {
    let out = run(root, args)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(Error::Git(message(&out)))
    }
}

pub fn is_repo(root: &Path) -> bool {
    root.join(".git").exists()
}

fn require_repo(root: &Path) -> Result<()> {
    if is_repo(root) {
        Ok(())
    } else {
        Err(Error::Git("this workspace is not a git repository".into()))
    }
}

/// Local-only status (no network): ahead/behind reflect the last fetch.
pub fn status(root: &Path) -> Result<GitStatus> {
    if !is_repo(root) {
        return Ok(GitStatus::default());
    }
    let out = exec(status_command(
        root,
        &[
            "status",
            "--porcelain=v2",
            "--branch",
            "--untracked-files=all",
            "-z",
        ],
    ))?;
    if !out.status.success() {
        return Err(Error::Git(message(&out)));
    }
    let out = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let mut st = parse_status(&out);
    st.is_repo = true;
    st.remote_url = ok(root, &["remote", "get-url", "origin"])
        .ok()
        .filter(|u| !u.is_empty());
    Ok(st)
}

fn parse_status(text: &str) -> GitStatus {
    let mut st = GitStatus::default();
    let mut records = text.split('\0');
    while let Some(rec) = records.next() {
        if let Some(h) = rec.strip_prefix("# branch.head ") {
            st.branch = h.to_string();
        } else if let Some(ab) = rec.strip_prefix("# branch.ab ") {
            for part in ab.split_whitespace() {
                if let Some(n) = part.strip_prefix('+') {
                    st.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = part.strip_prefix('-') {
                    st.behind = n.parse().unwrap_or(0);
                }
            }
        } else if let Some(p) = rec.strip_prefix("? ") {
            st.uncommitted.push(GitFileStatus {
                path: p.to_string(),
                status: "untracked".into(),
            });
        } else if rec.starts_with("1 ") || rec.starts_with("2 ") || rec.starts_with("u ") {
            let kind = rec.as_bytes()[0];
            let fields = rec.splitn(
                match kind {
                    b'1' => 9,
                    b'2' => 10,
                    _ => 11,
                },
                ' ',
            );
            let fields: Vec<&str> = fields.collect();
            let Some(path) = fields.last() else { continue };
            let xy = fields.get(1).copied().unwrap_or("..");
            let status = if kind == b'u' {
                "conflicted"
            } else if kind == b'2' {
                // The original path follows as its own NUL-separated record.
                records.next();
                "renamed"
            } else {
                let c = xy.chars().find(|c| *c != '.').unwrap_or('M');
                match c {
                    'A' => "added",
                    'D' => "deleted",
                    'R' => "renamed",
                    _ => "modified",
                }
            };
            st.uncommitted.push(GitFileStatus {
                path: path.to_string(),
                status: status.into(),
            });
        }
    }
    st
}

/// `git init` (if needed) and make sure `.gitignore` hides local state. With `remote`, sets
/// `origin` to it (adding or updating). No commit is made.
pub fn init(root: &Path, remote: Option<&str>) -> Result<()> {
    let remote = remote.map(str::trim).filter(|u| !u.is_empty());
    if remote.is_some_and(|u| u.starts_with('-')) {
        return Err(Error::InvalidInput(
            "the remote URL cannot start with '-'".into(),
        ));
    }
    let gate = gate(root);
    let _guard = lock(&gate);
    if !is_repo(root) {
        ok(root, &["init", "-q"])?;
    }
    storage::ensure_gitignore(root)?;
    if let Some(url) = remote {
        let verb = if ok(root, &["remote", "get-url", "origin"]).is_ok() {
            "set-url"
        } else {
            "add"
        };
        ok(root, &["remote", verb, "--", "origin", url])?;
    }
    Ok(())
}

/// Stage `specs` (adds, edits and deletions; only those paths) and commit exactly them, leaving
/// anything else the user staged alone. `None` when there is nothing to commit.
pub fn commit_paths(root: &Path, specs: &[&str], message: &str) -> Result<Option<String>> {
    let gate = gate(root);
    let _guard = lock(&gate);
    commit_paths_locked(root, specs, message)
}

/// [`commit_paths`] for auto-commit on save: when a manual git action holds the gate it gives
/// up with an error instead of blocking the save (the change stays uncommitted).
pub fn try_commit_paths(root: &Path, specs: &[&str], message: &str) -> Result<Option<String>> {
    let gate = gate(root);
    let _guard = match gate.try_lock() {
        Ok(g) => g,
        Err(TryLockError::Poisoned(e)) => e.into_inner(),
        Err(TryLockError::WouldBlock) => {
            return Err(Error::Git(
                "another git operation is in progress; the change was left uncommitted".into(),
            ))
        }
    };
    commit_paths_locked(root, specs, message)
}

fn commit_paths_locked(root: &Path, specs: &[&str], message: &str) -> Result<Option<String>> {
    require_repo(root)?;
    let mut pathspec: Vec<&str> = vec![];
    for spec in specs {
        let known = |root: &Path| -> Result<bool> {
            Ok(!ok(root, &["ls-files", "--", spec])?.is_empty()
                || ok(root, &["cat-file", "-e", &format!("HEAD:{spec}")]).is_ok())
        };
        if root.join(spec).exists() {
            ok(root, &["add", "-A", "--", spec])?;
        } else if known(root)? {
            // A tracked path that no longer exists: stage its deletion.
            ok(
                root,
                &["rm", "-r", "--cached", "-q", "--ignore-unmatch", "--", spec],
            )?;
        }
        // Git rejects pathspecs it has never heard of (e.g. an empty `endpoints/`).
        if known(root)? {
            pathspec.push(spec);
        }
    }
    if pathspec.is_empty() {
        return Ok(None);
    }
    let mut diff = vec!["diff", "--cached", "--quiet", "--"];
    diff.extend(&pathspec);
    if run(root, &diff)?.status.success() {
        return Ok(None);
    }
    let mut commit = vec!["commit", "-q", "-m", message, "--"];
    commit.extend(&pathspec);
    ok(root, &commit)?;
    ok(root, &["rev-parse", "HEAD"]).map(Some)
}

/// Commit all workspace content ([`TRACKED_PATHS`]).
pub fn commit(root: &Path, message: &str) -> Result<Option<String>> {
    commit_paths(root, &TRACKED_PATHS, message)
}

fn branch(root: &Path) -> Result<String> {
    let b = ok(root, &["symbolic-ref", "--short", "HEAD"])
        .map_err(|_| Error::Git("not on a branch (detached HEAD)".into()))?;
    Ok(b)
}

fn has_upstream(root: &Path) -> bool {
    ok(root, &["rev-parse", "--abbrev-ref", "@{upstream}"]).is_ok()
}

/// Fetch and merge the upstream (or `origin/<branch>`). Never resolves conflicts: on conflict
/// the merge is aborted, files are left as they were, and the error lists the paths. A remote
/// without the branch yet counts as up to date.
pub fn pull(root: &Path) -> Result<()> {
    let gate = gate(root);
    let _guard = lock(&gate);
    require_repo(root)?;
    let branch = branch(root)?;
    let mut args = vec!["pull", "--no-rebase", "--no-edit", "-q"];
    if !has_upstream(root) {
        args.extend(["origin", &branch]);
    }
    let out = run(root, &args)?;
    if out.status.success() {
        return Ok(());
    }
    let msg = message(&out);
    let conflicts = ok(root, &["diff", "--name-only", "--diff-filter=U"]).unwrap_or_default();
    if !conflicts.is_empty() {
        let _ = run(root, &["merge", "--abort"]);
        let paths: Vec<&str> = conflicts.lines().collect();
        return Err(Error::Git(format!(
            "pull aborted, merge conflicts in: {} (no files were changed)",
            paths.join(", ")
        )));
    }
    if msg.contains("couldn't find remote ref") {
        return Ok(());
    }
    Err(Error::Git(msg))
}

/// Push the current branch, setting the upstream on first push.
pub fn push(root: &Path) -> Result<()> {
    let gate = gate(root);
    let _guard = lock(&gate);
    require_repo(root)?;
    ok(root, &["push", "-q", "-u", "origin", "HEAD"]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_of(c: &Command, key: &str) -> Option<String> {
        c.get_envs()
            .find(|(k, _)| *k == key)
            .and_then(|(_, v)| v.map(|v| v.to_string_lossy().into_owned()))
    }

    #[test]
    fn status_command_disables_optional_locks() {
        let c = status_command(Path::new("/x"), &["status"]);
        assert_eq!(env_of(&c, "GIT_OPTIONAL_LOCKS").as_deref(), Some("0"));
        let c = base_command(Path::new("/x"), &["commit"]);
        assert_eq!(env_of(&c, "GIT_OPTIONAL_LOCKS"), None);
    }

    #[test]
    fn auto_commit_does_not_race_a_held_gate() {
        let g = gate(Path::new("/nonexistent"));
        let _held = lock(&g);
        let e = try_commit_paths(Path::new("/nonexistent"), &["service.yaml"], "m").unwrap_err();
        assert!(e.to_string().contains("in progress"), "{e}");
    }

    #[test]
    fn parses_porcelain_v2() {
        let text = [
            "# branch.oid abc",
            "# branch.head main",
            "# branch.upstream origin/main",
            "# branch.ab +2 -1",
            "1 .M N... 100644 100644 100644 a b endpoints/with space.yaml",
            "1 A. N... 000000 100644 100644 a b service.yaml",
            "1 .D N... 100644 100644 000000 a b gone.yaml",
            "2 R. N... 100644 100644 100644 a b R100 new.yaml",
            "old.yaml",
            "u UU N... 1 2 3 4 a b c conflict.yaml",
            "? notes.txt",
            "",
        ]
        .join("\0");
        let st = parse_status(&text);
        assert_eq!((st.branch.as_str(), st.ahead, st.behind), ("main", 2, 1));
        let got: Vec<(&str, &str)> = st
            .uncommitted
            .iter()
            .map(|f| (f.path.as_str(), f.status.as_str()))
            .collect();
        assert_eq!(
            got,
            vec![
                ("endpoints/with space.yaml", "modified"),
                ("service.yaml", "added"),
                ("gone.yaml", "deleted"),
                ("new.yaml", "renamed"),
                ("conflict.yaml", "conflicted"),
                ("notes.txt", "untracked"),
            ]
        );
    }
}
