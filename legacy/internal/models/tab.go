package models

import (
	"encoding/json"
	"fmt"
)

type BodyConfig struct {
	Type    string `json:"type" yaml:"type"`
	Content string `json:"content" yaml:"content"`
}

type RequestTab struct {
	ID           string            `json:"id" yaml:"id"`
	EndpointID   *string           `json:"endpointId,omitempty" yaml:"endpointId,omitempty"`
	Title        string            `json:"title" yaml:"title"`
	Method       string            `json:"method" yaml:"method"`
	URL          string            `json:"url" yaml:"url"`
	Params       []Param           `json:"params" yaml:"params"`
	Headers      []Header          `json:"headers" yaml:"headers"`
	Body         BodyConfig        `json:"body" yaml:"body"`
	Auth         AuthConfig        `json:"auth" yaml:"auth"`
	ActiveSubTab *string           `json:"activeSubTab,omitempty" yaml:"activeSubTab,omitempty"`
	ServiceID    *string           `json:"serviceId,omitempty" yaml:"serviceId,omitempty"`
	Preflight    *PreflightConfig  `json:"preflight,omitempty" yaml:"preflight,omitempty"`
	Variables    map[string]string `json:"variables,omitempty" yaml:"variables,omitempty"`
	IsEdited     bool              `json:"isEdited" yaml:"isEdited"`
}

type SettingsTab struct {
	ID          string  `json:"id" yaml:"id"`
	Title       string  `json:"title" yaml:"title"`
	ServiceID   string  `json:"serviceId" yaml:"serviceId"`
	ServiceData Service `json:"serviceData" yaml:"serviceData"`
	IsEdited    bool    `json:"isEdited" yaml:"isEdited"`
}

type Tab struct {
	Type     string       `json:"type" yaml:"type"`
	Request  *RequestTab  `json:"request,omitempty" yaml:"request,omitempty"`
	Settings *SettingsTab `json:"settings,omitempty" yaml:"settings,omitempty"`
}

func (t Tab) MarshalJSON() ([]byte, error) {
	if t.Type == "request" && t.Request != nil {
		type Alias RequestTab
		return json.Marshal(&struct {
			Type string `json:"type"`
			*Alias
		}{
			Type:  "request",
			Alias: (*Alias)(t.Request),
		})
	}
	if t.Type == "settings" && t.Settings != nil {
		type Alias SettingsTab
		return json.Marshal(&struct {
			Type string `json:"type"`
			*Alias
		}{
			Type:  "settings",
			Alias: (*Alias)(t.Settings),
		})
	}
	return nil, fmt.Errorf("invalid tab type")
}

func (t *Tab) UnmarshalJSON(data []byte) error {
	var tag struct {
		Type string `json:"type"`
	}
	if err := json.Unmarshal(data, &tag); err != nil {
		return err
	}
	t.Type = tag.Type
	switch tag.Type {
	case "request":
		var req RequestTab
		if err := json.Unmarshal(data, &req); err != nil {
			return err
		}
		t.Request = &req
	case "settings":
		var sett SettingsTab
		if err := json.Unmarshal(data, &sett); err != nil {
			return err
		}
		t.Settings = &sett
	default:
		return fmt.Errorf("unknown tab type: %s", tag.Type)
	}
	return nil
}

func (t Tab) MarshalYAML() (interface{}, error) {
	if t.Type == "request" && t.Request != nil {
		b, err := json.Marshal(t.Request)
		if err != nil {
			return nil, err
		}
		var m map[string]interface{}
		if err := json.Unmarshal(b, &m); err != nil {
			return nil, err
		}
		m["type"] = "request"
		return m, nil
	}
	if t.Type == "settings" && t.Settings != nil {
		b, err := json.Marshal(t.Settings)
		if err != nil {
			return nil, err
		}
		var m map[string]interface{}
		if err := json.Unmarshal(b, &m); err != nil {
			return nil, err
		}
		m["type"] = "settings"
		return m, nil
	}
	return nil, fmt.Errorf("invalid tab type")
}

func (t *Tab) UnmarshalYAML(unmarshal func(interface{}) error) error {
	var m map[string]interface{}
	if err := unmarshal(&m); err != nil {
		return err
	}
	tagVal, ok := m["type"]
	if !ok {
		return fmt.Errorf("missing type tag in Tab")
	}
	tag, ok := tagVal.(string)
	if !ok {
		return fmt.Errorf("type tag in Tab is not a string")
	}
	t.Type = tag
	b, err := json.Marshal(m)
	if err != nil {
		return err
	}
	switch tag {
	case "request":
		var req RequestTab
		if err := json.Unmarshal(b, &req); err != nil {
			return err
		}
		t.Request = &req
	case "settings":
		var sett SettingsTab
		if err := json.Unmarshal(b, &sett); err != nil {
			return err
		}
		t.Settings = &sett
	default:
		return fmt.Errorf("unknown tab type: %s", tag)
	}
	return nil
}

type TabState struct {
	ActiveTabID string `json:"activeTabId" yaml:"activeTabId"`
	Tabs        []Tab  `json:"tabs" yaml:"tabs"`
}
