//! Historical market data types.

use crate::contract::Contract;
use crate::message::OutgoingMessageId;
use crate::wire::{make_field, FieldIterator};

/// Bar size for historical data requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarSize {
    Sec1,
    Sec5,
    Sec15,
    Sec30,
    Min1,
    Min2,
    Min3,
    Min5,
    Min15,
    Min30,
    Hour1,
    Day1,
}

impl BarSize {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sec1 => "1 sec",
            Self::Sec5 => "5 secs",
            Self::Sec15 => "15 secs",
            Self::Sec30 => "30 secs",
            Self::Min1 => "1 min",
            Self::Min2 => "2 mins",
            Self::Min3 => "3 mins",
            Self::Min5 => "5 mins",
            Self::Min15 => "15 mins",
            Self::Min30 => "30 mins",
            Self::Hour1 => "1 hour",
            Self::Day1 => "1 day",
        }
    }
}

impl std::fmt::Display for BarSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// What type of data to show for historical bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhatToShow {
    #[default]
    Trades,
    Midpoint,
    Bid,
    Ask,
    BidAsk,
    HistoricalVolatility,
    OptionImpliedVolatility,
}

impl WhatToShow {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trades => "TRADES",
            Self::Midpoint => "MIDPOINT",
            Self::Bid => "BID",
            Self::Ask => "ASK",
            Self::BidAsk => "BID_ASK",
            Self::HistoricalVolatility => "HISTORICAL_VOLATILITY",
            Self::OptionImpliedVolatility => "OPTION_IMPLIED_VOLATILITY",
        }
    }
}

impl std::fmt::Display for WhatToShow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Duration for historical data requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    Seconds(u32),
    Days(u32),
    Weeks(u32),
    Months(u32),
    Years(u32),
}

impl Duration {
    pub fn as_string(&self) -> String {
        match self {
            Self::Seconds(n) => format!("{n} S"),
            Self::Days(n) => format!("{n} D"),
            Self::Weeks(n) => format!("{n} W"),
            Self::Months(n) => format!("{n} M"),
            Self::Years(n) => format!("{n} Y"),
        }
    }
}

impl std::fmt::Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// Date format for returned bars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DateFormat {
    /// Human-readable format: "yyyymmdd hh:mm:ss"
    #[default]
    String = 1,
    /// Unix timestamp (seconds since 1970-01-01 GMT)
    Unix = 2,
}

/// A single historical bar.
#[derive(Debug, Clone, Default)]
pub struct BarData {
    /// Bar timestamp
    pub date: String,
    /// Opening price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Closing price
    pub close: f64,
    /// Volume
    pub volume: f64,
    /// Weighted average price
    pub wap: f64,
    /// Number of trades in the bar
    pub bar_count: i32,
}

impl BarData {
    /// Parse a bar from message fields.
    pub fn parse(fields: &mut FieldIterator) -> Option<Self> {
        Some(Self {
            date: fields.next_string()?.to_string(),
            open: fields.next_f64(),
            high: fields.next_f64(),
            low: fields.next_f64(),
            close: fields.next_f64(),
            volume: fields.next_f64(),
            wap: fields.next_f64(),
            bar_count: fields.next_i32(),
        })
    }
}

/// Historical data request parameters.
#[derive(Debug, Clone)]
pub struct HistoricalDataRequest {
    /// Request ID for correlation
    pub req_id: i32,
    /// Contract to request data for
    pub contract: Contract,
    /// End date/time (empty for current time)
    /// Format: "yyyymmdd HH:mm:ss [timezone]"
    pub end_date_time: String,
    /// Duration of data to request
    pub duration: Duration,
    /// Bar size
    pub bar_size: BarSize,
    /// What data to show
    pub what_to_show: WhatToShow,
    /// Use regular trading hours only
    pub use_rth: bool,
    /// Date format for returned bars
    pub format_date: DateFormat,
    /// Keep up to date with new bars
    pub keep_up_to_date: bool,
}

impl HistoricalDataRequest {
    /// Create a new historical data request.
    pub fn new(req_id: i32, contract: Contract) -> Self {
        Self {
            req_id,
            contract,
            end_date_time: String::new(),
            duration: Duration::Days(1),
            bar_size: BarSize::Hour1,
            what_to_show: WhatToShow::Trades,
            use_rth: true,
            format_date: DateFormat::String,
            keep_up_to_date: false,
        }
    }

    /// Set the end date/time.
    pub fn end_date_time(mut self, end: &str) -> Self {
        self.end_date_time = end.to_string();
        self
    }

    /// Set the duration.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the bar size.
    pub fn bar_size(mut self, size: BarSize) -> Self {
        self.bar_size = size;
        self
    }

    /// Set what data to show.
    pub fn what_to_show(mut self, what: WhatToShow) -> Self {
        self.what_to_show = what;
        self
    }

    /// Set whether to use regular trading hours only.
    pub fn use_rth(mut self, rth: bool) -> Self {
        self.use_rth = rth;
        self
    }

    /// Set the date format.
    pub fn format_date(mut self, format: DateFormat) -> Self {
        self.format_date = format;
        self
    }

    /// Set whether to keep up to date.
    pub fn keep_up_to_date(mut self, keep: bool) -> Self {
        self.keep_up_to_date = keep;
        self
    }

    /// Encode the request as a message payload.
    ///
    /// Assumes server version >= 124 (MIN_SERVER_VER_SYNT_REALTIME_BARS).
    pub fn encode(&self) -> String {
        let mut msg = String::new();

        // Message ID
        msg.push_str(&make_field(OutgoingMessageId::ReqHistoricalData.as_u32()));

        // Request ID
        msg.push_str(&make_field(self.req_id));

        // Contract fields
        msg.push_str(&self.contract.encode());

        // includeExpired
        msg.push_str(&make_field(if self.contract.include_expired { 1 } else { 0 }));

        // Request parameters
        msg.push_str(&make_field(&self.end_date_time));
        msg.push_str(&make_field(self.bar_size.as_str()));
        msg.push_str(&make_field(self.duration.as_string()));
        msg.push_str(&make_field(if self.use_rth { 1 } else { 0 }));
        msg.push_str(&make_field(self.what_to_show.as_str()));
        msg.push_str(&make_field(self.format_date as i32));

        // keepUpToDate (server version >= 124)
        msg.push_str(&make_field(if self.keep_up_to_date { 1 } else { 0 }));

        // chartOptions (empty)
        msg.push_str(&make_field(""));

        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bar_size_str() {
        assert_eq!(BarSize::Min5.as_str(), "5 mins");
        assert_eq!(BarSize::Hour1.as_str(), "1 hour");
        assert_eq!(BarSize::Day1.as_str(), "1 day");
    }

    #[test]
    fn test_duration_str() {
        assert_eq!(Duration::Days(1).as_string(), "1 D");
        assert_eq!(Duration::Weeks(2).as_string(), "2 W");
        assert_eq!(Duration::Seconds(300).as_string(), "300 S");
    }

    #[test]
    fn test_request_encode() {
        let contract = Contract::stock("AAPL", "SMART", "USD");
        let request = HistoricalDataRequest::new(1, contract)
            .duration(Duration::Days(5))
            .bar_size(BarSize::Hour1);

        let encoded = request.encode();

        // Should start with message ID
        assert!(encoded.starts_with("20\0"));
        // Should contain contract
        assert!(encoded.contains("AAPL\0"));
        // Should contain bar size
        assert!(encoded.contains("1 hour\0"));
        // Should contain duration
        assert!(encoded.contains("5 D\0"));
    }
}
