//! Typed Rust client for the Interactive Brokers Client Portal Web API.
//!
//! [`Client`] executes any [`Endpoint`]; endpoints live under [`iserver`].
//! Canonical example: [`iserver::marketdata::history`].

pub mod client;
pub mod endpoint;
pub mod error;
pub mod iserver;

pub use client::Client;
pub use endpoint::Endpoint;
pub use error::Error;
