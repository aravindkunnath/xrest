package importlib

import (
	"os"
	"path/filepath"
	"testing"

	"xrest/internal/adapters"
	"xrest/internal/models"

	"gopkg.in/yaml.v3"
)

func TestLoadSettings_CreatesDefaults(t *testing.T) {
	settingsPath := filepath.Join(t.TempDir(), "settings.yaml")
	settings, err := LoadSettings(settingsPath)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if settings.Theme != "system" {
		t.Errorf("expected theme 'system', got %s", settings.Theme)
	}
	if len(settings.Services) != 0 {
		t.Errorf("expected 0 services, got %d", len(settings.Services))
	}

	// Verify file was created
	if _, err := os.Stat(settingsPath); os.IsNotExist(err) {
		t.Error("expected settings.yaml to be created on disk")
	}
}

func TestSaveAndLoadSettings(t *testing.T) {
	settingsPath := filepath.Join(t.TempDir(), "settings.yaml")
	settings := UserSettings{
		Theme: "dark",
		Services: []models.ServiceStub{
			{ID: "s1", Name: "Service 1", Directory: "/tmp/s1"},
			{ID: "s2", Name: "Service 2", Directory: "/tmp/s2"},
		},
	}
	if err := SaveSettings(settingsPath, settings); err != nil {
		t.Fatalf("expected no error saving settings, got %v", err)
	}

	loaded, err := LoadSettings(settingsPath)
	if err != nil {
		t.Fatalf("expected no error loading settings, got %v", err)
	}
	if loaded.Theme != "dark" {
		t.Errorf("expected theme 'dark', got %s", loaded.Theme)
	}
	if len(loaded.Services) != 2 {
		t.Fatalf("expected 2 services, got %d", len(loaded.Services))
	}
	if loaded.Services[0].ID != "s1" {
		t.Errorf("expected service ID s1, got %s", loaded.Services[0].ID)
	}
}

func TestServiceManager_LoadAndSave(t *testing.T) {
	sm := NewServiceManager()
	dir := t.TempDir()

	// Create a full service structure
	svc := models.Service{
		ID:              "s-test",
		Name:            "Test Service",
		Directory:       dir,
		IsAuthenticated: false,
		AuthType:        ptr(models.AuthNone),
		Auth: &models.AuthConfig{
			Type:   "none",
			Active: true,
		},
		Environments: []models.EnvironmentConfig{
			{
				Name:     "DEV",
				IsUnsafe: false,
				Variables: []models.Variable{
					{Name: "BASE_URL", Value: "http://localhost:3000", Enabled: true, Type: "plain"},
				},
			},
		},
		Endpoints: []models.Endpoint{
			{
				ID:            "e-1",
				ServiceID:     "s-test",
				Name:          "Get Users",
				Method:        "GET",
				URL:           "/users",
				Authenticated: false,
				AuthType:      "none",
				Metadata: models.EndpointMetadata{
					Version:     "1.0",
					LastUpdated: 1000,
				},
				Params: []models.Param{
					{Name: "page", Value: "1", Enabled: true, Type: "plain"},
				},
			},
		},
		SelectedEnvironment: ptr("DEV"),
	}

	// Save
	if err := sm.SaveService(&svc, "", nil); err != nil {
		t.Fatalf("expected no error saving service, got %v", err)
	}

	// Verify files exist
	if _, err := os.Stat(filepath.Join(dir, "service.yaml")); os.IsNotExist(err) {
		t.Error("expected service.yaml to exist")
	}
	if _, err := os.Stat(filepath.Join(dir, "environments.yaml")); os.IsNotExist(err) {
		t.Error("expected environments.yaml to exist")
	}
	if _, err := os.Stat(filepath.Join(dir, "endpoints", "e-1.yaml")); os.IsNotExist(err) {
		t.Error("expected endpoints/e-1.yaml to exist")
	}

	// Load back
	loaded, err := sm.LoadService(dir)
	if err != nil {
		t.Fatalf("expected no error loading service, got %v", err)
	}
	if loaded.ID != "s-test" {
		t.Errorf("expected ID s-test, got %s", loaded.ID)
	}
	if loaded.Name != "Test Service" {
		t.Errorf("expected name 'Test Service', got %s", loaded.Name)
	}
	if len(loaded.Endpoints) != 1 {
		t.Fatalf("expected 1 endpoint, got %d", len(loaded.Endpoints))
	}
	if loaded.Endpoints[0].Name != "Get Users" {
		t.Errorf("expected endpoint name 'Get Users', got %s", loaded.Endpoints[0].Name)
	}
	if len(loaded.Environments) != 1 {
		t.Fatalf("expected 1 environment, got %d", len(loaded.Environments))
	}
	if loaded.Environments[0].Name != "DEV" {
		t.Errorf("expected environment name 'DEV', got %s", loaded.Environments[0].Name)
	}
}

func TestServiceManager_LoadServiceFileNotFound(t *testing.T) {
	sm := NewServiceManager()
	_, err := sm.LoadService(t.TempDir())
	if err == nil {
		t.Fatal("expected error when service.yaml does not exist")
	}
}

func TestServiceManager_SaveWithGitCommit(t *testing.T) {
	sm := NewServiceManager()
	dir := t.TempDir()

	// Init git repo
	git := adapters.NewGitAdapter()
	if err := git.Init(dir, ""); err != nil {
		t.Fatalf("failed to init git: %v", err)
	}

	svc := models.Service{
		ID:        "s-git-test",
		Name:      "Git Test",
		Directory: dir,
	}

	// Save with commit message
	if err := sm.SaveService(&svc, "Initial import", git); err != nil {
		t.Fatalf("expected no error saving with git, got %v", err)
	}

	// Verify git has a commit
	status, err := git.Status(dir)
	if err != nil {
		t.Fatalf("expected no error getting git status, got %v", err)
	}
	if !status.IsGit {
		t.Error("expected the directory to be a git repo")
	}
}

func TestImportDomain_ImportFromDirectory(t *testing.T) {
	domain := NewImportDomain()
	settingsPath := filepath.Join(t.TempDir(), "settings.yaml")
	dir := t.TempDir()

	// Create service files
	svcFile := map[string]interface{}{
		"id":   "s-import-test",
		"name": "Imported Service",
	}
	data, _ := yaml.Marshal(svcFile)
	if err := os.WriteFile(filepath.Join(dir, "service.yaml"), data, 0644); err != nil {
		t.Fatalf("failed to write service.yaml: %v", err)
	}

	service, err := domain.ImportFromDirectory(settingsPath, dir, nil)
	if err != nil {
		t.Fatalf("expected no error importing directory, got %v", err)
	}
	if service.Name != "Imported Service" {
		t.Errorf("expected 'Imported Service', got %s", service.Name)
	}

	// Verify it was saved to settings
	settings, err := LoadSettings(settingsPath)
	if err != nil {
		t.Fatalf("expected no error loading settings, got %v", err)
	}
	if len(settings.Services) != 1 {
		t.Errorf("expected 1 service in settings, got %d", len(settings.Services))
	}
}

func TestImportDomain_ImportFromDirectory_Duplicate(t *testing.T) {
	domain := NewImportDomain()
	settingsPath := filepath.Join(t.TempDir(), "settings.yaml")
	dir := t.TempDir()

	svcFile := map[string]interface{}{"id": "s-dup", "name": "Duplicate"}
	data, _ := yaml.Marshal(svcFile)
	os.WriteFile(filepath.Join(dir, "service.yaml"), data, 0644)

	// First import should succeed
	_, err := domain.ImportFromDirectory(settingsPath, dir, nil)
	if err != nil {
		t.Fatalf("expected no error on first import, got %v", err)
	}

	// Second import should fail with duplicate error
	_, err = domain.ImportFromDirectory(settingsPath, dir, nil)
	if err == nil {
		t.Fatal("expected error for duplicate import")
	}
}

func TestImportDomain_ImportFromSwagger(t *testing.T) {
	domain := NewImportDomain()
	settingsPath := filepath.Join(t.TempDir(), "settings.yaml")
	dir := t.TempDir()

	spec := `{
		"openapi": "3.0.0",
		"info": {"title": "Pet Store", "version": "1.0.0"},
		"paths": {
			"/pets": {
				"get": {"summary": "List pets", "responses": {"200": {"description": "OK"}}}
			}
		}
	}`

	service, err := domain.ImportFromSwagger(settingsPath, "Pet Store", dir, spec, nil)
	if err != nil {
		t.Fatalf("expected no error importing swagger, got %v", err)
	}
	if service.Name != "Pet Store" {
		t.Errorf("expected name 'Pet Store', got %s", service.Name)
	}
	if len(service.Endpoints) != 1 {
		t.Fatalf("expected 1 endpoint, got %d", len(service.Endpoints))
	}
	if service.Endpoints[0].Name != "List pets" {
		t.Errorf("expected 'List pets', got %s", service.Endpoints[0].Name)
	}

	// Verify settings were saved
	settings, err := LoadSettings(settingsPath)
	if err != nil {
		t.Fatalf("expected no error loading settings, got %v", err)
	}
	if len(settings.Services) != 1 {
		t.Errorf("expected 1 service in settings, got %d", len(settings.Services))
	}
}

func TestImportDomain_ImportCurlEndpoint(t *testing.T) {
	domain := NewImportDomain()
	settingsPath := filepath.Join(t.TempDir(), "settings.yaml")
	dir := t.TempDir()

	// First create a service via swagger so it exists in settings
	svc, err := domain.ImportFromSwagger(settingsPath, "Test API", dir, `{
		"openapi": "3.0.0",
		"info": {"title": "Test", "version": "1.0"},
		"paths": {"/items": {"get": {"summary": "List items", "responses": {"200": {"description": "OK"}}}}}
	}`, nil)
	if err != nil {
		t.Fatalf("failed to create base service: %v", err)
	}

	// Now import a curl endpoint
	updated, err := domain.ImportCurlEndpoint(settingsPath, svc.ID, "curl -X POST -d '{\"name\":\"new\"}' http://api.example.com/items", nil)
	if err != nil {
		t.Fatalf("expected no error importing curl, got %v", err)
	}
	if len(updated.Endpoints) != 2 {
		t.Fatalf("expected 2 endpoints after curl import, got %d", len(updated.Endpoints))
	}
	if updated.Endpoints[1].Method != "POST" {
		t.Errorf("expected curl endpoint method POST, got %s", updated.Endpoints[1].Method)
	}
}

func TestImportDomain_ImportCurlEndpoint_ServiceNotFound(t *testing.T) {
	domain := NewImportDomain()
	settingsPath := filepath.Join(t.TempDir(), "settings.yaml")

	_, err := domain.ImportCurlEndpoint(settingsPath, "s-nonexistent", "curl http://example.com", nil)
	if err == nil {
		t.Fatal("expected error for nonexistent service")
	}
}

func TestDefaultEnvironments(t *testing.T) {
	envs := defaultEnvironments("https://api.example.com")
	if len(envs) != 3 {
		t.Fatalf("expected 3 environments, got %d", len(envs))
	}
	if envs[0].Name != "DEV" {
		t.Errorf("expected DEV, got %s", envs[0].Name)
	}
	if envs[1].Name != "STAGE" {
		t.Errorf("expected STAGE, got %s", envs[1].Name)
	}
	if envs[2].Name != "PROD" {
		t.Errorf("expected PROD, got %s", envs[2].Name)
	}
	if envs[2].IsUnsafe != true {
		t.Error("expected PROD to be unsafe")
	}
}

func TestStorageConversions(t *testing.T) {
	// Test ToStorageServiceFile and round-trip
	svc := &models.Service{
		ID:   "s1",
		Name: "Test",
	}

	sf := ToStorageServiceFile(svc)
	if sf.ID != "s1" {
		t.Errorf("expected ID s1, got %s", sf.ID)
	}

	// Test ToStorageEndpoint
	ep := models.Endpoint{
		ID:   "e1",
		Name: "Test EP",
	}
	storageEp := ToStorageEndpoint(ep)
	if storageEp.ID != "e1" {
		t.Errorf("expected ID e1, got %s", storageEp.ID)
	}

	// Test conversion back
	modelEp := storageEp.ToModelEndpoint()
	if modelEp.ID != "e1" {
		t.Errorf("expected ID e1, got %s", modelEp.ID)
	}
}

func TestPtrHelper(t *testing.T) {
	v := ptr("hello")
	if *v != "hello" {
		t.Errorf("expected hello, got %s", *v)
	}
}
