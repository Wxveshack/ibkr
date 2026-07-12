//! Typed Rust client for the Interactive Brokers Client Portal Web API.
//!
//! The [`Client`] + [`Endpoint`] pair is the engine: all HTTP mechanics live in one
//! place. Each endpoint under [`iserver`] is a self-contained component describing its
//! own request/response shape — the canonical example is
//! [`iserver::marketdata::history`].

pub mod client;
pub mod error;
pub mod iserver;

pub use client::{Client, Endpoint};
pub use error::Error;
