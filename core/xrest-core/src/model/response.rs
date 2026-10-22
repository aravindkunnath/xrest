//! The result of a send.

/// Result of a send.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub content_type: String,
    /// UTF-8 text, or base64 when `body_is_base64`.
    pub body: String,
    pub body_is_base64: bool,
    /// Re-indented JSON (key order and number formatting preserved) when the body is JSON.
    pub pretty_body: Option<String>,
    pub size_bytes: u64,
    pub time_ms: u64,
    pub final_url: String,
    /// Environment the request was resolved against.
    pub environment: Option<String>,
    /// `{{VAR}}` placeholders that had no value and were sent verbatim.
    pub unresolved_variables: Vec<String>,
}
