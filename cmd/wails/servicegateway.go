package main

import (
	"log"
	"xrest/internal/adapters"
	"xrest/internal/models"
)

// ServiceGateway handles service and Git repository management operations.
type ServiceGateway struct{}

// NewServiceGateway creates a new ServiceGateway.
func NewServiceGateway() *ServiceGateway {
	return &ServiceGateway{}
}

// LoadServices returns all stored services.
func (s *ServiceGateway) LoadServices() ([]models.Service, error) {
	log.Println("[ServiceGateway] LoadServices called")
	return []models.Service{}, nil
}

// SaveServices persists services with an optional commit message.
func (s *ServiceGateway) SaveServices(services []models.Service, commitMessage string) ([]models.Service, error) {
	log.Printf("[ServiceGateway] SaveServices called with %d services, message: %q\n", len(services), commitMessage)
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
	return models.Service{
		ID:        "s-imported",
		Name:      "Imported Service",
		Directory: directory,
	}, nil
}

// ImportCurl imports endpoints into a service from a cURL command.
func (s *ServiceGateway) ImportCurl(serviceId string, curlCommand string) (models.Service, error) {
	log.Printf("[ServiceGateway] ImportCurl called for service %s: %s\n", serviceId, curlCommand)
	return models.Service{
		ID:   serviceId,
		Name: "Imported Curl Service",
	}, nil
}

// TestPreflightConfig runs a test for the given preflight configuration.
func (s *ServiceGateway) TestPreflightConfig(config *models.PreflightConfig) (string, error) {
	log.Printf("[ServiceGateway] TestPreflightConfig called\n")
	client := &adapters.Http{}
	return client.TestPreflightConfig(config)
}

// ImportSwagger imports a service definition from a Swagger/OpenAPI file.
func (s *ServiceGateway) ImportSwagger(serviceId string, filePath string) (models.Service, error) {
	log.Printf("[ServiceGateway] ImportSwagger called for service %s, file: %s\n", serviceId, filePath)
	return models.Service{
		ID:   serviceId,
		Name: "Imported Swagger Service",
	}, nil
}
