package adapters

import (
	"crypto/tls"
	"fmt"
	"maps"
	"net/http"
	"path/filepath"
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
		Body:            resp.String(),
	}
}

func resolvePathParams(url string, params map[string]string) string {
	for k, v := range params {
		url = strings.ReplaceAll(url, ":"+k, v)
		url = strings.ReplaceAll(url, "{"+k+"}", v)
	}
	return url
}

func (h *Http) Send(req *models.Request) (*models.Response, error) {
	return h.sendInternal(req, false)
}

func (h *Http) sendInternal(req *models.Request, bypassPreflight bool) (*models.Response, error) {
	if h.client == nil {
		h.build()
	}
	client := h.client

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
	if len(req.Headers) > 0 {
		r.SetHeaders(req.Headers)
	}
	if len(req.QueryParams) > 0 {
		r.SetQueryParams(req.QueryParams)
	}

	// Resolve path parameters
	resolvedURL := req.URL
	if len(req.PathParams) > 0 {
		resolvedURL = resolvePathParams(req.URL, req.PathParams)
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
		r.SetBody(req.BodyRaw)
	case "urlencoded":
		r.SetFormData(req.BodyForm)
	case "form-data":
		var fields []*resty.MultipartField
		for _, item := range req.BodyFormData {
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
