//! `GET /iserver/marketdata/history` — historical market data for a contract.
//! Docs: <https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/#hist-md>

use crate::Endpoint;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Request {
    pub conid: u64,
    pub bar: BarSize,
    pub period: Option<String>,
    pub exchange: Option<String>,
    pub start_time: Option<String>,
    pub outside_rth: Option<bool>,
    pub source: Option<Source>,
}

/// Allowed `bar` values. The period-vs-bar "Step Size" matrix is not enforced here.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Trades,
    Midpoint,
    BidAsk,
}

impl Source {
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

/// `history-data` response. Fields are optional + `serde(default)` and unknown fields
/// ignored, so the docs' prose-vs-example drift never breaks the decode.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Response {
    pub server_id: Option<String>,
    pub symbol: Option<String>,
    pub text: Option<String>,
    pub price_factor: Option<i64>,
    pub start_time: Option<String>,
    /// Composite `%price/%volume/%minutes`, not a plain number.
    pub high: Option<String>,
    /// Formatted like [`Self::high`].
    pub low: Option<String>,
    pub time_period: Option<String>,
    pub bar_length: Option<i64>,
    pub md_availability: Option<String>,
    pub mkt_data_delay: Option<i64>,
    pub outside_rth: Option<bool>,
    pub trading_day_duration: Option<i64>,
    pub volume_factor: Option<i64>,
    pub price_display_rule: Option<i64>,
    pub price_display_value: Option<String>,
    pub chart_annotations: Option<String>,
    pub chart_pan_start_time: Option<String>,
    pub direction: Option<i64>,
    pub negative_capable: Option<bool>,
    pub message_version: Option<i64>,
    /// The historical bars.
    pub data: Vec<Bar>,
    /// Count of data points — not the bars themselves (those are [`Self::data`]).
    pub points: Option<i64>,
    pub travel_time: Option<i64>,
}

/// A single bar. Raw IBKR names; note the field order is o/c/h/l/v, not OHLC.
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
    /// Epoch unix timestamp.
    pub t: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Official example response from the docs.
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
        assert_eq!(resp.price_factor, Some(100));
        assert_eq!(resp.high.as_deref(), Some("17510/472117.45/0"));
        assert_eq!(resp.trading_day_duration, Some(1440));
        assert_eq!(resp.data.len(), 1);
        let bar = &resp.data[0];
        assert_eq!(bar.o, 173.4);
        assert_eq!(bar.t, 16923456000);
        assert_eq!(resp.points, Some(0));
    }

    #[test]
    fn ignores_unknown_fields() {
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
