use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<StorageError> for flightdeck_core::FlightDeckError {
    fn from(err: StorageError) -> Self {
        flightdeck_core::FlightDeckError::Storage(err.to_string())
    }
}