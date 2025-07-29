//! AWS Lambda instrumentation utilities.
//!
//! This module provides instrumentation layer for AWS Lambda functions.

pub mod context;
pub mod layer;

pub use layer::OtelLambdaLayer;
