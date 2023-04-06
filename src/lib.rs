use axum_tracing_opentelemetry::{otlp, resource::DetectResource};
use http::header::HeaderMap;
use opentelemetry_http::HeaderInjector;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{format::FmtSpan, writer::MakeWriterExt},
    layer::SubscriberExt,
};

pub use axum_tracing_opentelemetry::opentelemetry_tracing_layer;

pub mod propagation;

pub fn init_tracing_with_fallbacks(
    log_level: tracing::Level,
    service_name_fallback: &'static str,
    service_version_fallback: &'static str,
) {
    let otel_rsrc = DetectResource::default()
        .with_fallback_service_name(service_name_fallback)
        .with_fallback_service_version(service_version_fallback)
        .build();
    let otel_tracer =
        otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    opentelemetry::global::set_text_map_propagator(
        propagation::TextMapSplitPropagator::default(),
    );

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(std::io::stdout.with_max_level(log_level));

    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(LevelFilter::INFO)
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

pub fn inject_context(headers: &mut HeaderMap) {
    let context = tracing::Span::current().context();
    let mut injector = HeaderInjector(headers);

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut injector)
    });
}

pub fn shutdown_signal() {
    opentelemetry::global::shutdown_tracer_provider();
}
