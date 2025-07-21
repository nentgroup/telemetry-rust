//! A comprehensive OpenTelemetry telemetry library for Rust applications.
//!
//! This crate provides easy-to-use telemetry integration for Rust applications, with support for
//! OpenTelemetry tracing, metrics, and logging. It includes middleware for popular frameworks
//! like Axum and AWS Lambda, along with utilities for context propagation and configuration.
//!
//! # Features
//!
//! - **OTLP Integration**: Built-in support for OpenTelemetry Protocol (OTLP) exporters
//! - **Framework Middleware**: Ready-to-use middleware for Axum and AWS Lambda
//! - **Context Propagation**: HTTP header-based context propagation for distributed tracing
//! - **Flexible Configuration**: Environment-based configuration with sensible defaults
//! - **Multiple Backends**: Support for Zipkin, Jaeger, and other OpenTelemetry collectors
//!
//! # Quick Start
//!
//! ```rust
//! use telemetry_rust::{init_tracing, shutdown_tracer_provider};
//! use tracing::Level;
//!
//! // Initialize telemetry
//! let tracer_provider = init_tracing!(Level::INFO);
//!
//! // Your application code here...
//!
//! // Shutdown telemetry when done
//! shutdown_tracer_provider(&tracer_provider);
//! ```

// Initialization logic was retired from https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use tracing::level_filters::LevelFilter;
#[cfg(debug_assertions)]
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;

use opentelemetry::trace::TracerProvider as _;
pub use opentelemetry::{Array, Context, Key, KeyValue, StringValue, Value, global};
pub use opentelemetry_sdk::{
    Resource, error::OTelSdkError, trace::SdkTracerProvider as TracerProvider,
};
pub use opentelemetry_semantic_conventions::attribute as semconv;
pub use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};

/// JSON formatting utilities for structured logging with OpenTelemetry.
pub mod fmt;

/// HTTP utilities for OpenTelemetry context propagation through headers.
pub mod http;

/// Middleware integrations for popular web frameworks and platforms.
pub mod middleware;

/// OpenTelemetry Protocol (OTLP) configuration and initialization utilities.
pub mod otlp;

/// Context propagation utilities for distributed tracing across service boundaries.
pub mod propagation;

#[cfg(feature = "axum")]
pub use tracing_opentelemetry_instrumentation_sdk;

/// Testing utilities for OpenTelemetry integration testing and validation.
///
/// This module provides utilities for testing distributed tracing scenarios,
/// including trace context generation, HTTP header manipulation, and response
/// validation with trace information.
#[cfg(feature = "test")]
pub mod test;

/// Future instrumentation utilities for async operation monitoring.
///
/// This module provides wrapper types and traits for instrumenting async operations
/// with callbacks that execute when futures complete, enabling monitoring and
/// metrics collection for async workloads.
#[cfg(feature = "future")]
pub mod future;

mod filter;
mod util;

// Re-export the tracing level function for public use
pub use filter::read_tracing_level_from_env;

/// Resource detection utility for automatically configuring OpenTelemetry service metadata.
///
/// This struct helps detect and configure service information from environment variables
/// with fallback values. It supports the standard OpenTelemetry environment variables
/// as well as common service naming conventions.
///
/// # Environment Variables
///
/// The following environment variables are checked in order of priority:
/// - Service name: `OTEL_SERVICE_NAME`, `SERVICE_NAME`, `APP_NAME`
/// - Service version: `OTEL_SERVICE_VERSION`, `SERVICE_VERSION`, `APP_VERSION`
#[derive(Debug, Default)]
pub struct DetectResource {
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
}

impl DetectResource {
    /// `service.name` is first extracted from environment variables
    /// (in this order) `OTEL_SERVICE_NAME`, `SERVICE_NAME`, `APP_NAME`.
    /// But a default value can be provided with this method.
    pub fn new(
        fallback_service_name: &'static str,
        fallback_service_version: &'static str,
    ) -> Self {
        DetectResource {
            fallback_service_name,
            fallback_service_version,
        }
    }

    /// Builds the OpenTelemetry resource with detected service information.
    ///
    /// This method checks environment variables in order of priority and falls back
    /// to the provided default values if no environment variables are set.
    ///
    /// # Returns
    ///
    /// A configured [`Resource`] with service name and version attributes.
    pub fn build(self) -> Resource {
        let service_name = util::env_var("OTEL_SERVICE_NAME")
            .or_else(|| util::env_var("SERVICE_NAME"))
            .or_else(|| util::env_var("APP_NAME"))
            .or_else(|| Some(self.fallback_service_name.to_string()))
            .map(|v| KeyValue::new(semconv::SERVICE_NAME, v));
        let service_version = util::env_var("OTEL_SERVICE_VERSION")
            .or_else(|| util::env_var("SERVICE_VERSION"))
            .or_else(|| util::env_var("APP_VERSION"))
            .or_else(|| Some(self.fallback_service_version.to_string()))
            .map(|v| KeyValue::new(semconv::SERVICE_VERSION, v));

        let rsrc = Resource::builder()
            .with_attributes([service_name, service_version].into_iter().flatten())
            .build();

        // Debug
        rsrc.iter().for_each(
            |kv| tracing::debug!(target: "otel::setup::resource", key = %kv.0, value = %kv.1),
        );

        rsrc
    }
}

macro_rules! fmt_layer {
    () => {{
        let layer = tracing_subscriber::fmt::layer();

        #[cfg(debug_assertions)]
        let layer = layer.compact().with_span_events(FmtSpan::CLOSE);
        #[cfg(not(debug_assertions))]
        let layer = layer.json().event_format(fmt::JsonFormat);

        layer.with_writer(std::io::stdout)
    }};
}

/// Initializes tracing with OpenTelemetry integration and fallback service information.
///
/// This function sets up a complete tracing infrastructure including:
/// - A temporary subscriber for setup logging
/// - Resource detection from environment variables with fallbacks
/// - OTLP tracer provider initialization
/// - Global propagator configuration
/// - Final subscriber with both console output and OpenTelemetry export
///
/// # Arguments
///
/// - `log_level`: The minimum log level for events
/// - `fallback_service_name`: Default service name if not found in environment variables
/// - `fallback_service_version`: Default service version if not found in environment variables
///
/// # Returns
///
/// A configured [`TracerProvider`] that should be kept alive for the duration of the application
/// and passed to [`shutdown_tracer_provider`] on shutdown.
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::{init_tracing_with_fallbacks, shutdown_tracer_provider};
/// use tracing::Level;
///
/// let tracer_provider = init_tracing_with_fallbacks(
///     Level::INFO,
///     "my-service",
///     "1.0.0"
/// );
///
/// // Your application code here...
///
/// shutdown_tracer_provider(&tracer_provider);
/// ```
pub fn init_tracing_with_fallbacks(
    log_level: tracing::Level,
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
) -> TracerProvider {
    // set to debug to log detected resources, configuration read and infered
    let setup_subscriber = tracing_subscriber::registry()
        .with(Into::<LevelFilter>::into(log_level))
        .with(fmt_layer!());
    let _guard = tracing::subscriber::set_default(setup_subscriber);
    tracing::info!("init logging & tracing");

    let otel_rsrc =
        DetectResource::new(fallback_service_name, fallback_service_version).build();
    let tracer_provider =
        otlp::init_tracer(otel_rsrc, otlp::identity).expect("TracerProvider setup");

    global::set_tracer_provider(tracer_provider.clone());
    global::set_text_map_propagator(
        propagation::TextMapSplitPropagator::from_env().expect("TextMapPropagator setup"),
    );

    let otel_layer =
        OpenTelemetryLayer::new(tracer_provider.tracer(env!("CARGO_PKG_NAME")));
    let subscriber = tracing_subscriber::registry()
        .with(Into::<filter::TracingFilter>::into(log_level))
        .with(fmt_layer!())
        .with(otel_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();

    tracer_provider
}

/// Convenience macro for initializing tracing with package name and version as fallbacks.
///
/// This macro calls [`init_tracing_with_fallbacks`] using the current package's name and version
/// from `CARGO_PKG_NAME` and `CARGO_PKG_VERSION` environment variables as fallback values.
///
/// # Arguments
///
/// - `log_level`: The minimum log level for events (e.g., `Level::INFO`)
///
/// # Returns
///
/// A configured [`TracerProvider`] that should be kept alive for the duration of the application.
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::{init_tracing, shutdown_tracer_provider};
/// use tracing::Level;
///
/// let tracer_provider = init_tracing!(Level::INFO);
///
/// // Your application code here...
///
/// shutdown_tracer_provider(&tracer_provider);
/// ```
#[macro_export]
macro_rules! init_tracing {
    ($log_level:expr) => {
        $crate::init_tracing_with_fallbacks(
            $log_level,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        )
    };
}

/// Properly shuts down a tracer provider, flushing pending spans and cleaning up resources.
///
/// This function performs a graceful shutdown of the tracer provider by:
/// 1. Attempting to flush any pending spans to the exporter
/// 2. Shutting down the tracer provider and its associated resources
/// 3. Logging any errors that occur during the shutdown process
///
/// # Arguments
///
/// - `provider`: Reference to the [`TracerProvider`] to shut down
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::{init_tracing, shutdown_tracer_provider};
/// use tracing::Level;
///
/// let tracer_provider = init_tracing!(Level::INFO);
///
/// // Your application code here...
///
/// shutdown_tracer_provider(&tracer_provider);
/// ```
#[inline]
pub fn shutdown_tracer_provider(provider: &TracerProvider) {
    if let Err(err) = provider.force_flush() {
        tracing::warn!("failed to flush tracer provider: {err:?}");
    }
    if let Err(err) = provider.shutdown() {
        tracing::warn!("failed to shutdown tracer provider: {err:?}");
    } else {
        tracing::info!("tracer provider is shutdown")
    }
}
