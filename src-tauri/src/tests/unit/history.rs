use crate::core::traits::HistoryRepository;
use crate::core::types::HistoryEntry;
use crate::infra::history::SqliteHistoryRepository;
use rusqlite::Connection;

#[test]
fn test_history_lifecycle() {
    let conn = Connection::open_in_memory().unwrap();
    let repo = SqliteHistoryRepository::new(conn);

    // Init DB
    assert!(repo.init().is_ok());

    // Save entry
    let entry = HistoryEntry {
        id: "h1".to_string(),
        service_id: Some("s1".to_string()),
        endpoint_id: Some("e1".to_string()),
        method: "GET".to_string(),
        url: "/test".to_string(),
        request_headers: vec![],
        request_body: "".to_string(),
        response_status: 200,
        response_status_text: "OK".to_string(),
        response_headers: vec![],
        response_body: "body".to_string(),
        time_elapsed: 10,
        size: 4,
        created_at: "2023-01-01T00:00:00Z".to_string(),
    };

    assert!(repo.save(entry.clone()).is_ok());

    // Get history
    let history = repo.get_history(10, 0).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].id, "h1");

    // Clear history
    assert!(repo.clear().is_ok());
    let history = repo.get_history(10, 0).unwrap();
    assert_eq!(history.len(), 0);
}
