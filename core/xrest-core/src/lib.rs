//! XRest core: everything that must behave identically on every platform.
//!
//! Platform apps (SwiftUI today, GTK later) own all UI and talk to this crate through
//! `xrest-ffi`. See `docs/architecture.md` for the boundary.

pub mod curl;
pub mod diff;
pub mod error;
pub mod exec;
pub mod git;
pub mod history;
pub mod interpolate;
pub mod model;
pub mod policy;
pub mod preflight;
mod resolve;
pub mod secrets;
pub mod storage;
pub mod swagger;
pub mod versions;
pub mod workspace;

pub use diff::{diff_configs, ChangeKind, FieldChange};
pub use error::{Error, Result};
pub use exec::{HttpOptions, PreparedPreflightTest, PreparedSend};
pub use interpolate::InterpolationPreview;
pub use model::*;
pub use policy::SendPolicy;
pub use preflight::PreflightTestResult;
pub use resolve::MAX_BODY_FILE_BYTES;
pub use secrets::{MemorySecretStore, SecretStore};
pub use swagger::{ImportSummary, ImportedEndpoint, ImportedSpec};
pub use workspace::Workspace;
