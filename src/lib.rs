// Initialization logic was retired from https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use std::cmp::max;

use opentelemetry_sdk::{
    propagation::{
        BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
    },
    resource::{OsResourceDetector, ResourceDetector},
    Resource,
};

use opentelemetry::{propagation::TextMapPropagator, trace::TraceError};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{format::FmtSpan, writer::MakeWriterExt},
    layer::SubscriberExt,
};

pub mod middleware;
pub mod propagation;

pub mod http;

#[cfg(feature = "otlp")]
pub mod otlp;

#[cfg(feature = "integration_test")]
pub mod test;

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
        let service_version = std::env::var("SERVICE_VERSION")
            .or_else(|_| std::env::var("APP_VERSION"))
            .ok()
            .or_else(|| Some(self.fallback_service_version.to_string()))
            .map(|v| {
                opentelemetry_semantic_conventions::resource::SERVICE_VERSION.string(v)
            });
        Resource::new(vec![service_name, service_version].into_iter().flatten())
    }
}

pub fn init_tracing_with_fallbacks(
    log_level: tracing::Level,
    fallback_service_name: &'static str,
    fallback_service_version: &'static str,
) {
    // init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;
    let otel_rsrc =
        DetectResource::new(fallback_service_name, fallback_service_version).build();
    let otel_tracer =
        otlp::init_tracer(otel_rsrc, otlp::identity).expect("setup of Tracer");
    init_propagator().expect("setup of propagator");
    let otel_layer = tracing_opentelemetry::layer().with_tracer(otel_tracer);

    opentelemetry::global::set_text_map_propagator(
        propagation::TextMapSplitPropagator::default(),
    );

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(std::io::stdout.with_max_level(log_level));

    let level_filter: LevelFilter = max(log_level, tracing::Level::INFO).into();
    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(otel_layer)
        .with(level_filter);
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

/// Configure the global propagator based on content of the env variable [OTEL_PROPAGATORS](https://opentelemetry.io/docs/concepts/sdk-configuration/general-sdk-configuration/#otel_propagators)
/// Specifies Propagators to be used in a comma-separated list.
/// Default value: `"tracecontext,baggage"`
/// Example: `export OTEL_PROPAGATORS="b3"`
/// Accepted values for `OTEL_PROPAGATORS` are:
///
/// - "tracecontext": W3C Trace Context
/// - "baggage": W3C Baggage
/// - "b3": B3 Single (require feature "zipkin")
/// - "b3multi": B3 Multi (require feature "zipkin")
/// - "xray": AWS X-Ray (require feature "xray")
/// - "ottrace": OT Trace (third party) (not supported)
/// - "none": No automatically configured propagator.
///
/// # Errors
///
/// Will return `TraceError` if issue in reading or instanciate propagator.
pub fn init_propagator() -> Result<(), TraceError> {
    let value_from_env = std::env::var("OTEL_PROPAGATORS")
        .unwrap_or_else(|_| "tracecontext,baggage".to_string());
    let propagators: Vec<(Box<dyn TextMapPropagator + Send + Sync>, String)> =
        value_from_env
            .split(',')
            .map(|s| {
                let name = s.trim().to_lowercase();
                propagator_from_string(&name).map(|o| o.map(|b| (b, name)))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect();
    if !propagators.is_empty() {
        let (propagators_impl, propagators_name): (Vec<_>, Vec<_>) =
            propagators.into_iter().unzip();
        tracing::debug!(target: "otel::setup", OTEL_PROPAGATORS = propagators_name.join(","));
        let composite_propagator = TextMapCompositePropagator::new(propagators_impl);
        opentelemetry::global::set_text_map_propagator(composite_propagator);
    }
    Ok(())
}

#[allow(clippy::box_default)]
fn propagator_from_string(
    v: &str,
) -> Result<Option<Box<dyn TextMapPropagator + Send + Sync>>, TraceError> {
    match v {
        "tracecontext" => Ok(Some(Box::new(TraceContextPropagator::new()))),
        "baggage" => Ok(Some(Box::new(BaggagePropagator::new()))),
        #[cfg(feature = "zipkin")]
        "b3" => Ok(Some(Box::new(
            opentelemetry_zipkin::Propagator::with_encoding(
                opentelemetry_zipkin::B3Encoding::SingleHeader,
            ),
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'b3', try to enable compile feature 'zipkin'"
        )),
        #[cfg(feature = "zipkin")]
        "b3multi" => Ok(Some(Box::new(
            opentelemetry_zipkin::Propagator::with_encoding(
                opentelemetry_zipkin::B3Encoding::MultipleHeader,
            ),
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3multi" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'b3multi', try to enable compile feature 'zipkin'"
        )),
        #[cfg(feature = "jaeger")]
        "jaeger" => Ok(Some(Box::new(
            opentelemetry_jaeger::Propagator::default()
        ))),
        #[cfg(not(feature = "jaeger"))]
        "jaeger" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'jaeger', try to enable compile feature 'jaeger'"
        )),
        #[cfg(feature = "xray")]
        "xray" => Ok(Some(Box::new(
            opentelemetry_aws::trace::XrayPropagator::default(),
        ))),
        #[cfg(not(feature = "xray"))]
        "xray" => Err(TraceError::from(
            "unsupported propagators form env OTEL_PROPAGATORS: 'xray', try to enable compile feature 'xray'"
        )),
        "none" => Ok(None),
        unknown => Err(TraceError::from(format!(
            "unsupported propagators form env OTEL_PROPAGATORS: '{unknown}'"
        ))),
    }
}

#[cfg(test)]
#[cfg(feature = "tracer")]
mod tests {
    use assert2::let_assert;

    #[test]
    fn init_tracing_failed_on_invalid_propagator() {
        let_assert!(Err(_) = super::propagator_from_string("xxxxxx"));

        // std::env::set_var("OTEL_PROPAGATORS", "xxxxxx");
        // dbg!(std::env::var("OTEL_PROPAGATORS"));
        // let_assert!(Err(_) = init_tracing());
    }
}
