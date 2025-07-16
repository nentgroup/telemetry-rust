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
        let layer = layer
            .json()
            .flatten_event(true)
            .with_current_span(false)
            .with_span_list(true);

        layer.with_writer(std::io::stdout)
    }};
}

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
