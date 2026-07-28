//! OAuth 1.0a signing primitives: thin wrappers over the crypto crates, plus the
//! signature base string. IBKR's live-session-token sequencing lives in the parent module.
//! Reference: <https://www.interactivebrokers.com/campus/ibkr-api-page/oauth-1-0a-extended/>

use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use rand::{rngs::OsRng, RngCore};
use rsa::{Pkcs1v15Encrypt, Pkcs1v15Sign, RsaPrivateKey};
use sha1::Sha1;
use sha2::{Digest, Sha256};

use crate::error::Error;

/// Complement of RFC3986 unreserved: keep `- . _ ~`, encode everything else.
const UNRESERVED: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub(crate) fn percent_encode(s: &str) -> String {
    utf8_percent_encode(s, UNRESERVED).to_string()
}

/// `METHOD&enc(url)&enc(sorted "k=v" joined by "&")`. Params are single-encoded as a unit,
/// matching IBKR's reference. The LST caller prepends the decrypted-secret hex before signing.
pub(crate) fn signature_base_string(method: &str, url: &str, params: &[(String, String)]) -> String {
    let mut sorted = params.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let joined = sorted
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{method}&{}&{}", percent_encode(url), percent_encode(&joined))
}

pub(crate) fn nonce() -> String {
    let mut b = [0u8; 16];
    OsRng.fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}

pub(crate) fn timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before 1970")
        .as_secs()
        .to_string()
}

/// A positive random 256-bit integer — the Diffie-Hellman private exponent `a`.
pub(crate) fn dh_random() -> num_bigint::BigUint {
    let mut b = [0u8; 32];
    OsRng.fill_bytes(&mut b);
    num_bigint::BigUint::from_bytes_be(&b)
}

pub(crate) fn load_private_key(pem: &str) -> Result<RsaPrivateKey, Error> {
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::pkcs8::DecodePrivateKey;
    // openssl 3.x emits PKCS#8, 1.x emits PKCS#1; accept either.
    RsaPrivateKey::from_pkcs8_pem(pem)
        .or_else(|_| RsaPrivateKey::from_pkcs1_pem(pem))
        .map_err(|e| Error::Auth(format!("private key parse: {e}")))
}

/// RSASSA-PKCS1-v1_5 over SHA-256 — signs the LST request.
pub(crate) fn rsa_sha256_sign(key: &RsaPrivateKey, msg: &[u8]) -> Result<Vec<u8>, Error> {
    key.sign(Pkcs1v15Sign::new::<Sha256>(), &Sha256::digest(msg))
        .map_err(|e| Error::Auth(format!("rsa sign: {e}")))
}

/// RSA/PKCS1-v1_5 decrypt — recovers the access token secret from the portal ciphertext.
pub(crate) fn rsa_decrypt(key: &RsaPrivateKey, ciphertext: &[u8]) -> Result<Vec<u8>, Error> {
    key.decrypt(Pkcs1v15Encrypt, ciphertext)
        .map_err(|e| Error::Auth(format!("rsa decrypt: {e}")))
}

pub(crate) fn hmac_sha1(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha1>::new_from_slice(key).expect("hmac accepts any key length");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

pub(crate) fn hmac_sha256(key: &[u8], msg: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("hmac accepts any key length");
    mac.update(msg);
    mac.finalize().into_bytes().to_vec()
}

pub(crate) fn b64_encode(bytes: &[u8]) -> String {
    STANDARD.encode(bytes)
}

pub(crate) fn b64_decode(s: &str) -> Result<Vec<u8>, Error> {
    STANDARD
        .decode(s.trim())
        .map_err(|e| Error::Auth(format!("base64 decode: {e}")))
}

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Extract `(prime, generator)` from a `dhparam.pem`. PKCS#3 is
/// `SEQUENCE { prime INTEGER, base INTEGER, privateValueLength INTEGER OPTIONAL }`; we read the
/// first two and ignore any trailing optional field (OpenSSL sometimes emits it).
pub(crate) fn parse_dh_params(pem: &str) -> Result<(num_bigint::BigUint, num_bigint::BigUint), Error> {
    use der::{asn1::UintRef, Decode, Header, SliceReader, Tag};
    let (_, der) = pem_rfc7468::decode_vec(pem.as_bytes())
        .map_err(|e| Error::Auth(format!("dhparam pem: {e}")))?;
    let mut reader = SliceReader::new(&der).map_err(|e| Error::Auth(format!("dhparam der: {e}")))?;

    let header = Header::decode(&mut reader).map_err(|e| Error::Auth(format!("dhparam der: {e}")))?;
    if header.tag != Tag::Sequence {
        return Err(Error::Auth("dhparam: expected SEQUENCE".into()));
    }
    let prime = UintRef::decode(&mut reader).map_err(|e| Error::Auth(format!("dhparam prime: {e}")))?;
    let generator = UintRef::decode(&mut reader).map_err(|e| Error::Auth(format!("dhparam generator: {e}")))?;
    Ok((
        num_bigint::BigUint::from_bytes_be(prime.as_bytes()),
        num_bigint::BigUint::from_bytes_be(generator.as_bytes()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_reserved_and_unreserved() {
        assert_eq!(percent_encode("/:=&"), "%2F%3A%3D%26");
        assert_eq!(percent_encode("aZ0-._~"), "aZ0-._~");
    }

    // Official worked example (PDF §7.5.2): authenticated GET, proving query-param fold + sort.
    // This exact base string is reused by `request_signature_matches_pdf` below.
    const PDF_BASE_STRING: &str =
        "GET&http%3A%2F%2Flocalhost%3A12345%2Ftradingapi%2Fv1%2Fmarketdata%2Fsnapshot\
         &conid%3D8314%26oauth_consumer_key%3DTESTCONS%26oauth_nonce%3Daecef17086308940e861\
         %26oauth_signature_method%3DHMAC-SHA256%26oauth_timestamp%3D1473795686\
         %26oauth_token%3D6f531f8fd316915af53f";

    #[test]
    fn base_string_folds_query_params() {
        let params = [
            ("conid", "8314"),
            ("oauth_consumer_key", "TESTCONS"),
            ("oauth_nonce", "aecef17086308940e861"),
            ("oauth_signature_method", "HMAC-SHA256"),
            ("oauth_timestamp", "1473795686"),
            ("oauth_token", "6f531f8fd316915af53f"),
        ]
        .map(|(k, v)| (k.to_string(), v.to_string()));
        let got = signature_base_string(
            "GET",
            "http://localhost:12345/tradingapi/v1/marketdata/snapshot",
            &params,
        );
        assert_eq!(got, PDF_BASE_STRING);
    }

    // PDF §7.4.4 — the token validation `mint` performs: hex(HMAC_SHA1(LST_bytes, consumer_key))
    // must equal the signature the gateway returns.
    #[test]
    fn lst_validation_matches_pdf() {
        let lst = b64_decode("YBWbLw+9RYP2nWrPQHxHZkBb1aM=").unwrap();
        let signature = hex_encode(&hmac_sha1(&lst, b"TESTCONS"));
        assert_eq!(signature, "543c55477d6cbb0e792d1e4f8111cec7305ba3f4");
    }

    // PDF §7.5.3 — request signature = base64(HMAC_SHA256(base64_decode(LST), base_string)).
    #[test]
    fn request_signature_matches_pdf() {
        let lst = b64_decode("YBWbLw+9RYP2nWrPQHxHZkBb1aM=").unwrap();
        let sig = b64_encode(&hmac_sha256(&lst, PDF_BASE_STRING.as_bytes()));
        assert_eq!(sig, "+BdIuZDNooYZAbO9RZUCTC5F/3HjFOb04Tu4crpi0v8=");
    }

    #[test]
    fn parse_dh_params_extracts_prime_and_generator() {
        // SEQUENCE { prime 0x00DEADBEEF, generator 0x02, privateValueLength 0x05 } — the trailing
        // optional field must be ignored.
        let der = [
            0x30, 0x0d, 0x02, 0x05, 0x00, 0xde, 0xad, 0xbe, 0xef, 0x02, 0x01, 0x02, 0x02, 0x01,
            0x05,
        ];
        let pem = format!(
            "-----BEGIN DH PARAMETERS-----\n{}\n-----END DH PARAMETERS-----\n",
            b64_encode(&der)
        );
        let (prime, generator) = parse_dh_params(&pem).unwrap();
        assert_eq!(prime, num_bigint::BigUint::from(0xdeadbeefu32));
        assert_eq!(generator, num_bigint::BigUint::from(2u32));
    }
}
