package importlib

import (
	"fmt"
	"strings"
	"time"

	"xrest/internal/models"

	"github.com/getkin/kin-openapi/openapi2"
	"github.com/getkin/kin-openapi/openapi2conv"
	"github.com/getkin/kin-openapi/openapi3"
	"github.com/google/uuid"
	"gopkg.in/yaml.v3"
)

// ParseSpecContent parses an OpenAPI 3 or Swagger 2 spec and returns base URL + endpoints.
func ParseSpecContent(content, serviceID string) (string, []models.Endpoint, error) {
	if strings.TrimSpace(content) == "" {
		return "", nil, fmt.Errorf("empty spec content")
	}

	now := uint64(time.Now().Unix())

	// Try as OpenAPI 3 JSON
	{
		doc3 := &openapi3.T{}
		if err := doc3.UnmarshalJSON([]byte(content)); err == nil {
			return doc3ToEndpoints(doc3, serviceID, now)
		}
	}

	// Try as OpenAPI 3 YAML
	{
		loader := &openapi3.Loader{}
		doc3, err := loader.LoadFromData([]byte(content))
		if err == nil && doc3 != nil && doc3.Info != nil {
			return doc3ToEndpoints(doc3, serviceID, now)
		}
	}

	// Try as Swagger 2 JSON
	{
		doc2 := &openapi2.T{}
		if err := doc2.UnmarshalJSON([]byte(content)); err == nil && doc2.Swagger != "" {
			doc3, err := openapi2conv.ToV3(doc2)
			if err == nil {
				return doc3ToEndpoints(doc3, serviceID, now)
			}
		}
	}

	// Try as Swagger 2 YAML
	{
		doc2 := &openapi2.T{}
		if err := yaml.Unmarshal([]byte(content), doc2); err == nil && doc2.Swagger != "" {
			doc3, err := openapi2conv.ToV3(doc2)
			if err == nil {
				return doc3ToEndpoints(doc3, serviceID, now)
			}
		}
	}

	// Fallback: parse as generic map and handle manually
	var spec map[string]interface{}
	if err := yaml.Unmarshal([]byte(content), &spec); err != nil {
		return "", nil, fmt.Errorf("failed to parse spec as YAML: %w", err)
	}
	return parseGenericSpec(spec, serviceID, now)
}

// doc3ToEndpoints extracts endpoints from a parsed OpenAPI 3 document.
func doc3ToEndpoints(doc3 *openapi3.T, serviceID string, now uint64) (string, []models.Endpoint, error) {
	var baseURL string
	if len(doc3.Servers) > 0 {
		baseURL = doc3.Servers[0].URL
	} else {
		baseURL = "https://api.example.com"
	}

	var endpoints []models.Endpoint

	if doc3.Paths == nil {
		return baseURL, endpoints, nil
	}

	for _, path := range doc3.Paths.InMatchingOrder() {
		pi := doc3.Paths.Value(path)
		if pi == nil {
			continue
		}

		ops := map[string]*openapi3.Operation{
			"GET":     pi.Get,
			"POST":    pi.Post,
			"PUT":     pi.Put,
			"DELETE":  pi.Delete,
			"PATCH":   pi.Patch,
			"HEAD":    pi.Head,
			"OPTIONS": pi.Options,
		}

		for method, op := range ops {
			if op == nil {
				continue
			}
			endpointName := op.Summary
			if endpointName == "" {
				endpointName = op.OperationID
			}
			if endpointName == "" {
				endpointName = fmt.Sprintf("%s %s", method, path)
			}

			var params []models.Param
			var headers []models.Header

			for _, paramRef := range op.Parameters {
				if paramRef == nil || paramRef.Value == nil {
					continue
				}
				p := paramRef.Value
				nv := models.NameValue{
					Name:    p.Name,
					Value:   "",
					Enabled: true,
					Type:    "plain",
				}
				if p.Schema != nil && p.Schema.Value != nil {
					if defaultVal, ok := p.Schema.Value.Default.(string); ok {
						nv.Value = defaultVal
					}
				}
				switch p.In {
				case "query":
					params = append(params, nv)
				case "header":
					headers = append(headers, nv)
				}
			}

			endpoints = append(endpoints, models.Endpoint{
				ID:        "e-" + uuid.NewString(),
				ServiceID: serviceID,
				Name:      endpointName,
				Method:    method,
				URL:       path,
				Metadata: models.EndpointMetadata{
					Version:     "1.0",
					LastUpdated: now,
				},
				Params:      params,
				Headers:     headers,
				Preflight:   nil,
				LastVersion: 0,
				Versions:    nil,
			})
		}
	}

	return baseURL, endpoints, nil
}

// parseGenericSpec handles manually-parsed specs.
func parseGenericSpec(spec map[string]interface{}, serviceID string, now uint64) (string, []models.Endpoint, error) {
	baseURL := extractBaseURL(spec)

	var endpoints []models.Endpoint

	paths, ok := getMap(spec, "paths")
	if !ok {
		return "", nil, fmt.Errorf("spec has no 'paths' field")
	}

	for path, pathItem := range paths {
		pathStr, ok := pathItem.(map[string]interface{})
		if !ok {
			continue
		}
		for method, opValue := range pathStr {
			methodUpper := strings.ToUpper(method)
			if !isHTTPMethod(methodUpper) {
				continue
			}
			opMap, ok := opValue.(map[string]interface{})
			if !ok {
				continue
			}

			endpointName := getString(opMap, "summary")
			if endpointName == "" {
				endpointName = getString(opMap, "operationId")
			}
			if endpointName == "" {
				endpointName = fmt.Sprintf("%s %s", methodUpper, path)
			}

			var params []models.Param
			var headers []models.Header

			if rawParams, ok := getSlice(opMap, "parameters"); ok {
				for _, raw := range rawParams {
					pMap, ok := raw.(map[string]interface{})
					if !ok {
						continue
					}
					pName := getString(pMap, "name")
					pIn := getString(pMap, "in")
					if pName == "" {
						continue
					}
					nv := models.NameValue{
						Name:    pName,
						Value:   "",
						Enabled: true,
						Type:    "plain",
					}
					switch pIn {
					case "query":
						params = append(params, nv)
					case "header":
						headers = append(headers, nv)
					}
				}
			}

			endpoints = append(endpoints, models.Endpoint{
				ID:        "e-" + uuid.NewString(),
				ServiceID: serviceID,
				Name:      endpointName,
				Method:    methodUpper,
				URL:       path,
				Metadata: models.EndpointMetadata{
					Version:     "1.0",
					LastUpdated: now,
				},
				Params:      params,
				Headers:     headers,
				Preflight:   nil,
				LastVersion: 0,
				Versions:    nil,
			})
		}
	}

	if len(endpoints) == 0 {
		return "", nil, fmt.Errorf("no endpoints found in spec")
	}

	return baseURL, endpoints, nil
}

// extractBaseURL extracts a base URL from an OpenAPI/Swagger spec map.
func extractBaseURL(spec map[string]interface{}) string {
	// Try OpenAPI 3 servers
	if servers, ok := getSlice(spec, "servers"); ok && len(servers) > 0 {
		if server, ok := servers[0].(map[string]interface{}); ok {
			if u := getString(server, "url"); u != "" {
				return u
			}
		}
	}

	// Try Swagger 2
	host := getString(spec, "host")
	scheme := "https"
	if schemes, ok := getSlice(spec, "schemes"); ok && len(schemes) > 0 {
		if s, ok := schemes[0].(string); ok {
			scheme = s
		}
	}
	basePath := getString(spec, "basePath")

	if host != "" {
		return fmt.Sprintf("%s://%s%s", scheme, host, basePath)
	}

	return "https://api.example.com"
}

// ----- Generic map helpers -----

func getMap(m map[string]interface{}, key string) (map[string]interface{}, bool) {
	v, ok := m[key]
	if !ok {
		return nil, false
	}
	result, ok := v.(map[string]interface{})
	return result, ok
}

func getSlice(m map[string]interface{}, key string) ([]interface{}, bool) {
	v, ok := m[key]
	if !ok {
		return nil, false
	}
	result, ok := v.([]interface{})
	return result, ok
}

func getString(m map[string]interface{}, key string) string {
	v, ok := m[key]
	if !ok {
		return ""
	}
	s, _ := v.(string)
	return s
}
