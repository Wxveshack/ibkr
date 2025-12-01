//! IBKR Rust Client
//!
//! A clean, idiomatic Rust interface to Interactive Brokers' TWS API.

pub mod contract;
pub mod historical;
pub mod message;
pub mod wire;

pub use contract::Contract;
pub use historical::{BarData, BarSize, Duration, WhatToShow};
pub use message::{IncomingMessageId, OutgoingMessageId};
pub use wire::{extract_message, make_field, make_message, parse_fields};
