// Originally retired from davidB/tracing-opentelemetry-instrumentation-sdk
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use opentelemetry::Context;
use opentelemetry::propagation::{Extractor, Injector};
use tracing_opentelemetry_instrumentation_sdk as otel;

/// HTTP header injector for OpenTelemetry context propagation.
///
/// This struct implements the [`Injector`] trait to inject OpenTelemetry trace context
/// into HTTP headers. It wraps an HTTP header map and provides the necessary interface
/// for propagators to inject trace context information.
///
/// # Usage
///
/// Typically used internally by propagation functions, but can be used directly:
///
/// ```rust
/// use telemetry_rust::http::HeaderInjector;
/// use http::HeaderMap;
///
/// let mut headers = HeaderMap::new();
/// let mut injector = HeaderInjector(&mut headers);
/// // Use with OpenTelemetry propagators...
/// ```
pub struct HeaderInjector<'a>(pub &'a mut http::HeaderMap);

impl Injector for HeaderInjector<'_> {
    /// Set a key and value in the `HeaderMap`. Does nothing if the key or value are not valid inputs.
    fn set(&mut self, key: &str, value: String) {
        if let Ok(name) = http::header::HeaderName::from_bytes(key.as_bytes()) {
            if let Ok(val) = http::header::HeaderValue::from_str(&value) {
                self.0.insert(name, val);
            }
        }
    }
}

/// HTTP header extractor for OpenTelemetry context propagation.
///
/// This struct implements the [`Extractor`] trait to extract OpenTelemetry trace context
/// from HTTP headers. It wraps an HTTP header map and provides the necessary interface
/// for propagators to extract trace context information from incoming requests.
///
/// # Usage
///
/// Typically used internally by propagation functions, but can be used directly:
///
/// ```rust
/// use telemetry_rust::http::HeaderExtractor;
/// use http::HeaderMap;
///
/// let headers = HeaderMap::new();
/// let extractor = HeaderExtractor(&headers);
/// // Use with OpenTelemetry propagators...
/// ```
pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

impl Extractor for HeaderExtractor<'_> {
    /// Get a value for a key from the `HeaderMap`. If the value is not valid ASCII, returns None.
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    /// Collect all the keys from the `HeaderMap`.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(http::HeaderName::as_str)
            .collect::<Vec<_>>()
    }
}

/// Injects OpenTelemetry context from a specific context into HTTP headers.
///
/// This function takes an existing OpenTelemetry context and injects its trace
/// information into the provided HTTP headers using the globally configured
/// text map propagator.
///
/// # Arguments
///
/// - `context`: The OpenTelemetry context to inject
/// - `headers`: Mutable reference to HTTP headers where context will be injected
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::http::inject_context_on_context;
/// use opentelemetry::Context;
/// use http::HeaderMap;
///
/// let context = Context::current();
/// let mut headers = HeaderMap::new();
/// inject_context_on_context(&context, &mut headers);
/// ```
pub fn inject_context_on_context(context: &Context, headers: &mut http::HeaderMap) {
    let mut injector = HeaderInjector(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(context, &mut injector);
    });
}

/// Injects the current OpenTelemetry context into HTTP headers.
///
/// This convenience function automatically finds the current OpenTelemetry context
/// and injects its trace information into the provided HTTP headers using the
/// globally configured text map propagator.
///
/// # Arguments
///
/// - `headers`: Mutable reference to HTTP headers where context will be injected
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::http::inject_context;
/// use http::HeaderMap;
///
/// let mut headers = HeaderMap::new();
/// inject_context(&mut headers);
/// ```
pub fn inject_context(headers: &mut http::HeaderMap) {
    let mut injector = HeaderInjector(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&otel::find_current_context(), &mut injector);
    });
}

/// Extracts OpenTelemetry context from HTTP headers.
///
/// This function extracts trace context information from HTTP headers using the
/// globally configured text map propagator. If no trace context is found in the
/// headers, it returns an unsampled context.
///
/// # Arguments
///
/// - `headers`: Reference to HTTP headers to extract context from
///
/// # Returns
///
/// An OpenTelemetry [`Context`] containing the extracted trace information, or
/// an unsampled context if no trace data was found.
///
/// # Examples
///
/// ```rust
/// use telemetry_rust::http::extract_context;
/// use http::HeaderMap;
///
/// let headers = HeaderMap::new();
/// let context = extract_context(&headers);
/// ```
// If remote request has no span data the propagator defaults to an unsampled context
#[must_use]
pub fn extract_context(headers: &http::HeaderMap) -> Context {
    let extractor = HeaderExtractor(headers);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&extractor)
    })
}
