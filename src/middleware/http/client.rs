use std::error::Error;
use std::net::SocketAddr;

use http::{Request, Uri};
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span as _, SpanBuilder, SpanKind, Status, Tracer},
};
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::http::http_flavor;

use crate::{Context, KeyValue, OpenTelemetrySpanExt, semconv};

pub(crate) struct HttpClientSpan {
    span: BoxedSpan,
}

pub(crate) struct HttpClientSpanBuilder<'a> {
    inner: SpanBuilder,
    tracer: BoxedTracer,
    context: Option<&'a Context>,
}

impl<'a> HttpClientSpanBuilder<'a> {
    pub(crate) fn from_request<B>(request: &Request<B>) -> Self {
        let tracer = global::tracer("http_client");
        let method = request.method().to_string();
        let mut attributes =
            vec![KeyValue::new(semconv::HTTP_REQUEST_METHOD, method.clone())];

        if let Some(scheme) = request.uri().scheme_str() {
            attributes.push(KeyValue::new(semconv::URL_SCHEME, scheme.to_owned()));
        }
        if let Some(address) = server_address(request.uri()) {
            attributes.push(KeyValue::new(semconv::SERVER_ADDRESS, address));
        }
        if let Some(port) = request.uri().port_u16() {
            attributes.push(KeyValue::new(semconv::SERVER_PORT, i64::from(port)));
        }
        if !request.uri().path().is_empty() {
            attributes.push(KeyValue::new(
                semconv::URL_PATH,
                request.uri().path().to_owned(),
            ));
        }

        let full_url = request.uri().to_string();
        if !full_url.is_empty() {
            attributes.push(KeyValue::new(semconv::URL_FULL, full_url));
        }

        if let Some(query) = request.uri().query() {
            attributes.push(KeyValue::new(semconv::URL_QUERY, query.to_owned()));
        }

        if let Some(ua) = request
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
        {
            attributes.push(KeyValue::new(
                semconv::USER_AGENT_ORIGINAL,
                ua.to_owned(),
            ));
        }

        let inner = tracer
            .span_builder(method)
            .with_kind(SpanKind::Client)
            .with_attributes(attributes);

        Self {
            inner,
            tracer,
            context: None,
        }
    }

    #[inline]
    pub(crate) fn set_context(mut self, context: Option<&'a Context>) -> Self {
        self.context = context;
        self
    }

    #[inline(always)]
    fn start_with_context(self, parent_context: &Context) -> HttpClientSpan {
        HttpClientSpan {
            span: self.inner.start_with_context(&self.tracer, parent_context),
        }
    }

    #[inline]
    pub(crate) fn start(self) -> HttpClientSpan {
        match self.context {
            Some(context) => self.start_with_context(context),
            None => self.start_with_context(&Span::current().context()),
        }
    }
}

impl HttpClientSpan {
    pub(crate) fn end_response_parts(
        self,
        status: http::StatusCode,
        version: http::Version,
        remote_addr: Option<SocketAddr>,
    ) {
        let mut span = self.span;
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

        if status.is_server_error() {
            span.set_status(Status::error(status.to_string()));
        }
    }

    pub(crate) fn end_error<E>(self, error: &E)
    where
        E: Error + 'static,
    {
        let mut span = self.span;
        span.set_attribute(KeyValue::new(
            semconv::ERROR_TYPE,
            std::any::type_name::<E>(),
        ));
        span.record_error(error);
        span.set_status(Status::error(error.to_string()));
    }
}

fn server_address(uri: &Uri) -> Option<String> {
    uri.host().map(str::to_owned)
}
