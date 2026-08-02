//! `POST /tickle` — keep the session alive; returns the session id used for WebSocket auth.
//! Docs: <https://www.interactivebrokers.com/docs/web-api/api-reference/trading-session/get-session-token>

use crate::Endpoint;
use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct Request;

impl Endpoint for Request {
    type Response = Response;
    const METHOD: reqwest::Method = reqwest::Method::POST;

    fn path(&self) -> String {
        "/tickle".to_string()
    }
}

/// Fields optional + `serde(default)`, unknown ignored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Response {
    pub session: Option<String>,
}
