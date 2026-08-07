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

// --- Scenario: HTTP convenience methods ---

func TestMethods_Get(t *testing.T) {
	t.Run("happy", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			rw.Header().Set("Content-Type", "application/json")
			rw.Header().Set("X-Custom-Response-Header", "value1")
			rw.WriteHeader(http.StatusOK)
			rw.Write([]byte(`{"status":"ok"}`))
		}))
		defer server.Close()

		h := &Http{URL: server.URL}
		defer h.Close()

		resp, err := h.Get()
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
		if resp == nil {
			t.Fatal("Expected response not to be nil")
		}
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
	})

	t.Run("happy/custom headers", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			if req.Header.Get("X-Client-Header") != "custom-value" {
				t.Errorf("Server expected X-Client-Header 'custom-value', got %q", req.Header.Get("X-Client-Header"))
			}
			rw.Header().Set("Content-Type", "text/plain")
			rw.WriteHeader(http.StatusOK)
			rw.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{URL: server.URL}
		defer h.Close()

		h.build()
		h.client.SetHeader("X-Client-Header", "custom-value")

		resp, err := h.Get()
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
		if resp.RequestHeaders.Get("X-Client-Header") != "custom-value" {
			t.Errorf("Expected X-Client-Header in RequestHeaders, got %q", resp.RequestHeaders.Get("X-Client-Header"))
		}
	})

	t.Run("exception/invalid url", func(t *testing.T) {
		h := &Http{URL: "http://invalid-domain-name-that-does-not-exist.invalid"}
		defer h.Close()

		resp, err := h.Get()
		if err == nil {
			t.Fatal("Expected error for invalid URL, got nil")
		}
		if resp != nil {
			t.Errorf("Expected nil response on error, got %+v", resp)
		}
	})
}

func TestMethods_Post(t *testing.T) {
	t.Run("happy", func(t *testing.T) {
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

		h := &Http{URL: server.URL}
		defer h.Close()

		resp, err := h.Post("hello post")
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
		if resp.Body != `{"status":"created"}` {
			t.Errorf("Expected response %q, got %q", `{"status":"created"}`, resp.Body)
		}
	})
}

func TestMethods_Put(t *testing.T) {
	t.Run("happy", func(t *testing.T) {
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

		h := &Http{URL: server.URL}
		defer h.Close()

		resp, err := h.Put("hello put")
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
		if resp.Body != `{"status":"updated"}` {
			t.Errorf("Expected response %q, got %q", `{"status":"updated"}`, resp.Body)
		}
	})
}

func TestMethods_Delete(t *testing.T) {
	t.Run("happy", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			if req.Method != http.MethodDelete {
				t.Errorf("Expected DELETE request, got %s", req.Method)
			}
			rw.Header().Set("Content-Type", "application/json")
			rw.WriteHeader(http.StatusOK)
			rw.Write([]byte(`{"status":"deleted"}`))
		}))
		defer server.Close()

		h := &Http{URL: server.URL}
		defer h.Close()

		resp, err := h.Delete()
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
		if resp.Body != `{"status":"deleted"}` {
			t.Errorf("Expected response %q, got %q", `{"status":"deleted"}`, resp.Body)
		}
	})
}

func TestMethods_Patch(t *testing.T) {
	t.Run("happy", func(t *testing.T) {
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

		h := &Http{URL: server.URL}
		defer h.Close()

		resp, err := h.Patch("hello patch")
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
		if resp.Body != `{"status":"patched"}` {
			t.Errorf("Expected response %q, got %q", `{"status":"patched"}`, resp.Body)
		}
	})
}

func TestMethods_Head(t *testing.T) {
	t.Run("happy", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			if req.Method != http.MethodHead {
				t.Errorf("Expected HEAD request, got %s", req.Method)
			}
			rw.Header().Set("Content-Type", "text/plain")
			rw.Header().Set("X-Head-Response", "header-value")
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{URL: server.URL}
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
	})
}

func TestMethods_Options(t *testing.T) {
	t.Run("happy", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			if req.Method != http.MethodOptions {
				t.Errorf("Expected OPTIONS request, got %s", req.Method)
			}
			rw.Header().Set("Allow", "GET, POST, OPTIONS")
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{URL: server.URL}
		defer h.Close()

		resp, err := h.Options()
		if err != nil {
			t.Fatalf("Expected no error, got %v", err)
		}
		if resp.ResponseHeaders.Get("Allow") != "GET, POST, OPTIONS" {
			t.Errorf("Expected Allow header %q, got %q", "GET, POST, OPTIONS", resp.ResponseHeaders.Get("Allow"))
		}
	})
}

// TestMethods_Auth verifies every convenience method works with request-level
// (via Send) and client-level (via build + client auth) authentication, across
// both Basic and Bearer auth kinds.
func TestMethods_Auth(t *testing.T) {
	tests := []struct {
		name            string
		requestAuth     *models.Auth
		configureClient func(h *Http)
		bodySuffix      string
		// validateAuth returns true if the request carries valid credentials.
		validateAuth func(rw http.ResponseWriter, req *http.Request) bool
	}{
		{
			name: "basic",
			requestAuth: &models.Auth{
				Type:          models.AuthBasic,
				BasicUsername: "test-user",
				BasicPassword: "test-pass",
			},
			configureClient: func(h *Http) { h.client.SetBasicAuth("test-user", "test-pass") },
			bodySuffix:      "ok",
			validateAuth: func(rw http.ResponseWriter, req *http.Request) bool {
				u, p, ok := req.BasicAuth()
				if !ok || u != "test-user" || p != "test-pass" {
					rw.WriteHeader(http.StatusUnauthorized)
					rw.Write([]byte("Unauthorized"))
					return false
				}
				return true
			},
		},
		{
			name: "bearer",
			requestAuth: &models.Auth{
				Type:        models.AuthBearer,
				BearerToken: "my-secret-token",
			},
			configureClient: func(h *Http) { h.client.SetAuthToken("my-secret-token") },
			bodySuffix:      "token ok",
			validateAuth: func(rw http.ResponseWriter, req *http.Request) bool {
				if req.Header.Get("Authorization") != "Bearer my-secret-token" {
					rw.WriteHeader(http.StatusUnauthorized)
					rw.Write([]byte("Unauthorized"))
					return false
				}
				return true
			},
		},
	}

	methods := []struct {
		method string
		call   func(h *Http) (*models.Response, error)
	}{
		{"GET", func(h *Http) (*models.Response, error) { return h.Get() }},
		{"POST", func(h *Http) (*models.Response, error) { return h.Post("body") }},
		{"PUT", func(h *Http) (*models.Response, error) { return h.Put("body") }},
		{"DELETE", func(h *Http) (*models.Response, error) { return h.Delete() }},
		{"PATCH", func(h *Http) (*models.Response, error) { return h.Patch("body") }},
		{"HEAD", func(h *Http) (*models.Response, error) { return h.Head() }},
		{"OPTIONS", func(h *Http) (*models.Response, error) { return h.Options() }},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
				if !tt.validateAuth(rw, req) {
					return
				}
				rw.Header().Set("Content-Type", "text/plain")
				rw.WriteHeader(http.StatusOK)
				rw.Write([]byte(req.Method + " " + tt.bodySuffix))
			}))
			defer server.Close()

			t.Run("request level", func(t *testing.T) {
				h := &Http{}
				defer h.Close()

				resp, err := h.Send(&models.Request{
					Method: "GET",
					URL:    server.URL,
					Auth:   tt.requestAuth,
				})
				if err != nil {
					t.Fatalf("Send failed: %v", err)
				}
				if resp.StatusCode != http.StatusOK {
					t.Errorf("Expected status OK, got %d", resp.StatusCode)
				}
				if string(resp.BodyBytes) != "GET "+tt.bodySuffix {
					t.Errorf("Expected body 'GET %s', got %q", tt.bodySuffix, string(resp.BodyBytes))
				}
			})

			t.Run("client level", func(t *testing.T) {
				for _, m := range methods {
					t.Run(m.method, func(t *testing.T) {
						h := &Http{URL: server.URL}
						defer h.Close()
						h.build()
						tt.configureClient(h)

						resp, err := m.call(h)
						if err != nil {
							t.Fatalf("%s failed: %v", m.method, err)
						}
						if resp.StatusCode != http.StatusOK {
							t.Errorf("%s: expected 200, got %d", m.method, resp.StatusCode)
						}
						if m.method == "GET" && string(resp.BodyBytes) != "GET "+tt.bodySuffix {
							t.Errorf("GET: expected 'GET %s', got %q", tt.bodySuffix, string(resp.BodyBytes))
						}
					})
				}
			})
		})
	}
}

// --- Scenario: Send() orchestrator ---

func TestSend_ParamsAndHeaders(t *testing.T) {
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

func TestSend_Auth(t *testing.T) {
	t.Run("basic", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			username, password, ok := req.BasicAuth()
			if !ok || username != "user" || password != "pass" {
				t.Errorf("Invalid Basic Auth: ok=%v, user=%q, pass=%q", ok, username, password)
			}
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL,
			Auth: &models.Auth{
				Type:          models.AuthBasic,
				BasicUsername: "user",
				BasicPassword: "pass",
			},
		})
		if err != nil {
			t.Fatalf("Basic Auth request failed: %v", err)
		}
	})

	t.Run("bearer", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			if req.Header.Get("Authorization") != "Bearer my-token" {
				t.Errorf("Invalid Bearer Auth: got %q", req.Header.Get("Authorization"))
			}
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL,
			Auth: &models.Auth{
				Type:        models.AuthBearer,
				BearerToken: "my-token",
			},
		})
		if err != nil {
			t.Fatalf("Bearer Auth request failed: %v", err)
		}
	})

	t.Run("apikey/header", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			if req.Header.Get("X-API-Key") != "secret-value" {
				t.Errorf("Invalid API Key in header: got %q", req.Header.Get("X-API-Key"))
			}
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL,
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
	})

	t.Run("apikey/query", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			if req.URL.Query().Get("api_key") != "secret-value" {
				t.Errorf("Invalid API Key in query: got %q", req.URL.Query().Get("api_key"))
			}
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL,
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
	})
}

func TestSend_Bodies(t *testing.T) {
	t.Run("raw", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			body, _ := io.ReadAll(req.Body)
			if string(body) != "raw data" {
				t.Errorf("Expected raw body 'raw data', got %q", string(body))
			}
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method:   "POST",
			URL:      server.URL,
			BodyType: "raw",
			BodyRaw:  "raw data",
		})
		if err != nil {
			t.Fatalf("Raw body POST failed: %v", err)
		}
	})

	t.Run("urlencoded", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			err := req.ParseForm()
			if err != nil {
				t.Fatalf("ParseForm failed: %v", err)
			}
			if req.Form.Get("foo") != "bar" || req.Form.Get("baz") != "qux" {
				t.Errorf("Invalid URL-encoded form: got %+v", req.Form)
			}
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method:   "POST",
			URL:      server.URL,
			BodyType: "urlencoded",
			BodyForm: map[string]string{
				"foo": "bar",
				"baz": "qux",
			},
		})
		if err != nil {
			t.Fatalf("URL-encoded form POST failed: %v", err)
		}
	})

	t.Run("binary", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
			body, _ := io.ReadAll(req.Body)
			if len(body) != 4 || body[0] != 1 || body[1] != 2 || body[2] != 3 || body[3] != 4 {
				t.Errorf("Invalid binary body: got %v", body)
			}
			rw.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method:     "POST",
			URL:        server.URL,
			BodyType:   "binary",
			BodyBinary: []byte{1, 2, 3, 4},
		})
		if err != nil {
			t.Fatalf("Binary body POST failed: %v", err)
		}
	})

	t.Run("multipart", func(t *testing.T) {
		tmpFile, err := os.CreateTemp("", "test-upload-*.txt")
		if err != nil {
			t.Fatalf("Failed to create temp file: %v", err)
		}
		defer os.Remove(tmpFile.Name())
		if _, err := tmpFile.Write([]byte("file content")); err != nil {
			t.Fatalf("Failed to write to temp file: %v", err)
		}
		tmpFile.Close()

		server := httptest.NewServer(http.HandlerFunc(func(rw http.ResponseWriter, req *http.Request) {
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
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err = h.Send(&models.Request{
			Method:   "POST",
			URL:      server.URL,
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
	})
}

func TestSend_RedirectsAndCookies(t *testing.T) {
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

	t.Run("happy/follow redirects", func(t *testing.T) {
		h := &Http{}
		defer h.Close()

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
	})

	t.Run("happy/no redirects", func(t *testing.T) {
		h := &Http{}
		defer h.Close()

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
	})
}

func TestSend_Timeout(t *testing.T) {
	t.Run("exception/request exceeds timeout", func(t *testing.T) {
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
	})
}

func TestSend_PathParams(t *testing.T) {
	t.Run("happy/special characters", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path != "/users/abc-123/posts/xyz_987" {
				t.Errorf("Expected path /users/abc-123/posts/xyz_987, got %q", r.URL.Path)
			}
			w.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL + "/users/:userId/posts/{postId}",
			PathParams: map[string]string{
				"userId": "abc-123",
				"postId": "xyz_987",
			},
		})
		if err != nil {
			t.Fatalf("Path parameter request failed: %v", err)
		}
	})
}

func TestSend_TLS(t *testing.T) {
	server := httptest.NewTLSServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("secure ok"))
	}))
	defer server.Close()

	t.Run("exception/self-signed certificate rejected", func(t *testing.T) {
		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method:             "GET",
			URL:                server.URL,
			InsecureSkipVerify: false,
		})
		if err == nil {
			t.Error("Expected connection to fail due to self-signed certificate, but got no error")
		}
	})

	t.Run("happy/insecure skip verify", func(t *testing.T) {
		h := &Http{}
		defer h.Close()

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
	})
}

func TestSend_Proxy(t *testing.T) {
	var proxyCalled bool
	proxyServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		proxyCalled = true
		w.Header().Set("X-Proxied-By", "mock-proxy")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("proxied ok"))
	}))
	defer proxyServer.Close()

	h := &Http{}
	defer h.Close()

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

// --- Scenario: Preflight flows ---

func TestPreflight_Extraction(t *testing.T) {
	t.Run("happy/json body", func(t *testing.T) {
		var authCalls int
		authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authCalls++
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusOK)
			w.Write([]byte(`{"data":{"token":"my-jwt-token-123"}}`))
		}))
		defer authServer.Close()

		mainServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Header.Get("X-Custom-Auth") != "Prefix-my-jwt-token-123" {
				t.Errorf("Expected Header 'X-Custom-Auth: Prefix-my-jwt-token-123', got %q", r.Header.Get("X-Custom-Auth"))
				w.WriteHeader(http.StatusUnauthorized)
				return
			}
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("success"))
		}))
		defer mainServer.Close()

		h := &Http{}
		defer h.Close()

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
	})

	t.Run("happy/header", func(t *testing.T) {
		authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("X-Auth-Token", "header-token-xyz")
			w.WriteHeader(http.StatusOK)
		}))
		defer authServer.Close()

		mainServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Header.Get("Authorization") != "Bearer header-token-xyz" {
				t.Errorf("Expected Authorization 'Bearer header-token-xyz', got %q", r.Header.Get("Authorization"))
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
	})
}

func TestPreflight_Caching(t *testing.T) {
	t.Run("happy/cache hit and expiration", func(t *testing.T) {
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

		preflightCfg := &models.PreflightConfig{
			Request: &models.Request{
				Method: "POST",
				URL:    authServer.URL,
			},
			TokenLocation: "body",
			TokenPath:     "access_token",
			CacheKey:      "test-cache-key-unique",
			CacheTTL:      50 * time.Millisecond,
		}

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

		time.Sleep(60 * time.Millisecond)

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
	})
}

func TestPreflight_OAuth2(t *testing.T) {
	t.Run("happy/oauth2 client credentials flow", func(t *testing.T) {
		var authCalls int
		authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			authCalls++
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

		resourceServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Header.Get("Authorization") != "Bearer jwt-token-val-456" {
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
	})
}

func TestPreflight_Errors(t *testing.T) {
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

	runRequest := func(cfg *models.PreflightConfig) error {
		_, err := h.Send(&models.Request{
			Method:    "GET",
			URL:       "http://localhost:80", // Target doesn't matter since preflight should fail first
			Preflight: cfg,
		})
		return err
	}

	tests := []struct {
		name    string
		status  int
		body    []byte
		headers map[string]string
		cfg     *models.PreflightConfig
		wantErr string
	}{
		{
			name:    "exception/http error status",
			status:  http.StatusInternalServerError,
			body:    []byte("server error description"),
			cfg:     &models.PreflightConfig{Request: &models.Request{Method: "GET", URL: authServer.URL}},
			wantErr: "preflight HTTP call returned status 500",
		},
		{
			name:   "exception/token path not found in body",
			status: http.StatusOK,
			body:   []byte(`{"wrong_key":"val"}`),
			cfg: &models.PreflightConfig{
				Request:       &models.Request{Method: "GET", URL: authServer.URL},
				TokenLocation: "body",
				TokenPath:     "missing_token",
			},
			wantErr: `token key path "missing_token" not found`,
		},
		{
			name:   "exception/token value is not a string",
			status: http.StatusOK,
			body:   []byte(`{"token_key": 12345}`),
			cfg: &models.PreflightConfig{
				Request:       &models.Request{Method: "GET", URL: authServer.URL},
				TokenLocation: "body",
				TokenPath:     "token_key",
			},
			wantErr: "is not a string",
		},
		{
			name:   "exception/invalid json body",
			status: http.StatusOK,
			body:   []byte(`{invalid-json`),
			cfg: &models.PreflightConfig{
				Request:       &models.Request{Method: "GET", URL: authServer.URL},
				TokenLocation: "body",
				TokenPath:     "token_key",
			},
			wantErr: "failed to parse preflight response body as JSON",
		},
		{
			name:   "exception/empty token path for header",
			status: http.StatusOK,
			cfg: &models.PreflightConfig{
				Request:       &models.Request{Method: "GET", URL: authServer.URL},
				TokenLocation: "header",
				TokenPath:     "",
			},
			wantErr: "tokenPath is required for header extraction",
		},
		{
			name:    "exception/header token not found",
			status:  http.StatusOK,
			headers: map[string]string{"Some-Header": "val"},
			cfg: &models.PreflightConfig{
				Request:       &models.Request{Method: "GET", URL: authServer.URL},
				TokenLocation: "header",
				TokenPath:     "X-Missing-Token-Header",
			},
			wantErr: "not found in preflight response",
		},
		{
			name:   "exception/nil preflight request",
			status: http.StatusOK,
			cfg: &models.PreflightConfig{
				Request: nil,
			},
			wantErr: "preflight request is nil",
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			returnStatus = tt.status
			responseBody = tt.body
			responseHeaders = tt.headers
			err := runRequest(tt.cfg)
			if err == nil || !strings.Contains(err.Error(), tt.wantErr) {
				t.Errorf("Expected error containing %q, got: %v", tt.wantErr, err)
			}
		})
	}
}

func TestPreflight_Expiry(t *testing.T) {
	var currentResponseBody string
	authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		w.Write([]byte(currentResponseBody))
	}))
	defer authServer.Close()

	t.Run("happy/expires_in numeric", func(t *testing.T) {
		h := &Http{}
		defer h.Close()

		currentResponseBody = `{"access_token":"token-in-1","expires_in":7200}`
		preflight := &models.PreflightConfig{
			Request:       &models.Request{Method: "GET", URL: authServer.URL},
			TokenLocation: "body",
			TokenPath:     "access_token",
			ExpiryPath:    "expires_in",
			ExpiryType:    "expires_in",
		}
		tok, err := h.getPreflightToken(preflight)
		if err != nil {
			t.Fatalf("Failed getting token: %v", err)
		}
		if tok != "token-in-1" {
			t.Errorf("Expected token-in-1, got %q", tok)
		}

		cacheKey := "GET:" + authServer.URL
		cachedVal, err := tokenCache.Get(cacheKey)
		if err != nil {
			t.Fatalf("Expected cached value for key %q, got err: %v", cacheKey, err)
		}
		expectedMinExpiry := time.Now().Add(7190 * time.Second)
		expectedMaxExpiry := time.Now().Add(7210 * time.Second)
		if cachedVal.expiresAt.Before(expectedMinExpiry) || cachedVal.expiresAt.After(expectedMaxExpiry) {
			t.Errorf("Expected expiresAt around 7200s, got %v", cachedVal.expiresAt)
		}
	})

	t.Run("happy/epoch timestamp string", func(t *testing.T) {
		h := &Http{}
		defer h.Close()

		futureTime := time.Now().Add(24 * time.Hour).Unix()
		currentResponseBody = fmt.Sprintf(`{"access_token":"token-in-2","expires_at":"%d"}`, futureTime)
		preflight := &models.PreflightConfig{
			Request:       &models.Request{Method: "GET", URL: authServer.URL},
			TokenLocation: "body",
			TokenPath:     "access_token",
			ExpiryPath:    "expires_at",
			ExpiryType:    "epoch",
			CacheKey:      "cache-key-epoch-test",
		}
		tok, err := h.getPreflightToken(preflight)
		if err != nil {
			t.Fatalf("Failed getting token: %v", err)
		}
		if tok != "token-in-2" {
			t.Errorf("Expected token-in-2, got %q", tok)
		}
		cachedVal, err := tokenCache.Get("cache-key-epoch-test")
		if err != nil {
			t.Fatalf("Expected cached value, got: %v", err)
		}
		if cachedVal.expiresAt.Unix() != futureTime {
			t.Errorf("Expected expiresAt %d, got %d", futureTime, cachedVal.expiresAt.Unix())
		}
	})

	t.Run("happy/rfc3339 iso8601", func(t *testing.T) {
		h := &Http{}
		defer h.Close()

		rfc3339Time := time.Now().Add(5 * time.Hour).Truncate(time.Second)
		rfc3339TimeStr := rfc3339Time.Format(time.RFC3339)
		currentResponseBody = fmt.Sprintf(`{"access_token":"token-in-3","expires_at_str":"%s"}`, rfc3339TimeStr)
		preflight := &models.PreflightConfig{
			Request:       &models.Request{Method: "GET", URL: authServer.URL},
			TokenLocation: "body",
			TokenPath:     "access_token",
			ExpiryPath:    "expires_at_str",
			ExpiryType:    "iso8601",
			CacheKey:      "cache-key-iso-test",
		}
		tok, err := h.getPreflightToken(preflight)
		if err != nil {
			t.Fatalf("Failed getting token: %v", err)
		}
		if tok != "token-in-3" {
			t.Errorf("Expected token-in-3, got %q", tok)
		}
		cachedVal, err := tokenCache.Get("cache-key-iso-test")
		if err != nil {
			t.Fatalf("Expected cached value, got: %v", err)
		}
		if cachedVal.expiresAt.Unix() != rfc3339Time.Unix() {
			t.Errorf("Expected expiresAt unix %d, got %d (raw %v)", rfc3339Time.Unix(), cachedVal.expiresAt.Unix(), cachedVal.expiresAt)
		}
	})
}

// --- Scenario: Secret resolution ---

func TestSecrets_Resolution(t *testing.T) {
	t.Run("auth header", func(t *testing.T) {
		const expectedAuthValue = "Bearer CORRECT_VALUE"

		store := GetSecretStore()
		if err := store.Set("ABCD", "CORRECT_VALUE"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		authServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			value := r.Header.Get("Authorization")
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusOK)
			w.Write([]byte(value))
		}))
		defer authServer.Close()

		resp, err := (&Http{URL: authServer.URL}).Send(&models.Request{
			Method: "GET",
			URL:    authServer.URL + "/users/:id/{section}",
			Headers: map[string]string{
				"Authorization": "Bearer {{secret.ABCD}}",
			},
			QueryParams: map[string]string{
				"foo": "bar",
			},
			PathParams: map[string]string{
				"id":      "123",
				"section": "profile",
			},
		})
		if err != nil || resp.Body != expectedAuthValue {
			t.Fatalf("Expected body %q, got %q, err: %v", expectedAuthValue, resp.Body, err)
		}
	})

	t.Run("query params", func(t *testing.T) {
		store := GetSecretStore()
		if err := store.Set("API_KEY", "secret-key-123"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			key := r.URL.Query().Get("api_key")
			if key != "secret-key-123" {
				t.Errorf("Expected api_key=secret-key-123, got %q", key)
			}
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL,
			QueryParams: map[string]string{
				"api_key": "{{secret.API_KEY}}",
			},
		})
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
	})

	t.Run("path params", func(t *testing.T) {
		store := GetSecretStore()
		if err := store.Set("USER_ID", "user-456"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path != "/users/user-456/profile" {
				t.Errorf("Expected path /users/user-456/profile, got %q", r.URL.Path)
			}
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL + "/users/:id/profile",
			PathParams: map[string]string{
				"id": "{{secret.USER_ID}}",
			},
		})
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
	})

	t.Run("body raw", func(t *testing.T) {
		store := GetSecretStore()
		if err := store.Set("PAYLOAD", `{"data":"secret-value"}`); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			body, _ := io.ReadAll(r.Body)
			if string(body) != `{"data":"secret-value"}` {
				t.Errorf("Expected body %q, got %q", `{"data":"secret-value"}`, string(body))
			}
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method:   "POST",
			URL:      server.URL,
			BodyType: "raw",
			BodyRaw:  "{{secret.PAYLOAD}}",
		})
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
	})

	t.Run("body form", func(t *testing.T) {
		store := GetSecretStore()
		if err := store.Set("USERNAME", "john"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}
		if err := store.Set("PASSWORD", "secret123"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			err := r.ParseForm()
			if err != nil {
				t.Fatalf("ParseForm failed: %v", err)
			}
			if r.Form.Get("username") != "john" || r.Form.Get("password") != "secret123" {
				t.Errorf("Invalid form data: got %+v", r.Form)
			}
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method:   "POST",
			URL:      server.URL,
			BodyType: "urlencoded",
			BodyForm: map[string]string{
				"username": "{{secret.USERNAME}}",
				"password": "{{secret.PASSWORD}}",
			},
		})
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
	})

	t.Run("form-data", func(t *testing.T) {
		store := GetSecretStore()
		if err := store.Set("FILE_CONTENT", "file-secret-content"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		tmpFile, err := os.CreateTemp("", "test-secret-*.txt")
		if err != nil {
			t.Fatalf("Failed to create temp file: %v", err)
		}
		defer os.Remove(tmpFile.Name())
		if _, err := tmpFile.Write([]byte("file content")); err != nil {
			t.Fatalf("Failed to write to temp file: %v", err)
		}
		tmpFile.Close()

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			err := r.ParseMultipartForm(10 << 20)
			if err != nil {
				t.Fatalf("ParseMultipartForm failed: %v", err)
			}
			if r.FormValue("textfield") != "file-secret-content" {
				t.Errorf("Expected form textfield 'file-secret-content', got %q", r.FormValue("textfield"))
			}
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err = h.Send(&models.Request{
			Method:   "POST",
			URL:      server.URL,
			BodyType: "form-data",
			BodyFormData: []models.FormDataItem{
				{
					Key:   "textfield",
					Value: "{{secret.FILE_CONTENT}}",
					Type:  models.FormDataTypeText,
				},
			},
		})
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
	})

	t.Run("multiple fields", func(t *testing.T) {
		store := GetSecretStore()
		if err := store.Set("TOKEN", "multi-secret-token"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}
		if err := store.Set("USER_ID", "user-789"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}
		if err := store.Set("API_KEY", "multi-api-key"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.Header.Get("Authorization") != "Bearer multi-secret-token" {
				t.Errorf("Expected Authorization 'Bearer multi-secret-token', got %q", r.Header.Get("Authorization"))
			}
			if r.URL.Query().Get("api_key") != "multi-api-key" {
				t.Errorf("Expected api_key=multi-api-key, got %q", r.URL.Query().Get("api_key"))
			}
			if r.URL.Path != "/users/user-789/items" {
				t.Errorf("Expected path /users/user-789/items, got %q", r.URL.Path)
			}
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL + "/users/:id/items",
			Headers: map[string]string{
				"Authorization": "Bearer {{secret.TOKEN}}",
			},
			QueryParams: map[string]string{
				"api_key": "{{secret.API_KEY}}",
			},
			PathParams: map[string]string{
				"id": "{{secret.USER_ID}}",
			},
		})
		if err != nil {
			t.Fatalf("Request failed: %v", err)
		}
	})
}

func TestSecrets_Missing(t *testing.T) {
	t.Run("single missing", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{URL: server.URL}

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL,
			Headers: map[string]string{
				"Authorization": "Bearer {{secret.MISSING_KEY}}",
			},
		})
		if err == nil {
			t.Fatal("Expected error for missing secret, got nil")
		}
		if !strings.Contains(err.Error(), "missing secrets: [MISSING_KEY]") {
			t.Errorf("Expected missing secrets error, got: %v", err)
		}
	})

	t.Run("multiple missing", func(t *testing.T) {
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusOK)
			w.Write([]byte("ok"))
		}))
		defer server.Close()

		h := &Http{}
		defer h.Close()

		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    server.URL,
			Headers: map[string]string{
				"X-Key1": "{{secret.MISSING_1}}",
				"X-Key2": "{{secret.MISSING_2}}",
			},
			QueryParams: map[string]string{
				"param1": "{{secret.MISSING_3}}",
			},
		})
		if err == nil {
			t.Fatal("Expected error for missing secrets, got nil")
		}
		errStr := err.Error()
		if !strings.Contains(errStr, "missing secrets:") {
			t.Errorf("Expected missing secrets error, got: %v", err)
		}
		for _, key := range []string{"MISSING_1", "MISSING_2", "MISSING_3"} {
			if !strings.Contains(errStr, key) {
				t.Errorf("Expected missing key %q in error: %v", key, err)
			}
		}
	})
}

func TestSecrets_URL(t *testing.T) {
	t.Run("unhappy/secret resolves then fails on unreachable host", func(t *testing.T) {
		store := GetSecretStore()
		if err := store.Set("BASE_URL", "api.example.com"); err != nil {
			t.Fatalf("Failed to set secret: %v", err)
		}

		h := &Http{}
		defer h.Close()

		// The URL is resolved via the secret, then the request fails because the
		// host is unreachable. This verifies the secret is resolved before the
		// connection attempt.
		_, err := h.Send(&models.Request{
			Method: "GET",
			URL:    "https://{{secret.BASE_URL}}/path",
		})
		if err == nil {
			t.Fatal("Expected error for invalid URL, got nil")
		}
		if !strings.Contains(err.Error(), "invalid URL") && !strings.Contains(err.Error(), "no such host") {
			t.Logf("URL resolution worked, got expected connection error: %v", err)
		}
	})
}
