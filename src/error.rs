//! Error types for the IBKR client.

use thiserror::Error;

/// Errors that can occur when using the IBKR client.
#[derive(Debug, Error)]
pub enum Error {
    /// IO error during connection or communication.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Protocol error (malformed message, unexpected response, etc.)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// TWS/Gateway returned an error.
    #[error("TWS error {code}: {message}")]
    Tws { code: i32, message: String },

    /// Connection not established.
    #[error("Not connected")]
    NotConnected,

    /// Request timed out.
    #[error("Request timed out")]
    Timeout,
}

/// Result type alias for IBKR operations.
pub type Result<T> = std::result::Result<T, Error>;
