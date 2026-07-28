//! `POST /oauth/live_session_token` — Diffie-Hellman handshake that mints a live session token.
//! Signing here is bespoke (RSA-SHA256, not the generic request signer), so this route defines
//! only its path and response; [`crate::auth::mint_live_session_token`] drives the exchange.
//! Docs: <https://www.interactivebrokers.com/campus/ibkr-api-page/oauth-1-0a-extended/>

use serde::Deserialize;

pub const PATH: &str = "/oauth/live_session_token";

#[derive(Debug, Clone, Deserialize)]
pub struct Response {
    /// Diffie-Hellman response `B`, hex.
    pub diffie_hellman_response: String,
    /// Proof used to validate the locally-computed token.
    pub live_session_token_signature: String,
    /// Epoch-millis expiration (~24h out).
    pub live_session_token_expiration: i64,
}
