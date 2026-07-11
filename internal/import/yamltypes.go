package importlib

// Storage types that exactly mirror the Rust YAML serialization format.
// The Go models (internal/models) use a different PreflightConfig shape,
// so we need separate storage types for YAML round-tripping.

import "xrest/internal/models"

// ----- Storage (YAML) PreflightConfig -----
// Mirrors the Rust `PreflightConfig` in xrest-core/src/import/mod.rs.
// This is the flat format written to service.yaml and endpoint files.

type PreflightConfigStorage struct {
	Enabled           bool               `yaml:"enabled" json:"enabled"`
	Method            string             `yaml:"method" json:"method"`
	URL               string             `yaml:"url" json:"url"`
	Body              string             `yaml:"body" json:"body"`
	BodyType          string             `yaml:"bodyType" json:"bodyType"`
	BodyParams        []models.NameValue `yaml:"bodyParams" json:"bodyParams"`
	Headers           []models.NameValue `yaml:"headers" json:"headers"`
	CacheToken        bool               `yaml:"cacheToken" json:"cacheToken"`
	CacheDuration     string             `yaml:"cacheDuration" json:"cacheDuration"`
	CacheDurationKey  string             `yaml:"cacheDurationKey" json:"cacheDurationKey"`
	CacheDurationUnit string             `yaml:"cacheDurationUnit" json:"cacheDurationUnit"`
	TokenKey          string             `yaml:"tokenKey" json:"tokenKey"`
	TokenHeader       *string            `yaml:"tokenHeader" json:"tokenHeader"`
}

// ----- Storage (YAML) Endpoint -----
// Mirrors the Rust `Endpoint` (full endpoint written to endpoints/<id>.yaml)

type EndpointStorage struct {
	ID            string                   `yaml:"id" json:"id"`
	ServiceID     string                   `yaml:"serviceId" json:"serviceId"`
	Name          string                   `yaml:"name" json:"name"`
	Method        string                   `yaml:"method" json:"method"`
	URL           string                   `yaml:"url" json:"url"`
	Authenticated bool                     `yaml:"authenticated" json:"authenticated"`
	AuthType      string                   `yaml:"authType" json:"authType"`
	Metadata      models.EndpointMetadata  `yaml:"metadata" json:"metadata"`
	Params        []models.NameValue       `yaml:"params" json:"params"`
	Headers       []models.NameValue       `yaml:"headers" json:"headers"`
	Body          string                   `yaml:"body" json:"body"`
	Preflight     *PreflightConfigStorage  `yaml:"preflight,omitempty" json:"preflight,omitempty"`
	LastVersion   int32                    `yaml:"lastVersion" json:"lastVersion"`
	Versions      []models.EndpointVersion `yaml:"versions" json:"versions"`
}

// ----- Storage (YAML) ServiceFile -----
// Mirrors the Rust `ServiceFile` written to service.yaml

type StorageServiceFile struct {
	ID                  string                  `yaml:"id" json:"id"`
	Name                string                  `yaml:"name" json:"name"`
	IsAuthenticated     bool                    `yaml:"isAuthenticated" json:"isAuthenticated"`
	AuthType            *models.AuthType        `yaml:"authType,omitempty" json:"authType,omitempty"`
	Auth                *models.AuthConfig      `yaml:"auth,omitempty" json:"auth,omitempty"`
	Preflight           *PreflightConfigStorage `yaml:"preflight,omitempty" json:"preflight,omitempty"`
	Endpoints           []models.EndpointStub   `yaml:"endpoints" json:"endpoints"`
	Directory           string                  `yaml:"directory" json:"directory"`
	SelectedEnvironment *string                 `yaml:"selectedEnvironment,omitempty" json:"selectedEnvironment,omitempty"`
	GitURL              *string                 `yaml:"gitUrl,omitempty" json:"gitUrl,omitempty"`
}

// ----- Converters -----

// ToModelEndpoint converts a storage Endpoint to a models.Endpoint.
func (e *EndpointStorage) ToModelEndpoint() models.Endpoint {
	return models.Endpoint{
		ID:            e.ID,
		ServiceID:     e.ServiceID,
		Name:          e.Name,
		Method:        e.Method,
		URL:           e.URL,
		Authenticated: e.Authenticated,
		AuthType:      e.AuthType,
		Metadata:      e.Metadata,
		Params:        e.Params,
		Headers:       e.Headers,
		Body:          e.Body,
		Preflight:     storageToModelPreflight(e.Preflight),
		LastVersion:   e.LastVersion,
		Versions:      e.Versions,
	}
}

// ToModelService converts a StorageServiceFile + endpoints + environments to models.Service.
func (sf *StorageServiceFile) ToModelService(environments []models.EnvironmentConfig, endpoints []models.Endpoint) models.Service {
	return models.Service{
		ID:                  sf.ID,
		Name:                sf.Name,
		Environments:        environments,
		IsAuthenticated:     sf.IsAuthenticated,
		AuthType:            sf.AuthType,
		Auth:                sf.Auth,
		Endpoints:           endpoints,
		Directory:           sf.Directory,
		SelectedEnvironment: sf.SelectedEnvironment,
		GitURL:              sf.GitURL,
	}
}

// ToStorageServiceFile converts a models.Service (as it would be saved) to a StorageServiceFile.
func ToStorageServiceFile(svc *models.Service) StorageServiceFile {
	authType := svc.AuthType
	return StorageServiceFile{
		ID:                  svc.ID,
		Name:                svc.Name,
		IsAuthenticated:     svc.IsAuthenticated,
		AuthType:            authType,
		Auth:                svc.Auth,
		Preflight:           modelToStoragePreflight(svc.Preflight),
		Endpoints:           modelStubsFromEndpoints(svc.Endpoints),
		Directory:           svc.Directory,
		SelectedEnvironment: svc.SelectedEnvironment,
		GitURL:              svc.GitURL,
	}
}

// ToStorageEndpoint converts a models.Endpoint to a storage Endpoint.
func ToStorageEndpoint(ep models.Endpoint) EndpointStorage {
	return EndpointStorage{
		ID:            ep.ID,
		ServiceID:     ep.ServiceID,
		Name:          ep.Name,
		Method:        ep.Method,
		URL:           ep.URL,
		Authenticated: ep.Authenticated,
		AuthType:      ep.AuthType,
		Metadata:      ep.Metadata,
		Params:        ep.Params,
		Headers:       ep.Headers,
		Body:          ep.Body,
		Preflight:     modelToStoragePreflight(ep.Preflight),
		LastVersion:   ep.LastVersion,
		Versions:      ep.Versions,
	}
}

// ----- Preflight conversion helpers -----

func storageToModelPreflight(p *PreflightConfigStorage) *models.PreflightConfig {
	if p == nil {
		return nil
	}
	execHeaders := make(map[string]string)
	for _, h := range p.Headers {
		if h.Enabled {
			execHeaders[h.Name] = h.Value
		}
	}
	tokenHeader := "Authorization"
	if p.TokenHeader != nil && *p.TokenHeader != "" {
		tokenHeader = *p.TokenHeader
	}
	bodyType := p.BodyType
	if bodyType == "" {
		bodyType = "raw"
	}
	return &models.PreflightConfig{
		Request: &models.Request{
			Method:   p.Method,
			URL:      p.URL,
			BodyRaw:  p.Body,
			BodyType: bodyType,
			Headers:  execHeaders,
		},
		TokenLocation: "body",
		TokenPath:     p.TokenKey,
		TokenHeader:   tokenHeader,
		TokenPrefix:   "Bearer ",
		ExpiryPath:    p.CacheDurationKey,
		ExpiryType:    "expires_in",
		CacheKey:      p.Method + ":" + p.URL,
	}
}

func modelToStoragePreflight(p *models.PreflightConfig) *PreflightConfigStorage {
	if p == nil || p.Request == nil {
		return nil
	}
	tokenHeader := p.TokenHeader
	if tokenHeader == "" {
		tokenHeader = "Authorization"
	}
	nvs := make([]models.NameValue, 0, len(p.Request.Headers))
	for k, v := range p.Request.Headers {
		nvs = append(nvs, models.NameValue{Name: k, Value: v, Enabled: true, Type: "plain"})
	}
	return &PreflightConfigStorage{
		Enabled:           true,
		Method:            p.Request.Method,
		URL:               p.Request.URL,
		Body:              p.Request.BodyRaw,
		BodyType:          p.Request.BodyType,
		Headers:           nvs,
		CacheToken:        true,
		CacheDuration:     "derived",
		CacheDurationKey:  p.ExpiryPath,
		CacheDurationUnit: "seconds",
		TokenKey:          p.TokenPath,
		TokenHeader:       &tokenHeader,
	}
}

func modelStubsFromEndpoints(eps []models.Endpoint) []models.EndpointStub {
	stubs := make([]models.EndpointStub, len(eps))
	for i, ep := range eps {
		stubs[i] = models.EndpointStub{
			ID:   ep.ID,
			Name: ep.Name,
			URL:  ep.URL,
		}
	}
	return stubs
}
