// Originally retired from davidB/tracing-opentelemetry-instrumentation-sdk
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use std::{collections::HashMap, str::FromStr};

use opentelemetry::trace::TraceError;
use opentelemetry_http::hyper::HyperClient;
use opentelemetry_otlp::{ExportConfig, Protocol, SpanExporterBuilder};
use opentelemetry_sdk::{
    trace::{Sampler, Tracer},
    Resource,
};
use std::time::Duration;
use tracing::Level;

use crate::util;

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

    let (maybe_protocol, maybe_endpoint, maybe_timeout) = read_export_config_from_env();
    let export_config = infer_export_config(
        maybe_protocol.as_deref(),
        maybe_endpoint.as_deref(),
        maybe_timeout.as_deref(),
    )?;
    tracing::debug!(target: "otel::setup", export_config = format!("{export_config:?}"));
    let exporter: SpanExporterBuilder = match export_config.protocol {
        Protocol::HttpBinary => opentelemetry_otlp::new_exporter()
            .http()
            .with_http_client(HyperClient::new_with_timeout(
                hyper::Client::new(),
                export_config.timeout,
            ))
            .with_headers(read_headers_from_env())
            .with_export_config(export_config)
            .into(),
        Protocol::Grpc => opentelemetry_otlp::new_exporter()
            .tonic()
            .with_export_config(export_config)
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
        &util::env_var("OTEL_EXPORTER_OTLP_HEADERS").unwrap_or_default(),
    ));
    headers.extend(parse_headers(
        &util::env_var("OTEL_EXPORTER_OTLP_TRACES_HEADERS").unwrap_or_default(),
    ));
    headers
}
fn read_export_config_from_env() -> (Option<String>, Option<String>, Option<String>) {
    let maybe_endpoint = util::env_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT")
        .or_else(|| util::env_var("OTEL_EXPORTER_OTLP_ENDPOINT"));
    let maybe_protocol = util::env_var("OTEL_EXPORTER_OTLP_TRACES_PROTOCOL")
        .or_else(|| util::env_var("OTEL_EXPORTER_OTLP_PROTOCOL"));
    let maybe_timeout = util::env_var("OTEL_EXPORTER_OTLP_TRACES_TIMEOUT")
        .or_else(|| util::env_var("OTEL_EXPORTER_OTLP_TIMEOUT"));
    (maybe_protocol, maybe_endpoint, maybe_timeout)
}
pub fn read_otel_log_level_from_env() -> Level {
    let default_log_level = Level::INFO;
    if let Some(level_str) = util::env_var("OTEL_LOG_LEVEL") {
        level_str.parse().unwrap_or(default_log_level)
    } else {
        default_log_level
    }
}

/// see <https://opentelemetry.io/docs/reference/specification/sdk-environment-variables/#general-sdk-configuration>
/// TODO log error and infered sampler
fn read_sampler_from_env() -> Sampler {
    let mut name = util::env_var("OTEL_TRACES_SAMPLER")
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
    let v = util::env_var("OTEL_TRACES_SAMPLER_ARG")
        .map_or(default, |s| T::from_str(&s).unwrap_or(default));
    tracing::debug!(target: "otel::setup", OTEL_TRACES_SAMPLER_ARG = ?v);
    v
}

fn infer_export_config(
    maybe_protocol: Option<&str>,
    maybe_endpoint: Option<&str>,
    maybe_timeout: Option<&str>,
) -> Result<ExportConfig, TraceError> {
    let protocol = match maybe_protocol {
        Some("grpc") => Protocol::Grpc,
        Some("http") | Some("http/protobuf") => Protocol::HttpBinary,
        Some(other) => {
            return Err(TraceError::from(format!(
                "unsupported protocol {other:?} form env"
            )))
        }
        None => match maybe_endpoint {
            Some(e) if e.contains(":4317") => Protocol::Grpc,
            _ => Protocol::HttpBinary,
        },
    };

    let endpoint = match protocol {
        Protocol::HttpBinary => maybe_endpoint.unwrap_or("http://localhost:4318"),
        Protocol::Grpc => maybe_endpoint.unwrap_or("http://localhost:4317"),
    };

    let timeout = match maybe_timeout {
        Some(millis) => Duration::from_millis(millis.parse::<u64>().map_err(|err| {
            TraceError::from(format!("invalid timeout {millis:?} form env: {err}"))
        })?),
        None => {
            Duration::from_secs(opentelemetry_otlp::OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT)
        }
    };

    Ok(ExportConfig {
        endpoint: endpoint.to_owned(),
        protocol,
        timeout,
    })
}

#[cfg(test)]
mod tests {
    use assert2::assert;
    use rstest::rstest;

    use super::*;
    use Protocol::*;

    const TIMEOUT: Duration =
        Duration::from_secs(opentelemetry_otlp::OTEL_EXPORTER_OTLP_TIMEOUT_DEFAULT);

    #[rstest]
    #[case(None, None, None, HttpBinary, "http://localhost:4318", TIMEOUT)]
    #[case(
        Some("http/protobuf"),
        None,
        None,
        HttpBinary,
        "http://localhost:4318",
        TIMEOUT
    )]
    #[case(Some("http"), None, None, HttpBinary, "http://localhost:4318", TIMEOUT)]
    #[case(Some("grpc"), None, None, Grpc, "http://localhost:4317", TIMEOUT)]
    #[case(
        None,
        Some("http://localhost:4317"),
        None,
        Grpc,
        "http://localhost:4317",
        TIMEOUT
    )]
    #[case(
        Some("http/protobuf"),
        Some("http://localhost:4318"),
        None,
        HttpBinary,
        "http://localhost:4318",
        TIMEOUT
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4318"),
        None,
        HttpBinary,
        "https://examples.com:4318",
        TIMEOUT
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4317"),
        Some("12345"),
        HttpBinary,
        "https://examples.com:4317",
        Duration::from_millis(12345)
    )]
    fn test_infer_export_config(
        #[case] traces_protocol: Option<&str>,
        #[case] traces_endpoint: Option<&str>,
        #[case] traces_timeout: Option<&str>,
        #[case] expected_protocol: Protocol,
        #[case] expected_endpoint: &str,
        #[case] expected_timeout: Duration,
    ) {
        let ExportConfig {
            protocol,
            endpoint,
            timeout,
        } = infer_export_config(traces_protocol, traces_endpoint, traces_timeout)
            .unwrap();

        assert!(protocol == expected_protocol);
        assert!(endpoint == expected_endpoint);
        assert!(timeout == expected_timeout);
    }
}
