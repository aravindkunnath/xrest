package importlib

import (
	"fmt"
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

// AppConfig matches the application configuration format.
type AppConfig struct {
	Version   string `yaml:"version"`
	ZoomLevel int    `yaml:"zoomLevel"`
}

// ConfigPath returns the path to the config.yaml file.
func ConfigPath() string {
	if os.Getenv("XREST_ENV") == "test" {
		return filepath.Join(os.TempDir(), "xrest-test", "config.yaml")
	}
	return filepath.Join(os.Getenv("HOME"), ".xrest", "config.yaml")
}

// LoadConfig reads config from the YAML file, returning defaults if missing.
func LoadConfig(path string) (AppConfig, error) {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		cfg := AppConfig{
			Version:   "0.0.1",
			ZoomLevel: 0,
		}
		if err := SaveConfig(path, cfg); err != nil {
			return cfg, err
		}
		return cfg, nil
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return AppConfig{Version: "0.0.1", ZoomLevel: 0}, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg AppConfig
	if err := yaml.Unmarshal(data, &cfg); err != nil {
		return AppConfig{Version: "0.0.1", ZoomLevel: 0}, nil
	}
	return cfg, nil
}

// SaveConfig writes config to the YAML file.
func SaveConfig(path string, cfg AppConfig) error {
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create config dir: %w", err)
	}

	data, err := yaml.Marshal(&cfg)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	return os.WriteFile(path, data, 0644)
}
