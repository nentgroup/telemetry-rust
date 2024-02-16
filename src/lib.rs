// Initialization logic was retired from https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use opentelemetry_sdk::{
    resource::{EnvResourceDetector, OsResourceDetector, ResourceDetector},
    Resource,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt};

pub mod middleware;
pub mod propagation;

pub mod http;

#[cfg(feature = "axum")]
pub use tracing_opentelemetry_instrumentation_sdk;

pub mod otlp;

#[cfg(feature = "test")]
pub mod test;

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
                Box::new(EnvResourceDetector::new()),
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
        let service_name = util::env_var("OTEL_SERVICE_NAME")
            .or_else(|| util::env_var("SERVICE_NAME"))
            .or_else(|| util::env_var("APP_NAME"))
            .or_else(|| Some(self.fallback_service_name.to_string()))
            .map(|v| {
                opentelemetry_semantic_conventions::resource::SERVICE_NAME.string(v)
            });
        let service_version = util::env_var("OTEL_SERVICE_VERSION")
            .or_else(|| util::env_var("SERVICE_VERSION"))
            .or_else(|| util::env_var("APP_VERSION"))
            .or_else(|| Some(self.fallback_service_version.to_string()))
            .map(|v| {
                opentelemetry_semantic_conventions::resource::SERVICE_VERSION.string(v)
            });
        Resource::new(vec![service_name, service_version].into_iter().flatten())
    }
}

macro_rules! fmt_layer {
    ($log_level:expr) => {{
        let layer = tracing_subscriber::fmt::layer();

        #[cfg(debug_assertions)]
        let layer = layer
            .pretty()
            .with_line_number(true)
            .with_thread_names(true);
        #[cfg(not(debug_assertions))]
        let layer = layer.json();

        layer
            .with_timer(tracing_subscriber::fmt::time::uptime())
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_writer(std::io::stdout.with_max_level($log_level))
    }};
}

pub fn init_tracing_with_fallbacks(
    log_level: tracing::Level,
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
) {
    // set to debug to log detected resources, configuration read and infered
    let setup_subscriber = tracing_subscriber::registry()
        .with(Into::<LevelFilter>::into(log_level))
        .with(fmt_layer!(log_level));
    let _guard = tracing::subscriber::set_default(setup_subscriber);
    tracing::info!("init logging & tracing");

    let otel_rsrc =
        DetectResource::new(fallback_service_name, fallback_service_version).build();
    let otel_tracer =
        otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");

    opentelemetry::global::set_text_map_propagator(
        propagation::TextMapSplitPropagator::from_env().expect("setup of Propagation"),
    );

    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);
    let subscriber = tracing_subscriber::registry()
        .with(filter::TracingFilter::new(log_level))
        .with(fmt_layer!(log_level))
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
    std::thread::spawn(opentelemetry::global::shutdown_tracer_provider)
        .join()
        .unwrap();
}
