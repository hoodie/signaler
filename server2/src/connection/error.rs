use thiserror::Error;

/// The kind of an error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Session was destroyed already")]
    SessionGone,

    #[error("failed to parse incomming command")]
    Parsing(#[from] serde_json::Error),

    #[error("badly typed error")]
    Anyhow(#[from] anyhow::Error),
}
