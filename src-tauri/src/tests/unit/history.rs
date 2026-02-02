use crate::history::HistoryService;
use crate::types::HistoryEntry;
use rusqlite::Connection;

#[test]
fn test_history_lifecycle() {
    let conn = Connection::open_in_memory().unwrap();
    let service = HistoryService::new(conn);

    // Init DB
    assert!(service.init().is_ok());

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

    assert!(service.save(entry.clone()).is_ok());

    // Get history
    let history = service.get_history(10, 0).unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].id, "h1");

    // Clear history
    assert!(service.clear().is_ok());
    let history = service.get_history(10, 0).unwrap();
    assert_eq!(history.len(), 0);
}
