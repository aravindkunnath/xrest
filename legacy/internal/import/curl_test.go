package importlib

import (
	"testing"
)

func TestCurlToEndpoint_SimpleGET(t *testing.T) {
	ep, err := CurlToEndpoint("s1", "curl http://api.example.com/users", false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ep.Method != "GET" {
		t.Errorf("expected method GET, got %s", ep.Method)
	}
	if ep.URL != "http://api.example.com/users" {
		t.Errorf("expected URL http://api.example.com/users, got %s", ep.URL)
	}
	if ep.ServiceID != "s1" {
		t.Errorf("expected service ID s1, got %s", ep.ServiceID)
	}
	if ep.Name != "users" {
		t.Errorf("expected endpoint name 'users', got %s", ep.Name)
	}
}

func TestCurlToEndpoint_POSTWithBody(t *testing.T) {
	ep, err := CurlToEndpoint("s1", `curl -X POST -d '{"name":"test"}' http://api.example.com/users`, false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ep.Method != "POST" {
		t.Errorf("expected method POST, got %s", ep.Method)
	}
	if ep.Body != `{"name":"test"}` {
		t.Errorf("expected body '{\"name\":\"test\"}', got %s", ep.Body)
	}
}

func TestCurlToEndpoint_WithHeaders(t *testing.T) {
	ep, err := CurlToEndpoint("s1",
		`curl -H "Authorization: Bearer token123" -H "Content-Type: application/json" http://api.example.com/users`,
		false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if len(ep.Headers) != 2 {
		t.Fatalf("expected 2 headers, got %d", len(ep.Headers))
	}
	foundAuth := false
	foundContentType := false
	for _, h := range ep.Headers {
		if h.Name == "Authorization" && h.Value == "Bearer token123" {
			foundAuth = true
		}
		if h.Name == "Content-Type" && h.Value == "application/json" {
			foundContentType = true
		}
	}
	if !foundAuth {
		t.Error("expected Authorization header")
	}
	if !foundContentType {
		t.Error("expected Content-Type header")
	}
}

func TestCurlToEndpoint_PUTWithBody(t *testing.T) {
	ep, err := CurlToEndpoint("s1", `curl -X PUT -d '{"status":"updated"}' http://api.example.com/items/1`, false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ep.Method != "PUT" {
		t.Errorf("expected method PUT, got %s", ep.Method)
	}
	if ep.Body != `{"status":"updated"}` {
		t.Errorf("expected body '{\"status\":\"updated\"}', got %s", ep.Body)
	}
}

func TestCurlToEndpoint_DELETE(t *testing.T) {
	ep, err := CurlToEndpoint("s1", "curl -X DELETE http://api.example.com/items/1", false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ep.Method != "DELETE" {
		t.Errorf("expected method DELETE, got %s", ep.Method)
	}
}

func TestCurlToEndpoint_DataImpliesPOST(t *testing.T) {
	// Using -d without -X should default to POST
	ep, err := CurlToEndpoint("s1", `curl -d "key=value" http://api.example.com/submit`, false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ep.Method != "POST" {
		t.Errorf("expected method POST when -d is used, got %s", ep.Method)
	}
}

func TestCurlToEndpoint_Authenticated(t *testing.T) {
	authType := "bearer"
	ep, err := CurlToEndpoint("s1", "curl http://api.example.com/secure", true, &authType)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if !ep.Authenticated {
		t.Error("expected authenticated to be true")
	}
	if ep.AuthType != "bearer" {
		t.Errorf("expected auth type 'bearer', got %s", ep.AuthType)
	}
}

func TestCurlToEndpoint_QueryParams(t *testing.T) {
	ep, err := CurlToEndpoint("s1", "curl 'http://api.example.com/users?page=1&limit=10'", false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if len(ep.Params) != 2 {
		t.Fatalf("expected 2 query params, got %d", len(ep.Params))
	}
	if ep.Params[0].Name != "page" || ep.Params[0].Value != "1" {
		t.Errorf("expected param page=1, got %s=%s", ep.Params[0].Name, ep.Params[0].Value)
	}
	if ep.Params[1].Name != "limit" || ep.Params[1].Value != "10" {
		t.Errorf("expected param limit=10, got %s=%s", ep.Params[1].Name, ep.Params[1].Value)
	}
}

func TestCurlToEndpoint_NotACurlCommand(t *testing.T) {
	_, err := CurlToEndpoint("s1", "wget http://example.com", false, nil)
	if err == nil {
		t.Fatal("expected error for non-curl command")
	}
}

func TestCurlToEndpoint_EmptyCommand(t *testing.T) {
	_, err := CurlToEndpoint("s1", "", false, nil)
	if err == nil {
		t.Fatal("expected error for empty command")
	}
}

func TestCurlToEndpoint_WithURLEncodedData(t *testing.T) {
	ep, err := CurlToEndpoint("s1", `curl --data-urlencode "key=hello world" http://api.example.com/submit`, false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ep.Method != "POST" {
		t.Errorf("expected method POST, got %s", ep.Method)
	}
}

func TestCurlToEndpoint_VariousFlags(t *testing.T) {
	// Test that various curl flags are handled without error
	ep, err := CurlToEndpoint("s1",
		`curl -k -L -s -S --compressed --connect-timeout 10 --max-time 30 http://api.example.com`,
		false, nil)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ep.Method != "GET" {
		t.Errorf("expected method GET, got %s", ep.Method)
	}
}

func TestParseCurl_SingleQuotes(t *testing.T) {
	result, err := parseCurl(`curl -X POST 'http://api.example.com/data' -H 'Content-Type: application/json' -d '{"key":"value"}'`)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if result.method != "POST" {
		t.Errorf("expected method POST, got %s", result.method)
	}
	if result.body != `{"key":"value"}` {
		t.Errorf("expected body '{\"key\":\"value\"}', got %s", result.body)
	}
}

func TestExtractEndpointName_Root(t *testing.T) {
	name := extractEndpointName("https://api.example.com/")
	if name != "root" {
		t.Errorf("expected 'root', got %s", name)
	}
}

func TestExtractEndpointName_Deep(t *testing.T) {
	name := extractEndpointName("https://api.example.com/api/v2/users/list")
	if name != "api v2 users list" {
		t.Errorf("expected 'api v2 users list', got %s", name)
	}
}

func TestExtractEndpointName_WithQuery(t *testing.T) {
	name := extractEndpointName("https://api.example.com/users?page=1")
	if name != "users" {
		t.Errorf("expected 'users', got %s", name)
	}
}
