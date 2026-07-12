//! The HTTP engine. Every endpoint's mechanics live here exactly once; endpoint
//! modules only describe their shape via the [`Endpoint`] trait.

use crate::error::Error;

/// Base URL of the local Client Portal Gateway. `iserver`/`portfolio`/etc. are part
/// of each endpoint's `path()`, not the base, so non-iserver families stay reachable.
const BASE_URL: &str = "https://localhost:5000/v1/api";

/// A connected client. Holds the configured reqwest client and the base URL; clone
/// or share it across as many endpoint calls as you like.
pub struct Client {
    http: reqwest::blocking::Client,
    base: String,
}

/// A single API endpoint, described declaratively. Implementors carry no HTTP logic —
/// just the method, path, and how to render their inputs into a query/body.
pub trait Endpoint {
    /// The typed response this endpoint decodes into.
    type Response: serde::de::DeserializeOwned;
    /// HTTP method for the request.
    const METHOD: reqwest::Method;
    /// Path appended to the base URL, e.g. `/iserver/marketdata/history`.
    fn path(&self) -> String;
    /// Query-string parameters. Default: none.
    fn query(&self) -> Vec<(String, String)> {
        vec![]
    }
    /// JSON request body. Default: none.
    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}

impl Client {
    /// Build a client against the local gateway. The gateway serves a self-signed cert
    /// and 403s requests without a User-Agent, so both are configured here.
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

    /// Execute an endpoint and decode its typed response.
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
