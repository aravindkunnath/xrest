//! Git integration against the real `git` binary in temp directories, with a local bare repo as
//! the remote. Identity and config come from env so nothing depends on the machine's git config.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Once;

use xrest_core::*;

fn env_once() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        for (k, v) in [
            ("GIT_CONFIG_GLOBAL", "/dev/null"),
            ("GIT_CONFIG_NOSYSTEM", "1"),
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "init.defaultBranch"),
            ("GIT_CONFIG_VALUE_0", "main"),
            ("GIT_AUTHOR_NAME", "Test"),
            ("GIT_AUTHOR_EMAIL", "test@example.com"),
            ("GIT_COMMITTER_NAME", "Test"),
            ("GIT_COMMITTER_EMAIL", "test@example.com"),
        ] {
            std::env::set_var(k, v);
        }
    });
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn commit_count(dir: &Path) -> usize {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .unwrap_or(0)
}

fn spec(url: &str) -> RequestSpec {
    RequestSpec {
        method: "GET".into(),
        url: url.into(),
        ..Default::default()
    }
}

fn bare() -> tempfile::TempDir {
    env_once();
    let d = tempfile::tempdir().unwrap();
    git(d.path(), &["init", "--bare", "-q"]);
    d
}

/// A workspace that is a git repo with `remote` as origin and one pushed initial commit.
fn repo_with_remote(remote: &Path) -> (tempfile::TempDir, Workspace) {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.git_init(Some(remote.to_str().unwrap().into())).unwrap();
    assert!(ws.git_commit("Initial").unwrap().is_some());
    ws.git_push().unwrap();
    (d, ws)
}

#[test]
fn status_on_non_repo() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let ws = Workspace::create(d.path(), "svc").unwrap();
    let s = ws.git_status().unwrap();
    assert!(!s.is_repo);
    assert!(s.uncommitted.is_empty());
    assert_eq!((s.ahead, s.behind, s.last_sync), (0, 0, None));
    assert_eq!(s.remote_url, None);
}

#[test]
fn init_with_remote_and_status() {
    let remote = bare();
    let url = remote.path().to_str().unwrap().to_string();
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.git_init(Some(url.clone())).unwrap();
    let s = ws.git_status().unwrap();
    assert!(s.is_repo);
    assert_eq!(s.branch, "main");
    assert_eq!(s.remote_url.as_deref(), Some(url.as_str()));
    let paths: Vec<&str> = s.uncommitted.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"service.yaml"), "{paths:?}");
    assert!(paths.contains(&"environments.yaml"));
    assert!(s.uncommitted.iter().all(|f| f.status == "untracked"));
    // idempotent
    ws.git_init(Some(url)).unwrap();
}

#[test]
fn commit_stages_only_workspace_content() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.git_init(None).unwrap();
    ws.create_endpoint("List pets", spec("http://x/pets"))
        .unwrap();
    ws.create_scratch("tmp", spec("http://x/s")).unwrap();
    // even a .gitignore that no longer hides local state must not stage it
    fs::write(d.path().join(".gitignore"), "").unwrap();
    fs::write(d.path().join("notes.txt"), "mine").unwrap();
    git(d.path(), &["add", "notes.txt"]);

    let sha = ws.git_commit("Initial").unwrap().expect("a commit");
    assert_eq!(sha.len(), 40);
    let tracked = git(d.path(), &["ls-tree", "-r", "--name-only", "HEAD"]);
    assert!(tracked.contains("service.yaml"));
    assert!(tracked.contains("environments.yaml"));
    assert!(tracked.contains("endpoints/"));
    assert!(!tracked.contains("scratchpad"), "{tracked}");
    assert!(!tracked.contains(".xrest"), "{tracked}");
    assert!(!tracked.contains("notes.txt"), "user's other staged file");

    assert_eq!(ws.git_commit("Again").unwrap(), None);
}

#[test]
fn auto_mode_one_commit_per_save_touching_one_file() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.git_init(None).unwrap();
    ws.git_commit("Initial").unwrap();
    assert_eq!(ws.git_mode(), GitMode::Manual);
    ws.set_git_mode(GitMode::Auto).unwrap();
    let root = d.path();
    let last = || {
        (
            git(root, &["log", "-1", "--format=%s"]),
            git(root, &["show", "--name-only", "--format=", "HEAD"]),
        )
    };
    let mut n = commit_count(root);

    // a stray staged file must not leak into auto commits
    fs::write(root.join("notes.txt"), "mine").unwrap();
    git(root, &["add", "notes.txt"]);

    let ep = ws
        .create_endpoint("List pets", spec("http://x/pets"))
        .unwrap();
    n += 1;
    assert_eq!(commit_count(root), n);
    let file = format!("endpoints/{}.yaml", ep.id);
    assert_eq!(last(), ("Create endpoint: List pets".into(), file.clone()));

    ws.update_endpoint(&ep.id, None, spec("http://x/pets2"))
        .unwrap();
    n += 1;
    assert_eq!(commit_count(root), n);
    assert_eq!(last(), ("Update endpoint: List pets".into(), file.clone()));

    // no-op update: no commit
    ws.update_endpoint(&ep.id, None, spec("http://x/pets2"))
        .unwrap();
    assert_eq!(commit_count(root), n);

    let mut envs = ws.environments().to_vec();
    envs[0].variables.push(NameValue::new("K", "v"));
    ws.save_environments(envs).unwrap();
    n += 1;
    assert_eq!(commit_count(root), n);
    assert_eq!(
        last(),
        ("Update environments".into(), "environments.yaml".into())
    );

    let mut st = ws.service_settings();
    st.name = "Renamed".into();
    ws.update_service_settings(st).unwrap();
    n += 1;
    assert_eq!(commit_count(root), n);
    assert_eq!(
        last(),
        ("Update service settings".into(), "service.yaml".into())
    );

    ws.delete_endpoint(&ep.id).unwrap();
    n += 1;
    assert_eq!(commit_count(root), n);
    assert_eq!(last(), ("Delete endpoint: List pets".into(), file));

    assert!(ws.warnings().is_empty(), "{:?}", ws.warnings());
    assert!(git(root, &["diff", "--cached", "--name-only"]).contains("notes.txt"));
}

#[test]
fn manual_mode_never_commits() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.git_init(None).unwrap();
    ws.git_commit("Initial").unwrap();
    let n = commit_count(d.path());
    ws.create_endpoint("A", spec("http://x")).unwrap();
    assert_eq!(commit_count(d.path()), n);
}

#[test]
fn git_mode_persists_per_user() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.set_git_mode(GitMode::Auto).unwrap();
    assert!(fs::read_to_string(d.path().join(".xrest/state.yaml"))
        .unwrap()
        .contains("gitMode: auto"));
    assert_eq!(Workspace::open(d.path()).unwrap().git_mode(), GitMode::Auto);
}

#[test]
fn failed_auto_commit_is_a_warning_not_an_error() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.git_init(None).unwrap();
    ws.git_commit("Initial").unwrap();
    ws.set_git_mode(GitMode::Auto).unwrap();
    let hook = d.path().join(".git/hooks/pre-commit");
    fs::create_dir_all(hook.parent().unwrap()).unwrap();
    fs::write(&hook, "#!/bin/sh\necho nope >&2\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&hook, fs::Permissions::from_mode(0o755)).unwrap();
    }
    let ep = ws.create_endpoint("A", spec("http://x")).unwrap();
    assert!(ws.endpoint(&ep.id).is_ok());
    assert!(d.path().join(format!("endpoints/{}.yaml", ep.id)).is_file());
    assert!(
        ws.warnings().iter().any(|w| w.contains("auto-commit")),
        "{:?}",
        ws.warnings()
    );
}

#[test]
fn pull_brings_teammates_endpoint_and_reloads() {
    let remote = bare();
    let (a_dir, mut a) = repo_with_remote(remote.path());
    assert!(a.git_status().unwrap().last_sync.is_some());

    let b_dir = tempfile::tempdir().unwrap();
    let out = Command::new("git")
        .args(["clone", "-q"])
        .arg(remote.path())
        .arg(b_dir.path())
        .output()
        .unwrap();
    assert!(out.status.success());
    let mut b = Workspace::open(b_dir.path()).unwrap();
    let ep = b.create_endpoint("From B", spec("http://b")).unwrap();
    b.git_commit("Add from B").unwrap();
    b.git_push().unwrap();

    assert!(a.endpoints().is_empty());
    a.git_pull().unwrap();
    assert_eq!(a.endpoints().len(), 1);
    assert_eq!(a.endpoints()[0].id, ep.id);
    assert!(a_dir.path().join("endpoints").is_dir());
}

#[test]
fn conflicting_pull_changes_nothing_and_lists_paths() {
    let remote = bare();
    let (_a_dir, mut a) = repo_with_remote(remote.path());
    let b_dir = tempfile::tempdir().unwrap();
    Command::new("git")
        .args(["clone", "-q"])
        .arg(remote.path())
        .arg(b_dir.path())
        .output()
        .unwrap();
    let mut b = Workspace::open(b_dir.path()).unwrap();

    let rename = |ws: &mut Workspace, n: &str| {
        let mut st = ws.service_settings();
        st.name = n.into();
        ws.update_service_settings(st).unwrap();
    };
    rename(&mut b, "Theirs");
    b.git_commit("theirs").unwrap();
    b.git_push().unwrap();
    rename(&mut a, "Mine");
    a.git_commit("mine").unwrap();

    let before = fs::read_to_string(a.root().join("service.yaml")).unwrap();
    let err = a.git_pull().unwrap_err();
    assert!(matches!(err, Error::Git(_)), "{err:?}");
    assert!(err.to_string().contains("service.yaml"), "{err}");
    assert_eq!(
        fs::read_to_string(a.root().join("service.yaml")).unwrap(),
        before
    );
    assert!(!a.root().join(".git/MERGE_HEAD").exists());
    assert_eq!(a.service().name, "Mine");
}

#[test]
fn ahead_behind_and_push() {
    let remote = bare();
    let (a_dir, mut a) = repo_with_remote(remote.path());
    let s = a.git_status().unwrap();
    assert_eq!((s.ahead, s.behind), (0, 0));

    a.create_endpoint("X", spec("http://x")).unwrap();
    a.git_commit("x").unwrap();
    assert_eq!(a.git_status().unwrap().ahead, 1);

    let b_dir = tempfile::tempdir().unwrap();
    Command::new("git")
        .args(["clone", "-q"])
        .arg(remote.path())
        .arg(b_dir.path())
        .output()
        .unwrap();
    let mut b = Workspace::open(b_dir.path()).unwrap();
    b.create_endpoint("Y", spec("http://y")).unwrap();
    b.git_commit("y").unwrap();
    b.git_push().unwrap();

    git(a_dir.path(), &["fetch", "-q"]);
    let s = a.git_status().unwrap();
    assert_eq!((s.ahead, s.behind), (1, 1));

    a.git_sync().unwrap();
    let s = a.git_status().unwrap();
    assert_eq!((s.ahead, s.behind), (0, 0));
    assert_eq!(a.endpoints().len(), 2);
}

#[test]
fn sync_into_empty_remote_pushes() {
    let remote = bare();
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    ws.git_init(Some(remote.path().to_str().unwrap().into()))
        .unwrap();
    ws.git_commit("Initial").unwrap();
    ws.git_sync().unwrap();
    assert_eq!(git(remote.path(), &["rev-list", "--count", "main"]), "1");
}

#[test]
fn errors_are_git_errors() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    assert!(matches!(ws.git_pull(), Err(Error::Git(_))));
    ws.git_init(None).unwrap();
    assert!(matches!(ws.git_push(), Err(Error::Git(_))));
}

#[test]
fn remote_url_that_looks_like_an_option_is_refused() {
    env_once();
    let d = tempfile::tempdir().unwrap();
    let mut ws = Workspace::create(d.path(), "svc").unwrap();
    for url in ["--mirror=fetch", "-oProxyCommand=evil"] {
        let r = ws.git_init(Some(url.into()));
        assert!(matches!(r, Err(Error::InvalidInput(_))), "{url}: {r:?}");
    }
    assert!(!d.path().join(".git").exists(), "refused before git init");
}
