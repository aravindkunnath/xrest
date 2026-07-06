package models

type EnvironmentConfig struct {
	Name      string     `json:"name" yaml:"name"`
	IsUnsafe  bool       `json:"isUnsafe" yaml:"isUnsafe"`
	Variables []Variable `json:"variables" yaml:"variables"`
}

type EndpointMetadata struct {
	Version     string `json:"version" yaml:"version"`
	LastUpdated uint64 `json:"lastUpdated" yaml:"lastUpdated"`
}

type RequestConfig struct {
	Method        string           `json:"method" yaml:"method"`
	URL           string           `json:"url" yaml:"url"`
	Authenticated bool             `json:"authenticated" yaml:"authenticated"`
	AuthType      string           `json:"authType" yaml:"authType"`
	Params        []Param          `json:"params" yaml:"params"`
	Headers       []Header         `json:"headers" yaml:"headers"`
	Body          string           `json:"body" yaml:"body"`
	Preflight     *PreflightConfig `json:"preflight,omitempty" yaml:"preflight,omitempty"`
}

type EndpointVersion struct {
	Version     int32         `json:"version" yaml:"version"`
	Config      RequestConfig `json:"config" yaml:"config"`
	LastUpdated uint64        `json:"lastUpdated" yaml:"lastUpdated"`
}

type Endpoint struct {
	ID            string            `json:"id" yaml:"id"`
	ServiceID     string            `json:"serviceId" yaml:"serviceId"`
	Name          string            `json:"name" yaml:"name"`
	Method        string            `json:"method" yaml:"method"`
	URL           string            `json:"url" yaml:"url"`
	Authenticated bool              `json:"authenticated" yaml:"authenticated"`
	AuthType      string            `json:"authType" yaml:"authType"`
	Metadata      EndpointMetadata  `json:"metadata" yaml:"metadata"`
	Params        []Param           `json:"params" yaml:"params"`
	Headers       []Header          `json:"headers" yaml:"headers"`
	Body          string            `json:"body" yaml:"body"`
	Preflight     *PreflightConfig  `json:"preflight,omitempty" yaml:"preflight,omitempty"`
	LastVersion   int32             `json:"lastVersion" yaml:"lastVersion"`
	Versions      []EndpointVersion `json:"versions" yaml:"versions"`
}

type EndpointStub struct {
	ID   string `json:"id" yaml:"id"`
	Name string `json:"name" yaml:"name"`
	URL  string `json:"url" yaml:"url"`
}

type Service struct {
	ID                  string              `json:"id" yaml:"id"`
	Name                string              `json:"name" yaml:"name"`
	Environments        []EnvironmentConfig `json:"environments" yaml:"environments"`
	IsAuthenticated     bool                `json:"isAuthenticated" yaml:"isAuthenticated"`
	AuthType            *AuthType           `json:"authType,omitempty" yaml:"authType,omitempty"`
	Auth                *AuthConfig         `json:"auth,omitempty" yaml:"auth,omitempty"`
	Preflight           *PreflightConfig    `json:"preflight,omitempty" yaml:"preflight,omitempty"`
	Endpoints           []Endpoint          `json:"endpoints" yaml:"endpoints"`
	Directory           string              `json:"directory" yaml:"directory"`
	SelectedEnvironment *string             `json:"selectedEnvironment,omitempty" yaml:"selectedEnvironment,omitempty"`
	GitURL              *string             `json:"gitUrl,omitempty" yaml:"gitUrl,omitempty"`
}

type ServiceFile struct {
	ID                  string           `json:"id" yaml:"id"`
	Name                string           `json:"name" yaml:"name"`
	IsAuthenticated     bool             `json:"isAuthenticated" yaml:"isAuthenticated"`
	AuthType            *AuthType        `json:"authType,omitempty" yaml:"authType,omitempty"`
	Auth                *AuthConfig      `json:"auth,omitempty" yaml:"auth,omitempty"`
	Preflight           *PreflightConfig `json:"preflight,omitempty" yaml:"preflight,omitempty"`
	Endpoints           []EndpointStub   `json:"endpoints" yaml:"endpoints"`
	Directory           string           `json:"directory" yaml:"directory"`
	SelectedEnvironment *string          `json:"selectedEnvironment,omitempty" yaml:"selectedEnvironment,omitempty"`
	GitURL              *string          `json:"gitUrl,omitempty" yaml:"gitUrl,omitempty"`
}
