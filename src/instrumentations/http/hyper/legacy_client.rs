//! Async hyper-util legacy client instrumentation helpers.
//!
//! This module instruments `hyper_util::client::legacy::Client` requests
//! with OpenTelemetry client spans and trace-context propagation.
//!
//! # Example
//!
//! ```no_run
//! use bytes::Bytes;
//! use http_body_util::Empty;
//! use hyper::{Request, header::USER_AGENT};
//! use hyper_util::rt::TokioExecutor;
//! use telemetry_rust::instrumentations::http::hyper::HyperLegacyClientInstrument;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
//!     .build_http::<Empty<Bytes>>()
//!     .instrument();
//!
//! let response = client
//!     .request(
//!         Request::builder()
//!             .uri("http://127.0.0.1:3000/health")
//!             .header(USER_AGENT, "telemetry-rust")
//!             .body(Empty::<Bytes>::new())?,
//!     )
//!     .await?;
//! # let _ = response;
//! # Ok(())
//! # }
//! ```

use std::error::Error as StdError;
use std::future::Future;

use hyper::{
    Request, Response,
    body::{Body, Incoming},
};
use hyper_util::client::legacy;

use crate::{
    Context,
    future::InstrumentedFuture,
    http,
    instrumentations::http::client::{HttpClientSpanBuilder, HttpError},
};

impl HttpError for legacy::Error {
    fn error_type(&self) -> &'static str {
        if self.is_connect() {
            "connect"
        } else if let Some(hyper_error) = self
            .source()
            .and_then(|source| source.downcast_ref::<hyper::Error>())
        {
            hyper_error.error_type()
        } else {
            "_OTHER"
        }
    }
}

/// A trait for creating instrumented hyper-util legacy clients with
/// OpenTelemetry tracing.
pub trait HyperLegacyClientInstrument
where
    Self: Sized,
{
    /// The legacy client's connector type.
    type Connector;

    /// The legacy client's request body type.
    type Body;

    /// Wraps this client in an [`InstrumentedLegacyClient`] that can be reused
    /// to send traced requests.
    fn instrument(self) -> InstrumentedLegacyClient<Self::Connector, Self::Body>;
}

impl<C, B> HyperLegacyClientInstrument for legacy::Client<C, B> {
    type Connector = C;
    type Body = B;

    fn instrument(self) -> InstrumentedLegacyClient<C, B> {
        InstrumentedLegacyClient::new(self)
    }
}

/// A reusable wrapper around `hyper_util::client::legacy::Client` that records
/// client spans for each request sent through the client.
#[must_use = "Client does nothing until you call request()"]
pub struct InstrumentedLegacyClient<C, B> {
    inner: legacy::Client<C, B>,
    context: Option<Context>,
}

impl<C, B> InstrumentedLegacyClient<C, B> {
    /// Creates a new instrumented legacy hyper client.
    pub fn new(inner: legacy::Client<C, B>) -> Self {
        Self {
            inner,
            context: None,
        }
    }

    /// Sets the OpenTelemetry context for requests sent by this wrapper.
    pub fn context(mut self, context: &Context) -> Self {
        self.context = Some(context.clone());
        self
    }

    /// Sets the optional OpenTelemetry context for requests sent by this wrapper.
    pub fn set_context(mut self, context: Option<&Context>) -> Self {
        self.context = context.cloned();
        self
    }

    /// Returns the wrapped legacy hyper client.
    pub fn into_inner(self) -> legacy::Client<C, B> {
        self.inner
    }
}

impl<C, B> Clone for InstrumentedLegacyClient<C, B>
where
    legacy::Client<C, B>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            context: self.context.clone(),
        }
    }
}

impl<C, B> InstrumentedLegacyClient<C, B>
where
    C: legacy::connect::Connect + Clone + Send + Sync + 'static,
    B: Body + Send + Unpin + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn StdError + Send + Sync>>,
{
    /// Sends a constructed request and records an outbound HTTP client span
    /// around it.
    pub fn request(
        &self,
        mut request: Request<B>,
    ) -> impl Future<Output = Result<Response<Incoming>, legacy::Error>> + '_ {
        let span_builder = HttpClientSpanBuilder::from_http_request(&request);
        let span = span_builder.start(self.context.as_ref());

        http::inject_context_on_context(span.context(), request.headers_mut());

        let future = self.inner.request(request);
        InstrumentedFuture::new(future, span)
    }

    /// Sends a GET request to the supplied URI and records an outbound HTTP
    /// client span around it.
    pub fn get(
        &self,
        uri: hyper::Uri,
    ) -> impl Future<Output = Result<Response<Incoming>, legacy::Error>> + '_
    where
        B: Default,
    {
        let mut request = Request::new(B::default());
        *request.uri_mut() = uri;
        self.request(request)
    }
}

#[cfg(test)]
mod tests {
    use super::HyperLegacyClientInstrument;
    use crate::{Context, instrumentations::http::test_utils::*, semconv};
    use assert2::assert;
    use axum::http::StatusCode;
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::{Request, header::USER_AGENT};
    use hyper_util::rt::TokioExecutor;
    use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer, TracerProvider};
    use serial_test::serial;
    use tokio::net::TcpListener;
    use tracing_subscriber::{Registry, layer::SubscriberExt};

    #[tokio::test]
    #[serial]
    async fn instruments_successful_legacy_client_requests() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build_http::<Empty<Bytes>>()
            .instrument();
        let request_url = format!("{}/ok?ready=true", server.base_url);
        let response = client
            .request(
                Request::builder()
                    .uri(&request_url)
                    .header(USER_AGENT, "telemetry-rust-tests")
                    .body(Empty::<Bytes>::new())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK);

        let spans = force_flush_and_get_spans(&telemetry);
        let span = find_span(&spans, "GET");
        let traceparent = server.state.traceparent_for("/ok").unwrap();
        let (trace_id, span_id) = traceparent_ids(&traceparent);

        assert!(span.span_kind == SpanKind::Client);
        assert!(span.span_context.trace_id().to_string() == trace_id);
        assert!(span.span_context.span_id().to_string() == span_id);
        assert!(matches!(span.status, opentelemetry::trace::Status::Unset));
        assert!(string_attr(span, semconv::HTTP_REQUEST_METHOD) == Some("GET"));
        assert!(string_attr(span, semconv::URL_SCHEME) == Some("http"));
        assert!(string_attr(span, semconv::SERVER_ADDRESS) == Some("127.0.0.1"));
        assert!(
            i64_attr(span, semconv::SERVER_PORT) == Some(i64::from(server.addr.port()))
        );
        assert!(string_attr(span, semconv::URL_PATH) == Some("/ok"));
        assert!(string_attr(span, semconv::URL_QUERY) == Some("ready=true"));
        assert!(
            string_attr(span, semconv::USER_AGENT_ORIGINAL)
                == Some("telemetry-rust-tests")
        );
        assert!(string_attr(span, semconv::URL_FULL) == Some(request_url.as_str()));
        assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE) == Some(200));
        assert!(string_attr(span, semconv::NETWORK_PROTOCOL_VERSION).is_some());
    }

    #[tokio::test]
    #[serial]
    async fn legacy_client_uses_explicit_parent_context_when_provided() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build_http::<Empty<Bytes>>();
        let tracer = telemetry.provider.tracer("hyper-legacy-tests");
        let explicit_parent = tracer.start("explicit-parent");
        let explicit_parent_span_id = explicit_parent.span_context().span_id();
        let explicit_parent_cx = Context::current_with_span(explicit_parent);
        let tracing_tracer = telemetry.provider.tracer("tracing-tests");
        let subscriber = Registry::default()
            .with(tracing_opentelemetry::layer().with_tracer(tracing_tracer));
        let guard = tracing::subscriber::set_default(subscriber);
        let current_parent = tracing::info_span!("current-parent");

        tracing::Instrument::instrument(
            async {
                let client = client.instrument().context(&explicit_parent_cx);
                client
                    .request(
                        Request::builder()
                            .uri(format!("{}/ok", server.base_url))
                            .body(Empty::<Bytes>::new())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
            },
            current_parent,
        )
        .await;

        drop(guard);
        explicit_parent_cx.span().end();

        let spans = force_flush_and_get_spans(&telemetry);
        let hyper_span = find_span(&spans, "GET");
        let current_span = find_span(&spans, "current-parent");

        assert!(hyper_span.parent_span_id == explicit_parent_span_id);
        assert!(hyper_span.parent_span_id != current_span.span_context.span_id());
    }

    #[tokio::test]
    #[serial]
    async fn legacy_client_marks_error_responses_as_errors() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build_http::<Empty<Bytes>>()
            .instrument();
        let response = client
            .request(
                Request::builder()
                    .uri(format!("{}/not-found", server.base_url))
                    .body(Empty::<Bytes>::new())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status() == StatusCode::NOT_FOUND);

        let spans = force_flush_and_get_spans(&telemetry);
        let span = find_span(&spans, "GET");

        assert!(matches!(
            span.status,
            opentelemetry::trace::Status::Error { .. }
        ));
        assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE) == Some(404));
        assert!(string_attr(span, semconv::ERROR_TYPE) == Some("404"));
    }

    #[tokio::test]
    #[serial]
    async fn legacy_client_marks_transport_errors_as_errors() {
        let telemetry = configure_test_tracing();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
            .build_http::<Empty<Bytes>>()
            .instrument();
        let result = client
            .request(
                Request::builder()
                    .uri(format!("http://{addr}/unreachable"))
                    .body(Empty::<Bytes>::new())
                    .unwrap(),
            )
            .await;

        assert!(result.is_err());

        let spans = force_flush_and_get_spans(&telemetry);
        let span = find_span(&spans, "GET");

        assert!(matches!(
            span.status,
            opentelemetry::trace::Status::Error { .. }
        ));
        assert!(string_attr(span, semconv::ERROR_TYPE) == Some("connect"));
    }
}
