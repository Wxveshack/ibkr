//! Typed Rust client for the Interactive Brokers Client Portal Web API.
//!
//! [`Client`] executes any [`Endpoint`]; endpoints live under [`iserver`] and [`oauth`].
//! For headless use, [`auth`] mints a live session token from [`Credentials`] and
//! [`Client::oauth`] signs every request. Canonical example: [`iserver::marketdata::history`].

pub mod auth;
pub mod client;
pub mod endpoint;
pub mod error;
pub mod iserver;
pub mod oauth;
pub mod tickle;

pub use auth::{Credentials, LiveSessionToken};
pub use client::Client;
pub use endpoint::Endpoint;
pub use error::Error;
