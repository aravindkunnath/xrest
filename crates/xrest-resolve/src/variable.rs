
/// A variable with a key and its raw (unresolved) value.
#[derive(Debug, Clone)]
pub struct Variable {
    pub key: String,
    pub raw_value: String,
}

impl Variable {
    pub fn new(key: String, raw_value: String) -> Self {
        Self { key, raw_value }
    }

    /// Returns the inner content if the value is wrapped in `{{ }}`.
    pub fn get_template_content(&self) -> Option<&str> {
        let trimmed = self.raw_value.trim();
        if trimmed.starts_with("{{") && trimmed.ends_with("}}") {
            Some(trimmed[2..trimmed.len() - 2].trim())
        } else {
            None
        }
    }
}
