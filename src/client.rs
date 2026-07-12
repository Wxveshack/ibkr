//! The HTTP engine: executes any [`Endpoint`].

use crate::endpoint::Endpoint;
use crate::error::Error;

const BASE_URL: &str = "https://localhost:5000/v1/api";

pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
}

impl Client {
    pub fn new() -> Self {
        let http = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true) // gateway serves a self-signed cert
            .user_agent(concat!("ibkr/", env!("CARGO_PKG_VERSION"))) // gateway 403s with no User-Agent
            .build()
            .expect("build reqwest client");
        Self {
            http,
            base: BASE_URL.to_string(),
        }
    }

    /// Execute an endpoint and decode its response.
    pub fn send<E: Endpoint>(&self, ep: E) -> Result<E::Response, Error> {
        let mut req = self
            .http
            .request(E::METHOD, format!("{}{}", self.base, ep.path()))
            .query(&ep.query());
        if let Some(body) = ep.body() {
            req = req.json(&body);
        }
        Ok(req.send()?.error_for_status()?.json()?)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
