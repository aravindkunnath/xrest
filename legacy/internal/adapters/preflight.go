package adapters

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"
	"time"

	"xrest/internal/models"
)

func getValueAtPath(data any, path string) (any, bool) {
	if path == "" {
		return data, true
	}
	parts := strings.Split(path, ".")
	var current any = data
	for _, part := range parts {
		m, ok := current.(map[string]any)
		if !ok {
			return nil, false
		}
		current, ok = m[part]
		if !ok {
			return nil, false
		}
	}
	return current, true
}

func (h *Http) getPreflightToken(cfg *models.PreflightConfig) (string, error) {
	cacheKey := cfg.CacheKey
	if cacheKey == "" {
		if cfg.Request != nil {
			cacheKey = cfg.Request.Method + ":" + cfg.Request.URL
		}
	}

	if cacheKey != "" {
		if cached, err := tokenCache.Get(cacheKey); err == nil {
			if time.Now().Before(cached.expiresAt) {
				return cached.token, nil
			}
		}
	}

	// Fetch token
	token, expiresAt, err := h.fetchPreflightToken(cfg)
	if err != nil {
		return "", err
	}

	if cacheKey != "" {
		if expiresAt.IsZero() {
			ttl := cfg.CacheTTL
			if ttl <= 0 {
				ttl = 5 * time.Minute
			}
			expiresAt = time.Now().Add(ttl)
		}
		tokenCache.Put(cacheKey, cachedToken{
			token:     token,
			expiresAt: expiresAt,
		})
	}

	return token, nil
}

// submitPreflight performs the configured preflight HTTP request, bypassing
// the preflight check to avoid recursion, and validates the response status.
func (h *Http) submitPreflight(cfg *models.PreflightConfig) (*models.Response, error) {
	if cfg == nil || cfg.Request == nil {
		return nil, fmt.Errorf("preflight request is nil")
	}

	resp, err := h.sendInternal(cfg.Request, true)
	if err != nil {
		return nil, fmt.Errorf("preflight HTTP call failed: %w", err)
	}

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return resp, fmt.Errorf("preflight HTTP call returned status %d: %s", resp.StatusCode, resp.Body)
	}

	return resp, nil
}

// extractPreflightToken reads the bearer token (and its expiry) from the
// preflight response according to the configured token location.
func extractPreflightToken(resp *models.Response, cfg *models.PreflightConfig) (string, time.Time, error) {
	var expiresAt time.Time

	var bodyData any
	var jsonParsed bool

	// Helper to parse JSON body only once
	parseJSONBody := func() error {
		if jsonParsed {
			return nil
		}
		if err := json.Unmarshal(resp.BodyBytes, &bodyData); err != nil {
			return fmt.Errorf("failed to parse preflight response body as JSON: %w", err)
		}
		jsonParsed = true
		return nil
	}

	tokenStr := ""
	if strings.ToLower(cfg.TokenLocation) == "header" {
		tokenPath := cfg.TokenPath
		if tokenPath == "" {
			return "", expiresAt, fmt.Errorf("tokenPath is required for header extraction")
		}
		tokenStr = resp.ResponseHeaders.Get(tokenPath)
		if tokenStr == "" {
			return "", expiresAt, fmt.Errorf("header %q not found in preflight response", tokenPath)
		}
	} else {
		// Default to "body" JSON parsing
		if err := parseJSONBody(); err != nil {
			return "", expiresAt, err
		}

		tokenPath := cfg.TokenPath
		if tokenPath == "" {
			tokenPath = "access_token" // standard default
		}

		val, ok := getValueAtPath(bodyData, tokenPath)
		if !ok {
			return "", expiresAt, fmt.Errorf("token key path %q not found in preflight response JSON", tokenPath)
		}

		tokenStr, ok = val.(string)
		if !ok {
			return "", expiresAt, fmt.Errorf("token value at key path %q is not a string", tokenPath)
		}
	}

	// Extract expiration if ExpiryPath is configured
	if cfg.ExpiryPath != "" {
		if err := parseJSONBody(); err == nil {
			val, ok := getValueAtPath(bodyData, cfg.ExpiryPath)
			if ok && val != nil {
				switch strings.ToLower(cfg.ExpiryType) {
				case "expires_in":
					var seconds float64
					switch v := val.(type) {
					case float64:
						seconds = v
					case string:
						var temp float64
						if _, err := fmt.Sscan(v, &temp); err == nil {
							seconds = temp
						}
					}
					if seconds > 0 {
						expiresAt = time.Now().Add(time.Duration(seconds) * time.Second)
					}
				case "epoch":
					var timestamp int64
					switch v := val.(type) {
					case float64:
						timestamp = int64(v)
					case string:
						var temp int64
						if _, err := fmt.Sscan(v, &temp); err == nil {
							timestamp = temp
						}
					}
					if timestamp > 0 {
						expiresAt = time.Unix(timestamp, 0)
					}
				case "epoch_ms":
					var timestamp int64
					switch v := val.(type) {
					case float64:
						timestamp = int64(v)
					case string:
						var temp int64
						if _, err := fmt.Sscan(v, &temp); err == nil {
							timestamp = temp
						}
					}
					if timestamp > 0 {
						expiresAt = time.UnixMilli(timestamp)
					}
				case "iso8601", "rfc3339", "":
					if valStr, ok := val.(string); ok {
						if t, err := time.Parse(time.RFC3339, valStr); err == nil {
							expiresAt = t
						}
					}
				}
			}
		}
	}

	return tokenStr, expiresAt, nil
}

// fetchPreflightToken submits the preflight request and extracts the token.
func (h *Http) fetchPreflightToken(cfg *models.PreflightConfig) (string, time.Time, error) {
	resp, err := h.submitPreflight(cfg)
	if err != nil {
		return "", time.Time{}, err
	}
	return extractPreflightToken(resp, cfg)
}

// requestHeadersToModel converts a map[string]string of request headers into
// a list of models.Header entries.
func requestHeadersToModel(h map[string]string) []models.Header {
	var out []models.Header
	for k, v := range h {
		out = append(out, models.Header{Name: k, Value: v, Enabled: true, Type: "plain"})
	}
	return out
}

// responseHeadersToModel converts an http.Header into models.Header entries.
func responseHeadersToModel(h http.Header) []models.Header {
	var out []models.Header
	for k, vals := range h {
		for _, v := range vals {
			out = append(out, models.Header{Name: k, Value: v, Enabled: true, Type: "plain"})
		}
	}
	return out
}

// TestPreflightConfig runs the configured preflight sequence and returns a
// detailed result (success, token, and the full request/response) for the UI.
func (h *Http) TestPreflightConfig(cfg *models.PreflightConfig) (models.PreflightTestResult, error) {
	result := models.PreflightTestResult{}

	if cfg == nil || cfg.Request == nil {
		errMsg := "preflight request is nil"
		result.Success = false
		result.Error = &errMsg
		return result, nil
	}

	result.RequestURL = cfg.Request.URL
	result.RequestMethod = cfg.Request.Method
	result.RequestBody = cfg.Request.BodyRaw
	result.RequestHeaders = requestHeadersToModel(cfg.Request.Headers)

	start := time.Now()
	resp, err := h.submitPreflight(cfg)
	result.TimeElapsed = uint64(time.Since(start).Milliseconds())
	if err != nil {
		result.Success = false
		errMsg := err.Error()
		result.Error = &errMsg
		if resp != nil {
			result.ResponseStatus = uint16(resp.StatusCode)
			result.ResponseBody = resp.Body
			result.ResponseHeaders = responseHeadersToModel(resp.ResponseHeaders)
		}
		return result, nil
	}

	result.ResponseStatus = uint16(resp.StatusCode)
	result.ResponseBody = resp.Body
	result.ResponseHeaders = responseHeadersToModel(resp.ResponseHeaders)

	token, _, err := extractPreflightToken(resp, cfg)
	if err != nil {
		result.Success = false
		errMsg := err.Error()
		result.Error = &errMsg
		return result, nil
	}

	result.Success = true
	result.Token = &token
	return result, nil
}
