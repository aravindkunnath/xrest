#[cfg(test)]
mod tests {
    use xrest_core::service::dotenv::{load_dotenv_vars, parse_dotenv};
    use xrest_core::traits::MockFileSystem;
    use std::path::Path;

    // --- parse_dotenv tests ---

    #[test]
    fn test_basic_key_value() {
        let content = "KEY=value";
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some("value"));
    }

    #[test]
    fn test_comment_lines_skipped() {
        let content = "# this is a comment\nKEY=value";
        let map = parse_dotenv(content);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("KEY").map(String::as_str), Some("value"));
    }

    #[test]
    fn test_blank_lines_skipped() {
        let content = "\n\nKEY=value\n\n";
        let map = parse_dotenv(content);
        assert_eq!(map.len(), 1);
        assert_eq!(map.get("KEY").map(String::as_str), Some("value"));
    }

    #[test]
    fn test_export_prefix_stripped() {
        let content = "export KEY=val";
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some("val"));
    }

    #[test]
    fn test_double_quoted_value() {
        let content = r#"KEY="hello world""#;
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some("hello world"));
    }

    #[test]
    fn test_single_quoted_value() {
        let content = "KEY='hello world'";
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some("hello world"));
    }

    #[test]
    fn test_inline_comment_stripped_unquoted() {
        let content = "KEY=value # this is a comment";
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some("value"));
    }

    #[test]
    fn test_inline_comment_tab_stripped_unquoted() {
        let content = "KEY=value\t# this is a comment";
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some("value"));
    }

    #[test]
    fn test_hash_preserved_in_double_quoted_value() {
        let content = r#"KEY="value # not a comment""#;
        let map = parse_dotenv(content);
        assert_eq!(
            map.get("KEY").map(String::as_str),
            Some("value # not a comment")
        );
    }

    #[test]
    fn test_hash_preserved_in_single_quoted_value() {
        let content = "KEY='value # not a comment'";
        let map = parse_dotenv(content);
        assert_eq!(
            map.get("KEY").map(String::as_str),
            Some("value # not a comment")
        );
    }

    #[test]
    fn test_empty_value() {
        let content = "KEY=";
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some(""));
    }

    #[test]
    fn test_empty_key_skipped() {
        let content = "=value";
        let map = parse_dotenv(content);
        assert!(map.is_empty());
    }

    #[test]
    fn test_equals_in_value() {
        let content = "KEY=a=b=c";
        let map = parse_dotenv(content);
        assert_eq!(map.get("KEY").map(String::as_str), Some("a=b=c"));
    }

    #[test]
    fn test_multiple_entries() {
        let content = "FOO=bar\nBAZ=qux";
        let map = parse_dotenv(content);
        assert_eq!(map.get("FOO").map(String::as_str), Some("bar"));
        assert_eq!(map.get("BAZ").map(String::as_str), Some("qux"));
    }

    // --- load_dotenv_vars tests ---

    #[test]
    fn test_file_not_found_returns_empty_map() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs
            .expect_exists()
            .returning(|_| false);

        let result = load_dotenv_vars("/some/dir", "DEV", &mock_fs);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_env_name_lowercased_for_filename() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs
            .expect_exists()
            .withf(|p: &Path| p.ends_with("dev.env"))
            .returning(|_| true);
        mock_fs
            .expect_read_to_string()
            .withf(|p: &Path| p.ends_with("dev.env"))
            .returning(|_| Ok("BASE_URL=http://localhost:3000".to_string()));

        let result = load_dotenv_vars("/some/dir", "DEV", &mock_fs);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(
            map.get("BASE_URL").map(String::as_str),
            Some("http://localhost:3000")
        );
    }

    #[test]
    fn test_stage_env_name() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs
            .expect_exists()
            .withf(|p: &Path| p.ends_with("stage.env"))
            .returning(|_| true);
        mock_fs
            .expect_read_to_string()
            .withf(|p: &Path| p.ends_with("stage.env"))
            .returning(|_| Ok("API_KEY=secret123".to_string()));

        let result = load_dotenv_vars("/some/dir", "STAGE", &mock_fs);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.get("API_KEY").map(String::as_str), Some("secret123"));
    }

    #[test]
    fn test_load_parses_content_correctly() {
        let mut mock_fs = MockFileSystem::new();
        mock_fs
            .expect_exists()
            .returning(|_| true);
        mock_fs
            .expect_read_to_string()
            .returning(|_| {
                Ok("# comment\nFOO=bar\nexport BAZ=qux\nEMPTY=".to_string())
            });

        let result = load_dotenv_vars("/some/dir", "dev", &mock_fs);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.get("FOO").map(String::as_str), Some("bar"));
        assert_eq!(map.get("BAZ").map(String::as_str), Some("qux"));
        assert_eq!(map.get("EMPTY").map(String::as_str), Some(""));
        assert!(!map.contains_key("# comment"));
    }
}
