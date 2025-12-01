//! Contract definition for TWS API.
//!
//! A Contract uniquely identifies a tradeable instrument.

use crate::wire::make_field;

/// Security type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SecurityType {
    /// Stock or ETF
    #[default]
    Stock,
    /// Option
    Option,
    /// Future
    Future,
    /// Index
    Index,
    /// Forex pair
    Forex,
    /// Cash (for forex)
    Cash,
    /// Contract for difference
    Cfd,
    /// Combo/spread
    Bag,
}

impl SecurityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stock => "STK",
            Self::Option => "OPT",
            Self::Future => "FUT",
            Self::Index => "IND",
            Self::Forex => "FOREX",
            Self::Cash => "CASH",
            Self::Cfd => "CFD",
            Self::Bag => "BAG",
        }
    }
}

impl std::fmt::Display for SecurityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Option right (call or put).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OptionRight {
    #[default]
    None,
    Call,
    Put,
}

impl OptionRight {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Call => "C",
            Self::Put => "P",
        }
    }
}

impl std::fmt::Display for OptionRight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Contract specification for a tradeable instrument.
#[derive(Debug, Clone, Default)]
pub struct Contract {
    /// TWS contract identifier (0 if unspecified)
    pub con_id: i32,
    /// Ticker symbol
    pub symbol: String,
    /// Security type
    pub sec_type: SecurityType,
    /// Expiration date for derivatives (YYYYMMDD or YYYYMM)
    pub last_trade_date: String,
    /// Strike price for options
    pub strike: f64,
    /// Option right (call/put)
    pub right: OptionRight,
    /// Contract multiplier for derivatives
    pub multiplier: String,
    /// Exchange (e.g., "SMART", "NYSE", "NASDAQ")
    pub exchange: String,
    /// Primary exchange for SMART routing
    pub primary_exchange: String,
    /// Currency (e.g., "USD")
    pub currency: String,
    /// Local exchange symbol
    pub local_symbol: String,
    /// Trading class
    pub trading_class: String,
    /// Include expired contracts in searches
    pub include_expired: bool,
}

impl Contract {
    /// Create a new stock contract.
    pub fn stock(symbol: &str, exchange: &str, currency: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            sec_type: SecurityType::Stock,
            exchange: exchange.to_string(),
            currency: currency.to_string(),
            ..Default::default()
        }
    }

    /// Create a new forex contract.
    pub fn forex(pair: &str) -> Self {
        // Forex pairs are like "EUR.USD" -> symbol=EUR, currency=USD
        Self {
            symbol: pair.to_string(),
            sec_type: SecurityType::Cash,
            exchange: "IDEALPRO".to_string(),
            currency: "USD".to_string(),
            ..Default::default()
        }
    }

    /// Encode contract fields for a request message.
    ///
    /// This encodes the standard contract fields used in most requests.
    /// Server version assumed >= 68 (MIN_SERVER_VER_TRADING_CLASS).
    pub fn encode(&self) -> String {
        let mut msg = String::new();

        // conId (server version >= 68)
        msg.push_str(&make_field(self.con_id));

        // Core fields
        msg.push_str(&make_field(&self.symbol));
        msg.push_str(&make_field(self.sec_type.as_str()));
        msg.push_str(&make_field(&self.last_trade_date));

        // Strike: send empty string if 0.0
        if self.strike == 0.0 {
            msg.push_str(&make_field(""));
        } else {
            msg.push_str(&make_field(self.strike));
        }

        msg.push_str(&make_field(self.right.as_str()));
        msg.push_str(&make_field(&self.multiplier));
        msg.push_str(&make_field(&self.exchange));
        msg.push_str(&make_field(&self.primary_exchange));
        msg.push_str(&make_field(&self.currency));
        msg.push_str(&make_field(&self.local_symbol));

        // tradingClass (server version >= 68)
        msg.push_str(&make_field(&self.trading_class));

        msg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stock_contract() {
        let c = Contract::stock("AAPL", "SMART", "USD");
        assert_eq!(c.symbol, "AAPL");
        assert_eq!(c.sec_type, SecurityType::Stock);
        assert_eq!(c.exchange, "SMART");
        assert_eq!(c.currency, "USD");
    }

    #[test]
    fn test_contract_encode() {
        let c = Contract::stock("AAPL", "SMART", "USD");
        let encoded = c.encode();

        // Should contain null-separated fields
        assert!(encoded.contains("0\0"));      // con_id
        assert!(encoded.contains("AAPL\0"));   // symbol
        assert!(encoded.contains("STK\0"));    // sec_type
        assert!(encoded.contains("SMART\0")); // exchange
        assert!(encoded.contains("USD\0"));    // currency
    }
}
