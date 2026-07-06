package models

type AuthType string

const (
	AuthNone   AuthType = "none"
	AuthBasic  AuthType = "basic"
	AuthBearer AuthType = "bearer"
	AuthAPIKey AuthType = "apikey"
)

type Auth struct {
	Type          AuthType `json:"type" yaml:"type"`
	BasicUsername string   `json:"basicUsername,omitempty" yaml:"basicUsername,omitempty"`
	BasicPassword string   `json:"basicPassword,omitempty" yaml:"basicPassword,omitempty"`
	BearerToken   string   `json:"bearerToken,omitempty" yaml:"bearerToken,omitempty"`
	APIKeyKey     string   `json:"apiKeyKey,omitempty" yaml:"apiKeyKey,omitempty"`
	APIKeyValue   string   `json:"apiKeyValue,omitempty" yaml:"apiKeyValue,omitempty"`
	APIKeyAddTo   string   `json:"apiKeyAddTo,omitempty" yaml:"apiKeyAddTo,omitempty"` // "header" or "query"
}

type AuthConfig struct {
	Type           string `json:"type" yaml:"type"`
	Active         bool   `json:"active" yaml:"active"`
	BasicUser      string `json:"basicUser" yaml:"basicUser"`
	BasicPass      string `json:"basicPass" yaml:"basicPass"`
	BearerToken    string `json:"bearerToken" yaml:"bearerToken"`
	APIKeyName     string `json:"apiKeyName" yaml:"apiKeyName"`
	APIKeyValue    string `json:"apiKeyValue" yaml:"apiKeyValue"`
	APIKeyLocation string `json:"apiKeyLocation" yaml:"apiKeyLocation"`
}
