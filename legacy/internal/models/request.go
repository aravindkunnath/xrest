package models

import (
	"net/http"
	"time"
)

type Cookie struct {
	Name     string    `json:"name" yaml:"name"`
	Value    string    `json:"value" yaml:"value"`
	Path     string    `json:"path" yaml:"path"`
	Domain   string    `json:"domain" yaml:"domain"`
	Expires  time.Time `json:"expires" yaml:"expires"`
	MaxAge   int       `json:"maxAge" yaml:"maxAge"`
	Secure   bool      `json:"secure" yaml:"secure"`
	HttpOnly bool      `json:"httpOnly" yaml:"httpOnly"`
}

type Response struct {
	ContentType     string        `json:"contentType" yaml:"contentType"`
	TimeTaken       time.Duration `json:"timeTaken" yaml:"timeTaken"`
	RequestHeaders  http.Header   `json:"requestHeaders" yaml:"requestHeaders"`
	ResponseHeaders http.Header   `json:"responseHeaders" yaml:"responseHeaders"`
	StatusCode      int           `json:"statusCode" yaml:"statusCode"`
	StatusText      string        `json:"statusText" yaml:"statusText"`
	BodyBytes       []byte        `json:"bodyBytes" yaml:"bodyBytes"`
	Size            int64         `json:"size" yaml:"size"`
	Cookies         []Cookie      `json:"cookies" yaml:"cookies"`
	Body            string        `json:"body" yaml:"body"`
	Error           *string       `json:"error,omitempty" yaml:"error,omitempty"`
}

type FormDataType string

const (
	FormDataTypeText FormDataType = "text"
	FormDataTypeFile FormDataType = "file"
)

type FormDataItem struct {
	Key      string       `json:"key" yaml:"key"`
	Value    string       `json:"value" yaml:"value"`
	Type     FormDataType `json:"type" yaml:"type"`
	FilePath string       `json:"filePath,omitempty" yaml:"filePath,omitempty"`
}

type PreflightConfig struct {
	Request       *Request      `json:"request" yaml:"request"`
	TokenLocation string        `json:"tokenLocation" yaml:"tokenLocation"` // "body" or "header"
	TokenPath     string        `json:"tokenPath" yaml:"tokenPath"`         // JSON key path or header key name
	TokenHeader   string        `json:"tokenHeader" yaml:"tokenHeader"`     // Header name to write (defaults to "Authorization")
	TokenPrefix   string        `json:"tokenPrefix" yaml:"tokenPrefix"`     // Prefix (e.g. "Bearer ")
	CacheKey      string        `json:"cacheKey,omitempty" yaml:"cacheKey,omitempty"`
	CacheTTL      time.Duration `json:"cacheTtl,omitempty" yaml:"cacheTtl,omitempty"`
	ExpiryPath    string        `json:"expiryPath,omitempty" yaml:"expiryPath,omitempty"` // JSON key path for expiry
	ExpiryType    string        `json:"expiryType,omitempty" yaml:"expiryType,omitempty"` // "expires_in", "epoch", "epoch_ms", "iso8601"
}

type Request struct {
	Method             string            `json:"method" yaml:"method"`
	URL                string            `json:"url" yaml:"url"`
	Headers            map[string]string `json:"headers" yaml:"headers"`
	QueryParams        map[string]string `json:"queryParams" yaml:"queryParams"`
	PathParams         map[string]string `json:"pathParams" yaml:"pathParams"`
	Auth               *Auth             `json:"auth,omitempty" yaml:"auth,omitempty"`
	BodyType           string            `json:"bodyType" yaml:"bodyType"` // "none", "raw", "form-data", "urlencoded", "binary"
	BodyRaw            string            `json:"bodyRaw" yaml:"bodyRaw"`
	BodyForm           map[string]string `json:"bodyForm" yaml:"bodyForm"`
	BodyFormData       []FormDataItem    `json:"bodyFormData" yaml:"bodyFormData"`
	BodyBinary         []byte            `json:"bodyBinary" yaml:"bodyBinary"`
	Timeout            time.Duration     `json:"timeout" yaml:"timeout"`
	FollowRedirects    *bool             `json:"followRedirects" yaml:"followRedirects"`
	InsecureSkipVerify bool              `json:"insecureSkipVerify" yaml:"insecureSkipVerify"`
	ProxyURL           string            `json:"proxyUrl" yaml:"proxyUrl"`
	Preflight          *PreflightConfig  `json:"preflight,omitempty" yaml:"preflight,omitempty"`
}

type PreflightTestResult struct {
	Success         bool     `json:"success" yaml:"success"`
	Token           *string  `json:"token,omitempty" yaml:"token,omitempty"`
	Error           *string  `json:"error,omitempty" yaml:"error,omitempty"`
	RequestURL      string   `json:"requestUrl" yaml:"requestUrl"`
	RequestMethod   string   `json:"requestMethod" yaml:"requestMethod"`
	RequestHeaders  []Header `json:"requestHeaders" yaml:"requestHeaders"`
	RequestBody     string   `json:"requestBody" yaml:"requestBody"`
	ResponseStatus  uint16   `json:"responseStatus" yaml:"responseStatus"`
	ResponseBody    string   `json:"responseBody" yaml:"responseBody"`
	ResponseHeaders []Header `json:"responseHeaders" yaml:"responseHeaders"`
	TimeElapsed     uint64   `json:"timeElapsed" yaml:"timeElapsed"`
}
