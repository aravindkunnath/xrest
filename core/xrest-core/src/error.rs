use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not an XRest workspace (no service.yaml): {}", .0.display())]
    NotAWorkspace(PathBuf),
    #[error("a workspace already exists at {}", .0.display())]
    AlreadyExists(PathBuf),
    #[error("I/O error at {}: {source}", .path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid YAML in {}: {message}", .path.display())]
    Yaml { path: PathBuf, message: String },
    #[error("{what} not found: {id}")]
    NotFound { what: &'static str, id: String },
    #[error("invalid id {0:?}: ids may only contain letters, digits, '-', '_' and '.'")]
    InvalidId(String),
    #[error("invalid URL {url:?}: {message}")]
    InvalidUrl { url: String, message: String },
    #[error("environment {0:?} does not exist")]
    UnknownEnvironment(String),
    /// The selected environment is flagged unsafe and the caller did not pass `confirmed = true`.
    #[error("environment {environment:?} is unsafe: sending requires explicit confirmation")]
    ConfirmationRequired { environment: String },
    #[error("missing secrets: {}", .0.join(", "))]
    MissingSecrets(Vec<String>),
    #[error("preflight failed: {0}")]
    Preflight(String),
    #[error("request failed: {0}")]
    Http(String),
    #[error("invalid cURL command: {0}")]
    Curl(String),
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}
