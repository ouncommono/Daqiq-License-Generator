use thiserror::Error;

#[derive(Error, Debug)]
pub enum LicError {
    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Vault error: {0}")]
    Vault(String),

    #[error("HWID error: {0}")]
    Hwid(String),

    #[error("Date error: {0}")]
    Date(String),

    #[cfg(feature = "sqlite")]
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[cfg(not(feature = "sqlite"))]
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
