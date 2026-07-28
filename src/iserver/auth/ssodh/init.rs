//! `POST /iserver/auth/ssodh/init` — open (or re-establish) the brokerage session after auth.
//! Params travel as query so they're covered by the OAuth signature.
//! Docs: <https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/#ssodh-init>

use crate::Endpoint;
use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct Request {
    pub publish: bool,
    pub compete: bool,
}

impl Endpoint for Request {
    type Response = Response;
    const METHOD: reqwest::Method = reqwest::Method::POST;

    fn path(&self) -> String {
        "/iserver/auth/ssodh/init".to_string()
    }

    fn query(&self) -> Vec<(String, String)> {
        vec![
            ("publish".to_string(), self.publish.to_string()),
            ("compete".to_string(), self.compete.to_string()),
        ]
    }
}

/// Fields optional + `serde(default)`, unknown ignored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Response {
    pub authenticated: Option<bool>,
    pub competing: Option<bool>,
    pub connected: Option<bool>,
    pub message: Option<String>,
    pub fail: Option<String>,
}
