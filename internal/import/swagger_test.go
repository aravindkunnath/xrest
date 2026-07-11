package importlib

import (
	"testing"
)

func TestParseSpecContent_OpenAPI3JSON(t *testing.T) {
	spec := `{
		"openapi": "3.0.0",
		"info": {"title": "Pet Store", "version": "1.0.0"},
		"paths": {
			"/pets": {
				"get": {
					"summary": "List all pets",
					"operationId": "listPets",
					"parameters": [
						{"name": "limit", "in": "query", "schema": {"type": "integer"}},
						{"name": "Authorization", "in": "header", "schema": {"type": "string"}}
					],
					"responses": {"200": {"description": "OK"}}
				},
				"post": {
					"summary": "Create a pet",
					"responses": {"201": {"description": "Created"}}
				}
			},
			"/pets/{petId}": {
				"get": {
					"summary": "Get a pet by ID",
					"responses": {"200": {"description": "OK"}}
				}
			}
		}
	}`

	baseURL, endpoints, err := ParseSpecContent(spec, "s-test")
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if baseURL != "https://api.example.com" {
		t.Errorf("expected default base URL, got %s", baseURL)
	}
	if len(endpoints) != 3 {
		t.Fatalf("expected 3 endpoints, got %d", len(endpoints))
	}

	// Use a map to verify expected endpoints (iteration order varies)
	epMap := make(map[string]string) // "method:path" -> name
	paramCounts := make(map[string]int)
	headerCounts := make(map[string]int)
	for _, ep := range endpoints {
		key := ep.Method + ":" + ep.URL
		epMap[key] = ep.Name
		paramCounts[key] = len(ep.Params)
		headerCounts[key] = len(ep.Headers)
	}

	if epMap["GET:/pets"] != "List all pets" {
		t.Errorf("expected 'List all pets' for GET /pets, got %s", epMap["GET:/pets"])
	}
	if epMap["POST:/pets"] != "Create a pet" {
		t.Errorf("expected 'Create a pet' for POST /pets, got %s", epMap["POST:/pets"])
	}
	if epMap["GET:/pets/{petId}"] != "Get a pet by ID" {
		t.Errorf("expected 'Get a pet by ID' for GET /pets/{petId}, got %s", epMap["GET:/pets/{petId}"])
	}
	if paramCounts["GET:/pets"] != 1 {
		t.Errorf("expected 1 query param for GET /pets, got %d", paramCounts["GET:/pets"])
	}
	if headerCounts["GET:/pets"] != 1 {
		t.Errorf("expected 1 header param for GET /pets, got %d", headerCounts["GET:/pets"])
	}
}

func TestParseSpecContent_OpenAPI3WithServer(t *testing.T) {
	spec := `{
		"openapi": "3.0.0",
		"info": {"title": "API", "version": "1.0.0"},
		"servers": [{"url": "https://api.example.com/v2"}],
		"paths": {
			"/users": {
				"get": {
					"summary": "List users",
					"responses": {"200": {"description": "OK"}}
				}
			}
		}
	}`

	baseURL, endpoints, err := ParseSpecContent(spec, "s-test")
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if baseURL != "https://api.example.com/v2" {
		t.Errorf("expected server URL, got %s", baseURL)
	}
	if len(endpoints) != 1 {
		t.Fatalf("expected 1 endpoint, got %d", len(endpoints))
	}
}

func TestParseSpecContent_OpenAPI3YAML(t *testing.T) {
	spec := `openapi: "3.0.0"
info:
  title: Pet Store
  version: "1.0"
paths:
  "/pets":
    get:
      summary: List pets
      operationId: listPets
      responses:
        "200":
          description: OK
`

	_, endpoints, err := ParseSpecContent(spec, "s-test")
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if len(endpoints) != 1 {
		t.Fatalf("expected 1 endpoint, got %d", len(endpoints))
	}
	if endpoints[0].Name != "List pets" {
		t.Errorf("expected 'List pets', got %s", endpoints[0].Name)
	}
}

func TestParseSpecContent_Swagger2JSON(t *testing.T) {
	spec := `{
		"swagger": "2.0",
		"info": {"title": "Pet Store", "version": "1.0.0"},
		"host": "api.example.com",
		"basePath": "/v1",
		"schemes": ["https"],
		"paths": {
			"/pets": {
				"get": {
					"summary": "List pets",
					"operationId": "listPets",
					"parameters": [
						{"name": "limit", "in": "query", "type": "integer"}
					],
					"responses": {"200": {"description": "OK"}}
				}
			}
		}
	}`

	baseURL, endpoints, err := ParseSpecContent(spec, "s-test")
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if baseURL != "https://api.example.com" {
		t.Errorf("expected base URL, got %s", baseURL)
	}
	if len(endpoints) != 1 {
		t.Fatalf("expected 1 endpoint, got %d", len(endpoints))
	}
	if endpoints[0].Name != "List pets" {
		t.Errorf("expected 'List pets', got %s", endpoints[0].Name)
	}
}

func TestParseSpecContent_Swagger2YAML(t *testing.T) {
	spec := `swagger: "2.0"
info:
  title: Pet Store
  version: "1.0"
host: api.example.com
basePath: /v1
schemes:
  - https
paths:
  /pets:
    get:
      summary: List pets
      responses:
        "200":
          description: OK
    post:
      summary: Create pet
      responses:
        "201":
          description: Created
`

	_, endpoints, err := ParseSpecContent(spec, "s-test")
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if len(endpoints) != 2 {
		t.Fatalf("expected 2 endpoints, got %d", len(endpoints))
	}
}

func TestParseSpecContent_EmptySpec(t *testing.T) {
	_, _, err := ParseSpecContent("", "s-test")
	if err == nil {
		t.Fatal("expected error for empty spec")
	}
}

func TestParseSpecContent_InvalidJSON(t *testing.T) {
	_, _, err := ParseSpecContent("not valid json or yaml", "s-test")
	if err == nil {
		t.Fatal("expected error for invalid spec")
	}
}

func TestParseSpecContent_GenericFallback(t *testing.T) {
	spec := `{"paths": {"/items": {"get": {"summary": "Get all items", "responses": {"200": {"description": "OK"}}}}}}`
	baseURL, endpoints, err := ParseSpecContent(spec, "s-test")
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if baseURL != "https://api.example.com" {
		t.Errorf("expected default base URL, got %s", baseURL)
	}
	if len(endpoints) != 1 {
		t.Fatalf("expected 1 endpoint, got %d", len(endpoints))
	}
}

func TestParseSpecContent_NoPaths(t *testing.T) {
	spec := `{"openapi": "3.0.0", "info": {"title": "Empty", "version": "1.0"}}`
	_, endpoints, err := ParseSpecContent(spec, "s-test")
	if err != nil {
		t.Fatalf("expected no error for spec with no paths, got %v", err)
	}
	if len(endpoints) != 0 {
		t.Errorf("expected 0 endpoints, got %d", len(endpoints))
	}
}

func TestExtractBaseURL_OpenAPI3(t *testing.T) {
	spec := map[string]interface{}{
		"servers": []interface{}{
			map[string]interface{}{"url": "https://api.com/v2"},
		},
	}
	url := extractBaseURL(spec)
	if url != "https://api.com/v2" {
		t.Errorf("expected https://api.com/v2, got %s", url)
	}
}

func TestExtractBaseURL_Swagger2(t *testing.T) {
	spec := map[string]interface{}{
		"host":     "api.com",
		"schemes":  []interface{}{"https"},
		"basePath": "/v1",
	}
	url := extractBaseURL(spec)
	if url != "https://api.com/v1" {
		t.Errorf("expected https://api.com/v1, got %s", url)
	}
}

func TestExtractBaseURL_Default(t *testing.T) {
	spec := map[string]interface{}{}
	url := extractBaseURL(spec)
	if url != "https://api.example.com" {
		t.Errorf("expected default URL, got %s", url)
	}
}
