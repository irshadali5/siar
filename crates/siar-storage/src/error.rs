use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("stoolap error: {0}")]
    Backend(String),
    #[error("row was missing an expected column")]
    MissingColumn,
    #[error("stored id could not be parsed: {0}")]
    MalformedId(String),
}

impl StorageError {
    pub(crate) fn from_stoolap(err: impl std::fmt::Display) -> Self {
        StorageError::Backend(err.to_string())
    }
}
