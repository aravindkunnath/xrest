package importlib

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	"xrest/internal/adapters"
	"xrest/internal/models"

	"github.com/google/uuid"
	"gopkg.in/yaml.v3"
)

// SettingsPath returns the path to the user settings YAML file.
// Respects XREST_ENV=test for isolated test environments.
func SettingsPath() string {
	if os.Getenv("XREST_ENV") == "test" {
		return filepath.Join(os.TempDir(), "xrest-test", "settings.yaml")
	}
	return filepath.Join(os.Getenv("HOME"), ".config", "xrest", "settings.yaml")
}

// UserSettings matches the Rust UserSettings YAML format.
type UserSettings struct {
	Theme    string               `yaml:"theme" json:"theme"`
	Services []models.ServiceStub `yaml:"services" json:"services"`
}

func defaultSettings() UserSettings {
	return UserSettings{
		Theme:    "system",
		Services: []models.ServiceStub{},
	}
}

// LoadSettings reads settings from the YAML file, creating defaults if missing.
func LoadSettings(settingsPath string) (UserSettings, error) {
	if _, err := os.Stat(settingsPath); os.IsNotExist(err) {
		settings := defaultSettings()
		if saveErr := SaveSettings(settingsPath, settings); saveErr != nil {
			return settings, saveErr
		}
		return settings, nil
	}

	data, err := os.ReadFile(settingsPath)
	if err != nil {
		return defaultSettings(), fmt.Errorf("failed to read settings: %w", err)
	}

	var settings UserSettings
	if err := yaml.Unmarshal(data, &settings); err != nil {
		// Fall back to defaults on parse error (like Rust does)
		return defaultSettings(), nil
	}
	return settings, nil
}

// SaveSettings writes settings to the YAML file.
func SaveSettings(settingsPath string, settings UserSettings) error {
	dir := filepath.Dir(settingsPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create settings dir: %w", err)
	}

	data, err := yaml.Marshal(&settings)
	if err != nil {
		return fmt.Errorf("failed to marshal settings: %w", err)
	}

	return os.WriteFile(settingsPath, data, 0644)
}

// ServiceManager handles service CRUD with file system persistence.
type ServiceManager struct{}

// NewServiceManager creates a new ServiceManager.
func NewServiceManager() *ServiceManager {
	return &ServiceManager{}
}

// LoadService reads a service from its directory (service.yaml + environments.yaml + endpoints/).
func (sm *ServiceManager) LoadService(directory string) (models.Service, error) {
	basePath := filepath.Clean(directory)

	// Read service.yaml
	svcFilePath := filepath.Join(basePath, "service.yaml")
	data, err := os.ReadFile(svcFilePath)
	if err != nil {
		return models.Service{}, fmt.Errorf("service file not found at %s: %w", svcFilePath, err)
	}

	var svcFile StorageServiceFile
	if err := yaml.Unmarshal(data, &svcFile); err != nil {
		return models.Service{}, fmt.Errorf("failed to parse service.yaml: %w", err)
	}

	// Read environments.yaml (optional)
	var environments []models.EnvironmentConfig
	envPath := filepath.Join(basePath, "environments.yaml")
	if envData, err := os.ReadFile(envPath); err == nil {
		if err := yaml.Unmarshal(envData, &environments); err != nil {
			// Non-fatal — log and continue with empty
			fmt.Printf("Warning: failed to parse environments.yaml: %v\n", err)
		}
	}

	// Load endpoints
	var endpoints []models.Endpoint
	endpointsDir := filepath.Join(basePath, "endpoints")
	if epDirInfo, err := os.Stat(endpointsDir); err == nil && epDirInfo.IsDir() {
		entries, err := os.ReadDir(endpointsDir)
		if err == nil {
			for _, entry := range entries {
				if entry.IsDir() || filepath.Ext(entry.Name()) != ".yaml" {
					continue
				}
				epData, err := os.ReadFile(filepath.Join(endpointsDir, entry.Name()))
				if err != nil {
					continue
				}
				var storageEp EndpointStorage
				if err := yaml.Unmarshal(epData, &storageEp); err != nil {
					continue
				}
				endpoints = append(endpoints, storageEp.ToModelEndpoint())
			}
		}
	}

	// If no individual endpoint files, try using stubs to load from files
	if len(endpoints) == 0 && len(svcFile.Endpoints) > 0 {
		for _, stub := range svcFile.Endpoints {
			epPath := filepath.Join(endpointsDir, stub.ID+".yaml")
			if epData, err := os.ReadFile(epPath); err == nil {
				var storageEp EndpointStorage
				if err := yaml.Unmarshal(epData, &storageEp); err == nil {
					endpoints = append(endpoints, storageEp.ToModelEndpoint())
				}
			}
		}
	}

	return svcFile.ToModelService(environments, endpoints), nil
}

// SaveService persists a service to its directory (service.yaml + environments.yaml + endpoints/*.yaml).
func (sm *ServiceManager) SaveService(service *models.Service, commitMsg string, git *adapters.GitAdapter) error {
	dir := filepath.Clean(service.Directory)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create service directory: %w", err)
	}

	// Save environments.yaml
	if len(service.Environments) > 0 {
		envPath := filepath.Join(dir, "environments.yaml")
		envData, err := yaml.Marshal(&service.Environments)
		if err != nil {
			return fmt.Errorf("failed to marshal environments: %w", err)
		}
		if err := os.WriteFile(envPath, envData, 0644); err != nil {
			return fmt.Errorf("failed to write environments.yaml: %w", err)
		}
	}

	// Save individual endpoint files
	endpointsDir := filepath.Join(dir, "endpoints")
	if len(service.Endpoints) > 0 {
		if err := os.MkdirAll(endpointsDir, 0755); err != nil {
			return fmt.Errorf("failed to create endpoints dir: %w", err)
		}
		for _, ep := range service.Endpoints {
			storageEp := ToStorageEndpoint(ep)
			epData, err := yaml.Marshal(&storageEp)
			if err != nil {
				return fmt.Errorf("failed to marshal endpoint %s: %w", ep.ID, err)
			}
			epPath := filepath.Join(endpointsDir, ep.ID+".yaml")
			if err := os.WriteFile(epPath, epData, 0644); err != nil {
				return fmt.Errorf("failed to write endpoint %s: %w", ep.ID, err)
			}
		}
	}

	// Clean up stale endpoint files
	if entries, err := os.ReadDir(endpointsDir); err == nil {
		validIDs := make(map[string]bool)
		for _, ep := range service.Endpoints {
			validIDs[ep.ID] = true
		}
		for _, entry := range entries {
			if !entry.IsDir() && filepath.Ext(entry.Name()) == ".yaml" {
				stubID := entry.Name()[:len(entry.Name())-5] // Remove .yaml
				if !validIDs[stubID] {
					os.Remove(filepath.Join(endpointsDir, entry.Name()))
				}
			}
		}
	}

	// Save service.yaml
	svcFile := ToStorageServiceFile(service)
	svcData, err := yaml.Marshal(&svcFile)
	if err != nil {
		return fmt.Errorf("failed to marshal service file: %w", err)
	}
	svcPath := filepath.Join(dir, "service.yaml")
	if err := os.WriteFile(svcPath, svcData, 0644); err != nil {
		return fmt.Errorf("failed to write service.yaml: %w", err)
	}

	// Git auto-commit
	if git != nil && git.IsRepo(service.Directory) {
		msg := commitMsg
		if msg == "" {
			msg = "Update service configuration"
		}
		_ = git.Commit(service.Directory, msg)
	}

	return nil
}

// ImportDomain is the main import orchestrator, mirroring the Rust ImportDomain.
type ImportDomain struct {
	svcManager *ServiceManager
}

// NewImportDomain creates a new ImportDomain.
func NewImportDomain() *ImportDomain {
	return &ImportDomain{
		svcManager: NewServiceManager(),
	}
}

// ImportFromDirectory imports a service from a directory.
// Equivalent to Rust's ImportDomain::import_from_directory().
func (d *ImportDomain) ImportFromDirectory(settingsPath, directory string, git *adapters.GitAdapter) (models.Service, error) {
	service, err := d.svcManager.LoadService(directory)
	if err != nil {
		return models.Service{}, fmt.Errorf("failed to load service from directory %s: %w", directory, err)
	}

	settings, err := LoadSettings(settingsPath)
	if err != nil {
		return models.Service{}, fmt.Errorf("failed to load settings: %w", err)
	}

	// Check for duplicates
	for _, s := range settings.Services {
		if s.Directory == directory {
			return models.Service{}, fmt.Errorf("this directory is already imported as a service")
		}
	}

	// Add stub to settings
	settings.Services = append(settings.Services, models.ServiceStub{
		ID:        service.ID,
		Name:      service.Name,
		Directory: directory,
	})

	if err := SaveSettings(settingsPath, settings); err != nil {
		return models.Service{}, fmt.Errorf("failed to save settings: %w", err)
	}

	// Save service (triggers git commit if applicable)
	serviceName := service.Name
	if err := d.svcManager.SaveService(&service, fmt.Sprintf("Import service: %s", serviceName), git); err != nil {
		return models.Service{}, fmt.Errorf("failed to save service: %w", err)
	}

	return service, nil
}

// ImportFromSwagger imports a service from a Swagger/OpenAPI spec.
// Equivalent to Rust's ImportDomain::import_from_swagger().
func (d *ImportDomain) ImportFromSwagger(settingsPath, name, directory, content string, git *adapters.GitAdapter) (models.Service, error) {
	serviceID := fmt.Sprintf("s-%d", time.Now().UnixMilli())

	baseURL, endpoints, err := ParseSpecContent(content, serviceID)
	if err != nil {
		return models.Service{}, fmt.Errorf("failed to parse spec: %w", err)
	}

	// Set service ID on each endpoint
	for i := range endpoints {
		endpoints[i].ServiceID = serviceID
	}

	service := models.Service{
		ID:              serviceID,
		Name:            name,
		Directory:       directory,
		IsAuthenticated: false,
		AuthType:        ptr(models.AuthNone),
		Auth: &models.AuthConfig{
			Type:   "none",
			Active: true,
		},
		Environments:        defaultEnvironments(baseURL),
		Endpoints:           endpoints,
		SelectedEnvironment: ptr("DEV"),
	}

	serviceName := service.Name
	if err := d.svcManager.SaveService(&service, fmt.Sprintf("Import service from Swagger: %s", serviceName), git); err != nil {
		return models.Service{}, fmt.Errorf("failed to save service: %w", err)
	}

	settings, err := LoadSettings(settingsPath)
	if err != nil {
		return models.Service{}, fmt.Errorf("failed to load settings: %w", err)
	}

	settings.Services = append(settings.Services, models.ServiceStub{
		ID:        service.ID,
		Name:      service.Name,
		Directory: directory,
	})

	if err := SaveSettings(settingsPath, settings); err != nil {
		return models.Service{}, fmt.Errorf("failed to save settings: %w", err)
	}

	return service, nil
}

// ImportCurlEndpoint imports a cURL command into an existing service.
// Equivalent to Rust's ImportDomain::import_curl_endpoint().
func (d *ImportDomain) ImportCurlEndpoint(settingsPath, serviceID, curlCommand string, git *adapters.GitAdapter) (models.Service, error) {
	settings, err := LoadSettings(settingsPath)
	if err != nil {
		return models.Service{}, fmt.Errorf("failed to load settings: %w", err)
	}

	var directory string
	for _, s := range settings.Services {
		if s.ID == serviceID {
			directory = s.Directory
			break
		}
	}
	if directory == "" {
		return models.Service{}, fmt.Errorf("service not found: %s", serviceID)
	}

	service, err := d.svcManager.LoadService(directory)
	if err != nil {
		return models.Service{}, fmt.Errorf("failed to load service: %w", err)
	}

	authType := "none"
	if service.AuthType != nil {
		authType = string(*service.AuthType)
	}

	endpoint, err := CurlToEndpoint(serviceID, curlCommand, service.IsAuthenticated, &authType)
	if err != nil {
		return models.Service{}, fmt.Errorf("failed to parse curl: %w", err)
	}

	endpointName := endpoint.Name
	service.Endpoints = append(service.Endpoints, endpoint)

	if err := d.svcManager.SaveService(&service, fmt.Sprintf("Import endpoint from cURL: %s", endpointName), git); err != nil {
		return models.Service{}, fmt.Errorf("failed to save service: %w", err)
	}

	return service, nil
}

// ----- Helpers -----

func defaultEnvironments(baseURL string) []models.EnvironmentConfig {
	return []models.EnvironmentConfig{
		{
			Name:     "DEV",
			IsUnsafe: false,
			Variables: []models.Variable{
				{Name: "BASE_URL", Value: baseURL, Enabled: true, Type: "plain"},
			},
		},
		{
			Name:     "STAGE",
			IsUnsafe: false,
			Variables: []models.Variable{
				{Name: "BASE_URL", Value: baseURL, Enabled: true, Type: "plain"},
			},
		},
		{
			Name:     "PROD",
			IsUnsafe: true,
			Variables: []models.Variable{
				{Name: "BASE_URL", Value: baseURL, Enabled: true, Type: "plain"},
			},
		},
	}
}

func ptr[T any](v T) *T {
	return &v
}

// Ensure uuid is used
var _ = uuid.NewString
