//! Swagger / OpenAPI import.
//!
//! TODO(swagger): port `internal/import/swagger.go` (OpenAPI 2/3 JSON+YAML -> endpoints plus a
//! `BASE_URL` variable for the default environments).

use crate::error::{Error, Result};
use crate::model::Endpoint;

pub fn import_spec(_content: &str, _service_id: &str) -> Result<(String, Vec<Endpoint>)> {
    Err(Error::NotImplemented("Swagger/OpenAPI import"))
}
