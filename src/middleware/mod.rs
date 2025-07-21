#[cfg(feature = "aws-span")]
/// Instrumentation utilities for AWS SDK operations.
///
/// This module provides instrumentation for AWS services,
/// including span creation and context propagation for AWS SDK operations.
pub mod aws;

#[cfg(feature = "axum")]
/// Axum web framework middleware.
///
/// Provides middleware for the Axum web framework to automatically
/// instrument HTTP requests with OpenTelemetry tracing.
pub mod axum;

#[cfg(feature = "aws-lambda")]
/// AWS Lambda instrumentation utilities.
///
/// This module provides instrumentation layer for AWS Lambda functions.
pub mod lambda;
