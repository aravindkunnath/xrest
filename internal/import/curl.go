package importlib

import (
	"fmt"
	"regexp"
	"strings"
	"time"

	"xrest/internal/models"

	"github.com/google/uuid"
)

// CurlToEndpoint parses a cURL command and returns an Endpoint.
// This is a port of the Rust implementation in xrest-core/src/import/curl.rs.
func CurlToEndpoint(serviceID, curlCommand string, authenticated bool, authType *string) (models.Endpoint, error) {
	parsed, err := parseCurl(curlCommand)
	if err != nil {
		return models.Endpoint{}, fmt.Errorf("failed to parse cURL: %w", err)
	}

	now := uint64(time.Now().Unix())

	return models.Endpoint{
		ID:            "e-" + uuid.NewString(),
		ServiceID:     serviceID,
		Name:          extractEndpointName(parsed.url),
		Method:        parsed.method,
		URL:           parsed.url,
		Authenticated: authenticated,
		AuthType: func() string {
			if authType != nil {
				return *authType
			}
			return "none"
		}(),
		Metadata: models.EndpointMetadata{
			Version:     "1.0",
			LastUpdated: now,
		},
		Params:  parsed.queryParams,
		Headers: parsed.headers,
		Body:    parsed.body,
		Preflight: &models.PreflightConfig{
			Request: &models.Request{},
		},
		LastVersion: 0,
		Versions:    nil,
	}, nil
}

// curlParseResult holds the parsed components of a cURL command.
type curlParseResult struct {
	method      string
	url         string
	headers     []models.Header
	body        string
	queryParams []models.Param
}

// parseCurl parses a cURL command string using regex-based parsing.
func parseCurl(cmd string) (*curlParseResult, error) {
	cmd = strings.TrimSpace(cmd)

	// Remove leading "curl " prefix (case-insensitive)
	re := regexp.MustCompile(`(?i)^curl\s+`)
	if !re.MatchString(cmd) {
		return nil, fmt.Errorf("command does not start with 'curl'")
	}
	cmd = re.ReplaceAllString(cmd, "")

	result := &curlParseResult{
		method: "GET",
	}

	// Parse tokens handling quoted strings
	tokens := tokenize(cmd)

	for i := 0; i < len(tokens); i++ {
		tok := tokens[i]

		switch {
		case tok == "-X" || tok == "--request":
			if i+1 < len(tokens) {
				result.method = strings.ToUpper(tokens[i+1])
				i++
			}
		case tok == "-H" || tok == "--header":
			if i+1 < len(tokens) {
				headerStr := unquote(tokens[i+1])
				if colonIdx := strings.Index(headerStr, ":"); colonIdx > 0 {
					name := strings.TrimSpace(headerStr[:colonIdx])
					value := strings.TrimSpace(headerStr[colonIdx+1:])
					result.headers = append(result.headers, models.Header{
						Name:    name,
						Value:   value,
						Enabled: true,
						Type:    "plain",
					})
				}
				i++
			}
		case tok == "-d" || tok == "--data" || tok == "--data-raw" || tok == "--data-binary":
			if i+1 < len(tokens) {
				result.body = unquote(tokens[i+1])
				if result.method == "GET" {
					result.method = "POST"
				}
				i++
			}
		case tok == "--data-urlencode":
			if i+1 < len(tokens) {
				raw := unquote(tokens[i+1])
				result.body = raw
				if result.method == "GET" {
					result.method = "POST"
				}
				i++
			}
		case tok == "-u" || tok == "--user":
			// Skip basic auth user:password — it gets stored in the service auth config
			if i+1 < len(tokens) {
				i++
			}
		case tok == "-b" || tok == "--cookie":
			// Skip cookie header
			if i+1 < len(tokens) {
				i++
			}
		case tok == "-A" || tok == "--user-agent":
			if i+1 < len(tokens) {
				i++
			}
		case tok == "-k" || tok == "--insecure":
			// SSL skip — ignored for endpoint import
		case tok == "-L" || tok == "--location":
			// Follow redirects — ignored for endpoint import
		case tok == "-i" || tok == "--include":
			// Include response headers — ignored
		case tok == "-s" || tok == "--silent":
			// Silent — ignored
		case tok == "-S" || tok == "--show-error":
			// Show error — ignored
		case tok == "-o" || tok == "--output":
			// Output file — skip next token
			if i+1 < len(tokens) {
				i++
			}
		case tok == "-w" || tok == "--write-out":
			// Write-out format — skip next token
			if i+1 < len(tokens) {
				i++
			}
		case tok == "-v" || tok == "--verbose":
			// Verbose — ignored
		case tok == "--compressed":
			// Compressed — ignored
		case tok == "--connect-timeout":
			if i+1 < len(tokens) {
				i++
			}
		case tok == "--max-time":
			if i+1 < len(tokens) {
				i++
			}
		case tok == "--retry":
			if i+1 < len(tokens) {
				i++
			}
		case tok == "--retry-delay":
			if i+1 < len(tokens) {
				i++
			}
		case strings.HasPrefix(tok, "-"):
			// Unknown flag — if it expects an argument, skip
			if i+1 < len(tokens) && !strings.HasPrefix(tokens[i+1], "-") {
				i++
			}
		default:
			// Should be the URL
			if result.url == "" {
				raw := unquote(tok)
				// Handle inline method like "POST http://..." or "GET http://..."
				parts := strings.SplitN(raw, " ", 2)
				if len(parts) == 2 && isHTTPMethod(parts[0]) && strings.HasPrefix(parts[1], "http") {
					result.method = parts[0]
					result.url = parts[1]
				} else {
					result.url = raw
				}
			} else {
				// Skip any extra positional tokens
			}
		}
	}

	if result.url == "" {
		return nil, fmt.Errorf("no URL found in cURL command")
	}

	// Extract query params from URL
	result.queryParams = extractQueryParams(result.url)

	return result, nil
}

// tokenize splits a shell-like command into tokens, handling quotes.
func tokenize(cmd string) []string {
	var tokens []string
	var current strings.Builder
	inSingle := false
	inDouble := false
	escape := false

	for i := 0; i < len(cmd); i++ {
		ch := cmd[i]

		if escape {
			current.WriteByte(ch)
			escape = false
			continue
		}

		if ch == '\\' && inDouble {
			escape = true
			continue
		}

		if ch == '\\' && !inSingle && !inDouble {
			if i+1 < len(cmd) {
				i++
				current.WriteByte(cmd[i])
			}
			continue
		}

		if ch == '\'' && !inDouble {
			inSingle = !inSingle
			current.WriteByte(ch)
			continue
		}

		if ch == '"' && !inSingle {
			inDouble = !inDouble
			current.WriteByte(ch)
			continue
		}

		if ch == ' ' && !inSingle && !inDouble {
			if current.Len() > 0 {
				tokens = append(tokens, current.String())
				current.Reset()
			}
			continue
		}

		current.WriteByte(ch)
	}

	if current.Len() > 0 {
		tokens = append(tokens, current.String())
	}

	return tokens
}

// unquote removes surrounding quotes from a string.
func unquote(s string) string {
	if len(s) >= 2 {
		if (s[0] == '"' && s[len(s)-1] == '"') || (s[0] == '\'' && s[len(s)-1] == '\'') {
			return s[1 : len(s)-1]
		}
	}
	return s
}

// extractEndpointName derives a human-readable name from a URL path.
func extractEndpointName(urlStr string) string {
	// Find the path part
	if idx := strings.Index(urlStr, "://"); idx > 0 {
		noScheme := urlStr[idx+3:]
		if slashIdx := strings.Index(noScheme, "/"); slashIdx >= 0 {
			path := noScheme[slashIdx+1:]
			// Remove query string
			if qIdx := strings.Index(path, "?"); qIdx >= 0 {
				path = path[:qIdx]
			}
			if path == "" {
				return "root"
			}
			// Replace / with spaces, capitalize words
			path = strings.ReplaceAll(path, "/", " ")
			path = strings.ReplaceAll(path, "-", " ")
			path = strings.ReplaceAll(path, "_", " ")
			// Collapse multiple spaces
			spaceRE := regexp.MustCompile(`\s+`)
			path = strings.TrimSpace(spaceRE.ReplaceAllString(path, " "))
			if path == "" {
				return "New Endpoint"
			}
			return path
		}
		return "root"
	}
	return "New Endpoint"
}

// extractQueryParams parses query parameters from a URL string.
func extractQueryParams(urlStr string) []models.Param {
	if qIdx := strings.Index(urlStr, "?"); qIdx >= 0 {
		query := urlStr[qIdx+1:]
		// Remove fragment
		if fIdx := strings.Index(query, "#"); fIdx >= 0 {
			query = query[:fIdx]
		}
		var params []models.Param
		for _, pair := range strings.Split(query, "&") {
			if pair == "" {
				continue
			}
			kv := strings.SplitN(pair, "=", 2)
			p := models.Param{
				Name:    kv[0],
				Enabled: true,
				Type:    "plain",
			}
			if len(kv) > 1 {
				p.Value = kv[1]
			}
			params = append(params, p)
		}
		return params
	}
	return nil
}

// isHTTPMethod checks if a string is an HTTP method.
func isHTTPMethod(s string) bool {
	switch s {
	case "GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS", "CONNECT", "TRACE":
		return true
	}
	return false
}
