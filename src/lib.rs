//! IBKR Rust Client
//!
//! A clean, idiomatic Rust interface to Interactive Brokers' TWS API.
//!
//! # Example
//!
//! ```no_run
//! use ibkr::{Client, Contract, BarSize, Duration, WhatToShow};
//!
//! #[tokio::main]
//! async fn main() -> ibkr::Result<()> {
//!     let client = Client::connect("127.0.0.1:7496", 1).await?;
//!
//!     let contract = Contract::stock("AAPL", "SMART", "USD");
//!     let bars = client.historical_data(
//!         contract,
//!         Duration::Days(5),
//!         BarSize::Day1,
//!         WhatToShow::Trades,
//!         true,
//!     ).await?;
//!
//!     for bar in bars {
//!         println!("{}: O:{} H:{} L:{} C:{}",
//!             bar.date, bar.open, bar.high, bar.low, bar.close);
//!     }
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod contract;
pub mod error;
pub mod historical;
pub mod message;
pub mod wire;

pub use client::Client;
pub use contract::{Contract, OptionRight, SecurityType};
pub use error::{Error, Result};
pub use historical::{BarData, BarSize, Duration, WhatToShow};
pub use message::{IncomingMessageId, OutgoingMessageId};
pub use wire::{extract_message, make_field, make_message, parse_fields, FieldIterator};
