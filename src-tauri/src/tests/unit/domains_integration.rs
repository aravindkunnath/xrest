use crate::domains::auth::AuthConfig;
use crate::domains::service::service::ServiceDomain;
use crate::domains::settings::SettingsDomain;
use crate::io::MockFileSystem;
use crate::types::{AuthType, Endpoint, EndpointMetadata, PreflightConfig, Service};
use mockall::predicate::*;
use std::path::PathBuf;

#[test]
fn test_load_save_settings() {
    let mut mock_fs = MockFileSystem::new();
    let settings_path = PathBuf::from("/tmp/settings.yaml");

    // Test default creation
    mock_fs.expect_exists().returning(|_| false);
    mock_fs.expect_create_dir_all().returning(|_| Ok(()));
    mock_fs.expect_write().returning(|_, _| Ok(()));

    let domain = SettingsDomain::new(&mock_fs);
    let settings = domain.load_settings(&settings_path).unwrap();
    assert_eq!(settings.theme, "system");

    // Test invalid YAML fallback
    let mut mock_fs = MockFileSystem::new();
    mock_fs.expect_exists().returning(|_| true);
    mock_fs
        .expect_read_to_string()
        .returning(|_| Ok("invalid: yaml: :".to_string()));
    let domain = SettingsDomain::new(&mock_fs);
    let settings = domain.load_settings(&settings_path).unwrap();
    assert_eq!(settings.theme, "system");
}

#[test]
fn test_load_save_collections() {
    let mut mock_fs = MockFileSystem::new();
    let path = PathBuf::from("/tmp/collections.yaml");

    mock_fs.expect_exists().returning(|_| false);
    mock_fs.expect_create_dir_all().returning(|_| Ok(()));
    mock_fs.expect_write().returning(|_, _| Ok(()));

    let domain = ServiceDomain::new(&mock_fs);
    let collections = domain.load_collections(&path).unwrap();
    assert_eq!(collections.len(), 0);

    let mut mock_fs = MockFileSystem::new();
    mock_fs.expect_exists().returning(|_| true);
    mock_fs
        .expect_read_to_string()
        .returning(|_| Ok("[]".to_string()));
    let domain = ServiceDomain::new(&mock_fs);
    let collections = domain.load_collections(&path).unwrap();
    assert_eq!(collections.len(), 0);
}

#[test]
fn test_load_save_tab_state() {
    let mut mock_fs = MockFileSystem::new();
    let path = PathBuf::from("/tmp/tabstate.yaml");

    mock_fs.expect_exists().returning(|_| false);
    let domain = SettingsDomain::new(&mock_fs);
    let state = domain.load_tab_state(&path).unwrap();
    assert!(state.is_none());
}

#[test]
fn test_load_service_success() {
    let mut mock_fs = MockFileSystem::new();
    let service_dir = "/tmp/test-service";

    // Mock service.yaml
    mock_fs
        .expect_exists()
        .with(eq(PathBuf::from(service_dir).join("service.yaml")))
        .returning(|_| true);

    let service_file_json = r#"{
        "id": "s1",
        "name": "Test Service",
        "isAuthenticated": false,
        "endpoints": [],
        "directory": "/tmp/test-service"
    }"#;

    mock_fs
        .expect_read_to_string()
        .with(eq(PathBuf::from(service_dir).join("service.yaml")))
        .returning(move |_| Ok(service_file_json.to_string()));

    // Mock environments.yaml (doesn't exist)
    mock_fs
        .expect_exists()
        .with(eq(PathBuf::from(service_dir).join("environments.yaml")))
        .returning(|_| false);

    // Mock endpoints dir (doesn't exist)
    mock_fs
        .expect_exists()
        .with(eq(PathBuf::from(service_dir).join("endpoints")))
        .returning(|_| false);

    // Mock endpoints.yaml (doesn't exist)
    mock_fs
        .expect_exists()
        .with(eq(PathBuf::from(service_dir).join("endpoints.yaml")))
        .returning(|_| false);

    let domain = ServiceDomain::new(&mock_fs);
    let result = domain.load_service(service_dir);
    assert!(result.is_ok());
    let service = result.unwrap();
    assert_eq!(service.id, "s1");
    assert_eq!(service.name, "Test Service");
}

#[test]
fn test_save_service_versioning() {
    let mut mock_fs = MockFileSystem::new();
    let service_dir = "/tmp/test-service";

    let mut service = Service {
        id: "s1".to_string(),
        name: "Test Service".to_string(),
        environments: vec![],
        is_authenticated: false,
        auth_type: Some(AuthType::None),
        auth: AuthConfig {
            r#type: "none".to_string(),
            active: true,
            basic_user: "".to_string(),
            basic_pass: "".to_string(),
            bearer_token: "".to_string(),
            api_key_name: "".to_string(),
            api_key_value: "".to_string(),
            api_key_location: "header".to_string(),
        },
        preflight: PreflightConfig {
            enabled: false,
            method: "POST".to_string(),
            url: "".to_string(),
            body: "".to_string(),
            body_type: "application/json".to_string(),
            body_params: vec![],
            headers: vec![],
            cache_token: true,
            cache_duration: "".to_string(),
            cache_duration_key: "".to_string(),
            cache_duration_unit: "seconds".to_string(),
            token_key: "".to_string(),
            token_header: None,
        },
        endpoints: vec![Endpoint {
            id: "e1".to_string(),
            service_id: "s1".to_string(),
            name: "Get Items".to_string(),
            method: "GET".to_string(),
            url: "/items".to_string(),
            authenticated: false,
            auth_type: "none".to_string(),
            metadata: EndpointMetadata {
                version: "0".to_string(),
                last_updated: 0,
            },
            params: vec![],
            headers: vec![],
            body: "".to_string(),
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
            last_version: 0,
            versions: vec![],
        }],
        directory: service_dir.to_string(),
        selected_environment: None,
        git_url: None,
    };

    mock_fs.expect_exists().returning(|_| true);
    mock_fs.expect_write().returning(|_, _| Ok(()));
    mock_fs.expect_create_dir_all().returning(|_| Ok(()));

    // First save should create version 1
    let domain = ServiceDomain::new(&mock_fs);
    let result = domain.save_service(&mut service, None);
    assert!(result.is_ok());
    assert_eq!(service.endpoints[0].last_version, 1);
    assert_eq!(service.endpoints[0].versions.len(), 1);

    // Save again with no changes should NOT create a new version
    let result = domain.save_service(&mut service, None);
    assert!(result.is_ok());
    assert_eq!(service.endpoints[0].last_version, 1);
    assert_eq!(service.endpoints[0].versions.len(), 1);

    // Change URL and save should create version 2
    service.endpoints[0].url = "/new-items".to_string();
    let result = domain.save_service(&mut service, None);
    assert!(result.is_ok());
    assert_eq!(service.endpoints[0].last_version, 2);
    assert_eq!(service.endpoints[0].versions.len(), 2);
}
