package adapters

import (
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"
	"time"

	"xrest/internal/models"
)

func TestHttpGet(t *testing.T) {
	// Start a local HTTP server
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		rw.Header().Set("Content-Type", "application/json")
		rw.Header().Set("X-Custom-Response-Header", "value1")
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte(`{"status":"ok"}`))
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	resp, err := h.Get()
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if resp == nil {
		t.Fatal("Expected response not to be nil")
	}

	// Assertions
	if resp.Body != `{"status":"ok"}` {
		t.Errorf("Expected response %q, got %q", `{"status":"ok"}`, resp.Body)
	}
	if resp.ContentType != "application/json" {
		t.Errorf("Expected ContentType %q, got %q", "application/json", resp.ContentType)
	}
	if resp.TimeTaken <= 0 {
		t.Errorf("Expected positive TimeTaken, got %v", resp.TimeTaken)
	}
	if resp.RequestHeaders.Get("User-Agent") == "" {
		t.Error("Expected RequestHeaders to have User-Agent")
	}
	if resp.ResponseHeaders.Get("X-Custom-Response-Header") != "value1" {
		t.Errorf("Expected X-Custom-Response-Header %q, got %q", "value1", resp.ResponseHeaders.Get("X-Custom-Response-Header"))
	}
}

func TestHttpGetCustomHeaders(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Header.Get("X-Client-Header") != "custom-value" {
			t.Errorf("Server expected X-Client-Header 'custom-value', got %q", req.Header.Get("X-Client-Header"))
		}
		rw.Header().Set("Content-Type", "text/plain")
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte("ok"))
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	// Initialize resty client first to configure it or set headers
	h.build()
	h.client.SetHeader("X-Client-Header", "custom-value")

	resp, err := h.Get()
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}

	if resp.RequestHeaders.Get("X-Client-Header") != "custom-value" {
		t.Errorf("Expected X-Client-Header in RequestHeaders, got %q", resp.RequestHeaders.Get("X-Client-Header"))
	}
}

func TestHttpGetError(t *testing.T) {
	h := &Http{
		URL: "http://invalid-domain-name-that-does-not-exist.invalid",
	}
	defer h.Close()

	resp, err := h.Get()
	if err == nil {
		t.Fatal("Expected error for invalid URL, got nil")
	}
	if resp != nil {
		t.Errorf("Expected nil response on error, got %+v", resp)
	}
}

func TestHttpPost(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Method != http.MethodPost {
			t.Errorf("Expected POST request, got %s", req.Method)
		}
		bodyBytes, err := io.ReadAll(req.Body)
		if err != nil {
			t.Fatalf("Failed to read body: %v", err)
		}
		bodyStr := string(bodyBytes)
		if bodyStr != "hello post" {
			t.Errorf("Expected body 'hello post', got %q", bodyStr)
		}
		rw.Header().Set("Content-Type", "application/json")
		rw.WriteHeader(http.StatusCreated)
		rw.Write([]byte(`{"status":"created"}`))
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	resp, err := h.Post("hello post")
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}
	if resp.Body != `{"status":"created"}` {
		t.Errorf("Expected response %q, got %q", `{"status":"created"}`, resp.Body)
	}
}

func TestHttpPut(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Method != http.MethodPut {
			t.Errorf("Expected PUT request, got %s", req.Method)
		}
		bodyBytes, err := io.ReadAll(req.Body)
		if err != nil {
			t.Fatalf("Failed to read body: %v", err)
		}
		bodyStr := string(bodyBytes)
		if bodyStr != "hello put" {
			t.Errorf("Expected body 'hello put', got %q", bodyStr)
		}
		rw.Header().Set("Content-Type", "application/json")
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte(`{"status":"updated"}`))
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	resp, err := h.Put("hello put")
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}
	if resp.Body != `{"status":"updated"}` {
		t.Errorf("Expected response %q, got %q", `{"status":"updated"}`, resp.Body)
	}
}

func TestHttpDelete(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Method != http.MethodDelete {
			t.Errorf("Expected DELETE request, got %s", req.Method)
		}
		rw.Header().Set("Content-Type", "application/json")
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte(`{"status":"deleted"}`))
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	resp, err := h.Delete()
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}
	if resp.Body != `{"status":"deleted"}` {
		t.Errorf("Expected response %q, got %q", `{"status":"deleted"}`, resp.Body)
	}
}

func TestHttpPatch(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Method != http.MethodPatch {
			t.Errorf("Expected PATCH request, got %s", req.Method)
		}
		bodyBytes, err := io.ReadAll(req.Body)
		if err != nil {
			t.Fatalf("Failed to read body: %v", err)
		}
		bodyStr := string(bodyBytes)
		if bodyStr != "hello patch" {
			t.Errorf("Expected body 'hello patch', got %q", bodyStr)
		}
		rw.Header().Set("Content-Type", "application/json")
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte(`{"status":"patched"}`))
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	resp, err := h.Patch("hello patch")
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}
	if resp.Body != `{"status":"patched"}` {
		t.Errorf("Expected response %q, got %q", `{"status":"patched"}`, resp.Body)
	}
}

func TestHttpHead(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Method != http.MethodHead {
			t.Errorf("Expected HEAD request, got %s", req.Method)
		}
		rw.Header().Set("Content-Type", "text/plain")
		rw.Header().Set("X-Head-Response", "header-value")
		rw.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	resp, err := h.Head()
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}
	if resp.ContentType != "text/plain" {
		t.Errorf("Expected ContentType %q, got %q", "text/plain", resp.ContentType)
	}
	if resp.ResponseHeaders.Get("X-Head-Response") != "header-value" {
		t.Errorf("Expected X-Head-Response %q, got %q", "header-value", resp.ResponseHeaders.Get("X-Head-Response"))
	}
}

func TestHttpOptions(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Method != http.MethodOptions {
			t.Errorf("Expected OPTIONS request, got %s", req.Method)
		}
		rw.Header().Set("Allow", "GET, POST, OPTIONS")
		rw.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	h := &Http{
		URL: server.URL,
	}
	defer h.Close()

	resp, err := h.Options()
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}
	if resp.ResponseHeaders.Get("Allow") != "GET, POST, OPTIONS" {
		t.Errorf("Expected Allow header %q, got %q", "GET, POST, OPTIONS", resp.ResponseHeaders.Get("Allow"))
	}
}

func TestHttpSendQueryParamsAndHeaders(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.Header.Get("X-Custom-Req") != "hello" {
			t.Errorf("Expected X-Custom-Req header 'hello', got %q", req.Header.Get("X-Custom-Req"))
		}
		if req.URL.Query().Get("foo") != "bar" {
			t.Errorf("Expected query param foo=bar, got %q", req.URL.Query().Get("foo"))
		}
		if req.URL.Path != "/users/123/profile" {
			t.Errorf("Expected path /users/123/profile, got %q", req.URL.Path)
		}
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte("ok"))
	}))
	defer server.Close()

	h := &Http{}
	defer h.Close()

	resp, err := h.Send(&models.Request{
		Method: "GET",
		URL:    server.URL + "/users/:id/{section}",
		Headers: map[string]string{
			"X-Custom-Req": "hello",
		},
		QueryParams: map[string]string{
			"foo": "bar",
		},
		PathParams: map[string]string{
			"id":      "123",
			"section": "profile",
		},
	})
	if err != nil {
		t.Fatalf("Expected no error, got %v", err)
	}
	if string(resp.BodyBytes) != "ok" {
		t.Errorf("Expected body 'ok', got %q", string(resp.BodyBytes))
	}
}

func TestHttpSendAuth(t *testing.T) {
	// Test Basic Auth
	serverBasic := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		username, password, ok := req.BasicAuth()
		if !ok || username != "user" || password != "pass" {
			t.Errorf("Invalid Basic Auth: ok=%v, user=%q, pass=%q", ok, username, password)
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverBasic.Close()

	h := &Http{}
	defer h.Close()

	_, err := h.Send(&models.Request{
		Method: "GET",
		URL:    serverBasic.URL,
		Auth: &models.Auth{
			Type:          models.AuthBasic,
			BasicUsername: "user",
			BasicPassword: "pass",
		},
	})
	if err != nil {
		t.Fatalf("Basic Auth request failed: %v", err)
	}

	// Test Bearer Auth
	serverBearer := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		authHeader := req.Header.Get("Authorization")
		if authHeader != "Bearer my-token" {
			t.Errorf("Invalid Bearer Auth: got %q", authHeader)
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverBearer.Close()

	_, err = h.Send(&models.Request{
		Method: "GET",
		URL:    serverBearer.URL,
		Auth: &models.Auth{
			Type:        models.AuthBearer,
			BearerToken: "my-token",
		},
	})
	if err != nil {
		t.Fatalf("Bearer Auth request failed: %v", err)
	}

	// Test API Key Auth in header
	serverAPIKeyHeader := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		keyHeader := req.Header.Get("X-API-Key")
		if keyHeader != "secret-value" {
			t.Errorf("Invalid API Key in header: got %q", keyHeader)
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverAPIKeyHeader.Close()

	_, err = h.Send(&models.Request{
		Method: "GET",
		URL:    serverAPIKeyHeader.URL,
		Auth: &models.Auth{
			Type:        models.AuthAPIKey,
			APIKeyKey:   "X-API-Key",
			APIKeyValue: "secret-value",
			APIKeyAddTo: "header",
		},
	})
	if err != nil {
		t.Fatalf("API Key Header request failed: %v", err)
	}

	// Test API Key Auth in query
	serverAPIKeyQuery := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		keyQuery := req.URL.Query().Get("api_key")
		if keyQuery != "secret-value" {
			t.Errorf("Invalid API Key in query: got %q", keyQuery)
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverAPIKeyQuery.Close()

	_, err = h.Send(&models.Request{
		Method: "GET",
		URL:    serverAPIKeyQuery.URL,
		Auth: &models.Auth{
			Type:        models.AuthAPIKey,
			APIKeyKey:   "api_key",
			APIKeyValue: "secret-value",
			APIKeyAddTo: "query",
		},
	})
	if err != nil {
		t.Fatalf("API Key Query request failed: %v", err)
	}
}

func TestHttpSendBodies(t *testing.T) {
	// 1. Raw Body
	serverRaw := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		body, _ := io.ReadAll(req.Body)
		if string(body) != "raw data" {
			t.Errorf("Expected raw body 'raw data', got %q", string(body))
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverRaw.Close()

	h := &Http{}
	defer h.Close()

	_, err := h.Send(&models.Request{
		Method:   "POST",
		URL:      serverRaw.URL,
		BodyType: "raw",
		BodyRaw:  "raw data",
	})
	if err != nil {
		t.Fatalf("Raw body POST failed: %v", err)
	}

	// 2. URL-encoded Form
	serverUrlencoded := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		err := req.ParseForm()
		if err != nil {
			t.Fatalf("ParseForm failed: %v", err)
		}
		if req.Form.Get("foo") != "bar" || req.Form.Get("baz") != "qux" {
			t.Errorf("Invalid URL-encoded form: got %+v", req.Form)
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverUrlencoded.Close()

	_, err = h.Send(&models.Request{
		Method:   "POST",
		URL:      serverUrlencoded.URL,
		BodyType: "urlencoded",
		BodyForm: map[string]string{
			"foo": "bar",
			"baz": "qux",
		},
	})
	if err != nil {
		t.Fatalf("URL-encoded form POST failed: %v", err)
	}

	// 3. Binary Body
	serverBinary := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		body, _ := io.ReadAll(req.Body)
		if len(body) != 4 || body[0] != 1 || body[1] != 2 || body[2] != 3 || body[3] != 4 {
			t.Errorf("Invalid binary body: got %v", body)
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverBinary.Close()

	_, err = h.Send(&models.Request{
		Method:     "POST",
		URL:        serverBinary.URL,
		BodyType:   "binary",
		BodyBinary: []byte{1, 2, 3, 4},
	})
	if err != nil {
		t.Fatalf("Binary body POST failed: %v", err)
	}

	// 4. Multipart/form-data with file & text
	tmpFile, err := os.CreateTemp("", "test-upload-*.txt")
	if err != nil {
		t.Fatalf("Failed to create temp file: %v", err)
	}
	defer os.Remove(tmpFile.Name())
	if _, err := tmpFile.Write([]byte("file content")); err != nil {
		t.Fatalf("Failed to write to temp file: %v", err)
	}
	tmpFile.Close()

	serverMultipart := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		err := req.ParseMultipartForm(10 << 20)
		if err != nil {
			t.Fatalf("ParseMultipartForm failed: %v", err)
		}
		if req.FormValue("textfield") != "textvalue" {
			t.Errorf("Expected form textfield 'textvalue', got %q", req.FormValue("textfield"))
		}
		file, header, err := req.FormFile("filefield")
		if err != nil {
			t.Fatalf("Failed to get filefield: %v", err)
		}
		defer file.Close()
		content, _ := io.ReadAll(file)
		if string(content) != "file content" {
			t.Errorf("Expected uploaded file content 'file content', got %q", string(content))
		}
		if !strings.HasSuffix(header.Filename, ".txt") {
			t.Errorf("Expected filename suffix '.txt', got %q", header.Filename)
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer serverMultipart.Close()

	_, err = h.Send(&models.Request{
		Method:   "POST",
		URL:      serverMultipart.URL,
		BodyType: "form-data",
		BodyFormData: []models.FormDataItem{
			{
				Key:   "textfield",
				Value: "textvalue",
				Type:  models.FormDataTypeText,
			},
			{
				Key:      "filefield",
				Type:     models.FormDataTypeFile,
				FilePath: tmpFile.Name(),
			},
		},
	})
	if err != nil {
		t.Fatalf("Multipart form-data POST failed: %v", err)
	}
}

func TestHttpSendRedirectsCookiesAndConfig(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		if req.URL.Path == "/redirect" {
			http.Redirect(rw, req, "/target", http.StatusFound)
			return
		}
		if req.URL.Path == "/target" {
			http.SetCookie(rw, &http.Cookie{
				Name:     "session",
				Value:    "abc123xyz",
				Path:     "/",
				Domain:   "localhost",
				Expires:  time.Now().Add(24 * time.Hour),
				Secure:   true,
				HttpOnly: true,
			})
			rw.WriteHeader(http.StatusAccepted)
			rw.Write([]byte("redirect target"))
			return
		}
		rw.WriteHeader(http.StatusOK)
	}))
	defer server.Close()

	h := &Http{}
	defer h.Close()

	// 1. Test Follow Redirects = true
	follow := true
	resp, err := h.Send(&models.Request{
		Method:          "GET",
		URL:             server.URL + "/redirect",
		FollowRedirects: &follow,
	})
	if err != nil {
		t.Fatalf("Follow redirect failed: %v", err)
	}
	if resp.StatusCode != http.StatusAccepted {
		t.Errorf("Expected status code %d, got %d", http.StatusAccepted, resp.StatusCode)
	}
	if resp.StatusText == "" {
		t.Error("Expected StatusText to be populated")
	}
	if resp.Size <= 0 {
		t.Errorf("Expected size > 0, got %d", resp.Size)
	}

	var sessionCookie *models.Cookie
	for _, c := range resp.Cookies {
		if c.Name == "session" {
			sessionCookie = &c
			break
		}
	}
	if sessionCookie == nil {
		t.Fatal("Expected 'session' cookie, got none")
	}
	if sessionCookie.Value != "abc123xyz" {
		t.Errorf("Expected cookie value 'abc123xyz', got %q", sessionCookie.Value)
	}
	if !sessionCookie.Secure || !sessionCookie.HttpOnly {
		t.Errorf("Expected Secure and HttpOnly to be true, got Secure=%t HttpOnly=%t", sessionCookie.Secure, sessionCookie.HttpOnly)
	}

	// 2. Test Follow Redirects = false
	dontFollow := false
	respNoRedirect, err := h.Send(&models.Request{
		Method:          "GET",
		URL:             server.URL + "/redirect",
		FollowRedirects: &dontFollow,
	})
	if err != nil {
		t.Fatalf("No-redirect request failed: %v", err)
	}
	if respNoRedirect.StatusCode != http.StatusFound {
		t.Errorf("Expected status code %d for no-redirect, got %d", http.StatusFound, respNoRedirect.StatusCode)
	}
}

func TestPreflightAuthBodyJSON(t *testing.T) {
	// Preflight auth server
	var authCalls int
	authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authCalls++
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"data":{"token":"my-jwt-token-123"}}`))
	}))
	defer authServer.Close()

	// Main server
	mainServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("X-Custom-Auth")
		if authHeader != "Prefix-my-jwt-token-123" {
			t.Errorf("Expected Header 'X-Custom-Auth: Prefix-my-jwt-token-123', got %q", authHeader)
			w.WriteHeader(http.StatusUnauthorized)
			return
		}
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("success"))
	}))
	defer mainServer.Close()

	h := &Http{}
	defer h.Close()

	// Configure request with preflight
	resp, err := h.Send(&models.Request{
		Method: "GET",
		URL:    mainServer.URL,
		Preflight: &models.PreflightConfig{
			Request: &models.Request{
				Method: "POST",
				URL:    authServer.URL,
			},
			TokenLocation: "body",
			TokenPath:     "data.token",
			TokenHeader:   "X-Custom-Auth",
			TokenPrefix:   "Prefix-",
		},
	})

	if err != nil {
		t.Fatalf("Request failed: %v", err)
	}

	if resp.StatusCode != http.StatusOK {
		t.Errorf("Expected status OK, got %d", resp.StatusCode)
	}

	if string(resp.BodyBytes) != "success" {
		t.Errorf("Expected body 'success', got %q", string(resp.BodyBytes))
	}

	if authCalls != 1 {
		t.Errorf("Expected preflight server to be called 1 time, called %d times", authCalls)
	}
}

func TestPreflightAuthHeader(t *testing.T) {
	// Preflight auth server
	authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("X-Auth-Token", "header-token-xyz")
		w.WriteHeader(http.StatusOK)
	}))
	defer authServer.Close()

	// Main server
	mainServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		if authHeader != "Bearer header-token-xyz" {
			t.Errorf("Expected Authorization 'Bearer header-token-xyz', got %q", authHeader)
			w.WriteHeader(http.StatusUnauthorized)
			return
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer mainServer.Close()

	h := &Http{}
	defer h.Close()

	resp, err := h.Send(&models.Request{
		Method: "GET",
		URL:    mainServer.URL,
		Preflight: &models.PreflightConfig{
			Request: &models.Request{
				Method: "GET",
				URL:    authServer.URL,
			},
			TokenLocation: "header",
			TokenPath:     "X-Auth-Token",
			// Defaults to TokenHeader: Authorization, TokenPrefix: Bearer
		},
	})

	if err != nil {
		t.Fatalf("Request failed: %v", err)
	}

	if resp.StatusCode != http.StatusOK {
		t.Errorf("Expected status OK, got %d", resp.StatusCode)
	}
}

func TestPreflightCachingAndExpiration(t *testing.T) {
	var authCalls int
	authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authCalls++
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"access_token":"token-val"}`))
	}))
	defer authServer.Close()

	mainServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer mainServer.Close()

	h := &Http{}
	defer h.Close()

	// Define preflight with cache key
	preflightCfg := &models.PreflightConfig{
		Request: &models.Request{
			Method: "POST",
			URL:    authServer.URL,
		},
		TokenLocation: "body",
		TokenPath:     "access_token",
		CacheKey:      "test-cache-key-unique",
		CacheTTL:      50 * time.Millisecond, // very short TTL
	}

	// 1st request - should trigger preflight
	resp, err := h.Send(&models.Request{
		Method:    "GET",
		URL:       mainServer.URL,
		Preflight: preflightCfg,
	})
	if err != nil {
		t.Fatalf("First request failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("First request got status %d", resp.StatusCode)
	}

	// 2nd request - should hit cache, NOT trigger preflight
	resp2, err := h.Send(&models.Request{
		Method:    "GET",
		URL:       mainServer.URL,
		Preflight: preflightCfg,
	})
	if err != nil {
		t.Fatalf("Second request failed: %v", err)
	}
	if resp2.StatusCode != http.StatusOK {
		t.Errorf("Second request got status %d", resp2.StatusCode)
	}

	if authCalls != 1 {
		t.Errorf("Expected exactly 1 auth call due to cache, got %d", authCalls)
	}

	// Wait for cache to expire
	time.Sleep(60 * time.Millisecond)

	// 3rd request - cache expired, should trigger preflight again
	resp3, err := h.Send(&models.Request{
		Method:    "GET",
		URL:       mainServer.URL,
		Preflight: preflightCfg,
	})
	if err != nil {
		t.Fatalf("Third request failed: %v", err)
	}
	if resp3.StatusCode != http.StatusOK {
		t.Errorf("Third request got status %d", resp3.StatusCode)
	}

	if authCalls != 2 {
		t.Errorf("Expected exactly 2 auth calls after cache expiration, got %d", authCalls)
	}
}

func TestHttpAllMethodsWithBasicAuth(t *testing.T) {
	// A mock server that enforces basic authentication for all methods
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		u, p, ok := req.BasicAuth()
		if !ok || u != "test-user" || p != "test-pass" {
			rw.WriteHeader(http.StatusUnauthorized)
			rw.Write([]byte("Unauthorized"))
			return
		}
		rw.Header().Set("Content-Type", "text/plain")
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte(req.Method + " ok"))
	}))
	defer server.Close()

	// 1. Test using Send(req) with Request-level Auth basic
	h := &Http{}
	defer h.Close()

	req := &models.Request{
		Method: "GET",
		URL:    server.URL,
		Auth: &models.Auth{
			Type:          models.AuthBasic,
			BasicUsername: "test-user",
			BasicPassword: "test-pass",
		},
	}
	resp, err := h.Send(req)
	if err != nil {
		t.Fatalf("Send failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("Expected status OK, got %d", resp.StatusCode)
	}
	if string(resp.BodyBytes) != "GET ok" {
		t.Errorf("Expected body 'GET ok', got %q", string(resp.BodyBytes))
	}

	// 2. Test using convenience wrapper methods with Client-level Auth basic
	hClient := &Http{
		URL: server.URL,
	}
	defer hClient.Close()
	hClient.build()
	hClient.client.SetBasicAuth("test-user", "test-pass")

	// GET
	respGet, err := hClient.Get()
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if respGet.StatusCode != http.StatusOK {
		t.Errorf("GET: expected 200, got %d", respGet.StatusCode)
	}
	if string(respGet.BodyBytes) != "GET ok" {
		t.Errorf("GET: expected 'GET ok', got %q", string(respGet.BodyBytes))
	}

	// POST
	respPost, err := hClient.Post("body")
	if err != nil {
		t.Fatalf("Post failed: %v", err)
	}
	if respPost.StatusCode != http.StatusOK {
		t.Errorf("POST: expected 200, got %d", respPost.StatusCode)
	}
	if string(respPost.BodyBytes) != "POST ok" {
		t.Errorf("POST: expected 'POST ok', got %q", string(respPost.BodyBytes))
	}

	// PUT
	respPut, err := hClient.Put("body")
	if err != nil {
		t.Fatalf("Put failed: %v", err)
	}
	if respPut.StatusCode != http.StatusOK {
		t.Errorf("PUT: expected 200, got %d", respPut.StatusCode)
	}

	// DELETE
	respDelete, err := hClient.Delete()
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}
	if respDelete.StatusCode != http.StatusOK {
		t.Errorf("DELETE: expected 200, got %d", respDelete.StatusCode)
	}

	// PATCH
	respPatch, err := hClient.Patch("body")
	if err != nil {
		t.Fatalf("Patch failed: %v", err)
	}
	if respPatch.StatusCode != http.StatusOK {
		t.Errorf("PATCH: expected 200, got %d", respPatch.StatusCode)
	}

	// HEAD
	respHead, err := hClient.Head()
	if err != nil {
		t.Fatalf("Head failed: %v", err)
	}
	if respHead.StatusCode != http.StatusOK {
		t.Errorf("HEAD: expected 200, got %d", respHead.StatusCode)
	}

	// OPTIONS
	respOptions, err := hClient.Options()
	if err != nil {
		t.Fatalf("Options failed: %v", err)
	}
	if respOptions.StatusCode != http.StatusOK {
		t.Errorf("OPTIONS: expected 200, got %d", respOptions.StatusCode)
	}
}

func TestHttpAllMethodsWithBearerToken(t *testing.T) {
	// A mock server that enforces Bearer token authentication for all methods
	server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
		auth := req.Header.Get("Authorization")
		if auth != "Bearer my-secret-token" {
			rw.WriteHeader(http.StatusUnauthorized)
			rw.Write([]byte("Unauthorized"))
			return
		}
		rw.Header().Set("Content-Type", "text/plain")
		rw.WriteHeader(http.StatusOK)
		rw.Write([]byte(req.Method + " token ok"))
	}))
	defer server.Close()

	// 1. Test using Send(req) with Request-level Auth bearer
	h := &Http{}
	defer h.Close()

	req := &models.Request{
		Method: "GET",
		URL:    server.URL,
		Auth: &models.Auth{
			Type:        models.AuthBearer,
			BearerToken: "my-secret-token",
		},
	}
	resp, err := h.Send(req)
	if err != nil {
		t.Fatalf("Send failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("Expected status OK, got %d", resp.StatusCode)
	}
	if string(resp.BodyBytes) != "GET token ok" {
		t.Errorf("Expected body 'GET token ok', got %q", string(resp.BodyBytes))
	}

	// 2. Test using convenience wrapper methods with Client-level Bearer token
	hClient := &Http{
		URL: server.URL,
	}
	defer hClient.Close()
	hClient.build()
	hClient.client.SetAuthToken("my-secret-token")

	// GET
	respGet, err := hClient.Get()
	if err != nil {
		t.Fatalf("Get failed: %v", err)
	}
	if respGet.StatusCode != http.StatusOK {
		t.Errorf("GET: expected 200, got %d", respGet.StatusCode)
	}
	if string(respGet.BodyBytes) != "GET token ok" {
		t.Errorf("GET: expected 'GET token ok', got %q", string(respGet.BodyBytes))
	}

	// POST
	respPost, err := hClient.Post("body")
	if err != nil {
		t.Fatalf("Post failed: %v", err)
	}
	if respPost.StatusCode != http.StatusOK {
		t.Errorf("POST: expected 200, got %d", respPost.StatusCode)
	}

	// PUT
	respPut, err := hClient.Put("body")
	if err != nil {
		t.Fatalf("Put failed: %v", err)
	}
	if respPut.StatusCode != http.StatusOK {
		t.Errorf("PUT: expected 200, got %d", respPut.StatusCode)
	}

	// DELETE
	respDelete, err := hClient.Delete()
	if err != nil {
		t.Fatalf("Delete failed: %v", err)
	}
	if respDelete.StatusCode != http.StatusOK {
		t.Errorf("DELETE: expected 200, got %d", respDelete.StatusCode)
	}

	// PATCH
	respPatch, err := hClient.Patch("body")
	if err != nil {
		t.Fatalf("Patch failed: %v", err)
	}
	if respPatch.StatusCode != http.StatusOK {
		t.Errorf("PATCH: expected 200, got %d", respPatch.StatusCode)
	}

	// HEAD
	respHead, err := hClient.Head()
	if err != nil {
		t.Fatalf("Head failed: %v", err)
	}
	if respHead.StatusCode != http.StatusOK {
		t.Errorf("HEAD: expected 200, got %d", respHead.StatusCode)
	}

	// OPTIONS
	respOptions, err := hClient.Options()
	if err != nil {
		t.Fatalf("Options failed: %v", err)
	}
	if respOptions.StatusCode != http.StatusOK {
		t.Errorf("OPTIONS: expected 200, got %d", respOptions.StatusCode)
	}
}

func TestHttpOAuth2PreflightFlow(t *testing.T) {
	var authCalls int
	// 1. Simulating OAuth 2.0 Auth Server returning JSON body
	authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authCalls++
		// Verify OAuth2 client credentials can be sent as basic auth
		u, p, ok := r.BasicAuth()
		if ok {
			if u != "client-id" || p != "client-secret" {
				w.WriteHeader(http.StatusUnauthorized)
				return
			}
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(`{"oauth_token":{"token_str":"jwt-token-val-456"},"expires_in":3600}`))
	}))
	defer authServer.Close()

	// 2. Resource Server requiring the extracted token
	resourceServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		if authHeader != "Bearer jwt-token-val-456" {
			w.WriteHeader(http.StatusUnauthorized)
			return
		}
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("resource ok"))
	}))
	defer resourceServer.Close()

	h := &Http{}
	defer h.Close()

	preflightCfg := &models.PreflightConfig{
		Request: &models.Request{
			Method: "POST",
			URL:    authServer.URL,
			Auth: &models.Auth{
				Type:          models.AuthBasic,
				BasicUsername: "client-id",
				BasicPassword: "client-secret",
			},
		},
		TokenLocation: "body",
		TokenPath:     "oauth_token.token_str",
		TokenHeader:   "Authorization",
		TokenPrefix:   "Bearer ",
		CacheKey:      "oauth2-preflight-cache-key",
		CacheTTL:      10 * time.Minute,
	}

	// First execution should fetch the token
	resp1, err := h.Send(&models.Request{
		Method:    "GET",
		URL:       resourceServer.URL,
		Preflight: preflightCfg,
	})
	if err != nil {
		t.Fatalf("First Send failed: %v", err)
	}
	if resp1.StatusCode != http.StatusOK {
		t.Errorf("First Send got status %d", resp1.StatusCode)
	}
	if string(resp1.BodyBytes) != "resource ok" {
		t.Errorf("Expected 'resource ok', got %q", string(resp1.BodyBytes))
	}

	// Second execution should hit the cache, authCalls should still be 1
	resp2, err := h.Send(&models.Request{
		Method:    "GET",
		URL:       resourceServer.URL,
		Preflight: preflightCfg,
	})
	if err != nil {
		t.Fatalf("Second Send failed: %v", err)
	}
	if resp2.StatusCode != http.StatusOK {
		t.Errorf("Second Send got status %d", resp2.StatusCode)
	}
	if authCalls != 1 {
		t.Errorf("Expected 1 auth call, got %d", authCalls)
	}
}

func TestHttpPreflightErrors(t *testing.T) {
	// A server that returns various responses to trigger preflight errors
	var returnStatus int
	var responseBody []byte
	var responseHeaders map[string]string

	authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		for k, v := range responseHeaders {
			w.Header().Set(k, v)
		}
		w.WriteHeader(returnStatus)
		w.Write(responseBody)
	}))
	defer authServer.Close()

	h := &Http{}
	defer h.Close()

	// Helper to make request
	runRequest := func(cfg *models.PreflightConfig) error {
		_, err := h.Send(&models.Request{
			Method:    "GET",
			URL:       "http://localhost:80", // Target doesn't matter since preflight should fail first
			Preflight: cfg,
		})
		return err
	}

	// Error 1: Auth server returns error status code
	returnStatus = http.StatusInternalServerError
	responseBody = []byte("server error description")
	err := runRequest(&models.PreflightConfig{
		Request: &models.Request{Method: "GET", URL: authServer.URL},
	})
	if err == nil || !strings.Contains(err.Error(), "preflight HTTP call returned status 500") {
		t.Errorf("Expected 500 status error, got: %v", err)
	}

	// Error 2: Token path not found in JSON body
	returnStatus = http.StatusOK
	responseBody = []byte(`{"wrong_key":"val"}`)
	err = runRequest(&models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "body",
		TokenPath:     "missing_token",
	})
	if err == nil || !strings.Contains(err.Error(), "token key path \"missing_token\" not found") {
		t.Errorf("Expected key path missing error, got: %v", err)
	}

	// Error 3: Token value is not a string
	responseBody = []byte(`{"token_key": 12345}`)
	err = runRequest(&models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "body",
		TokenPath:     "token_key",
	})
	if err == nil || !strings.Contains(err.Error(), "is not a string") {
		t.Errorf("Expected token value type error, got: %v", err)
	}

	// Error 4: Failed to parse JSON
	responseBody = []byte(`{invalid-json`)
	err = runRequest(&models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "body",
		TokenPath:     "token_key",
	})
	if err == nil || !strings.Contains(err.Error(), "failed to parse preflight response body as JSON") {
		t.Errorf("Expected JSON parse error, got: %v", err)
	}

	// Error 5: Header token path required but empty
	err = runRequest(&models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "header",
		TokenPath:     "",
	})
	if err == nil || !strings.Contains(err.Error(), "tokenPath is required for header extraction") {
		t.Errorf("Expected tokenPath required error, got: %v", err)
	}

	// Error 6: Header not found in response
	responseHeaders = map[string]string{"Some-Header": "val"}
	err = runRequest(&models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "header",
		TokenPath:     "X-Missing-Token-Header",
	})
	if err == nil || !strings.Contains(err.Error(), "not found in preflight response") {
		t.Errorf("Expected header not found error, got: %v", err)
	}

	// Error 7: Nil preflight request
	err = runRequest(&models.PreflightConfig{
		Request: nil,
	})
	if err == nil || !strings.Contains(err.Error(), "preflight request is nil") {
		t.Errorf("Expected nil preflight request error, got: %v", err)
	}
}

func TestHttpEdgeCasesAndConfigs(t *testing.T) {
	// 1. Test timeout behavior
	slowServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		time.Sleep(50 * time.Millisecond)
		w.WriteHeader(http.StatusOK)
	}))
	defer slowServer.Close()

	h := &Http{}
	defer h.Close()

	_, err := h.Send(&models.Request{
		Method:  "GET",
		URL:     slowServer.URL,
		Timeout: 5 * time.Millisecond,
	})
	if err == nil {
		t.Error("Expected timeout error, but got nil")
	}

	// 2. Test path parameter resolution edge cases
	serverParams := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/users/abc-123/posts/xyz_987" {
			t.Errorf("Expected path /users/abc-123/posts/xyz_987, got %q", r.URL.Path)
		}
		w.WriteHeader(http.StatusOK)
	}))
	defer serverParams.Close()

	_, err = h.Send(&models.Request{
		Method: "GET",
		URL:    serverParams.URL + "/users/:userId/posts/{postId}",
		PathParams: map[string]string{
			"userId": "abc-123",
			"postId": "xyz_987",
		},
	})
	if err != nil {
		t.Fatalf("Path parameter request failed: %v", err)
	}
}

func TestHttpTLSInsecureSkipVerify(t *testing.T) {
	// Start a local HTTPS (TLS) server using self-signed certs
	server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("secure ok"))
	}))
	defer server.Close()

	h := &Http{}
	defer h.Close()

	// 1. Without InsecureSkipVerify: should fail (due to self-signed cert validation)
	_, err := h.Send(&models.Request{
		Method:             "GET",
		URL:                server.URL,
		InsecureSkipVerify: false,
	})
	if err == nil {
		t.Error("Expected connection to fail due to self-signed certificate, but got no error")
	}

	// 2. With InsecureSkipVerify: should succeed
	resp, err := h.Send(&models.Request{
		Method:             "GET",
		URL:                server.URL,
		InsecureSkipVerify: true,
	})
	if err != nil {
		t.Fatalf("Expected connection to succeed with InsecureSkipVerify: true, got: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Errorf("Expected status 200, got %d", resp.StatusCode)
	}
	if string(resp.BodyBytes) != "secure ok" {
		t.Errorf("Expected body 'secure ok', got %q", string(resp.BodyBytes))
	}
}

func TestHttpProxy(t *testing.T) {
	var proxyCalled bool
	// Mock proxy server
	proxyServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		proxyCalled = true
		w.Header().Set("X-Proxied-By", "mock-proxy")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("proxied ok"))
	}))
	defer proxyServer.Close()

	h := &Http{}
	defer h.Close()

	// Make request specifying the proxy server URL
	resp, err := h.Send(&models.Request{
		Method:   "GET",
		URL:      "http://dummy-target-url-that-does-not-exist.local/hello",
		ProxyURL: proxyServer.URL,
	})
	if err != nil {
		t.Fatalf("Proxy request failed: %v", err)
	}
	if !proxyCalled {
		t.Error("Expected proxy server to be called, but it was not")
	}
	if resp.ResponseHeaders.Get("X-Proxied-By") != "mock-proxy" {
		t.Errorf("Expected X-Proxied-By header 'mock-proxy', got %q", resp.ResponseHeaders.Get("X-Proxied-By"))
	}
	if string(resp.BodyBytes) != "proxied ok" {
		t.Errorf("Expected body 'proxied ok', got %q", string(resp.BodyBytes))
	}
}

func TestHttpPreflightExpiryParsing(t *testing.T) {
	var currentResponseBody string
	authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(currentResponseBody))
	}))
	defer authServer.Close()

	h := &Http{}
	defer h.Close()

	// Case 1: expires_in as numeric float64
	currentResponseBody = `{"access_token":"token-in-1","expires_in":7200}`
	preflight1 := &models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "body",
		TokenPath:     "access_token",
		ExpiryPath:    "expires_in",
		ExpiryType:    "expires_in",
	}
	tok1, err := h.getPreflightToken(preflight1)
	if err != nil {
		t.Fatalf("Failed getting token 1: %v", err)
	}
	if tok1 != "token-in-1" {
		t.Errorf("Expected token-in-1, got %q", tok1)
	}
	// Verify it got cached with an expiry in the future
	cacheKey1 := "GET:" + authServer.URL
	cachedVal, err := tokenCache.Get(cacheKey1)
	if err != nil {
		t.Fatalf("Expected cached value for key %q, got err: %v", cacheKey1, err)
	}
	expectedMinExpiry := time.Now().Add(7190 * time.Second) // allow some delay
	expectedMaxExpiry := time.Now().Add(7210 * time.Second)
	if cachedVal.expiresAt.Before(expectedMinExpiry) || cachedVal.expiresAt.After(expectedMaxExpiry) {
		t.Errorf("Expected expiresAt around 7200s, got %v", cachedVal.expiresAt)
	}

	// Case 2: epoch timestamp as string
	futureTime := time.Now().Add(24 * time.Hour).Unix()
	currentResponseBody = fmt.Sprintf(`{"access_token":"token-in-2","expires_at":"%d"}`, futureTime)
	preflight2 := &models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "body",
		TokenPath:     "access_token",
		ExpiryPath:    "expires_at",
		ExpiryType:    "epoch",
		CacheKey:      "cache-key-epoch-test",
	}
	tok2, err := h.getPreflightToken(preflight2)
	if err != nil {
		t.Fatalf("Failed getting token 2: %v", err)
	}
	if tok2 != "token-in-2" {
		t.Errorf("Expected token-in-2, got %q", tok2)
	}
	cachedVal2, err := tokenCache.Get("cache-key-epoch-test")
	if err != nil {
		t.Fatalf("Expected cached value, got: %v", err)
	}
	if cachedVal2.expiresAt.Unix() != futureTime {
		t.Errorf("Expected expiresAt %d, got %d", futureTime, cachedVal2.expiresAt.Unix())
	}

	// Case 3: RFC3339 string (iso8601)
	rfc3339Time := time.Now().Add(5 * time.Hour).Truncate(time.Second)
	rfc3339TimeStr := rfc3339Time.Format(time.RFC3339)
	currentResponseBody = fmt.Sprintf(`{"access_token":"token-in-3","expires_at_str":"%s"}`, rfc3339TimeStr)
	preflight3 := &models.PreflightConfig{
		Request:       &models.Request{Method: "GET", URL: authServer.URL},
		TokenLocation: "body",
		TokenPath:     "access_token",
		ExpiryPath:    "expires_at_str",
		ExpiryType:    "iso8601",
		CacheKey:      "cache-key-iso-test",
	}
	tok3, err := h.getPreflightToken(preflight3)
	if err != nil {
		t.Fatalf("Failed getting token 3: %v", err)
	}
	if tok3 != "token-in-3" {
		t.Errorf("Expected token-in-3, got %q", tok3)
	}
	cachedVal3, err := tokenCache.Get("cache-key-iso-test")
	if err != nil {
		t.Fatalf("Expected cached value, got: %v", err)
	}
	// Compare unix timestamp to avoid location/timezone local offset issues
	if cachedVal3.expiresAt.Unix() != rfc3339Time.Unix() {
		t.Errorf("Expected expiresAt unix %d, got %d (raw %v)", rfc3339Time.Unix(), cachedVal3.expiresAt.Unix(), cachedVal3.expiresAt)
	}
}
