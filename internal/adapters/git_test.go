package adapters

import (
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func TestGitAdapter_Functional(t *testing.T) {
	git := NewGitAdapter()

	// 1. Create a temporary directory representing the local workspace
	localDir := t.TempDir()

	// Verify not a repository initially
	if git.IsRepo(localDir) {
		t.Fatalf("expected directory %s to not be a git repo initially", localDir)
	}

	// 2. Initialize repository
	err := git.Init(localDir, "")
	if err != nil {
		t.Fatalf("failed to init git repository: %v", err)
	}

	if !git.IsRepo(localDir) {
		t.Fatalf("expected directory to be recognized as a git repo after Init")
	}

	// 3. Verify status of clean repository
	status, err := git.Status(localDir)
	if err != nil {
		t.Fatalf("failed to get status: %v", err)
	}
	if !status.IsGit {
		t.Errorf("expected isGit to be true, got %t", status.IsGit)
	}
	if status.HasUncommittedChanges {
		t.Errorf("expected no uncommitted changes initially, got true")
	}

	// 4. Make uncommitted changes (create a new file)
	testFilePath := filepath.Join(localDir, "test.txt")
	err = os.WriteFile(testFilePath, []byte("hello git"), 0644)
	if err != nil {
		t.Fatalf("failed to write test file: %v", err)
	}

	// Verify status detects uncommitted changes
	status, err = git.Status(localDir)
	if err != nil {
		t.Fatalf("failed to get status after modification: %v", err)
	}
	if !status.HasUncommittedChanges {
		t.Errorf("expected status to show uncommitted changes after file creation")
	}

	// 5. Commit changes
	err = git.Commit(localDir, "first user commit")
	if err != nil {
		t.Fatalf("failed to commit changes: %v", err)
	}

	// Verify status is clean again
	status, err = git.Status(localDir)
	if err != nil {
		t.Fatalf("failed to get status after commit: %v", err)
	}
	if status.HasUncommittedChanges {
		t.Errorf("expected status to be clean after commit")
	}

	// 6. Test Remote & Push/Pull/Sync setup
	// Create another temp dir to act as the remote bare repository
	remoteDir := t.TempDir()
	
	// Create bare remote repository
	cmd := execCommand(remoteDir, "git", "init", "--bare")
	if err := cmd.Run(); err != nil {
		t.Fatalf("failed to init bare remote repository: %v", err)
	}

	// Set remote URL in local repository
	cmdAdd := execCommand(localDir, "git", "remote", "add", "origin", remoteDir)
	err = cmdAdd.Run()
	if err != nil {
		t.Fatalf("failed to add remote: %v", err)
	}

	// Push to remote
	err = git.Push(localDir)
	if err != nil {
		t.Fatalf("failed to push: %v", err)
	}

	// Make sure remote has the commits (ahead/behind checks)
	status, err = git.Status(localDir)
	if err != nil {
		t.Fatalf("failed to get status after remote configuration: %v", err)
	}
	if status.HasUnpushedCommits {
		t.Errorf("expected no unpushed commits after pushing, got true")
	}

	// Make another modification to test Sync
	err = os.WriteFile(testFilePath, []byte("hello sync"), 0644)
	if err != nil {
		t.Fatalf("failed to modify file: %v", err)
	}

	err = git.Sync(localDir)
	if err != nil {
		t.Fatalf("failed to sync repository: %v", err)
	}

	status, err = git.Status(localDir)
	if err != nil {
		t.Fatalf("failed to get status after sync: %v", err)
	}
	if status.HasUncommittedChanges || status.HasUnpushedCommits {
		t.Errorf("expected sync to commit, pull, and push successfully leaving a clean state")
	}
}

// helper command generator
func execCommand(dir string, name string, args ...string) *exec.Cmd {
	c := exec.Command(name, args...)
	c.Dir = dir
	return c
}
