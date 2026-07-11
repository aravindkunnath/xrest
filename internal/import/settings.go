package importlib

import (
	"fmt"
	"os"
	"path/filepath"
	"xrest/internal/models"

	"gopkg.in/yaml.v3"
)

// TabStatePath returns the path to the tab state YAML file.
// Respects XREST_ENV=test for isolated test environments.
func TabStatePath() string {
	if os.Getenv("XREST_ENV") == "test" {
		return filepath.Join(os.TempDir(), "xrest-test", "tab_state.yaml")
	}
	return filepath.Join(os.Getenv("HOME"), ".config", "xrest", "tab_state.yaml")
}

// LoadTabState reads TabState from the YAML file. Returns nil, nil if the file does not exist.
func LoadTabState(path string) (*models.TabState, error) {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		return nil, nil
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read tab state: %w", err)
	}

	var state models.TabState
	if err := yaml.Unmarshal(data, &state); err != nil {
		return nil, fmt.Errorf("failed to unmarshal tab state: %w", err)
	}

	return &state, nil
}

// SaveTabState writes TabState to the YAML file.
func SaveTabState(path string, state *models.TabState) error {
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create tab state dir: %w", err)
	}

	data, err := yaml.Marshal(state)
	if err != nil {
		return fmt.Errorf("failed to marshal tab state: %w", err)
	}

	return os.WriteFile(path, data, 0644)
}

// UpdateTheme loads current settings, updates the theme, and saves back.
func UpdateTheme(settingsPath string, theme string) error {
	settings, err := LoadSettings(settingsPath)
	if err != nil {
		return fmt.Errorf("failed to load settings for theme update: %w", err)
	}
	settings.Theme = theme
	if err := SaveSettings(settingsPath, settings); err != nil {
		return fmt.Errorf("failed to save settings for theme update: %w", err)
	}
	return nil
}
