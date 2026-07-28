//! Crate-wide error type.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("auth error: {0}")]
    Auth(String),
    // TODO: Api variant for the gateway's {"error": ...} 429/500 bodies.
}
