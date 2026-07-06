package main

import (
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

	status, err := sg.GetGitStatus("some/dir")
	if err != nil {
		t.Fatalf("expected no error getting git status, got %v", err)
	}
	if !status.IsGit {
		t.Errorf("expected IsGit to be true, got %t", status.IsGit)
	}

	if err := sg.InitGit("some/dir", "http://remote"); err != nil {
		t.Errorf("expected no error on InitGit, got %v", err)
	}

	if err := sg.SyncGit("some/dir"); err != nil {
		t.Errorf("expected no error on SyncGit, got %v", err)
	}

	if err := sg.PullGit("some/dir"); err != nil {
		t.Errorf("expected no error on PullGit, got %v", err)
	}

	if err := sg.PushGit("some/dir"); err != nil {
		t.Errorf("expected no error on PushGit, got %v", err)
	}

	if err := sg.CommitGit("some/dir", "message"); err != nil {
		t.Errorf("expected no error on CommitGit, got %v", err)
	}

	imported, err := sg.ImportService("some/dir")
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
