//! Domain model and serde mapping for the on-disk YAML format.
//!
//! Field names are `camelCase` on disk. Files written by the legacy Go/Wails app (format
//! version 0) still load; see `docs/architecture.md` for the format history.
//!
//! - [`request`]: what a request is (`RequestSpec`, `Body`, `RequestOptions`, `Preflight`).
//! - [`service`]: the workspace's service file, auth and environments.
//! - [`endpoint`]: versioned endpoints, scratch requests and version snapshots.
//! - [`response`]: the result of a send.

mod endpoint;
mod request;
mod response;
mod service;

pub use endpoint::*;
pub use request::*;
pub use response::*;
pub use service::*;

/// The on-disk format version this build reads and writes (`formatVersion` in `service.yaml`).
///
/// - `0`: written by the legacy Go app (no `formatVersion` key); bodies are plain strings.
/// - `1`: typed request bodies and per-request options.
///
/// Workspaces with a newer version are refused rather than partially read and rewritten.
pub const FORMAT_VERSION: u32 = 1;

pub(crate) fn yes() -> bool {
    true
}
