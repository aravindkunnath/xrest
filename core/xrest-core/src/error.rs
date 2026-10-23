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
    /// The workspace was written by a newer XRest; refusing to read (and later rewrite) it.
    #[error(
        "{} uses workspace format {found}, but this version of XRest supports up to {supported}; update XRest",
        .path.display()
    )]
    UnsupportedFormat {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
    /// A request body could not be built (e.g. a file outside the workspace or missing).
    #[error("invalid request body: {0}")]
    InvalidBody(String),
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
    /// A write API was given values it refuses (e.g. duplicate or empty environment names).
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid cURL command: {0}")]
    Curl(String),
    /// A git command failed; carries git's stderr (or "git not found").
    #[error("git: {0}")]
    Git(String),
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
