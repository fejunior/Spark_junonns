//! Error types for OpenFire authentication and communication

use thiserror::Error;

/// Result type alias for OpenFire operations
pub type Result<T> = std::result::Result<T, OpenFireError>;

/// OpenFire-specific error types
#[derive(Error, Debug)]
pub enum OpenFireError {
    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Connection error: {message}")]
    ConnectionError { message: String },

    #[error("XMPP protocol error: {message}")]
    XmppProtocolError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("TLS/SSL error: {message}")]
    TlsError { message: String },

    #[error("Timeout error: operation timed out after {seconds} seconds")]
    TimeoutError { seconds: u64 },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("JNI error: {message}")]
    JniError { message: String },

    #[error("Invalid credentials: {message}")]
    InvalidCredentials { message: String },

    #[error("Server not reachable: {server}")]
    ServerUnreachable { server: String },

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl From<anyhow::Error> for OpenFireError {
    fn from(err: anyhow::Error) -> Self {
        OpenFireError::Unknown {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for OpenFireError {
    fn from(err: serde_json::Error) -> Self {
        OpenFireError::SerializationError {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for OpenFireError {
    fn from(err: std::io::Error) -> Self {
        OpenFireError::ConnectionError {
            message: err.to_string(),
        }
    }
}