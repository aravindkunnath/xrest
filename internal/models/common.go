package models

type NameValue struct {
	Name      string `json:"name" yaml:"name"`
	Value     string `json:"value" yaml:"value"`
	Enabled   bool   `json:"enabled" yaml:"enabled"`
	SecretKey string `json:"secretKey,omitempty" yaml:"secretKey,omitempty"`
	Type      string `json:"type" yaml:"type"`
}

type Param = NameValue
type Header = NameValue
type Variable = NameValue

type UserSettings struct {
	Version  string        `json:"version" yaml:"version"`
	Theme    string        `json:"theme" yaml:"theme"`
	Services []ServiceStub `json:"services" yaml:"services"`
}

type ServiceStub struct {
	ID        string `json:"id" yaml:"id"`
	Name      string `json:"name" yaml:"name"`
	Directory string `json:"directory" yaml:"directory"`
}
