use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlightDeckError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Event parsing error: {0}")]
    EventParsing(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Adapter error: {0}")]
    Adapter(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, FlightDeckError>;