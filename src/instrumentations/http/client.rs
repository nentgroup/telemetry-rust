use std::{error::Error, net::SocketAddr};

use http::{HeaderMap, Method};
use opentelemetry::{
    global,
    trace::{SpanKind, Status, TraceContextExt, Tracer},
};
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::http::http_flavor;

use crate::{
    Context, KeyValue, OpenTelemetrySpanExt, Value, future::InstrumentedFutureContext,
    semconv, util::as_attribute,
};

const OTHER_HTTP_METHOD: &str = "_OTHER";
const HTTP_SPAN_NAME: &str = "HTTP";

pub(crate) trait UrlParts {
    fn full_url(&self) -> Option<impl Into<Value>>;
    fn path(&self) -> Option<impl Into<Value>>;
    fn host(&self) -> Option<impl Into<Value>>;
    fn scheme(&self) -> Option<impl Into<Value>>;
    fn port(&self) -> Option<impl Into<Value>>;
    fn query(&self) -> Option<impl Into<Value>>;
}

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
    span_name: &'static str,
}

impl HttpClientSpanBuilder {
    pub(crate) fn from_parts(
        method: &Method,
        headers: &HeaderMap,
        url: &impl UrlParts,
    ) -> Self {
        let semantic_method = semantic_method(method);
        let (span_name, original_method) = match semantic_method {
            OTHER_HTTP_METHOD => (HTTP_SPAN_NAME, Some(method.to_string())),
            verb => (verb, None),
        };

        let user_agent = headers
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);

        let attributes = [
            Some(KeyValue::new(semconv::HTTP_REQUEST_METHOD, semantic_method)),
            as_attribute(semconv::URL_FULL, url.full_url()),
            as_attribute(semconv::URL_PATH, url.path()),
            as_attribute(semconv::SERVER_ADDRESS, url.host()),
            as_attribute(semconv::URL_SCHEME, url.scheme()),
            as_attribute(semconv::SERVER_PORT, url.port()),
            as_attribute(semconv::URL_QUERY, url.query()),
            as_attribute(semconv::HTTP_REQUEST_METHOD_ORIGINAL, original_method),
            as_attribute(semconv::USER_AGENT_ORIGINAL, user_agent),
        ];

        Self {
            attributes: attributes.into_iter().flatten().collect(),
            span_name,
        }
    }

    pub(crate) fn start(self, parent_cx: Option<&Context>) -> HttpClientSpan {
        match parent_cx {
            Some(cx) => self.start_with_context(cx),
            None => self.start_with_context(&Span::current().context()),
        }
    }

    pub(crate) fn start_with_context(self, parent_cx: &Context) -> HttpClientSpan {
        let tracer = global::tracer("http_client");
        let span = tracer
            .span_builder(self.span_name)
            .with_kind(SpanKind::Client)
            .with_attributes(self.attributes)
            .start_with_context(&tracer, parent_cx);

        HttpClientSpan {
            context: parent_cx.with_span(span),
        }
    }
}

impl HttpClientSpan {
    /// Returns the context carrying this span, suitable for trace propagation
    /// injection.
    pub(crate) fn context(&self) -> &Context {
        &self.context
    }

    pub(crate) fn end_response<R: HttpResponse>(self, response: &R) {
        let status = response.status();
        let span = self.context.span();
        span.set_attribute(KeyValue::new(
            semconv::HTTP_RESPONSE_STATUS_CODE,
            i64::from(status.as_u16()),
        ));
        span.set_attribute(KeyValue::new(
            semconv::NETWORK_PROTOCOL_VERSION,
            http_flavor(response.version()).into_owned(),
        ));

        if let Some(addr) = response.remote_addr() {
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

    pub(crate) fn end_error<E: HttpError>(self, error: &E) {
        let span = self.context.span();
        span.set_attribute(KeyValue::new(semconv::ERROR_TYPE, error.error_type()));
        span.record_error(error);
        span.set_status(Status::error(error.to_string()));
        span.end();
    }
}

fn semantic_method(method: &Method) -> &'static str {
    match method.as_str() {
        "CONNECT" => "CONNECT",
        "DELETE" => "DELETE",
        "GET" => "GET",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        "PATCH" => "PATCH",
        "POST" => "POST",
        "PUT" => "PUT",
        "TRACE" => "TRACE",
        _ => OTHER_HTTP_METHOD,
    }
}

pub(crate) trait HttpResponse {
    fn status(&self) -> http::StatusCode;
    fn version(&self) -> http::Version;
    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }
}

impl<T> HttpResponse for http::Response<T> {
    fn status(&self) -> http::StatusCode {
        self.status()
    }

    fn version(&self) -> http::Version {
        self.version()
    }
}

pub(crate) trait HttpError: Error + 'static {
    fn error_type(&self) -> &'static str;
}

impl<R, E> InstrumentedFutureContext<Result<R, E>> for HttpClientSpan
where
    R: HttpResponse,
    E: HttpError,
{
    fn on_result(self, result: &Result<R, E>) {
        match &result {
            Ok(response) => self.end_response(response),
            Err(error) => self.end_error(error),
        }
    }
}
