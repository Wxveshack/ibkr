//! Crate-wide error type. One place for every failure mode the client can surface.

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The HTTP request failed to send, or the gateway returned a non-2xx status.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// The response body could not be decoded into the endpoint's `Response` type.
    #[error("decode error: {0}")]
    Decode(#[from] serde_json::Error),
    // A future `Api` variant can capture the gateway's `{ "error": "..." }` 429/500 bodies.
}
