//! Error types for the FarmLink companion client.

use thiserror::Error;

/// Enumeration of all recoverable errors occurring in the FarmLink client.
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("zip archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("xml parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("http client error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("modDesc.xml missing from archive '{0}'")]
    ModDescNotFound(String),

    #[error("sha256 hash mismatch for file '{file}': expected {expected}, calculated {actual}")]
    HashMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("synchronization failed: {0}")]
    SyncFailed(String),

    #[error("server not found or returned error: status {status}, message: {message}")]
    ServerApiError { status: u16, message: String },

    #[error("game launch failed: {0}")]
    GameLaunchFailed(String),
}

/// Convenience result type alias for FarmLink operations.
pub type Result<T> = std::result::Result<T, ClientError>;
