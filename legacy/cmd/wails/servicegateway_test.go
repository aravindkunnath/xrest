package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"testing"
	"xrest/internal/models"

	"gopkg.in/yaml.v3"
)

func TestServiceGateway(t *testing.T) {
	sg := NewServiceGateway()

	// Override settings path to use isolated temp dir
	origSettingsPath := settingsPath
	settingsDir := t.TempDir()
	testSettingsPath := filepath.Join(settingsDir, "settings.yaml")
	settingsPath = func() string { return testSettingsPath }
	defer func() { settingsPath = origSettingsPath }()

	// 1. LoadServices returns empty initially
	svcs, err := sg.LoadServices()
	if err != nil {
		t.Fatalf("expected no error loading services, got %v", err)
	}
	if len(svcs) != 0 {
		t.Errorf("expected 0 loaded services, got %d", len(svcs))
	}

	// Create a service directory with proper files
	svcDir := t.TempDir()
	svcID := "s-test-1"

	// Write service.yaml
	svcFile := map[string]interface{}{
		"id":              svcID,
		"name":            "Test Service",
		"isAuthenticated": false,
		"authType":        "none",
		"auth": map[string]interface{}{
			"type":           "none",
			"active":         true,
			"basicUser":      "",
			"basicPass":      "",
			"bearerToken":    "",
			"apiKeyName":     "",
			"apiKeyValue":    "",
			"apiKeyLocation": "header",
		},
		"endpoints": []interface{}{},
		"directory": svcDir,
	}
	svcData, _ := yaml.Marshal(svcFile)
	if err := os.WriteFile(filepath.Join(svcDir, "service.yaml"), svcData, 0644); err != nil {
		t.Fatalf("failed to write service.yaml: %v", err)
	}

	// Write environments.yaml
	envs := []models.EnvironmentConfig{
		{
			Name:     "DEV",
			IsUnsafe: false,
			Variables: []models.Variable{
				{Name: "BASE_URL", Value: "http://localhost:3000", Enabled: true, Type: "plain"},
			},
		},
	}
	envData, _ := yaml.Marshal(envs)
	os.WriteFile(filepath.Join(svcDir, "environments.yaml"), envData, 0644)

	// 2. SaveServices persists the service to a separate directory
	saveDir := t.TempDir()
	testSvcs := []models.Service{
		{ID: svcID, Name: "Test Service", Directory: saveDir},
	}
	saved, err := sg.SaveServices(testSvcs, "test commit")
	if err != nil {
		t.Fatalf("expected no error saving services, got %v", err)
	}
	if len(saved) != 1 || saved[0].ID != svcID {
		t.Errorf("expected saved services to match input, got %v", saved)
	}

	// 3. LoadServices now returns the service
	svcs, err = sg.LoadServices()
	if err != nil {
		t.Fatalf("expected no error loading services, got %v", err)
	}
	if len(svcs) != 1 {
		t.Errorf("expected 1 loaded service, got %d", len(svcs))
	}

	// 4. Git operations
	tempDir := t.TempDir()

	if err := sg.InitGit(tempDir, ""); err != nil {
		t.Errorf("expected no error on InitGit, got %v", err)
	}

	status, err := sg.GetGitStatus(tempDir)
	if err != nil {
		t.Fatalf("expected no error getting git status, got %v", err)
	}
	if !status.IsGit {
		t.Errorf("expected IsGit to be true, got %t", status.IsGit)
	}

	// Create a dummy file to commit/sync/pull/push
	testFile := filepath.Join(tempDir, "dummy.txt")
	if err := os.WriteFile(testFile, []byte("test"), 0644); err != nil {
		t.Fatalf("expected no error writing dummy file, got %v", err)
	}

	if err := sg.CommitGit(tempDir, "message"); err != nil {
		t.Errorf("expected no error on CommitGit, got %v", err)
	}

	// Set up a bare remote repo to test Sync/Pull/Push
	remoteDir := t.TempDir()
	c := exec.Command("git", "init", "--bare")
	c.Dir = remoteDir
	if err := c.Run(); err != nil {
		t.Fatalf("failed to init bare remote repository: %v", err)
	}

	// Add remote
	cmdAdd := exec.Command("git", "remote", "add", "origin", remoteDir)
	cmdAdd.Dir = tempDir
	if err := cmdAdd.Run(); err != nil {
		t.Fatalf("failed to add remote: %v", err)
	}

	if err := sg.PushGit(tempDir); err != nil {
		t.Errorf("expected no error on PushGit, got %v", err)
	}

	if err := sg.PullGit(tempDir); err != nil {
		t.Errorf("expected no error on PullGit, got %v", err)
	}

	if err := sg.SyncGit(tempDir); err != nil {
		t.Errorf("expected no error on SyncGit, got %v", err)
	}

	// 5. ImportService from a different directory (not yet in settings)
	importDir := t.TempDir()
	importSvcID := "s-imported-2"

	importFile := map[string]interface{}{
		"id":              importSvcID,
		"name":            "Imported Service",
		"isAuthenticated": false,
		"authType":        "none",
		"auth": map[string]interface{}{
			"type":           "none",
			"active":         true,
			"basicUser":      "",
			"basicPass":      "",
			"bearerToken":    "",
			"apiKeyName":     "",
			"apiKeyValue":    "",
			"apiKeyLocation": "header",
		},
		"endpoints": []interface{}{},
		"directory": importDir,
	}
	importData, _ := yaml.Marshal(importFile)
	if err := os.WriteFile(filepath.Join(importDir, "service.yaml"), importData, 0644); err != nil {
		t.Fatalf("failed to write import service.yaml: %v", err)
	}
	importEnvData, _ := yaml.Marshal([]models.EnvironmentConfig{})
	os.WriteFile(filepath.Join(importDir, "environments.yaml"), importEnvData, 0644)

	imported, err := sg.ImportService(importDir)
	if err != nil {
		t.Fatalf("expected no error importing service, got %v", err)
	}
	if imported.ID != importSvcID {
		t.Errorf("expected imported ID to be %s, got %s", importSvcID, imported.ID)
	}
	if imported.Name != "Imported Service" {
		t.Errorf("expected imported name to be 'Imported Service', got %s", imported.Name)
	}

	// 6. ImportCurl into existing service
	importedCurl, err := sg.ImportCurl(svcID, "curl http://api.example.com/users")
	if err != nil {
		t.Fatalf("expected no error importing curl, got %v", err)
	}
	if importedCurl.ID != svcID {
		t.Errorf("expected imported curl service ID to be %s, got %s", svcID, importedCurl.ID)
	}
	if len(importedCurl.Endpoints) == 0 {
		t.Errorf("expected at least 1 endpoint after curl import, got 0")
	}
	if importedCurl.Endpoints[0].Method != "GET" {
		t.Errorf("expected curl endpoint method to be GET, got %s", importedCurl.Endpoints[0].Method)
	}

	// 7. ImportSwagger from a local OpenAPI 3 file
	swaggerContent := `{
		"openapi": "3.0.0",
		"info": {"title": "Pet Store", "version": "1.0.0"},
		"paths": {
			"/pets": {
				"get": {
					"summary": "List all pets",
					"operationId": "listPets",
					"responses": {"200": {"description": "OK"}}
				}
			}
		}
	}`
	swaggerFile := filepath.Join(t.TempDir(), "petstore.json")
	if err := os.WriteFile(swaggerFile, []byte(swaggerContent), 0644); err != nil {
		t.Fatalf("failed to write swagger file: %v", err)
	}

	importedSwagger, err := sg.ImportSwagger("Pet Store API", swaggerFile)
	if err != nil {
		t.Fatalf("expected no error importing swagger, got %v", err)
	}
	if importedSwagger.Name != "Pet Store API" {
		t.Errorf("expected swagger imported name to be 'Pet Store API', got %s", importedSwagger.Name)
	}
	if len(importedSwagger.Endpoints) != 1 {
		t.Errorf("expected 1 endpoint from swagger, got %d", len(importedSwagger.Endpoints))
	}
	if importedSwagger.Endpoints[0].Name != "List all pets" {
		t.Errorf("expected endpoint name 'List all pets', got %s", importedSwagger.Endpoints[0].Name)
	}
}
