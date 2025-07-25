//! Testing utilities for OpenTelemetry integration testing and validation.
//!
//! This module provides utilities for testing OpenTelemetry instrumentation,
//! including trace header manipulation, Jaeger trace data structures for
//! validation, and HTTP response testing helpers.
//!
//! The module contains tools for:
//! - Parsing and generating trace headers (traceparent, tracestate)
//! - Deserializing Jaeger trace data for validation
//! - Testing HTTP responses with trace context
//! - Generating test trace IDs and span IDs

pub mod jaegar;

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{
    HeaderMap, Response,
    body::{Body, Incoming},
    header::HeaderValue,
};

pub use opentelemetry_api::trace::{SpanId, TraceId};
use rand::Rng;

/// HTTP response wrapper that includes OpenTelemetry trace information.
///
/// This struct wraps an HTTP response and provides easy access to the associated
/// trace ID and span ID for testing and debugging purposes. It's particularly
/// useful in integration tests where you need to verify trace propagation.
///
/// # Example
///
/// ```rust
/// use telemetry_rust::test::{TracedResponse, Traceparent};
///
/// async fn send_traced_request() -> TracedResponse<&'static str> {
///     let traceparent = Traceparent::generate();
///
///     // Send request and get response
///     let resp = hyper::Response::new("Hello world!");
///
///     TracedResponse::new(resp, traceparent)
/// }
/// ```
#[derive(Debug)]
pub struct TracedResponse<T = Incoming> {
    resp: Response<T>,
    /// The OpenTelemetry trace ID associated with this response
    pub trace_id: TraceId,
    /// The OpenTelemetry span ID associated with this response
    pub span_id: SpanId,
}

impl<T> TracedResponse<T> {
    /// Creates a new traced response from an HTTP response and trace parent information.
    ///
    /// # Arguments
    ///
    /// - `resp`: The HTTP response to wrap
    /// - `traceparent`: The trace parent containing trace and span IDs
    ///
    /// # Returns
    ///
    /// A new [`TracedResponse`] instance
    pub fn new(resp: Response<T>, traceparent: Traceparent) -> Self {
        Self {
            resp,
            trace_id: traceparent.trace_id,
            span_id: traceparent.span_id,
        }
    }

    /// Consumes the traced response and returns the inner HTTP response.
    ///
    /// # Returns
    ///
    /// The wrapped [`hyper::Response`] instance.
    pub async fn into_inner(self) -> Response<T> {
        self.resp
    }
}

impl<E, T: Body<Data = Bytes, Error = E>> TracedResponse<T> {
    /// Consumes the response and returns the body as bytes.
    ///
    /// # Returns
    ///
    /// A future that resolves to the response body as [`bytes::Bytes`]
    ///
    /// # Errors
    ///
    /// Returns an error if the response body cannot be read
    pub async fn into_bytes(self) -> Result<Bytes, E> {
        Ok(self.resp.into_body().collect().await?.to_bytes())
    }
}

impl<T> std::ops::Deref for TracedResponse<T> {
    type Target = Response<T>;

    fn deref(&self) -> &Self::Target {
        &self.resp
    }
}

impl<T> std::ops::DerefMut for TracedResponse<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resp
    }
}

/// Enumeration of supported tracing header formats for testing.
///
/// This enum represents the different trace context propagation formats
/// that can be used in HTTP headers for testing distributed tracing scenarios.
pub enum TracingHeaderKind {
    /// W3C Trace Context format using the `traceparent` header
    Traceparent,
    /// B3 single header format using the `b3` header
    B3Single,
    /// B3 multiple header format using separate `X-B3-*` headers
    B3Multi,
}

/// A container for OpenTelemetry trace parent information used in testing.
///
/// This struct holds a trace ID and span ID pair that represents a trace context
/// relationship. It's commonly used for generating test trace headers and
/// validating trace propagation in integration tests.
///
/// # Example
///
/// ```rust
/// use telemetry_rust::test::{Traceparent, TracingHeaderKind};
///
/// // Generate a new trace parent for testing
/// let traceparent = Traceparent::generate();
///
/// // Create HTTP headers for trace propagation
/// let headers = traceparent.get_headers(TracingHeaderKind::Traceparent);
///
/// // Use in HTTP request testing
/// let mut req = hyper::Request::new(());
/// for (key, value) in headers {
///     if let Some(header_name) = key {
///         req.headers_mut().insert(header_name, value);
///     }
/// }
/// ```
pub struct Traceparent {
    /// The OpenTelemetry trace ID
    pub trace_id: TraceId,
    /// The OpenTelemetry span ID
    pub span_id: SpanId,
}

impl Traceparent {
    /// Generates a new random trace parent with random trace and span IDs.
    ///
    /// This method creates a new trace parent with randomly generated IDs,
    /// useful for creating test scenarios with unique trace contexts.
    ///
    /// # Returns
    ///
    /// A new [`Traceparent`] with randomly generated trace and span IDs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use telemetry_rust::test::Traceparent;
    ///
    /// let traceparent = Traceparent::generate();
    /// println!("Trace ID: {}", traceparent.trace_id);
    /// ```
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let trace_id = TraceId::from_u128(rng.random());
        let span_id = SpanId::from_u64(rng.random());
        Self { trace_id, span_id }
    }

    /// Generates HTTP headers containing trace context in the specified format.
    ///
    /// This method creates HTTP headers with trace context information formatted
    /// according to the specified tracing header kind. This is useful for testing
    /// trace propagation with different header formats.
    ///
    /// # Arguments
    ///
    /// - `kind`: The format to use for the trace headers
    ///
    /// # Returns
    ///
    /// A [`HeaderMap`] containing the appropriately formatted trace headers
    ///
    /// # Examples
    ///
    /// ```rust
    /// use telemetry_rust::test::{Traceparent, TracingHeaderKind};
    ///
    /// let traceparent = Traceparent::generate();
    /// let headers = traceparent.get_headers(TracingHeaderKind::Traceparent);
    /// ```
    ///
    /// # Panics
    ///
    /// This function will panic if the trace ID or span ID cannot be converted
    /// to a valid HTTP header value format.
    pub fn get_headers(&self, kind: TracingHeaderKind) -> HeaderMap {
        let mut map = HeaderMap::new();

        match kind {
            TracingHeaderKind::Traceparent => {
                let value = format!("00-{}-{}-01", self.trace_id, self.span_id);
                map.append("traceparent", HeaderValue::from_str(&value).unwrap());
            }
            TracingHeaderKind::B3Single => {
                let value = format!("{}-{}-1", self.trace_id, self.span_id);
                map.append("b3", HeaderValue::from_str(&value).unwrap());
            }
            TracingHeaderKind::B3Multi => {
                map.append(
                    "X-B3-TraceId",
                    HeaderValue::from_str(&self.trace_id.to_string()).unwrap(),
                );
                map.append(
                    "X-B3-SpanId",
                    HeaderValue::from_str(&self.span_id.to_string()).unwrap(),
                );
                map.append("X-B3-Sampled", HeaderValue::from_str("1").unwrap());
            }
        }

        map
    }
}
