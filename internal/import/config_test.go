package importlib

import (
	"os"
	"path/filepath"
	"testing"
)

func TestConfigLoadAndSave(t *testing.T) {
	os.Setenv("XREST_ENV", "test")
	defer os.Unsetenv("XREST_ENV")

	path := ConfigPath()
	defer os.RemoveAll(filepath.Dir(path))

	cfg, err := LoadConfig(path)
	if err != nil {
		t.Fatalf("expected no error loading, got %v", err)
	}

	if cfg.ZoomLevel != 0 {
		t.Errorf("expected zoom level 0, got %d", cfg.ZoomLevel)
	}

	cfg.ZoomLevel = 3
	err = SaveConfig(path, cfg)
	if err != nil {
		t.Fatalf("expected no error saving, got %v", err)
	}

	loaded, err := LoadConfig(path)
	if err != nil {
		t.Fatalf("expected no error loading, got %v", err)
	}

	if loaded.ZoomLevel != 3 {
		t.Errorf("expected loaded zoom level 3, got %d", loaded.ZoomLevel)
	}
}
