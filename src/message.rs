//! TWS API message identifiers.
//!
//! Only includes message types currently supported by this crate.

/// Outgoing message IDs (client -> TWS)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum OutgoingMessageId {
    /// Request account data subscription
    ReqAccountData = 6,
    /// Request historical bar data
    ReqHistoricalData = 20,
    /// Cancel historical data request
    CancelHistoricalData = 25,
    /// Start API connection
    StartApi = 71,
}

impl OutgoingMessageId {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

impl std::fmt::Display for OutgoingMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

/// Incoming message IDs (TWS -> client)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum IncomingMessageId {
    /// Error message
    Error = 4,
    /// Account value update
    AccountValue = 6,
    /// Portfolio value update
    PortfolioValue = 7,
    /// Account download end marker
    AccountDownloadEnd = 8,
    /// Next valid order ID
    NextValidId = 9,
    /// Managed accounts list
    ManagedAccounts = 15,
    /// Historical bar data
    HistoricalData = 17,
    /// Historical data update (for keepUpToDate)
    HistoricalDataUpdate = 90,
    /// Historical data end marker
    HistoricalDataEnd = 108,
}

impl IncomingMessageId {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            4 => Some(Self::Error),
            6 => Some(Self::AccountValue),
            7 => Some(Self::PortfolioValue),
            8 => Some(Self::AccountDownloadEnd),
            9 => Some(Self::NextValidId),
            15 => Some(Self::ManagedAccounts),
            17 => Some(Self::HistoricalData),
            90 => Some(Self::HistoricalDataUpdate),
            108 => Some(Self::HistoricalDataEnd),
            _ => None,
        }
    }

    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

impl std::fmt::Display for IncomingMessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}
