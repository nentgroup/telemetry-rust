// Initialization logic was retired from https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use opentelemetry_sdk::{
    resource::{OsResourceDetector, ResourceDetector},
    Resource,
};
use tracing::Subscriber;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, registry::LookupSpan, Layer,
};

pub mod middleware;
pub mod propagation;

pub mod http;

#[cfg(feature = "axum")]
pub use tracing_opentelemetry_instrumentation_sdk;

#[cfg(feature = "otlp")]
pub mod otlp;

#[cfg(feature = "integration_test")]
pub mod test;

mod filter;

#[derive(Debug, Default)]
pub struct DetectResource {
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
}

impl DetectResource {
    /// `service.name` is first extracted from environment variables
    /// (in this order) `OTEL_SERVICE_NAME`, `SERVICE_NAME`, `APP_NAME`.
    /// But a default value can be provided with this method.

    /// `service.name` is first extracted from environment variables
    /// (in this order) `SERVICE_VERSION`, `APP_VERSION`.
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
        let base = Resource::default();
        let fallback = Resource::from_detectors(
            std::time::Duration::from_secs(0),
            vec![
                Box::new(ServiceInfoDetector {
                    fallback_service_name: self.fallback_service_name,
                    fallback_service_version: self.fallback_service_version,
                }),
                Box::new(OsResourceDetector),
                //Box::new(ProcessResourceDetector),
            ],
        );
        let rsrc = base.merge(&fallback); // base has lower priority

        // Debug
        rsrc.iter().for_each(
            |kv| tracing::debug!(target: "otel::setup::resource", key = %kv.0, value = %kv.1),
        );

        rsrc
    }
}

#[derive(Debug)]
pub struct ServiceInfoDetector {
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
}

impl ResourceDetector for ServiceInfoDetector {
    fn detect(&self, _timeout: std::time::Duration) -> Resource {
        let service_name = std::env::var("OTEL_SERVICE_NAME")
            .or_else(|_| std::env::var("SERVICE_NAME"))
            .or_else(|_| std::env::var("APP_NAME"))
            .ok()
            .or_else(|| Some(self.fallback_service_name.to_string()))
            .map(|v| {
                opentelemetry_semantic_conventions::resource::SERVICE_NAME.string(v)
            });
        let service_version = std::env::var("OTEL_SERVICE_VERSION")
            .or_else(|_| std::env::var("SERVICE_VERSION"))
            .or_else(|_| std::env::var("APP_VERSION"))
            .ok()
            .or_else(|| Some(self.fallback_service_version.to_string()))
            .map(|v| {
                opentelemetry_semantic_conventions::resource::SERVICE_VERSION.string(v)
            });
        Resource::new(vec![service_name, service_version].into_iter().flatten())
    }
}

pub fn build_logger_text<S>(
    log_level: tracing::Level,
) -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    if cfg!(debug_assertions) {
        Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_line_number(true)
                .with_thread_names(true)
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(std::io::stdout.with_max_level(log_level)),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .with_timer(tracing_subscriber::fmt::time::uptime())
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_writer(std::io::stdout.with_max_level(log_level)),
        )
    }
}

pub fn init_tracing_with_fallbacks(
    log_level: tracing::Level,
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
) {
    let otel_rsrc =
        DetectResource::new(fallback_service_name, fallback_service_version).build();
    let otel_tracer =
        otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");

    opentelemetry::global::set_text_map_propagator(
        propagation::TextMapSplitPropagator::default(),
    );

    let otel_layer = tracing_opentelemetry::layer()
        .with_tracer(otel_tracer)
        .with_filter(filter::OtelFilter::new(log_level));
    let subscriber = tracing_subscriber::registry()
        .with(build_logger_text(log_level))
        .with(otel_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[macro_export]
macro_rules! init_tracing {
    ($log_level:expr) => {
        $crate::init_tracing_with_fallbacks(
            $log_level,
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        );
    };
}

#[inline]
pub fn shutdown_signal() {
    opentelemetry::global::shutdown_tracer_provider();
}
