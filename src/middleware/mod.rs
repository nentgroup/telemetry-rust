#[cfg(feature = "aws-span")]
/// AWS-specific middleware and instrumentation utilities.
///
/// This module provides middleware and instrumentation for AWS services,
/// including span creation and context propagation for AWS SDK operations.
pub mod aws;

#[cfg(feature = "axum")]
/// Axum web framework middleware integration.
///
/// Provides middleware for the Axum web framework to automatically
/// instrument HTTP requests with OpenTelemetry tracing.
pub mod axum;

#[cfg(feature = "aws-lambda")]
/// AWS Lambda runtime middleware and instrumentation.
///
/// This module provides instrumentation utilities specifically designed
/// for AWS Lambda functions, including request/response tracing and
/// context propagation in serverless environments.
pub mod lambda;
