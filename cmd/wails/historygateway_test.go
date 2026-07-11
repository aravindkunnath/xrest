package main

import (
	"path/filepath"
	"testing"
	"xrest/internal/models"
)

func TestHistoryGateway_Functional(t *testing.T) {
	// Setup isolated test database path
	tempDir := t.TempDir()
	dbPath := filepath.Join(tempDir, "gateway_history.db")

	// Create HistoryGateway but override dbPath for strict isolation
	hg := NewHistoryGateway()
	hg.dbPath = dbPath

	// Ensure cleanup at the end
	defer func() {
		if hg.repo != nil {
			_ = hg.repo.Close()
		}
	}()

	// 1. Initial State: get empty list
	history, err := hg.GetHistory(10, 0)
	if err != nil {
		t.Fatalf("expected no error loading history, got %v", err)
	}
	if len(history) != 0 {
		t.Errorf("expected 0 loaded history items, got %d", len(history))
	}

	// 2. Add an item
	entry := models.HistoryEntry{
		Method:             "GET",
		URL:                "https://api.example.com",
		ResponseStatus:     200,
		ResponseStatusText: "OK",
	}

	saved, err := hg.AddHistory(entry)
	if err != nil {
		t.Fatalf("expected no error saving history, got %v", err)
	}
	if saved.ID == "" {
		t.Error("expected generated ID for history entry")
	}
	if saved.CreatedAt == "" {
		t.Error("expected generated CreatedAt for history entry")
	}

	// 3. Retrieve and verify
	history, err = hg.GetHistory(10, 0)
	if err != nil {
		t.Fatalf("expected no error loading history, got %v", err)
	}
	if len(history) != 1 {
		t.Errorf("expected 1 history item, got %d", len(history))
	} else {
		if history[0].ID != saved.ID {
			t.Errorf("expected history item ID to be %s, got %s", saved.ID, history[0].ID)
		}
		if history[0].URL != "https://api.example.com" {
			t.Errorf("expected history item URL to be https://api.example.com, got %s", history[0].URL)
		}
	}

	// 4. Clear
	if err := hg.ClearHistory(); err != nil {
		t.Fatalf("expected no error on ClearHistory, got %v", err)
	}

	history, err = hg.GetHistory(10, 0)
	if err != nil {
		t.Fatalf("expected no error loading history after clear, got %v", err)
	}
	if len(history) != 0 {
		t.Errorf("expected 0 loaded history items after clear, got %d", len(history))
	}
}
