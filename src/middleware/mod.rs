//! Middleware integrations for popular web frameworks and platforms.

#[cfg(feature = "aws-span")]
pub mod aws;

#[cfg(feature = "axum")]
pub mod axum;

#[cfg(feature = "reqwest")]
pub mod http;

#[cfg(feature = "aws-lambda")]
pub mod lambda;
