//! `POST /iserver/auth/ssodh/init` — open (or re-establish) the brokerage session after auth.
//! `publish`/`compete` travel as a JSON body (not signed), matching IBKR's reference client.
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

    fn body(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({ "publish": self.publish, "compete": self.compete }))
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
