// Originally retired from davidB/tracing-opentelemetry-instrumentation-sdk
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use opentelemetry_otlp::{
    ExportConfig, ExporterBuildError, Protocol, SpanExporter, WithExportConfig,
    WithHttpConfig,
};
use opentelemetry_sdk::{
    Resource,
    trace::{Sampler, SdkTracerProvider as TracerProvider, TracerProviderBuilder},
};
use std::{collections::HashMap, num::ParseIntError, str::FromStr, time::Duration};

pub use crate::filter::read_tracing_level_from_env as read_otel_log_level_from_env;
use crate::util;

/// Error types that can occur during OpenTelemetry tracer initialization.
///
/// This enum represents the various failure modes when setting up an OTLP
/// tracer provider, including configuration errors and exporter build failures.
#[derive(thiserror::Error, Debug)]
pub enum InitTracerError {
    /// An unsupported protocol was specified in environment variables.
    ///
    /// This error occurs when the `OTEL_EXPORTER_OTLP_PROTOCOL` or
    /// `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` environment variable contains
    /// a protocol that is not supported by this library.
    #[error("unsupported protocol {0:?} form env")]
    UnsupportedEnvProtocol(String),

    /// An invalid timeout value was provided in environment variables.
    ///
    /// This error occurs when the timeout specified in `OTEL_EXPORTER_OTLP_TIMEOUT`
    /// or `OTEL_EXPORTER_OTLP_TRACES_TIMEOUT` cannot be parsed as a valid integer.
    #[error("invalid timeout {0:?} form env: {1}")]
    InvalidEnvTimeout(String, #[source] ParseIntError),

    /// An error occurred while building the OTLP exporter.
    ///
    /// This error wraps underlying exporter build errors that may occur during
    /// the construction of the OTLP span exporter.
    #[error(transparent)]
    ExporterBuildError(#[from] ExporterBuildError),
}

/// Identity transformation function for tracer provider builders.
///
/// This function accepts a [`TracerProviderBuilder`] and returns it unchanged.
/// It serves as a default transformation function when no custom configuration
/// is needed during tracer provider initialization.
///
/// # Arguments
///
/// - `v`: The tracer provider builder to return unchanged
///
/// # Returns
///
/// The same tracer provider builder that was passed in
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::otlp::{identity, init_tracer};
/// use opentelemetry_sdk::Resource;
///
/// let resource = Resource::builder().build();
/// let tracer_provider = init_tracer(resource, identity).unwrap();
/// ```
#[must_use]
pub fn identity(v: TracerProviderBuilder) -> TracerProviderBuilder {
    v
}

/// Initializes an OpenTelemetry tracer provider with OTLP exporter configuration.
///
/// This function creates a fully configured tracer provider with an OTLP exporter
/// that reads configuration from environment variables. It supports both HTTP and
/// gRPC protocols and allows for custom transformation of the tracer provider builder.
///
/// # Environment Variables
///
/// The function reads configuration from the following environment variables:
/// - `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` / `OTEL_EXPORTER_OTLP_ENDPOINT`: Exporter endpoint
/// - `OTEL_EXPORTER_OTLP_TRACES_PROTOCOL` / `OTEL_EXPORTER_OTLP_PROTOCOL`: Protocol (grpc, http, http/protobuf)
/// - `OTEL_EXPORTER_OTLP_TRACES_TIMEOUT` / `OTEL_EXPORTER_OTLP_TIMEOUT`: Timeout in milliseconds
/// - `OTEL_EXPORTER_OTLP_HEADERS` / `OTEL_EXPORTER_OTLP_TRACES_HEADERS`: Additional headers
/// - `OTEL_TRACES_SAMPLER`: Sampling strategy configuration
/// - `OTEL_TRACES_SAMPLER_ARG`: Sampling rate for ratio-based samplers
///
/// # Arguments
///
/// - `resource`: OpenTelemetry resource containing service metadata
/// - `transform`: Function to customize the tracer provider builder before building
///
/// # Returns
///
/// A configured [`TracerProvider`] on success, or an [`InitTracerError`] on failure
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::otlp::{identity, init_tracer};
/// use opentelemetry_sdk::Resource;
///
/// let resource = Resource::builder().build();
/// let tracer_provider = init_tracer(resource, identity)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
// see https://opentelemetry.io/docs/reference/specification/protocol/exporter/
pub fn init_tracer<F>(
    resource: Resource,
    transform: F,
) -> Result<TracerProvider, InitTracerError>
where
    F: FnOnce(TracerProviderBuilder) -> TracerProviderBuilder,
{
    let (maybe_protocol, maybe_endpoint, maybe_timeout) = read_export_config_from_env();
    let export_config = infer_export_config(
        maybe_protocol.as_deref(),
        maybe_endpoint.as_deref(),
        maybe_timeout.as_deref(),
    )?;
    tracing::debug!(target: "otel::setup", export_config = format!("{export_config:?}"));
    let exporter: SpanExporter = match export_config.protocol {
        Protocol::HttpBinary => SpanExporter::builder()
            .with_http()
            .with_headers(read_headers_from_env())
            .with_export_config(export_config)
            .build()?,
        Protocol::Grpc => SpanExporter::builder()
            .with_tonic()
            .with_export_config(export_config)
            .build()?,
        Protocol::HttpJson => unreachable!("HttpJson protocol is not supported"),
    };

    let tracer_provider_builder = TracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .with_sampler(read_sampler_from_env());

    Ok(transform(tracer_provider_builder).build())
}

/// turn a string of "k1=v1,k2=v2,..." into an iterator of (key, value) tuples
fn parse_headers(val: &str) -> impl Iterator<Item = (String, String)> + '_ {
    val.split(',').filter_map(|kv| {
        kv.split_once('=')
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
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
) -> Result<ExportConfig, InitTracerError> {
    let protocol = match maybe_protocol {
        Some("grpc") => Protocol::Grpc,
        Some("http") | Some("http/protobuf") => Protocol::HttpBinary,
        Some(other) => {
            return Err(InitTracerError::UnsupportedEnvProtocol(other.to_owned()));
        }
        None => match maybe_endpoint {
            Some(e) if e.contains(":4317") => Protocol::Grpc,
            _ => Protocol::HttpBinary,
        },
    };

    let timeout = maybe_timeout
        .map(|millis| {
            millis
                .parse::<u64>()
                .map_err(|err| InitTracerError::InvalidEnvTimeout(millis.to_owned(), err))
        })
        .transpose()?
        .map(Duration::from_millis);

    Ok(ExportConfig {
        endpoint: maybe_endpoint.map(ToOwned::to_owned),
        protocol,
        timeout,
    })
}

#[cfg(test)]
mod tests {
    use assert2::{assert, let_assert};
    use rstest::rstest;

    use super::*;
    use Protocol::*;

    #[rstest]
    #[case(None, None, None, HttpBinary, None, None)]
    #[case(Some("http/protobuf"), None, None, HttpBinary, None, None)]
    #[case(Some("http"), None, None, HttpBinary, None, None)]
    #[case(Some("grpc"), None, None, Grpc, None, None)]
    #[case(
        None,
        Some("http://localhost:4317"),
        None,
        Grpc,
        Some("http://localhost:4317"),
        None
    )]
    #[case(
        Some("http/protobuf"),
        Some("http://localhost:4318"),
        None,
        HttpBinary,
        Some("http://localhost:4318"),
        None
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4318"),
        None,
        HttpBinary,
        Some("https://examples.com:4318"),
        None
    )]
    #[case(
        Some("http/protobuf"),
        Some("https://examples.com:4317"),
        Some("12345"),
        HttpBinary,
        Some("https://examples.com:4317"),
        Some(Duration::from_millis(12345))
    )]
    fn test_infer_export_config(
        #[case] traces_protocol: Option<&str>,
        #[case] traces_endpoint: Option<&str>,
        #[case] traces_timeout: Option<&str>,
        #[case] expected_protocol: Protocol,
        #[case] expected_endpoint: Option<&str>,
        #[case] expected_timeout: Option<Duration>,
    ) {
        let ExportConfig {
            protocol,
            endpoint,
            timeout,
        } = infer_export_config(traces_protocol, traces_endpoint, traces_timeout)
            .unwrap();

        assert!(protocol == expected_protocol);
        assert!(endpoint.as_deref() == expected_endpoint);
        assert!(timeout == expected_timeout);
    }

    #[rstest]
    #[case(Some("tonic"), None, r#"unsupported protocol "tonic" form env"#)]
    #[case(
        Some("http/protobuf"),
        Some("-1"),
        r#"invalid timeout "-1" form env: invalid digit found in string"#
    )]
    fn test_infer_export_config_error(
        #[case] traces_protocol: Option<&str>,
        #[case] traces_timeout: Option<&str>,
        #[case] expected_error: &str,
    ) {
        let result = infer_export_config(traces_protocol, None, traces_timeout);

        let_assert!(Err(err) = result);

        assert!(format!("{}", err) == expected_error);
    }
}
