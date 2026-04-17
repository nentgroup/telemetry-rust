//! Instrumentation helpers for outbound clients and SDKs.

#[cfg(any(feature = "reqwest", feature = "hyper-http1", feature = "hyper-http2"))]
pub mod http;
