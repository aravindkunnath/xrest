package adapters

import (
	"crypto/tls"
	"encoding/base64"
	"fmt"
	"maps"
	"net/http"
	"path/filepath"
	"regexp"
	"strings"

	"xrest/internal/models"

	"resty.dev/v3"
)

type Http struct {
	client *resty.Client
	URL    string
}

var tokenCache TokenCache

func init() {
	tokenCache = TokenCache{}
}

func (h *Http) build() *Http {
	h.client = resty.New()
	return h
}

func (h *Http) toResponse(resp *resty.Response) *models.Response {
	reqHeaders := make(http.Header)
	if resp.Request != nil {
		maps.Copy(reqHeaders, resp.Request.Header)
	}

	respHeaders := make(http.Header)
	maps.Copy(respHeaders, resp.Header())

	var cookies []models.Cookie
	if resp.RawResponse != nil {
		for _, c := range resp.RawResponse.Cookies() {
			cookies = append(cookies, models.Cookie{
				Name:     c.Name,
				Value:    c.Value,
				Path:     c.Path,
				Domain:   c.Domain,
				Expires:  c.Expires,
				MaxAge:   c.MaxAge,
				Secure:   c.Secure,
				HttpOnly: c.HttpOnly,
			})
		}
	}

	var headers []models.Header
	for k, vals := range respHeaders {
		for _, v := range vals {
			headers = append(headers, models.Header{
				Name:    k,
				Value:   v,
				Enabled: true,
				Type:    "plain",
			})
		}
	}

	bodyStr := resp.String()
	ct := strings.ToLower(resp.Header().Get("Content-Type"))
	if strings.HasPrefix(ct, "image/") || strings.HasPrefix(ct, "audio/") || strings.HasPrefix(ct, "video/") || strings.HasPrefix(ct, "application/octet-stream") || strings.HasPrefix(ct, "application/pdf") {
		bodyStr = base64.StdEncoding.EncodeToString(resp.Bytes())
	}

	return &models.Response{
		ContentType:     resp.Header().Get("Content-Type"),
		TimeTaken:       resp.Duration(),
		RequestHeaders:  reqHeaders,
		ResponseHeaders: respHeaders,
		StatusCode:      resp.StatusCode(),
		StatusText:      resp.Status(),
		BodyBytes:       resp.Bytes(),
		Size:            resp.Size(),
		Cookies:         cookies,
		Body:            bodyStr,
	}
}

func resolvePathParams(url string, params map[string]string) string {
	for k, v := range params {
		url = strings.ReplaceAll(url, ":"+k, v)
		url = strings.ReplaceAll(url, "{"+k+"}", v)
	}
	return url
}

var secretPattern = regexp.MustCompile(`\{\{secret\.([^}]+)\}\}`)

// resolveSecrets replaces {{secret.KEY}} patterns with values from SecretStore
// Returns (resolvedString, missingKeys)
func resolveSecrets(input string, store SecretStore) (string, []string) {
	if input == "" {
		return input, nil
	}

	var missingKeys []string
	result := secretPattern.ReplaceAllStringFunc(input, func(match string) string {
		// Extract key from {{secret.KEY}}
		matches := secretPattern.FindStringSubmatch(match)
		if len(matches) < 2 {
			return match
		}
		key := matches[1]
		value, err := store.Get(key)
		if err != nil {
			missingKeys = append(missingKeys, key)
			return match // Keep placeholder if secret not found
		}
		return value
	})

	return result, missingKeys
}

// resolveSecretsInMap resolves secrets in all values of a map
func resolveSecretsInMap(m map[string]string, store SecretStore) (map[string]string, []string) {
	if len(m) == 0 {
		return m, nil
	}

	var allMissing []string
	resolved := make(map[string]string, len(m))
	for k, v := range m {
		resolvedVal, missing := resolveSecrets(v, store)
		if len(missing) > 0 {
			allMissing = append(allMissing, missing...)
		}
		resolved[k] = resolvedVal
	}
	return resolved, allMissing
}

func (h *Http) Send(req *models.Request) (*models.Response, error) {
	return h.sendInternal(req, false)
}

func (h *Http) sendInternal(req *models.Request, bypassPreflight bool) (*models.Response, error) {
	if h.client == nil {
		h.build()
	}
	client := h.client

	// Resolve secrets in request fields
	store := GetSecretStore()
	var allMissingKeys []string

	// Resolve URL
	resolvedURL, missing := resolveSecrets(req.URL, store)
	if len(missing) > 0 {
		allMissingKeys = append(allMissingKeys, missing...)
	}

	// Resolve Headers
	resolvedHeaders, missing := resolveSecretsInMap(req.Headers, store)
	if len(missing) > 0 {
		allMissingKeys = append(allMissingKeys, missing...)
	}

	// Resolve QueryParams
	resolvedQueryParams, missing := resolveSecretsInMap(req.QueryParams, store)
	if len(missing) > 0 {
		allMissingKeys = append(allMissingKeys, missing...)
	}

	// Resolve PathParams
	resolvedPathParams, missing := resolveSecretsInMap(req.PathParams, store)
	if len(missing) > 0 {
		allMissingKeys = append(allMissingKeys, missing...)
	}

	// Resolve BodyRaw
	resolvedBodyRaw, missing := resolveSecrets(req.BodyRaw, store)
	if len(missing) > 0 {
		allMissingKeys = append(allMissingKeys, missing...)
	}

	// Resolve BodyForm
	resolvedBodyForm, missing := resolveSecretsInMap(req.BodyForm, store)
	if len(missing) > 0 {
		allMissingKeys = append(allMissingKeys, missing...)
	}

	// Resolve BodyFormData text values
	resolvedBodyFormData := req.BodyFormData
	if len(resolvedBodyFormData) > 0 {
		resolvedBodyFormData = make([]models.FormDataItem, len(req.BodyFormData))
		copy(resolvedBodyFormData, req.BodyFormData)
		for i, item := range resolvedBodyFormData {
			if item.Type == models.FormDataTypeText {
				resolvedVal, missing := resolveSecrets(item.Value, store)
				if len(missing) > 0 {
					allMissingKeys = append(allMissingKeys, missing...)
				}
				resolvedBodyFormData[i].Value = resolvedVal
			}
		}
	}

	// Return error if any secrets are missing
	if len(allMissingKeys) > 0 {
		return nil, fmt.Errorf("missing secrets: %v", allMissingKeys)
	}

	// Apply client-level configurations
	if req.Timeout > 0 {
		client.SetTimeout(req.Timeout)
	}
	if req.InsecureSkipVerify {
		client.SetTLSClientConfig(&tls.Config{InsecureSkipVerify: true})
	}
	if req.ProxyURL != "" {
		client.SetProxy(req.ProxyURL)
	}
	if req.FollowRedirects != nil && !*req.FollowRedirects {
		client.SetRedirectPolicy(resty.RedirectNoPolicy())
	}

	// Create request
	r := client.R()

	// Apply request-level configurations
	if len(resolvedHeaders) > 0 {
		r.SetHeaders(resolvedHeaders)
	}
	if len(resolvedQueryParams) > 0 {
		r.SetQueryParams(resolvedQueryParams)
	}

	// Resolve path parameters (using already secret-resolved path params)
	if len(resolvedPathParams) > 0 {
		resolvedURL = resolvePathParams(resolvedURL, resolvedPathParams)
	}

	// Apply preflight authentication
	if !bypassPreflight && req.Preflight != nil {
		token, err := h.getPreflightToken(req.Preflight)
		if err != nil {
			return nil, fmt.Errorf("preflight failed: %w", err)
		}

		headerName := req.Preflight.TokenHeader
		if headerName == "" {
			headerName = "Authorization"
		}

		prefix := req.Preflight.TokenPrefix
		if prefix == "" && strings.ToLower(headerName) == "authorization" {
			prefix = "Bearer "
		}

		r.SetHeader(headerName, prefix+token)
	}

	// Apply authentication
	if req.Auth != nil {
		switch req.Auth.Type {
		case models.AuthBasic:
			r.SetBasicAuth(req.Auth.BasicUsername, req.Auth.BasicPassword)
		case models.AuthBearer:
			r.SetAuthToken(req.Auth.BearerToken)
		case models.AuthAPIKey:
			if strings.ToLower(req.Auth.APIKeyAddTo) == "query" {
				r.SetQueryParam(req.Auth.APIKeyKey, req.Auth.APIKeyValue)
			} else {
				r.SetHeader(req.Auth.APIKeyKey, req.Auth.APIKeyValue)
			}
		}
	}

	// Apply body based on type
	switch req.BodyType {
	case "raw":
		r.SetBody(resolvedBodyRaw)
	case "urlencoded":
		r.SetFormData(resolvedBodyForm)
	case "form-data":
		var fields []*resty.MultipartField
		for _, item := range resolvedBodyFormData {
			if item.Type == models.FormDataTypeFile {
				fields = append(fields, &resty.MultipartField{
					Name:     item.Key,
					FilePath: item.FilePath,
					FileName: filepath.Base(item.FilePath),
				})
			} else {
				fields = append(fields, &resty.MultipartField{
					Name:   item.Key,
					Reader: strings.NewReader(item.Value),
				})
			}
		}
		r.SetMultipartFields(fields...)
	case "binary":
		r.SetBody(req.BodyBinary)
	}

	// Execute request
	method := strings.ToUpper(req.Method)
	if method == "" {
		method = "GET"
	}

	resp, err := r.Execute(method, resolvedURL)
	if err != nil {
		return nil, err
	}

	return h.toResponse(resp), nil
}

func (h *Http) Get() (*models.Response, error) {
	return h.Send(&models.Request{
		Method: "GET",
		URL:    h.URL,
	})
}

func (h *Http) Post(body string) (*models.Response, error) {
	return h.Send(&models.Request{
		Method:   "POST",
		URL:      h.URL,
		BodyType: "raw",
		BodyRaw:  body,
	})
}

func (h *Http) Put(body string) (*models.Response, error) {
	return h.Send(&models.Request{
		Method:   "PUT",
		URL:      h.URL,
		BodyType: "raw",
		BodyRaw:  body,
	})
}

func (h *Http) Delete() (*models.Response, error) {
	return h.Send(&models.Request{
		Method: "DELETE",
		URL:    h.URL,
	})
}

func (h *Http) Patch(body string) (*models.Response, error) {
	return h.Send(&models.Request{
		Method:   "PATCH",
		URL:      h.URL,
		BodyType: "raw",
		BodyRaw:  body,
	})
}

func (h *Http) Head() (*models.Response, error) {
	return h.Send(&models.Request{
		Method: "HEAD",
		URL:    h.URL,
	})
}

func (h *Http) Options() (*models.Response, error) {
	return h.Send(&models.Request{
		Method: "OPTIONS",
		URL:    h.URL,
	})
}

func (h *Http) Close() error {
	if h.client != nil {
		return h.client.Close()
	}
	return nil
}
