package main

import (
	"fmt"
	"log"
	"os"
	"path/filepath"
	"xrest/internal/adapters"
	importlib "xrest/internal/import"
	"xrest/internal/models"

	"github.com/adrg/xdg"
)

// ServiceGateway handles service and Git repository management operations.
type ServiceGateway struct{}

// NewServiceGateway creates a new ServiceGateway.
func NewServiceGateway() *ServiceGateway {
	return &ServiceGateway{}
}

// settingsPath returns the path to the user settings YAML file.
var settingsPath = func() string {
	if os.Getenv("XREST_ENV") == "test" {
		return filepath.Join(os.TempDir(), "xrest-test", "settings.yaml")
	}
	return filepath.Join(xdg.ConfigHome, "xrest", "settings.yaml")
}

// LoadServices returns all stored services.
func (s *ServiceGateway) LoadServices() ([]models.Service, error) {
	log.Println("[ServiceGateway] LoadServices called")
	settings, err := importlib.LoadSettings(settingsPath())
	if err != nil {
		return nil, fmt.Errorf("failed to load settings: %w", err)
	}

	svcManager := importlib.NewServiceManager()
	var services []models.Service
	for _, stub := range settings.Services {
		svc, err := svcManager.LoadService(stub.Directory)
		if err != nil {
			log.Printf("[ServiceGateway] Warning: failed to load service %s (%s): %v\n", stub.Name, stub.Directory, err)
			continue
		}
		services = append(services, svc)
	}
	return services, nil
}

// SaveServices persists services with an optional commit message.
func (s *ServiceGateway) SaveServices(services []models.Service, commitMessage string) ([]models.Service, error) {
	log.Printf("[ServiceGateway] SaveServices called with %d services, message: %q\n", len(services), commitMessage)

	settings, err := importlib.LoadSettings(settingsPath())
	if err != nil {
		return nil, err
	}

	var stubs []models.ServiceStub
	svcManager := importlib.NewServiceManager()
	git := adapters.NewGitAdapter()

	for i := range services {
		svc := services[i]
		if err := svcManager.SaveService(&svc, commitMessage, git); err != nil {
			return nil, fmt.Errorf("failed to save service %s: %w", svc.Name, err)
		}
		stubs = append(stubs, models.ServiceStub{
			ID:        svc.ID,
			Name:      svc.Name,
			Directory: svc.Directory,
		})
		services[i] = svc
	}

	settings.Services = stubs
	if err := importlib.SaveSettings(settingsPath(), settings); err != nil {
		return nil, fmt.Errorf("failed to save settings: %w", err)
	}

	return services, nil
}

// GetGitStatus retrieves the Git status of the specified directory.
func (s *ServiceGateway) GetGitStatus(directory string) (models.GitStatus, error) {
	log.Printf("[ServiceGateway] GetGitStatus called for: %s\n", directory)
	git := adapters.NewGitAdapter()
	return git.Status(directory)
}

// InitGit initializes a Git repository in the directory.
func (s *ServiceGateway) InitGit(directory string, remoteUrl string) error {
	log.Printf("[ServiceGateway] InitGit called for: %s, remote: %s\n", directory, remoteUrl)
	git := adapters.NewGitAdapter()
	return git.Init(directory, remoteUrl)
}

// SyncGit synchronizes the Git repository (pull then push).
func (s *ServiceGateway) SyncGit(directory string) error {
	log.Printf("[ServiceGateway] SyncGit called for: %s\n", directory)
	git := adapters.NewGitAdapter()
	return git.Sync(directory)
}

// PullGit pulls from the remote Git repository.
func (s *ServiceGateway) PullGit(directory string) error {
	log.Printf("[ServiceGateway] PullGit called for: %s\n", directory)
	git := adapters.NewGitAdapter()
	return git.Pull(directory)
}

// PushGit pushes commits to the remote Git repository.
func (s *ServiceGateway) PushGit(directory string) error {
	log.Printf("[ServiceGateway] PushGit called for: %s\n", directory)
	git := adapters.NewGitAdapter()
	return git.Push(directory)
}

// CommitGit commits uncommitted changes to the Git repository.
func (s *ServiceGateway) CommitGit(directory string, message string) error {
	log.Printf("[ServiceGateway] CommitGit called for: %s, message: %s\n", directory, message)
	git := adapters.NewGitAdapter()
	return git.Commit(directory, message)
}

// ImportService imports a service from the specified directory.
func (s *ServiceGateway) ImportService(directory string) (models.Service, error) {
	log.Printf("[ServiceGateway] ImportService called for: %s\n", directory)
	domain := importlib.NewImportDomain()
	git := adapters.NewGitAdapter()
	return domain.ImportFromDirectory(settingsPath(), directory, git)
}

// ImportCurl imports endpoints into a service from a cURL command.
func (s *ServiceGateway) ImportCurl(serviceId string, curlCommand string) (models.Service, error) {
	log.Printf("[ServiceGateway] ImportCurl called for service %s: %s\n", serviceId, curlCommand)
	domain := importlib.NewImportDomain()
	git := adapters.NewGitAdapter()
	return domain.ImportCurlEndpoint(settingsPath(), serviceId, curlCommand, git)
}

// TestPreflightConfig runs a test for the given preflight configuration.
func (s *ServiceGateway) TestPreflightConfig(config *models.PreflightConfig) (string, error) {
	log.Printf("[ServiceGateway] TestPreflightConfig called\n")
	client := &adapters.Http{}
	return client.TestPreflightConfig(config)
}

// ImportSwagger imports a service definition from a Swagger/OpenAPI file.
// The frontend passes (name string, filePath string) where filePath may be a URL or file path.
func (s *ServiceGateway) ImportSwagger(name string, filePath string) (models.Service, error) {
	log.Printf("[ServiceGateway] ImportSwagger called for name: %s, file: %s\n", name, filePath)

	var content string
	var directory string

	if isURL(filePath) {
		// Fetch from URL
		client := &adapters.Http{}
		resp, err := client.Send(&models.Request{
			Method: "GET",
			URL:    filePath,
		})
		if err != nil {
			return models.Service{}, fmt.Errorf("failed to fetch swagger URL: %w", err)
		}
		content = resp.Body
		directory = filepath.Join(xdg.ConfigHome, "xrest", "services", name)
	} else {
		// Read local file
		data, err := os.ReadFile(filePath)
		if err != nil {
			return models.Service{}, fmt.Errorf("failed to read swagger file: %w", err)
		}
		content = string(data)
		directory = filepath.Dir(filePath)
	}

	domain := importlib.NewImportDomain()
	git := adapters.NewGitAdapter()
	return domain.ImportFromSwagger(settingsPath(), name, directory, content, git)
}

// isURL checks if a string looks like a URL.
func isURL(s string) bool {
	return len(s) > 8 && (s[:7] == "http://" || s[:8] == "https://")
}
