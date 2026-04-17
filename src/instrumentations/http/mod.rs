//! HTTP client instrumentation utilities.

#[cfg(any(feature = "reqwest", feature = "hyper-http1", feature = "hyper-http2"))]
mod client;

#[cfg(feature = "reqwest")]
pub mod reqwest;

#[cfg(any(feature = "hyper-http1", feature = "hyper-http2"))]
pub mod hyper;
