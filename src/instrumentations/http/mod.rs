//! HTTP client instrumentation utilities.

mod client;

#[cfg(feature = "reqwest")]
pub mod reqwest;

#[cfg(any(
    feature = "hyper-http1",
    feature = "hyper-http2",
    feature = "hyper-client-legacy"
))]
pub mod hyper;

#[cfg(test)]
mod test_utils;
