//! First-party OAuth 1.0a: credentials and per-request signing. The signing primitives live in
//! the private `crypto` submodule; minting the token is a route, and lives in
//! [`crate::oauth::live_session_token`].

pub(crate) mod crypto;

use serde::Deserialize;

use crate::error::Error;

/// The credential bundle, deserialized from a JSON payload (source is the caller's concern).
/// `Debug` is hand-written to keep the private keys and token secret out of logs.
#[derive(Clone, Deserialize)]
pub struct Credentials {
    pub consumer_key: String,
    /// `test_realm` only for the `TESTCONS` demo key; otherwise `limited_poa`.
    pub realm: String,
    pub access_token: String,
    /// base64 ciphertext from the portal, decrypted with the private encryption key.
    pub access_token_secret: String,
    pub private_signature_pem: String,
    pub private_encryption_pem: String,
    /// Diffie-Hellman parameters (`dhparam.pem` contents); prime and generator are parsed out.
    pub dhparam_pem: String,
}

impl Credentials {
    pub fn from_json(s: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(s)?)
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("consumer_key", &self.consumer_key)
            .field("realm", &self.realm)
            .finish_non_exhaustive()
    }
}

/// A minted live session token, valid ~24h. `Debug` redacts the token: it keys every
/// request signature.
#[derive(Clone)]
pub struct LiveSessionToken {
    /// base64-encoded token.
    pub token: String,
    /// Epoch-millis expiration reported by the gateway.
    pub expiration: i64,
}

impl std::fmt::Debug for LiveSessionToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveSessionToken")
            .field("expiration", &self.expiration)
            .finish_non_exhaustive()
    }
}

/// Signs authenticated requests with HMAC-SHA256 keyed by the live session token.
pub struct Signer {
    consumer_key: String,
    realm: String,
    access_token: String,
    lst: Vec<u8>,
}

impl Signer {
    pub fn new(creds: &Credentials, lst: &LiveSessionToken) -> Result<Self, Error> {
        Ok(Signer {
            consumer_key: creds.consumer_key.clone(),
            realm: creds.realm.clone(),
            access_token: creds.access_token.clone(),
            lst: crypto::b64_decode(&lst.token)?,
        })
    }

    /// Build the `Authorization: OAuth …` header for a request. `query` params are folded into
    /// the signature base string but not the header (they travel in the URL).
    pub fn authorization(&self, method: &str, url: &str, query: &[(String, String)]) -> String {
        let oauth = vec![
            ("oauth_consumer_key".into(), self.consumer_key.clone()),
            ("oauth_nonce".into(), crypto::nonce()),
            ("oauth_signature_method".into(), "HMAC-SHA256".into()),
            ("oauth_timestamp".into(), crypto::timestamp()),
            ("oauth_token".into(), self.access_token.clone()),
        ];

        let mut signed = oauth.clone();
        signed.extend_from_slice(query);
        let base = crypto::signature_base_string(method, url, &signed);
        let signature = crypto::percent_encode(&crypto::b64_encode(&crypto::hmac_sha256(
            &self.lst,
            base.as_bytes(),
        )));

        signed_header(oauth, signature, &self.realm)
    }
}

/// Close out an `Authorization: OAuth …` header: append the signature and realm, then render
/// `k1="v1", k2="v2", …` with keys sorted. Both signing paths — HMAC here, RSA in
/// [`crate::oauth::live_session_token`] — end this way.
pub(crate) fn signed_header(
    mut params: Vec<(String, String)>,
    signature: String,
    realm: &str,
) -> String {
    params.push(("oauth_signature".into(), signature));
    params.push(("realm".into(), realm.into()));
    params.sort();
    let inner = params
        .iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("OAuth {inner}")
}
