//! First-party OAuth 1.0a: credentials, per-request signing, and live-session-token minting.
//! The signing primitives live in [`crypto`]; the wire route in [`crate::oauth::live_session_token`].

mod crypto;

use num_bigint::BigUint;
use reqwest::header::AUTHORIZATION;
use serde::Deserialize;

use crate::error::Error;
use crate::oauth::live_session_token;

/// The credential bundle, deserialized from a JSON payload (source is the caller's concern).
#[derive(Debug, Clone, Deserialize)]
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

/// A minted live session token, valid ~24h.
#[derive(Debug, Clone)]
pub struct LiveSessionToken {
    /// base64-encoded token.
    pub token: String,
    /// Epoch-millis expiration reported by the gateway.
    pub expiration: i64,
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

        let mut header = oauth;
        header.push(("oauth_signature".into(), signature));
        header.push(("realm".into(), self.realm.clone()));
        authorization_header(&header)
    }
}

/// Mint a live session token: Diffie-Hellman challenge, RSA-signed request to
/// [`live_session_token::PATH`], then compute and validate the token per IBKR's scheme.
pub fn mint_live_session_token(
    creds: &Credentials,
    http: &reqwest::blocking::Client,
    base: &str,
) -> Result<LiveSessionToken, Error> {
    let (prime, generator) = crypto::parse_dh_params(&creds.dhparam_pem)?;
    let a = crypto::dh_random();
    let challenge = generator.modpow(&a, &prime).to_str_radix(16);

    // Decrypt the access token secret; its hex prepends the base string, its bytes key the LST.
    let enc_key = crypto::load_private_key(&creds.private_encryption_pem)?;
    let secret = crypto::rsa_decrypt(&enc_key, &crypto::b64_decode(&creds.access_token_secret)?)?;
    let prepend = crypto::hex_encode(&secret);

    let url = format!("{base}{}", live_session_token::PATH);
    let params = vec![
        ("diffie_hellman_challenge".into(), challenge),
        ("oauth_consumer_key".into(), creds.consumer_key.clone()),
        ("oauth_nonce".into(), crypto::nonce()),
        ("oauth_signature_method".into(), "RSA-SHA256".into()),
        ("oauth_timestamp".into(), crypto::timestamp()),
        ("oauth_token".into(), creds.access_token.clone()),
    ];

    let base_string = format!("{prepend}{}", crypto::signature_base_string("POST", &url, &params));
    let sig_key = crypto::load_private_key(&creds.private_signature_pem)?;
    let signature =
        crypto::percent_encode(&crypto::b64_encode(&crypto::rsa_sha256_sign(&sig_key, base_string.as_bytes())?));

    let mut header = params;
    header.push(("oauth_signature".into(), signature));
    header.push(("realm".into(), creds.realm.clone()));

    let http_resp = http
        .post(&url)
        .header(AUTHORIZATION, authorization_header(&header))
        .body("") // gateway 411s an empty POST without an explicit Content-Length: 0
        .send()?;
    let status = http_resp.status();
    let text = http_resp.text()?;
    if !status.is_success() {
        return Err(Error::Auth(format!("live_session_token {status}: {text}")));
    }
    let resp: live_session_token::Response = serde_json::from_str(&text)?;

    // K = B^a mod p; LST = HMAC_SHA1(K_bytes, secret_bytes).
    let b = BigUint::parse_bytes(resp.diffie_hellman_response.as_bytes(), 16)
        .ok_or_else(|| Error::Auth("diffie_hellman_response is not valid hex".into()))?;
    let k = b.modpow(&a, &prime);
    let lst = crypto::hmac_sha1(&k.to_bytes_be(), &secret);

    // Validate against the signature the gateway returned.
    let check = crypto::hex_encode(&crypto::hmac_sha1(&lst, creds.consumer_key.as_bytes()));
    if check != resp.live_session_token_signature {
        return Err(Error::Auth("live session token validation failed".into()));
    }

    Ok(LiveSessionToken {
        token: crypto::b64_encode(&lst),
        expiration: resp.live_session_token_expiration,
    })
}

/// `OAuth k1="v1", k2="v2", …` with keys sorted alphabetically.
fn authorization_header(params: &[(String, String)]) -> String {
    let mut sorted = params.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let inner = sorted
        .iter()
        .map(|(k, v)| format!("{k}=\"{v}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("OAuth {inner}")
}
