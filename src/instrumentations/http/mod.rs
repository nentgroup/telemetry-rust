//! HTTP client instrumentation utilities.

#[cfg(feature = "reqwest")]
mod client;

#[cfg(feature = "reqwest")]
pub mod reqwest;
