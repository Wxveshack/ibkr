//! `GET /iserver/marketdata/history` — historical market data for a contract.
//!
//! Official docs: <https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/#hist-md>
//! Limit: 5 concurrent requests (429 on excess); max 1000 data points per response.
//!
//! This is the canonical shape every other endpoint component imitates: a `Request`
//! that renders its inputs into a query, a `Response` that mirrors the wire object,
//! and `impl Endpoint for Request` tying the two together. No HTTP logic lives here.

use crate::client::Endpoint;
use serde::Deserialize;

/// Request parameters. `conid` and `bar` are required; the rest are optional.
#[derive(Debug, Clone)]
pub struct Request {
    /// Contract identifier for the ticker of interest. Required.
    pub conid: u64,
    /// Individual bar size. Required. See [`BarSize`].
    pub bar: BarSize,
    /// Overall duration to return. Gateway defaults to `1w` when omitted.
    /// Format: `{1-30}min, {1-8}h, {1-1000}d, {1-792}w, {1-182}m, {1-15}y`.
    pub period: Option<String>,
    /// Exchange to source data from. Empty/None = the contract's primary exchange.
    pub exchange: Option<String>,
    /// Start of the request duration, UTC, formatted `YYYYMMDD-HH:mm:ss`.
    pub start_time: Option<String>,
    /// Include data outside regular trading hours.
    pub outside_rth: Option<bool>,
    /// Type of data to return. Gateway defaults to `Trades` when omitted.
    pub source: Option<Source>,
}

/// Allowed bar sizes. Modeling these as an enum makes an invalid `bar` value
/// unrepresentable on the client side. (The docs' period-vs-bar "Step Size"
/// compatibility matrix is not enforced here — the gateway rejects bad combos.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarSize {
    Min1,
    Min2,
    Min3,
    Min5,
    Min10,
    Min15,
    Min30,
    Hour1,
    Hour2,
    Hour3,
    Hour4,
    Hour8,
    Day1,
    Week1,
    Month1,
}

impl BarSize {
    /// The wire representation the gateway expects.
    pub fn as_str(self) -> &'static str {
        match self {
            BarSize::Min1 => "1min",
            BarSize::Min2 => "2min",
            BarSize::Min3 => "3min",
            BarSize::Min5 => "5min",
            BarSize::Min10 => "10min",
            BarSize::Min15 => "15min",
            BarSize::Min30 => "30min",
            BarSize::Hour1 => "1h",
            BarSize::Hour2 => "2h",
            BarSize::Hour3 => "3h",
            BarSize::Hour4 => "4h",
            BarSize::Hour8 => "8h",
            BarSize::Day1 => "1d",
            BarSize::Week1 => "1w",
            BarSize::Month1 => "1m",
        }
    }
}

/// Type of historical data to return.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Trades,
    Midpoint,
    BidAsk,
}

impl Source {
    /// The wire representation the gateway expects.
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Trades => "Trades",
            Source::Midpoint => "Midpoint",
            Source::BidAsk => "Bid_Ask",
        }
    }
}

impl Endpoint for Request {
    type Response = Response;
    const METHOD: reqwest::Method = reqwest::Method::GET;

    fn path(&self) -> String {
        "/iserver/marketdata/history".to_string()
    }

    fn query(&self) -> Vec<(String, String)> {
        let mut q = vec![
            ("conid".to_string(), self.conid.to_string()),
            ("bar".to_string(), self.bar.as_str().to_string()),
        ];
        if let Some(period) = &self.period {
            q.push(("period".to_string(), period.clone()));
        }
        if let Some(exchange) = &self.exchange {
            q.push(("exchange".to_string(), exchange.clone()));
        }
        if let Some(start_time) = &self.start_time {
            q.push(("startTime".to_string(), start_time.clone()));
        }
        if let Some(outside_rth) = self.outside_rth {
            q.push(("outsideRth".to_string(), outside_rth.to_string()));
        }
        if let Some(source) = self.source {
            q.push(("source".to_string(), source.as_str().to_string()));
        }
        q
    }
}

/// The `history-data` response object. Every metadata field is optional so a missing
/// or renamed field never fails the decode; `data` carries the bars.
///
/// Fields are the union of the docs' prose table and its JSON example (which drift):
/// `tradingDayDuration`, `chartPanStartTime`, and `direction` appear only in the
/// example. serde ignores any field present in neither, so undocumented additions
/// are also safe.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Response {
    /// Internal request identifier.
    pub server_id: Option<String>,
    /// Ticker symbol of the contract.
    pub symbol: Option<String>,
    /// Long name of the ticker symbol.
    pub text: Option<String>,
    /// Price increment from the display rules. Docs' prose says String; the JSON
    /// example shows a number — kept as raw JSON to absorb either form.
    pub price_factor: Option<serde_json::Value>,
    /// Initial time of the request, UTC, `YYYYMMDD-HH:mm:ss`.
    pub start_time: Option<String>,
    /// High over the series, formatted `%h/%v/%t` (price scaled by priceFactor / volume /
    /// minutes-from-start), e.g. `"17510/472117.45/0"`.
    pub high: Option<String>,
    /// Low over the series, formatted `%l/%v/%t`.
    pub low: Option<String>,
    /// Duration of the historical data request.
    pub time_period: Option<String>,
    /// Number of seconds in a bar.
    pub bar_length: Option<i64>,
    /// Market data availability code (see the docs' Market Data Availability section).
    pub md_availability: Option<String>,
    /// Delay, in milliseconds, to process the request.
    pub mkt_data_delay: Option<i64>,
    /// Whether the returned data was outside regular trading hours.
    pub outside_rth: Option<bool>,
    /// Trading day duration, in minutes. (Example-only field.)
    pub trading_day_duration: Option<i64>,
    /// Factor the volume is multiplied by.
    pub volume_factor: Option<i64>,
    /// Price display rule (internal use).
    pub price_display_rule: Option<i64>,
    /// Price display value (internal use).
    pub price_display_value: Option<String>,
    /// Chart pan start time. (Example-only field.)
    pub chart_pan_start_time: Option<String>,
    /// Direction of the series. (Example-only field.)
    pub direction: Option<i64>,
    /// Whether the data can return negative values.
    pub negative_capable: Option<bool>,
    /// Message version (internal use).
    pub message_version: Option<i64>,
    /// The historical bars for the requested period.
    pub data: Vec<Bar>,
    /// Total number of data points returned (a count, not the bars themselves).
    pub points: Option<i64>,
    /// Time taken to return the details.
    pub travel_time: Option<i64>,
}

/// A single OHLCV bar. Raw IBKR field names, 1:1 with the wire.
#[derive(Debug, Clone, Deserialize)]
pub struct Bar {
    /// Open.
    pub o: f64,
    /// Close.
    pub c: f64,
    /// High.
    pub h: f64,
    /// Low.
    pub l: f64,
    /// Volume.
    pub v: f64,
    /// Epoch unix timestamp of the bar.
    pub t: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Official example response from the docs — proves both structs decode without a
    /// live gateway.
    const SAMPLE: &str = r#"{
        "serverId":"20477","symbol":"AAPL","text":"APPLE INC","priceFactor":100,
        "startTime":"20230818-08:00:00","high":"17510/472117.45/0","low":"17170/472117.45/0",
        "timePeriod":"1d","barLength":86400,"mdAvailability":"S","mktDataDelay":0,"outsideRth":true,
        "tradingDayDuration":1440,"volumeFactor":1,"priceDisplayRule":1,"priceDisplayValue":"2",
        "chartPanStartTime":"20230821-13:30:00","direction":-1,"negativeCapable":false,
        "messageVersion":2,
        "data":[{"o":173.4,"c":174.7,"h":175.1,"l":171.7,"v":472117.45,"t":16923456000}],
        "points":0,"travelTime":48
    }"#;

    #[test]
    fn decodes_official_sample() {
        let resp: Response = serde_json::from_str(SAMPLE).expect("decode sample");
        assert_eq!(resp.symbol.as_deref(), Some("AAPL"));
        assert_eq!(resp.high.as_deref(), Some("17510/472117.45/0"));
        assert_eq!(resp.trading_day_duration, Some(1440)); // example-only field is captured
        assert_eq!(resp.data.len(), 1);
        let bar = &resp.data[0];
        assert_eq!(bar.o, 173.4);
        assert_eq!(bar.t, 16923456000);
        assert_eq!(resp.points, Some(0)); // count field, distinct from data.len()
    }

    #[test]
    fn ignores_unknown_fields() {
        // A field in neither the prose table nor the example must not break the decode.
        let resp: Response =
            serde_json::from_str(r#"{"symbol":"AAPL","data":[],"futureField":123}"#)
                .expect("decode with unknown field");
        assert_eq!(resp.symbol.as_deref(), Some("AAPL"));
        assert!(resp.data.is_empty());
    }

    #[test]
    fn query_includes_required_and_set_optionals() {
        let req = Request {
            conid: 265598,
            bar: BarSize::Day1,
            period: Some("1w".to_string()),
            exchange: None,
            start_time: None,
            outside_rth: Some(true),
            source: Some(Source::Midpoint),
        };
        let q = req.query();
        assert!(q.contains(&("conid".to_string(), "265598".to_string())));
        assert!(q.contains(&("bar".to_string(), "1d".to_string())));
        assert!(q.contains(&("period".to_string(), "1w".to_string())));
        assert!(q.contains(&("outsideRth".to_string(), "true".to_string())));
        assert!(q.contains(&("source".to_string(), "Midpoint".to_string())));
        // unset optionals are absent
        assert!(!q.iter().any(|(k, _)| k == "exchange"));
    }
}
