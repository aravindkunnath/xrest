package adapters

import (
	"os"
	"path/filepath"
	"testing"
	"xrest/internal/models"
)

func TestSqliteHistoryRepository_Lifecycle(t *testing.T) {
	// Create isolated test environment using t.TempDir()
	tempDir := t.TempDir()
	dbPath := filepath.Join(tempDir, "test_history.db")

	repo, err := NewSqliteHistoryRepository(dbPath)
	if err != nil {
		t.Fatalf("failed to create repo: %v", err)
	}
	defer repo.Close()

	// Initial get must be empty
	entries, err := repo.GetHistory(10, 0)
	if err != nil {
		t.Fatalf("failed to get history: %v", err)
	}
	if len(entries) != 0 {
		t.Errorf("expected 0 entries, got %d", len(entries))
	}

	serviceID := "svc-1"
	endpointID := "end-1"
	entry1 := models.HistoryEntry{
		ID:                 "id-1",
		ServiceID:          &serviceID,
		EndpointID:         &endpointID,
		Method:             "GET",
		URL:                "https://api.example.com/users",
		RequestHeaders:     []models.Header{{Name: "Authorization", Value: "Bearer token", Enabled: true}},
		RequestBody:        "",
		ResponseStatus:     200,
		ResponseStatusText: "OK",
		ResponseHeaders:    []models.Header{{Name: "Content-Type", Value: "application/json", Enabled: true}},
		ResponseBody:       `{"status":"ok"}`,
		TimeElapsed:        120,
		Size:               15,
		CreatedAt:          "2026-07-06T12:00:00Z",
	}

	entry2 := models.HistoryEntry{
		ID:                 "id-2",
		ServiceID:          &serviceID,
		EndpointID:         &endpointID,
		Method:             "POST",
		URL:                "https://api.example.com/users",
		RequestHeaders:     []models.Header{{Name: "Content-Type", Value: "application/json", Enabled: true}},
		RequestBody:        `{"name":"Alice"}`,
		ResponseStatus:     201,
		ResponseStatusText: "Created",
		ResponseHeaders:    []models.Header{{Name: "Content-Type", Value: "application/json", Enabled: true}},
		ResponseBody:       `{"id":1}`,
		TimeElapsed:        250,
		Size:               8,
		CreatedAt:          "2026-07-06T12:05:00Z",
	}

	// Save entries
	if err := repo.Save(entry1); err != nil {
		t.Fatalf("failed to save entry1: %v", err)
	}
	if err := repo.Save(entry2); err != nil {
		t.Fatalf("failed to save entry2: %v", err)
	}

	// Fetch entries and verify order (created_at DESC)
	entries, err = repo.GetHistory(10, 0)
	if err != nil {
		t.Fatalf("failed to get history: %v", err)
	}

	if len(entries) != 2 {
		t.Fatalf("expected 2 entries, got %d", len(entries))
	}

	// Since order is created_at DESC, entry2 should be first
	if entries[0].ID != "id-2" {
		t.Errorf("expected first entry to be id-2, got %s", entries[0].ID)
	}
	if entries[1].ID != "id-1" {
		t.Errorf("expected second entry to be id-1, got %s", entries[1].ID)
	}

	// Verify header decoding
	if len(entries[1].RequestHeaders) != 1 || entries[1].RequestHeaders[0].Name != "Authorization" {
		t.Errorf("header decoding failed, got request headers: %v", entries[1].RequestHeaders)
	}

	// Test limit and offset
	limitedEntries, err := repo.GetHistory(1, 1)
	if err != nil {
		t.Fatalf("failed to get history with limit/offset: %v", err)
	}
	if len(limitedEntries) != 1 {
		t.Fatalf("expected 1 entry, got %d", len(limitedEntries))
	}
	if limitedEntries[0].ID != "id-1" {
		t.Errorf("expected entry at offset 1 to be id-1, got %s", limitedEntries[0].ID)
	}

	// Clear history
	if err := repo.Clear(); err != nil {
		t.Fatalf("failed to clear: %v", err)
	}

	entries, err = repo.GetHistory(10, 0)
	if err != nil {
		t.Fatalf("failed to get history after clear: %v", err)
	}
	if len(entries) != 0 {
		t.Errorf("expected 0 entries after clear, got %d", len(entries))
	}
}

func TestSqliteHistoryRepository_AutoDirCreation(t *testing.T) {
	// Verifies directories are created automatically
	tempDir := t.TempDir()
	nestedDbPath := filepath.Join(tempDir, "nested", "sub", "history.db")

	repo, err := NewSqliteHistoryRepository(nestedDbPath)
	if err != nil {
		t.Fatalf("failed to initialize database in nested dir: %v", err)
	}
	defer repo.Close()

	if _, err := os.Stat(nestedDbPath); os.IsNotExist(err) {
		t.Error("database file was not created")
	}
}
