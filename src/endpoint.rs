/// An endpoint described declaratively; [`crate::Client`] executes it.
pub trait Endpoint {
    type Response: serde::de::DeserializeOwned;
    const METHOD: reqwest::Method;
    /// Path appended to the base URL, e.g. `/iserver/marketdata/history`.
    fn path(&self) -> String;
    fn query(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn body(&self) -> Option<serde_json::Value> {
        None
    }
}
