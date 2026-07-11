package importlib

import (
	"os"
	"path/filepath"
	"testing"
	"xrest/internal/models"
)

func TestTabStatePath_RespectsTestEnv(t *testing.T) {
	os.Setenv("XREST_ENV", "test")
	path := TabStatePath()
	expectedPrefix := filepath.Join(os.TempDir(), "xrest-test")
	if !filepath.HasPrefix(path, expectedPrefix) {
		t.Errorf("expected path to start with %s, got %s", expectedPrefix, path)
	}
}

func TestLoadTabState_NonExistentReturnsNil(t *testing.T) {
	tempPath := filepath.Join(t.TempDir(), "nonexistent_tab_state.yaml")
	state, err := LoadTabState(tempPath)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if state != nil {
		t.Errorf("expected nil tab state, got %+v", state)
	}
}

func TestSaveAndLoadTabState(t *testing.T) {
	tempPath := filepath.Join(t.TempDir(), "tab_state.yaml")
	originalState := &models.TabState{
		ActiveTabID: "tab-123",
		Tabs: []models.Tab{
			{
				Type: "request",
				Request: &models.RequestTab{
					ID:    "tab-123",
					Title: "Test Tab",
				},
			},
		},
	}

	err := SaveTabState(tempPath, originalState)
	if err != nil {
		t.Fatalf("failed to save tab state: %v", err)
	}

	loadedState, err := LoadTabState(tempPath)
	if err != nil {
		t.Fatalf("failed to load tab state: %v", err)
	}

	if loadedState == nil {
		t.Fatal("expected loaded tab state to be non-nil")
	}

	if loadedState.ActiveTabID != originalState.ActiveTabID {
		t.Errorf("expected active tab id %s, got %s", originalState.ActiveTabID, loadedState.ActiveTabID)
	}

	if len(loadedState.Tabs) != 1 {
		t.Fatalf("expected 1 tab, got %d", len(loadedState.Tabs))
	}

	if loadedState.Tabs[0].Request == nil || loadedState.Tabs[0].Request.Title != "Test Tab" {
		t.Errorf("expected tab name 'Test Tab', got %v", loadedState.Tabs[0].Request)
	}
}

func TestUpdateTheme(t *testing.T) {
	tempSettingsPath := filepath.Join(t.TempDir(), "settings.yaml")

	// Create initial default settings
	_, err := LoadSettings(tempSettingsPath)
	if err != nil {
		t.Fatalf("failed to create default settings: %v", err)
	}

	err = UpdateTheme(tempSettingsPath, "dark")
	if err != nil {
		t.Fatalf("failed to update theme: %v", err)
	}

	settings, err := LoadSettings(tempSettingsPath)
	if err != nil {
		t.Fatalf("failed to reload settings: %v", err)
	}

	if settings.Theme != "dark" {
		t.Errorf("expected theme 'dark', got %s", settings.Theme)
	}
}
