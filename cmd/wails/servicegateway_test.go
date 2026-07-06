package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"testing"
	"xrest/internal/models"
)

func TestServiceGateway(t *testing.T) {
	sg := NewServiceGateway()

	svcs, err := sg.LoadServices()
	if err != nil {
		t.Fatalf("expected no error loading services, got %v", err)
	}
	if len(svcs) != 0 {
		t.Errorf("expected 0 loaded services, got %d", len(svcs))
	}

	testSvcs := []models.Service{
		{ID: "s1", Name: "Test Service"},
	}
	saved, err := sg.SaveServices(testSvcs, "test commit")
	if err != nil {
		t.Fatalf("expected no error saving services, got %v", err)
	}
	if len(saved) != 1 || saved[0].ID != "s1" {
		t.Errorf("expected saved services to match input, got %v", saved)
	}

	tempDir := t.TempDir()

	if err := sg.InitGit(tempDir, ""); err != nil {
		t.Errorf("expected no error on InitGit, got %v", err)
	}

	status, err := sg.GetGitStatus(tempDir)
	if err != nil {
		t.Fatalf("expected no error getting git status, got %v", err)
	}
	if !status.IsGit {
		t.Errorf("expected IsGit to be true, got %t", status.IsGit)
	}

	// Create a dummy file to commit/sync/pull/push
	testFile := filepath.Join(tempDir, "dummy.txt")
	if err := os.WriteFile(testFile, []byte("test"), 0644); err != nil {
		t.Fatalf("expected no error writing dummy file, got %v", err)
	}

	if err := sg.CommitGit(tempDir, "message"); err != nil {
		t.Errorf("expected no error on CommitGit, got %v", err)
	}

	// Set up a bare remote repo to test Sync/Pull/Push
	remoteDir := t.TempDir()
	c := exec.Command("git", "init", "--bare")
	c.Dir = remoteDir
	if err := c.Run(); err != nil {
		t.Fatalf("failed to init bare remote repository: %v", err)
	}

	// Add remote
	cmdAdd := exec.Command("git", "remote", "add", "origin", remoteDir)
	cmdAdd.Dir = tempDir
	if err := cmdAdd.Run(); err != nil {
		t.Fatalf("failed to add remote: %v", err)
	}

	if err := sg.PushGit(tempDir); err != nil {
		t.Errorf("expected no error on PushGit, got %v", err)
	}

	if err := sg.PullGit(tempDir); err != nil {
		t.Errorf("expected no error on PullGit, got %v", err)
	}

	if err := sg.SyncGit(tempDir); err != nil {
		t.Errorf("expected no error on SyncGit, got %v", err)
	}

	imported, err := sg.ImportService(tempDir)
	if err != nil {
		t.Fatalf("expected no error importing service, got %v", err)
	}
	if imported.ID != "s-imported" {
		t.Errorf("expected imported ID to be s-imported, got %s", imported.ID)
	}

	importedCurl, err := sg.ImportCurl("s1", "curl http://url")
	if err != nil {
		t.Fatalf("expected no error importing curl, got %v", err)
	}
	if importedCurl.ID != "s1" {
		t.Errorf("expected imported curl service ID to be s1, got %s", importedCurl.ID)
	}
}
