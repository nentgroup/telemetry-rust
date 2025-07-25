#![warn(missing_docs, clippy::missing_panics_doc)]

//! A comprehensive OpenTelemetry telemetry library for Rust applications.
//!
//! This crate provides easy-to-use telemetry integration for Rust applications, with support for
//! OpenTelemetry tracing, metrics, and logging. It includes middleware for popular frameworks
//! like Axum and AWS Lambda, along with utilities for context propagation and configuration.
//!
//! # Features
//!
//! - OpenTelemetry tracing instrumentation
//! - Formatted logs with tracing metadata
//! - Context Propagation for incoming and outgoing HTTP requests
//! - Axum middleware to instrument http services
//! - AWS Lambda instrumentation layer
//! - AWS SDK instrumentation
//! - Integration testing tools
//!
//! # Available Feature Flags
//!
//! - `axum`: Axum web framework middleware support
//! - `aws-span`: AWS SDK span creation utilities
//! - `aws-instrumentation`: AWS SDK automatic instrumentation
//! - `aws-lambda`: AWS Lambda runtime middleware
//! - `aws`: All AWS features (span + instrumentation)
//! - `aws-full`: All AWS features including Lambda
//! - `future`: Future instrumentation utilities
//! - `test`: Testing utilities for OpenTelemetry validation
//! - `zipkin`: Zipkin context propagation support (enabled by default)
//! - `full`: All features enabled
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
    Resource,
    error::OTelSdkError,
    resource::{EnvResourceDetector, ResourceDetector, TelemetryResourceDetector},
    trace::SdkTracerProvider as TracerProvider,
};
pub use opentelemetry_semantic_conventions::attribute as semconv;
pub use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};

pub mod fmt;
pub mod http;
pub mod middleware;
pub mod otlp;
pub mod propagation;

#[cfg(feature = "axum")]
pub use tracing_opentelemetry_instrumentation_sdk;

#[cfg(feature = "test")]
pub mod test;

#[cfg(feature = "future")]
pub mod future;

mod filter;
mod util;

/// Resource detection utility for automatically configuring OpenTelemetry service metadata.
///
/// This struct helps detect and configure service information from environment variables
/// with fallback values. It supports the standard OpenTelemetry environment variables
/// as well as common service naming conventions.
///
/// # Environment Variables
///
/// The following environment variables are checked in order of priority:
/// - Service name: `OTEL_SERVICE_NAME`, `OTEL_RESOURCE_ATTRIBUTES`, `SERVICE_NAME`, `APP_NAME`
/// - Service version: `OTEL_SERVICE_VERSION`, `OTEL_RESOURCE_ATTRIBUTES`, `SERVICE_VERSION`, `APP_VERSION`
#[derive(Debug, Default)]
pub struct DetectResource {
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
}

impl DetectResource {
    /// Creates a new `DetectResource` with the provided fallback service name and version.
    ///
    /// # Arguments
    ///
    /// * `fallback_service_name` - The default service name to use if not found in environment variables.
    /// * `fallback_service_version` - The default service version to use if not found in environment variables.
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
        let env_detector = EnvResourceDetector::new();
        let env_resource = env_detector.detect();

        let read_from_env = |key| util::env_var(key).map(Into::into);

        let service_name_key = Key::new(semconv::SERVICE_NAME);
        let service_name_value = read_from_env("OTEL_SERVICE_NAME")
            .or_else(|| env_resource.get(&service_name_key))
            .or_else(|| read_from_env("SERVICE_NAME"))
            .or_else(|| read_from_env("APP_NAME"))
            .unwrap_or_else(|| self.fallback_service_name.into());

        let service_version_key = Key::new(semconv::SERVICE_VERSION);
        let service_version_value = read_from_env("OTEL_SERVICE_VERSION")
            .or_else(|| env_resource.get(&service_version_key))
            .or_else(|| read_from_env("SERVICE_VERSION"))
            .or_else(|| read_from_env("APP_VERSION"))
            .unwrap_or_else(|| self.fallback_service_version.into());

        let resource = Resource::builder_empty()
            .with_detectors(&[
                Box::new(TelemetryResourceDetector),
                Box::new(env_detector),
            ])
            .with_attributes([
                KeyValue::new(service_name_key, service_name_value),
                KeyValue::new(service_version_key, service_version_value),
            ])
            .build();

        // Debug
        resource.iter().for_each(
            |kv| tracing::debug!(target: "otel::setup::resource", key = %kv.0, value = %kv.1),
        );

        resource
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
/// let tracer_provider = init_tracing_with_fallbacks(Level::INFO, "my-service", "1.0.0");
///
/// // Your application code here...
///
/// shutdown_tracer_provider(&tracer_provider);
/// ```
///
/// # Panics
///
/// This function will panic if:
/// - The OTLP tracer provider cannot be initialized
/// - The text map propagator cannot be configured
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
