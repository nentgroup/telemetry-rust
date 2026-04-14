use std::error::Error;
use std::net::SocketAddr;

use http::{HeaderMap, Method};
use opentelemetry::{
    global,
    trace::{SpanKind, Status, TraceContextExt, Tracer},
};
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::http::http_flavor;
use url::Url;

use crate::{Context, KeyValue, OpenTelemetrySpanExt, semconv};

const OTHER_HTTP_METHOD: &str = "_OTHER";
const HTTP_SPAN_NAME: &str = "HTTP";

/// An active HTTP client span with its associated [`Context`].
///
/// The context carries the span so that callers can inject trace propagation
/// headers (e.g. `traceparent`) that reference *this* client span rather than
/// its parent.
pub(crate) struct HttpClientSpan {
    context: Context,
}

pub(crate) struct HttpClientSpanBuilder {
    attributes: Vec<KeyValue>,
    span_name: String,
}

impl HttpClientSpanBuilder {
    pub(crate) fn from_parts(method: &Method, headers: &HeaderMap, url: &Url) -> Self {
        let (semantic_method, original_method) = semantic_method(method);
        let span_name = if semantic_method == OTHER_HTTP_METHOD {
            HTTP_SPAN_NAME
        } else {
            semantic_method
        };

        let mut attributes = vec![
            KeyValue::new(semconv::HTTP_REQUEST_METHOD, semantic_method.to_owned()),
            KeyValue::new(semconv::URL_FULL, url.as_str().to_owned()),
            KeyValue::new(semconv::URL_PATH, url.path().to_owned()),
        ];

        if let Some(host) = url.host_str() {
            attributes.push(KeyValue::new(semconv::SERVER_ADDRESS, host.to_owned()));
        }

        if !url.scheme().is_empty() {
            let scheme = url.scheme();
            attributes.push(KeyValue::new(semconv::URL_SCHEME, scheme.to_owned()));
        }

        if let Some(method) = original_method {
            attributes.push(KeyValue::new(
                semconv::HTTP_REQUEST_METHOD_ORIGINAL,
                method.to_owned(),
            ));
        }

        if let Some(port) = url.port_or_known_default() {
            attributes.push(KeyValue::new(semconv::SERVER_PORT, i64::from(port)));
        }

        if let Some(query) = url.query() {
            attributes.push(KeyValue::new(semconv::URL_QUERY, query.to_owned()));
        }

        if let Some(ua) = headers.get("user-agent").and_then(|v| v.to_str().ok()) {
            attributes.push(KeyValue::new(semconv::USER_AGENT_ORIGINAL, ua.to_owned()));
        }

        Self {
            attributes,
            span_name: span_name.to_owned(),
        }
    }

    pub(crate) fn start(self) -> HttpClientSpan {
        self.start_with_context(&Span::current().context())
    }

    pub(crate) fn start_with_context(self, parent_cx: &Context) -> HttpClientSpan {
        let tracer = global::tracer("http_client");
        let span = tracer
            .span_builder(self.span_name)
            .with_kind(SpanKind::Client)
            .with_attributes(self.attributes)
            .start_with_context(&tracer, parent_cx);

        HttpClientSpan {
            context: parent_cx.clone().with_span(span),
        }
    }
}

impl HttpClientSpan {
    /// Returns the context carrying this span, suitable for trace propagation
    /// injection.
    pub(crate) fn context(&self) -> &Context {
        &self.context
    }

    pub(crate) fn end_response(
        self,
        status: http::StatusCode,
        version: http::Version,
        remote_addr: Option<SocketAddr>,
    ) {
        let span = self.context.span();
        span.set_attribute(KeyValue::new(
            semconv::HTTP_RESPONSE_STATUS_CODE,
            i64::from(status.as_u16()),
        ));
        span.set_attribute(KeyValue::new(
            semconv::NETWORK_PROTOCOL_VERSION,
            http_flavor(version).into_owned(),
        ));

        if let Some(addr) = remote_addr {
            span.set_attribute(KeyValue::new(
                semconv::NETWORK_PEER_ADDRESS,
                addr.ip().to_string(),
            ));
            span.set_attribute(KeyValue::new(
                semconv::NETWORK_PEER_PORT,
                i64::from(addr.port()),
            ));
        }

        if status.is_client_error() || status.is_server_error() {
            span.set_attribute(KeyValue::new(
                semconv::ERROR_TYPE,
                status.as_u16().to_string(),
            ));
            span.set_status(Status::error(""));
        }

        span.end();
    }

    pub(crate) fn end_error<E>(self, error_type: &str, error: &E)
    where
        E: Error + 'static,
    {
        let span = self.context.span();
        span.set_attribute(KeyValue::new(semconv::ERROR_TYPE, error_type.to_owned()));
        span.record_error(error);
        span.set_status(Status::error(error.to_string()));
        span.end();
    }
}

fn semantic_method(method: &Method) -> (&str, Option<&str>) {
    match method.as_str() {
        "CONNECT" | "DELETE" | "GET" | "HEAD" | "OPTIONS" | "PATCH" | "POST" | "PUT"
        | "TRACE" => (method.as_str(), None),
        other => (OTHER_HTTP_METHOD, Some(other)),
    }
}
