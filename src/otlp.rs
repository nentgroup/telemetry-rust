// Originally retired from davidB/tracing-opentelemetry-instrumentation-sdk
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use std::{collections::HashMap, str::FromStr};

use opentelemetry::trace::TraceError;
use opentelemetry_http::hyper::HyperClient;
use opentelemetry_otlp::SpanExporterBuilder;
use opentelemetry_sdk::{
    trace::{Sampler, Tracer},
    Resource,
};
use std::time::Duration;
use tracing::Level;

#[derive(Default, Debug, PartialEq)]
pub enum Protocol {
    #[default]
    Grpc,
    HttpProtobuf,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[must_use]
pub fn identity(
    v: opentelemetry_otlp::OtlpTracePipeline<SpanExporterBuilder>,
) -> opentelemetry_otlp::OtlpTracePipeline<SpanExporterBuilder> {
    v
}

// see https://opentelemetry.io/docs/reference/specification/protocol/exporter/
pub fn init_tracer<F>(resource: Resource, transform: F) -> Result<Tracer, TraceError>
where
    F: FnOnce(
        opentelemetry_otlp::OtlpTracePipeline<SpanExporterBuilder>,
    ) -> opentelemetry_otlp::OtlpTracePipeline<SpanExporterBuilder>,
{
    use opentelemetry_otlp::WithExportConfig;

    let (maybe_protocol, maybe_endpoint) = read_protocol_and_endpoint_from_env();
    let (protocol, endpoint) = infer_protocol_and_endpoint(
        maybe_protocol.as_deref(),
        maybe_endpoint.as_deref(),
    )?;
    tracing::debug!(target: "otel::setup", OTEL_EXPORTER_OTLP_TRACES_ENDPOINT = endpoint);
    tracing::debug!(target: "otel::setup", OTEL_EXPORTER_OTLP_TRACES_PROTOCOL = protocol.to_string());
    let exporter: SpanExporterBuilder = match protocol {
        Protocol::HttpProtobuf => opentelemetry_otlp::new_exporter()
            .http()
            .with_http_client(HyperClient::new_with_timeout(
                hyper::Client::new(),
                Duration::from_millis(1500), // TODO: make configurable
            ))
            .with_endpoint(endpoint)
            .with_headers(read_headers_from_env())
            .into(),
        Protocol::Grpc => opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint)
            .into(),
    };

    let mut pipeline = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(resource)
                .with_sampler(read_sampler_from_env()),
        );
    pipeline = transform(pipeline);
    pipeline.install_batch(opentelemetry_sdk::runtime::Tokio)
}

/// turn a string of "k1=v1,k2=v2,..." into an iterator of (key, value) tuples
fn parse_headers(val: &str) -> impl Iterator<Item = (String, String)> + '_ {
    val.split(',').filter_map(|kv| {
        let s = kv
            .split_once('=')
            .map(|(k, v)| (k.to_owned(), v.to_owned()));
        s
    })
}
fn read_headers_from_env() -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.extend(parse_headers(
        &std::env::var("OTEL_EXPORTER_OTLP_HEADERS").unwrap_or_default(),
    ));
    headers.extend(parse_headers(
        &std::env::var("OTEL_EXPORTER_OTLP_TRACES_HEADERS").unwrap_or_default(),
    ));
    headers
}
fn read_protocol_and_endpoint_from_env() -> (Option<String>, Option<String>) {
    let maybe_endpoint = std::env::var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT"))
        .ok();
    let maybe_protocol = std::env::var("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL"))
        .ok();
    (maybe_protocol, maybe_endpoint)
}
pub fn read_otel_log_level_from_env() -> Level {
    let default_log_level = Level::INFO;
    if let Ok(level_str) = std::env::var("OTEL_LOG_LEVEL") {
        level_str.parse().unwrap_or(default_log_level)
    } else {
        default_log_level
    }
}

/// see <https://opentelemetry.io/docs/reference/specification/sdk-environment-variables/#general-sdk-configuration>
/// TODO log error and infered sampler
fn read_sampler_from_env() -> Sampler {
    let mut name = std::env::var("OTEL_TRACES_SAMPLER")
        .ok()
        .unwrap_or_default()
        .to_lowercase();
    let v = match name.as_str() {
        "always_on" => Sampler::AlwaysOn,
        "always_off" => Sampler::AlwaysOff,
        "traceidratio" => Sampler::TraceIdRatioBased(read_sampler_arg_from_env(1f64)),
        "parentbased_always_on" => Sampler::ParentBased(Box::new(Sampler::AlwaysOn)),
        "parentbased_always_off" => Sampler::ParentBased(Box::new(Sampler::AlwaysOff)),
        "parentbased_traceidratio" => Sampler::ParentBased(Box::new(
            Sampler::TraceIdRatioBased(read_sampler_arg_from_env(1f64)),
        )),
        "jaeger_remote" => todo!("unsupported: OTEL_TRACES_SAMPLER='jaeger_remote'"),
        "xray" => todo!("unsupported: OTEL_TRACES_SAMPLER='xray'"),
        _ => {
            name = "parentbased_always_on".to_string();
            Sampler::ParentBased(Box::new(Sampler::AlwaysOn))
        }
    };
    tracing::debug!(target: "otel::setup", OTEL_TRACES_SAMPLER = ?name);
    v
}

fn read_sampler_arg_from_env<T>(default: T) -> T
where
    T: FromStr + Copy + std::fmt::Debug,
{
    //TODO Log for invalid value (how to log)
    let v = std::env::var("OTEL_TRACES_SAMPLER_ARG")
        .map_or(default, |s| T::from_str(&s).unwrap_or(default));
    tracing::debug!(target: "otel::setup", OTEL_TRACES_SAMPLER_ARG = ?v);
    v
}

fn infer_protocol_and_endpoint(
    maybe_protocol: Option<&str>,
    maybe_endpoint: Option<&str>,
) -> Result<(Protocol, String), TraceError> {
    let protocol = match maybe_protocol {
        Some("grpc") => Protocol::Grpc,
        Some("http") | Some("http/protobuf") => Protocol::HttpProtobuf,
        Some(other) => {
            return Err(TraceError::from(format!(
                "unsupported protocol {other:?} form env"
            )))
        }
        None => {
            if maybe_endpoint.map_or(false, |e| e.contains(":4317")) {
                Protocol::Grpc
            } else {
                Protocol::HttpProtobuf
            }
        }
    };

    let endpoint = match protocol {
        Protocol::HttpProtobuf => maybe_endpoint.unwrap_or("http://localhost:4318"), //Devskim: ignore DS137138
        Protocol::Grpc => maybe_endpoint.unwrap_or("http://localhost:4317"), //Devskim: ignore DS137138
    };

    Ok((protocol, endpoint.to_owned()))
}

#[cfg(test)]
mod tests {
    use assert2::assert;
    use rstest::rstest;

    use super::*;
    use Protocol::*;

    #[rstest]
    #[case(None, None, HttpProtobuf, "http://localhost:4318")] //Devskim: ignore DS137138
    #[case(Some("http/protobuf"), None, HttpProtobuf, "http://localhost:4318")] //Devskim: ignore DS137138
    #[case(Some("http"), None, HttpProtobuf, "http://localhost:4318")] //Devskim: ignore DS137138
    #[case(Some("grpc"), None, Grpc, "http://localhost:4317")] //Devskim: ignore DS137138
    #[case(None, Some("http://localhost:4317"), Grpc, "http://localhost:4317")]
    #[case(
        Some("http/protobuf"),
        Some("http://localhost:4318"), //Devskim: ignore DS137138
        HttpProtobuf,
        "http://localhost:4318" //Devskim: ignore DS137138
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4318"),
        HttpProtobuf,
        "https://examples.com:4318"
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4317"),
        HttpProtobuf,
        "https://examples.com:4317"
    )]
    fn test_infer_protocol_and_endpoint(
        #[case] traces_protocol: Option<&str>,
        #[case] traces_endpoint: Option<&str>,
        #[case] expected_protocol: Protocol,
        #[case] expected_endpoint: &str,
    ) {
        assert!(
            infer_protocol_and_endpoint(traces_protocol, traces_endpoint).unwrap()
                == (expected_protocol, expected_endpoint.to_string())
        );
    }
}
