package main

import (
	"log"
	importlib "xrest/internal/import"
	"xrest/internal/models"
)

// SettingsGateway handles user settings and tab state operations.
type SettingsGateway struct{}

// NewSettingsGateway creates a new SettingsGateway.
func NewSettingsGateway() *SettingsGateway {
	return &SettingsGateway{}
}

// LoadSettings loads the user settings.
func (g *SettingsGateway) LoadSettings() (importlib.UserSettings, error) {
	log.Println("[SettingsGateway] LoadSettings called")
	return importlib.LoadSettings(settingsPath())
}

// SaveSettings saves the user settings.
func (g *SettingsGateway) SaveSettings(settings importlib.UserSettings) error {
	log.Println("[SettingsGateway] SaveSettings called")
	return importlib.SaveSettings(settingsPath(), settings)
}

// LoadTabState loads the tab state.
func (g *SettingsGateway) LoadTabState() (*models.TabState, error) {
	log.Println("[SettingsGateway] LoadTabState called")
	return importlib.LoadTabState(importlib.TabStatePath())
}

// SaveTabState saves the tab state.
func (g *SettingsGateway) SaveTabState(state *models.TabState) error {
	log.Println("[SettingsGateway] SaveTabState called")
	return importlib.SaveTabState(importlib.TabStatePath(), state)
}

// UpdateTheme updates the user's theme in the settings file.
func (g *SettingsGateway) UpdateTheme(theme string) error {
	log.Printf("[SettingsGateway] UpdateTheme called: %s\n", theme)
	return importlib.UpdateTheme(settingsPath(), theme)
}

// LoadZoomLevel loads the zoom level configuration.
func (g *SettingsGateway) LoadZoomLevel() (int, error) {
	log.Println("[SettingsGateway] LoadZoomLevel called")
	cfg, err := importlib.LoadConfig(importlib.ConfigPath())
	if err != nil {
		return 0, err
	}
	return cfg.ZoomLevel, nil
}

// SaveZoomLevel saves the zoom level configuration.
func (g *SettingsGateway) SaveZoomLevel(level int) error {
	log.Printf("[SettingsGateway] SaveZoomLevel called: %d\n", level)
	path := importlib.ConfigPath()
	cfg, err := importlib.LoadConfig(path)
	if err != nil {
		return err
	}
	cfg.ZoomLevel = level
	return importlib.SaveConfig(path, cfg)
}

