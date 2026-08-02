//! The HTTP engine: executes any [`Endpoint`], over the local gateway or OAuth to `api.ibkr.com`.

use crate::auth::{Credentials, LiveSessionToken, Signer};
use crate::endpoint::Endpoint;
use crate::error::Error;
use crate::{iserver::auth::ssodh, tickle};

const GATEWAY_URL: &str = "https://localhost:5000/v1/api";
const OAUTH_URL: &str = "https://api.ibkr.com/v1/api";

pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
    signer: Option<Signer>,
}

impl Client {
    /// Talk to a local Client Portal Gateway (browser-authenticated, self-signed cert).
    pub fn new() -> Self {
        let http = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true) // gateway serves a self-signed cert
            .user_agent(concat!("ibkr/", env!("CARGO_PKG_VERSION"))) // gateway 403s with no User-Agent
            .build()
            .expect("build reqwest client");
        Self {
            http,
            base: GATEWAY_URL.to_string(),
            signer: None,
        }
    }

    /// Mint a live session token against `api.ibkr.com` from the given credentials.
    pub fn mint(creds: &Credentials) -> Result<LiveSessionToken, Error> {
        let http = reqwest::blocking::Client::builder()
            .user_agent(concat!("ibkr/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("build reqwest client");
        crate::oauth::live_session_token::mint(creds, &http, OAUTH_URL)
    }

    /// Talk to `api.ibkr.com` headless, signing each request with the live session token.
    pub fn oauth(creds: &Credentials, lst: &LiveSessionToken) -> Result<Self, Error> {
        let http = reqwest::blocking::Client::builder()
            .user_agent(concat!("ibkr/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("build reqwest client");
        Ok(Self {
            http,
            base: OAUTH_URL.to_string(),
            signer: Some(Signer::new(creds, lst)?),
        })
    }

    /// Execute an endpoint and decode its response.
    pub fn send<E: Endpoint>(&self, ep: E) -> Result<E::Response, Error> {
        let url = format!("{}{}", self.base, ep.path());
        let query = ep.query();
        let mut req = self.http.request(E::METHOD, &url).query(&query);
        if let Some(signer) = &self.signer {
            req = req.header(
                reqwest::header::AUTHORIZATION,
                signer.authorization(E::METHOD.as_str(), &url, &query),
            );
        }
        match ep.body() {
            Some(body) => req = req.json(&body),
            // Empty POSTs need an explicit Content-Length: 0 or the gateway 411s.
            None if E::METHOD == reqwest::Method::POST => req = req.body(""),
            None => {}
        }
        Ok(req.send()?.error_for_status()?.json()?)
    }

    /// Open the brokerage session (`compete`/`publish` both true). OAuth mode only.
    pub fn init_session(&self) -> Result<ssodh::init::Response, Error> {
        self.send(ssodh::init::Request {
            publish: true,
            compete: true,
        })
    }

    /// Keep the session alive; returns the session id. OAuth mode only.
    pub fn tickle(&self) -> Result<tickle::Response, Error> {
        self.send(tickle::Request)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
