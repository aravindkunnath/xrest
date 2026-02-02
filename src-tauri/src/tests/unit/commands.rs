use crate::commands::parse_spec_content;

#[test]
fn test_parse_openapi_3_json() {
    let content = r#"{
        "openapi": "3.0.0",
        "info": { "title": "Test API", "version": "1.0.0" },
        "servers": [{ "url": "https://api.test.com" }],
        "paths": {
            "/users": {
                "get": {
                    "summary": "Get Users",
                    "parameters": [
                        { "name": "limit", "in": "query", "schema": { "type": "integer" } }
                    ],
                    "responses": {}
                }
            }
        }
    }"#;
    let (base_url, endpoints) = parse_spec_content(content, "s1").unwrap();
    assert_eq!(base_url, "https://api.test.com");
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].name, "Get Users");
    assert_eq!(endpoints[0].method, "GET");
    assert_eq!(endpoints[0].url, "/users");
    assert_eq!(endpoints[0].params.len(), 1);
    assert_eq!(endpoints[0].params[0].name, "limit");
}

#[test]
fn test_parse_swagger_2_yaml() {
    let content = r#"
swagger: "2.0"
host: "api.test.com"
basePath: "/v1"
schemes:
  - "https"
paths:
  /items:
    post:
      summary: "Create Item"
      parameters:
        - name: "X-Request-ID"
          in: "header"
          type: "string"
"#;
    let (base_url, endpoints) = parse_spec_content(content, "s1").unwrap();
    assert_eq!(base_url, "https://api.test.com/v1");
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0].name, "Create Item");
    assert_eq!(endpoints[0].method, "POST");
    assert_eq!(endpoints[0].headers.len(), 1);
    assert_eq!(endpoints[0].headers[0].name, "X-Request-ID");
}

#[test]
fn test_curl_to_endpoint_parsing() {
    use crate::commands::curl_to_endpoint;

    // Test simple GET
    let curl = "curl https://api.example.com/users";
    let endpoint = curl_to_endpoint("s1".to_string(), curl, false, None).unwrap();
    assert_eq!(endpoint.method, "GET");
    assert_eq!(endpoint.url, "https://api.example.com/users");
    assert_eq!(endpoint.name, "users");

    // Test POST with body and headers
    let curl = r#"curl -X POST https://api.example.com/login -H "Content-Type: application/json" -d '{"user":"test"}'"#;
    let endpoint =
        curl_to_endpoint("s1".to_string(), curl, true, Some("bearer".to_string())).unwrap();
    assert_eq!(endpoint.method, "POST");
    assert_eq!(endpoint.url, "https://api.example.com/login");
    assert_eq!(endpoint.name, "login");
    assert_eq!(endpoint.body, r#"{"user":"test"}"#);
    assert!(endpoint
        .headers
        .iter()
        .any(|h| h.name == "content-type" && h.value == "application/json"));
    assert_eq!(endpoint.authenticated, true);
    assert_eq!(endpoint.auth_type, "bearer");
}
