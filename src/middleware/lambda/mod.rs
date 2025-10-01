//! AWS Lambda instrumentation utilities.
//!
//! This module provides instrumentation layer for AWS Lambda functions.

/// Context providers for different AWS Lambda event sources.
///
/// This module contains trait definitions and implementations for creating
/// appropriate OpenTelemetry spans based on the Lambda event source type.
pub mod context;

/// OpenTelemetry layer implementation for AWS Lambda functions.
///
/// This module contains the main layer implementation that wraps Lambda
/// services to provide automatic tracing instrumentation.
pub mod layer;

pub use layer::OtelLambdaLayer;
