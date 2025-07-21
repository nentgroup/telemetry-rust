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

use http_body_util::BodyExt;
use hyper::{HeaderMap, Result, header::HeaderValue};

pub use opentelemetry_api::trace::{SpanId, TraceId};
use rand::Rng;

type Response = hyper::Response<hyper::body::Incoming>;

/// HTTP response wrapper that includes OpenTelemetry trace information.
///
/// This struct wraps an HTTP response and provides easy access to the associated
/// trace ID and span ID for testing and debugging purposes. It's particularly
/// useful in integration tests where you need to verify trace propagation.
#[derive(Debug)]
pub struct TracedResponse {
    resp: Response,
    /// The OpenTelemetry trace ID associated with this response
    pub trace_id: TraceId,
    /// The OpenTelemetry span ID associated with this response
    pub span_id: SpanId,
}

impl TracedResponse {
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
    pub fn new(resp: Response, traceparent: Traceparent) -> Self {
        Self {
            resp,
            trace_id: traceparent.trace_id,
            span_id: traceparent.span_id,
        }
    }

    /// Consumes the response and returns the body as bytes.
    ///
    /// # Returns
    ///
    /// A future that resolves to the response body as [`bytes::Bytes`]
    ///
    /// # Errors
    ///
    /// Returns an error if the response body cannot be read
    pub async fn into_bytes(self) -> Result<bytes::Bytes> {
        Ok(self.resp.into_body().collect().await?.to_bytes())
    }
}

impl std::ops::Deref for TracedResponse {
    type Target = Response;

    fn deref(&self) -> &Self::Target {
        &self.resp
    }
}

impl std::ops::DerefMut for TracedResponse {
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
