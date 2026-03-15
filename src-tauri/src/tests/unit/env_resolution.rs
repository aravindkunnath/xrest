#[cfg(test)]
mod tests {
    use crate::core::traits::{MockFileSystem, MockHttpClient, MockSecretStore};
    use crate::core::types::{AuthConfig, BodyConfig, PreflightConfig, QResponse, RequestTab};
    use mockall::predicate;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_mock_tab(method: &str, url: &str) -> RequestTab {
        RequestTab {
            id: "test-id".to_string(),
            endpoint_id: None,
            title: "Test Tab".to_string(),
            method: method.to_string(),
            url: url.to_string(),
            params: vec![],
            headers: vec![],
            body: BodyConfig {
                r#type: "none".to_string(),
                content: "".to_string(),
            },
            auth: AuthConfig {
                r#type: "none".to_string(),
                active: true,
                bearer_token: "".to_string(),
                basic_user: "".to_string(),
                basic_pass: "".to_string(),
                api_key_name: "".to_string(),
                api_key_value: "".to_string(),
                api_key_location: "header".to_string(),
            },
            active_sub_tab: None,
            service_id: None,
            preflight: PreflightConfig {
                enabled: false,
                method: "GET".to_string(),
                url: "".to_string(),
                body: "".to_string(),
                body_type: "application/json".to_string(),
                body_params: vec![],
                headers: vec![],
                cache_token: true,
                cache_duration: "3600".to_string(),
                cache_duration_key: "".to_string(),
                cache_duration_unit: "seconds".to_string(),
                token_key: "access_token".to_string(),
                token_header: None,
            },
            variables: None,
            is_edited: false,
        }
    }

    #[tokio::test]
    async fn test_send_request_with_env_file_resolution() {
        let mut mock_http = MockHttpClient::new();
        let mut mock_fs = MockFileSystem::new();
        let mock_secrets = MockSecretStore::new();

        let settings_path = PathBuf::from("/settings.yaml");
        let _service_dir = "/service";

        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_read_to_string().returning(|p| {
            let path = p.to_str().unwrap();
            if path.contains("settings.yaml") {
                Ok("theme: dark\nservices: [{id: s1, name: ts, directory: /service}]".to_string())
            } else if path.contains("service.yaml") {
                Ok("id: s1\nname: ts\nisAuthenticated: false\ndirectory: /service\nselectedEnvironment: DEV\nendpoints: []".to_string())
            } else if path.contains("environments.yaml") {
                Ok(r#"
- name: DEV
  variables:
  - name: TEST_ID
    value: env:TEST_ID
    enabled: true
"# .to_string())
            } else if path.contains("dev.env") {
                Ok("TEST_ID=123".to_string())
            } else {
                Ok("[]".to_string())
            }
        });

        mock_http
            .expect_send_request()
            .with(
                predicate::eq("GET"),
                predicate::eq("https://api.com/123"),
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _, _| {
                Box::pin(async {
                    Ok(QResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers: vec![],
                        body: "ok".to_string(),
                        error: None,
                        time_elapsed: 1,
                        size: 2,
                    })
                })
            });

        let mut tab = create_mock_tab("GET", "https://api.com/{{TEST_ID}}");
        tab.service_id = Some("s1".to_string());

        let (_response, history) = crate::core::request::send_request_with_context(
            &mock_http,
            &mock_fs,
            &mock_secrets,
            &settings_path,
            None,
            tab,
        )
        .await
        .unwrap();

        assert_eq!(history.url, "https://api.com/123");
    }

    #[tokio::test]
    async fn test_send_request_with_tab_env_prefix_resolution() {
        let mut mock_http = MockHttpClient::new();
        let mut mock_fs = MockFileSystem::new();
        let mock_secrets = MockSecretStore::new();

        let settings_path = PathBuf::from("/settings.yaml");

        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_read_to_string().returning(|p| {
            let path = p.to_str().unwrap();
            if path.contains("settings.yaml") {
                Ok("theme: dark\nservices: [{id: s1, name: ts, directory: /service}]".to_string())
            } else if path.contains("service.yaml") {
                Ok("id: s1\nname: ts\nisAuthenticated: false\ndirectory: /service\nselectedEnvironment: DEV\nendpoints: []".to_string())
            } else if path.contains("environments.yaml") {
                Ok("[]".to_string())
            } else if path.contains("dev.env") {
                Ok("TEST_KEY=secret_value".to_string())
            } else {
                Ok("[]".to_string())
            }
        });

        mock_http
            .expect_send_request()
            .with(
                predicate::always(),
                predicate::eq("https://api.com/secret_value"),
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _, _| {
                Box::pin(async {
                    Ok(QResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers: vec![],
                        body: "ok".to_string(),
                        error: None,
                        time_elapsed: 1,
                        size: 2,
                    })
                })
            });

        let mut tab = create_mock_tab("GET", "https://api.com/{{MY_VAR}}");
        tab.service_id = Some("s1".to_string());
        let mut vars = HashMap::new();
        vars.insert("MY_VAR".to_string(), "env:TEST_KEY".to_string());
        tab.variables = Some(vars);

        let (_response, history) = crate::core::request::send_request_with_context(
            &mock_http,
            &mock_fs,
            &mock_secrets,
            &settings_path,
            None,
            tab,
        )
        .await
        .unwrap();

        assert_eq!(history.url, "https://api.com/secret_value");
    }
    #[tokio::test]
    async fn test_send_request_with_secret_type_resolution() {
        let mut mock_http = MockHttpClient::new();
        let mut mock_fs = MockFileSystem::new();
        let mut mock_secrets = MockSecretStore::new();

        let settings_path = PathBuf::from("/settings.yaml");
        
        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_read_to_string().returning(|p| {
            let path = p.to_str().unwrap();
            if path.contains("settings.yaml") {
                Ok("theme: dark\nservices: [{id: s1, name: ts, directory: /service}]".to_string())
            } else if path.contains("service.yaml") {
                Ok("id: s1\nname: ts\nisAuthenticated: false\ndirectory: /service\nselectedEnvironment: DEV\nendpoints: []".to_string())
            } else if path.contains("environments.yaml") {
                Ok(r#"
- name: DEV
  variables:
  - name: MY_SECRET
    value: GCP_API_KEY
    type: secret
    enabled: true
"# .to_string())
            } else {
                Ok("[]".to_string())
            }
        });

        // Mock the secret store return
        mock_secrets.expect_get().with(predicate::eq("GCP_API_KEY")).returning(|_| Ok("secret_123_abc".to_string()));

        mock_http.expect_send_request()
            .with(
                predicate::always(),
                predicate::eq("https://api.com/secret_123_abc"),
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _, _| {
                Box::pin(async {
                    Ok(QResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers: vec![],
                        body: "ok".to_string(),
                        error: None,
                        time_elapsed: 1,
                        size: 2,
                    })
                })
            });

        let mut tab = create_mock_tab("GET", "https://api.com/{{MY_SECRET}}");
        tab.service_id = Some("s1".to_string());

        let (_response, history) = crate::core::request::send_request_with_context(
            &mock_http,
            &mock_fs,
            &mock_secrets,
            &settings_path,
            None,
            tab,
        ).await.unwrap();

        assert_eq!(history.url, "https://api.com/secret_123_abc");
    }

    #[tokio::test]
    async fn test_send_request_with_direct_env_interpolation() {
        let mut mock_http = MockHttpClient::new();
        let mut mock_fs = MockFileSystem::new();
        let mock_secrets = MockSecretStore::new();

        let settings_path = PathBuf::from("/settings.yaml");

        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_read_to_string().returning(|p| {
            let path = p.to_str().unwrap();
            if path.contains("settings.yaml") {
                Ok("theme: dark\nservices: [{id: s1, name: ts, directory: /service}]".to_string())
            } else if path.contains("service.yaml") {
                Ok("id: s1\nname: ts\nisAuthenticated: false\ndirectory: /service\nselectedEnvironment: DEV\nendpoints: []".to_string())
            } else if path.contains("environments.yaml") {
                Ok("[]".to_string())
            } else if path.contains("dev.env") {
                Ok("DIRECT_VAL=999".to_string())
            } else {
                Ok("[]".to_string())
            }
        });

        mock_http
            .expect_send_request()
            .with(
                predicate::always(),
                predicate::eq("https://api.com/999/999"),
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _, _| {
                Box::pin(async {
                    Ok(QResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers: vec![],
                        body: "ok".to_string(),
                        error: None,
                        time_elapsed: 1,
                        size: 2,
                    })
                })
            });

        // Test both {{env:DIRECT_VAL}} and {{env.DIRECT_VAL}}
        let mut tab =
            create_mock_tab("GET", "https://api.com/{{env:DIRECT_VAL}}/{{env.DIRECT_VAL}}");
        tab.service_id = Some("s1".to_string());

        let (_response, history) = crate::core::request::send_request_with_context(
            &mock_http,
            &mock_fs,
            &mock_secrets,
            &settings_path,
            None,
            tab,
        ).await.unwrap();
        assert_eq!(history.url, "https://api.com/999/999");
    }

    #[tokio::test]
    async fn test_send_request_with_direct_secret_interpolation() {
        let mut mock_http = MockHttpClient::new();
        let mut mock_fs = MockFileSystem::new();
        let mut mock_secrets = MockSecretStore::new();

        let settings_path = PathBuf::from("/settings.yaml");

        mock_fs.expect_exists().returning(|_| true);
        mock_fs.expect_read_to_string().returning(|p| {
            let path = p.to_str().unwrap();
            if path.contains("settings.yaml") {
                Ok("theme: dark\nservices: [{id: s1, name: ts, directory: /service}]".to_string())
            } else if path.contains("service.yaml") {
                Ok("id: s1\nname: ts\nisAuthenticated: false\ndirectory: /service\nselectedEnvironment: DEV\nendpoints: []".to_string())
            } else {
                Ok("[]".to_string())
            }
        });

        // Mock the secret store return
        mock_secrets.expect_get().with(predicate::eq("DIRECT_SECRET")).returning(|_| Ok("secret_999".to_string()));

        mock_http.expect_send_request()
            .with(
                predicate::always(),
                predicate::eq("https://api.com/secret_999/secret_999"),
                predicate::always(),
                predicate::always(),
                predicate::always(),
            )
            .times(1)
            .returning(|_, _, _, _, _| {
                Box::pin(async {
                    Ok(QResponse {
                        status: 200,
                        status_text: "OK".to_string(),
                        headers: vec![],
                        body: "ok".to_string(),
                        error: None,
                        time_elapsed: 1,
                        size: 2,
                    })
                })
            });

        // Test both {{secret:KEY}} and {{secret.KEY}}
        let mut tab = create_mock_tab("GET", "https://api.com/{{secret:DIRECT_SECRET}}/{{secret.DIRECT_SECRET}}");
        tab.service_id = Some("s1".to_string());

        let (_response, history) = crate::core::request::send_request_with_context(
            &mock_http,
            &mock_fs,
            &mock_secrets,
            &settings_path,
            None,
            tab,
        ).await.unwrap();

        assert_eq!(history.url, "https://api.com/secret_999/secret_999");
    }
}
