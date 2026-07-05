use thiserror::Error;

#[derive(Error, Debug)]
pub enum CollectorError {
    #[error("Adapter error: {0}")]
    Adapter(String),

    #[error("Storage error: {0}")]
    Storage(#[from] flightdeck_storage::StorageError),

    #[error("Session not active")]
    SessionNotActive,

    #[error("Adapter not found: {0}")]
    AdapterNotFound(String),
}