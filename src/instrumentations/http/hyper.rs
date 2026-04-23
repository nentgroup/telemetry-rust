//! Async hyper connection instrumentation helpers.
//!
//! This module instruments `hyper::client::conn::http1::SendRequest` and
//! `hyper::client::conn::http2::SendRequest`.
//!
//! # Example
//!
//! ```no_run
//! use bytes::Bytes;
//! use http_body_util::Empty;
//! use hyper::{Request, header::HOST};
//! use hyper_util::rt::TokioIo;
//! use telemetry_rust::instrumentations::http::hyper::HyperSendRequestInstrument;
//! use tokio::net::TcpStream;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # #[cfg(feature = "hyper-http1")]
//! # {
//! let stream = TcpStream::connect("127.0.0.1:3000").await?;
//! let io = TokioIo::new(stream);
//! let (send_request, connection) = hyper::client::conn::http1::handshake(io).await?;
//! tokio::spawn(async move {
//!     let _ = connection.await;
//! });
//!
//! // Create an instrumented sender once and reuse it for requests on this connection.
//! let mut instrumented_sender = send_request.instrument();
//! let request = Request::builder()
//!     .uri("/health")
//!     .header(HOST, "127.0.0.1:3000")
//!     .body(Empty::<Bytes>::new())?;
//! let response = instrumented_sender.send_request(request).await?;
//! # let _ = response;
//! # }
//! # Ok(())
//! # }
//! ```

use std::{
    future::Future,
    task::{Context as TaskContext, Poll},
};

use hyper::{
    Request, Response,
    body::{Body, Incoming},
    client::conn,
};

use crate::{
    Context, Value, http,
    instrumentations::http::client::{HttpClientSpanBuilder, UrlParts},
};

impl UrlParts for hyper::Uri {
    fn full_url(&self) -> Option<impl Into<Value>> {
        Some(self.to_string())
    }

    fn path(&self) -> Option<impl Into<Value>> {
        Some(self.path().to_owned())
    }

    fn host(&self) -> Option<impl Into<Value>> {
        self.host().map(ToOwned::to_owned)
    }

    fn scheme(&self) -> Option<impl Into<Value>> {
        self.scheme_str().map(ToOwned::to_owned)
    }

    fn port(&self) -> Option<impl Into<Value>> {
        self.port_u16().map(|p| p as i64)
    }

    fn query(&self) -> Option<impl Into<Value>> {
        self.query().map(ToOwned::to_owned)
    }
}

impl HttpClientSpanBuilder {
    pub(crate) fn from_http_request<B>(request: &hyper::Request<B>) -> Self {
        Self::from_parts(request.method(), request.headers(), request.uri())
    }
}

/// A trait for creating instrumented hyper connection senders with OpenTelemetry tracing.
pub trait HyperSendRequestInstrument
where
    Self: Sized,
{
    /// Wraps this sender in an [`InstrumentedSendRequest`] that can be reused
    /// to send traced requests on the same connection.
    fn instrument(self) -> InstrumentedSendRequest<Self>;
}

#[cfg(feature = "hyper-http1")]
impl<B> HyperSendRequestInstrument for conn::http1::SendRequest<B> {
    fn instrument(self) -> InstrumentedSendRequest<Self> {
        InstrumentedSendRequest::new(self)
    }
}

#[cfg(feature = "hyper-http2")]
impl<B> HyperSendRequestInstrument for conn::http2::SendRequest<B> {
    fn instrument(self) -> InstrumentedSendRequest<Self> {
        InstrumentedSendRequest::new(self)
    }
}

/// A reusable wrapper around hyper `SendRequest` that records client spans for
/// each request sent through the connection.
#[must_use = "SendRequest does nothing until you call send_request()"]
pub struct InstrumentedSendRequest<S> {
    inner: S,
    context: Option<Context>,
}

impl<S> InstrumentedSendRequest<S> {
    /// Creates a new instrumented hyper sender.
    pub fn new(inner: S) -> Self {
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

    /// Returns the wrapped hyper sender.
    pub fn into_inner(self) -> S {
        self.inner
    }
}

impl<S> Clone for InstrumentedSendRequest<S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            context: self.context.clone(),
        }
    }
}

macro_rules! impl_instrumented_send_request {
    ($sender:ty) => {
        impl<B> InstrumentedSendRequest<$sender>
        where
            B: Body + 'static,
        {
            /// Polls until the underlying sender is ready to send a request.
            pub fn poll_ready(
                &mut self,
                cx: &mut TaskContext<'_>,
            ) -> Poll<hyper::Result<()>> {
                self.inner.poll_ready(cx)
            }

            /// Waits until the underlying sender is ready to send a request.
            pub async fn ready(&mut self) -> hyper::Result<()> {
                self.inner.ready().await
            }

            /// Returns whether the underlying sender appears ready.
            pub fn is_ready(&self) -> bool {
                self.inner.is_ready()
            }

            /// Returns whether the underlying connection has closed.
            pub fn is_closed(&self) -> bool {
                self.inner.is_closed()
            }

            /// Sends a request and records an outbound HTTP client span around it.
            pub fn send_request(
                &mut self,
                mut request: Request<B>,
            ) -> impl Future<Output = hyper::Result<Response<Incoming>>> + '_ {
                let span_builder = HttpClientSpanBuilder::from_http_request(&request);
                let span = match self.context.as_ref() {
                    Some(context) => span_builder.start_with_context(context),
                    None => span_builder.start(),
                };

                http::inject_context_on_context(span.context(), request.headers_mut());

                let inner = &mut self.inner;

                async move {
                    let result = inner.send_request(request).await;
                    match &result {
                        Ok(response) => {
                            span.end_response(response.status(), response.version(), None)
                        }
                        Err(error) => span.end_error(hyper_error_type(error), error),
                    }
                    result
                }
            }
        }
    };
}

#[cfg(feature = "hyper-http1")]
impl_instrumented_send_request!(conn::http1::SendRequest<B>);
#[cfg(feature = "hyper-http2")]
impl_instrumented_send_request!(conn::http2::SendRequest<B>);

fn hyper_error_type(error: &hyper::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_closed() {
        "closed"
    } else if error.is_canceled() {
        "canceled"
    } else if error.is_parse_status() {
        "parse_status"
    } else if error.is_parse() {
        "parse"
    } else if error.is_incomplete_message() {
        "incomplete_message"
    } else if error.is_body_write_aborted() {
        "body_write_aborted"
    } else if error.is_shutdown() {
        "shutdown"
    } else if error.is_user() {
        "user"
    } else {
        "_OTHER"
    }
}

#[cfg(test)]
mod tests {
    use super::HyperSendRequestInstrument;
    use crate::{Context, Value, semconv};
    use assert2::assert;
    use axum::{
        Router,
        extract::State,
        http::{HeaderMap, StatusCode},
        response::{IntoResponse, Redirect},
        routing::get,
    };
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::{
        Request,
        header::{HOST, USER_AGENT},
    };
    #[cfg(feature = "hyper-http2")]
    use hyper_util::rt::TokioExecutor;
    use hyper_util::rt::TokioIo;
    use opentelemetry::{
        global,
        trace::{Span as _, SpanKind, TraceContextExt, Tracer as _, TracerProvider as _},
    };
    use opentelemetry_sdk::{
        propagation::TraceContextPropagator,
        trace::{InMemorySpanExporter, SdkTracerProvider as TracerProvider},
    };
    use serial_test::serial;
    use std::sync::{Arc, Mutex};
    use tokio::{
        net::{TcpListener, TcpStream},
        task::JoinHandle,
    };
    use tracing_subscriber::{Registry, layer::SubscriberExt};

    #[derive(Clone, Default)]
    struct TestState {
        traceparents: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl TestState {
        fn record(&self, path: &str, headers: &HeaderMap) {
            if let Some(traceparent) = headers
                .get("traceparent")
                .and_then(|value| value.to_str().ok())
            {
                self.traceparents
                    .lock()
                    .unwrap()
                    .push((path.to_owned(), traceparent.to_owned()));
            }
        }

        fn traceparent_for(&self, path: &str) -> Option<String> {
            self.traceparents
                .lock()
                .unwrap()
                .iter()
                .rev()
                .find(|(recorded_path, _)| recorded_path == path)
                .map(|(_, traceparent)| traceparent.clone())
        }
    }

    #[cfg(feature = "hyper-http1")]
    #[tokio::test]
    #[serial]
    async fn instruments_successful_http1_requests() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let io = TokioIo::new(TcpStream::connect(server.addr).await.unwrap());
        let (send_request, connection) =
            hyper::client::conn::http1::handshake(io).await.unwrap();

        tokio::spawn(async move {
            connection.await.unwrap();
        });

        let mut send_request = send_request.instrument();
        let request_url = format!("{}/ok?ready=true", server.base_url);
        let response = send_request
            .send_request(
                Request::builder()
                    .uri(&request_url)
                    .header(HOST, server.authority())
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
        assert!(string_attr(span, semconv::NETWORK_PEER_ADDRESS).is_none());
        assert!(i64_attr(span, semconv::NETWORK_PEER_PORT).is_none());
    }

    #[cfg(feature = "hyper-http2")]
    #[tokio::test]
    #[serial]
    async fn instruments_successful_http2_requests() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let io = TokioIo::new(TcpStream::connect(server.addr).await.unwrap());
        let (send_request, connection) =
            hyper::client::conn::http2::Builder::new(TokioExecutor::new())
                .handshake(io)
                .await
                .unwrap();

        tokio::spawn(async move {
            connection.await.unwrap();
        });

        let mut send_request = send_request.instrument();
        let request_url = format!("{}/ok?ready=true", server.base_url);
        let response = send_request
            .send_request(
                Request::builder()
                    .uri(&request_url)
                    .header(HOST, server.authority())
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
        assert!(string_attr(span, semconv::URL_FULL) == Some(request_url.as_str()));
        assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE) == Some(200));
        assert!(string_attr(span, semconv::NETWORK_PROTOCOL_VERSION).is_some());
    }

    #[cfg(feature = "hyper-http1")]
    #[tokio::test]
    #[serial]
    async fn uses_explicit_parent_context_when_provided() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let io = TokioIo::new(TcpStream::connect(server.addr).await.unwrap());
        let (send_request, connection) =
            hyper::client::conn::http1::handshake(io).await.unwrap();
        let tracer = telemetry.provider.tracer("hyper-tests");
        let explicit_parent = tracer.start("explicit-parent");
        let explicit_parent_span_id = explicit_parent.span_context().span_id();
        let explicit_parent_cx = Context::current_with_span(explicit_parent);
        let tracing_tracer = telemetry.provider.tracer("tracing-tests");
        let subscriber = Registry::default()
            .with(tracing_opentelemetry::layer().with_tracer(tracing_tracer));
        let guard = tracing::subscriber::set_default(subscriber);
        let current_parent = tracing::info_span!("current-parent");

        tokio::spawn(async move {
            connection.await.unwrap();
        });

        tracing::Instrument::instrument(
            async {
                let mut send_request =
                    send_request.instrument().context(&explicit_parent_cx);
                send_request
                    .send_request(
                        Request::builder()
                            .uri(format!("{}/ok", server.base_url))
                            .header(HOST, server.authority())
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

    #[cfg(feature = "hyper-http1")]
    #[tokio::test]
    #[serial]
    async fn marks_error_responses_as_errors() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let io = TokioIo::new(TcpStream::connect(server.addr).await.unwrap());
        let (send_request, connection) =
            hyper::client::conn::http1::handshake(io).await.unwrap();

        tokio::spawn(async move {
            connection.await.unwrap();
        });

        let mut send_request = send_request.instrument();
        let response = send_request
            .send_request(
                Request::builder()
                    .uri(format!("{}/not-found", server.base_url))
                    .header(HOST, server.authority())
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

    struct TestServer {
        addr: std::net::SocketAddr,
        base_url: String,
        state: TestState,
        _handle: JoinHandle<()>,
    }

    impl TestServer {
        fn authority(&self) -> String {
            self.addr.to_string()
        }
    }

    async fn spawn_server() -> TestServer {
        async fn ok(
            State(state): State<TestState>,
            headers: HeaderMap,
        ) -> impl IntoResponse {
            state.record("/ok", &headers);
            StatusCode::OK
        }

        async fn not_found(
            State(state): State<TestState>,
            headers: HeaderMap,
        ) -> impl IntoResponse {
            state.record("/not-found", &headers);
            StatusCode::NOT_FOUND
        }

        async fn redirect(
            State(state): State<TestState>,
            headers: HeaderMap,
        ) -> impl IntoResponse {
            state.record("/redirect", &headers);
            Redirect::temporary("/final")
        }

        async fn final_route(
            State(state): State<TestState>,
            headers: HeaderMap,
        ) -> impl IntoResponse {
            state.record("/final", &headers);
            StatusCode::OK
        }

        let state = TestState::default();
        let app = Router::new()
            .route("/ok", get(ok))
            .route("/not-found", get(not_found))
            .route("/redirect", get(redirect))
            .route("/final", get(final_route))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        TestServer {
            addr,
            base_url: format!("http://{addr}"),
            state,
            _handle: handle,
        }
    }

    fn configure_test_tracing() -> TestTelemetry {
        let exporter = InMemorySpanExporter::default();
        let provider = TracerProvider::builder()
            .with_simple_exporter(exporter.clone())
            .build();
        global::set_tracer_provider(provider.clone());
        global::set_text_map_propagator(TraceContextPropagator::new());
        TestTelemetry { exporter, provider }
    }

    fn force_flush_and_get_spans(
        telemetry: &TestTelemetry,
    ) -> Vec<opentelemetry_sdk::trace::SpanData> {
        telemetry.provider.force_flush().unwrap();
        telemetry.exporter.get_finished_spans().unwrap()
    }

    fn find_span<'a>(
        spans: &'a [opentelemetry_sdk::trace::SpanData],
        name: &str,
    ) -> &'a opentelemetry_sdk::trace::SpanData {
        spans.iter().find(|span| span.name == name).unwrap()
    }

    fn string_attr<'a>(
        span: &'a opentelemetry_sdk::trace::SpanData,
        key: &str,
    ) -> Option<&'a str> {
        match attr(span, key) {
            Some(Value::String(value)) => Some(value.as_str()),
            _ => None,
        }
    }

    fn i64_attr(span: &opentelemetry_sdk::trace::SpanData, key: &str) -> Option<i64> {
        match attr(span, key) {
            Some(Value::I64(value)) => Some(*value),
            _ => None,
        }
    }

    fn attr<'a>(
        span: &'a opentelemetry_sdk::trace::SpanData,
        key: &str,
    ) -> Option<&'a Value> {
        span.attributes
            .iter()
            .find(|kv| kv.key.as_str() == key)
            .map(|kv| &kv.value)
    }

    fn traceparent_ids(traceparent: &str) -> (&str, &str) {
        let mut parts = traceparent.split('-');
        let _version = parts.next().unwrap();
        let trace_id = parts.next().unwrap();
        let span_id = parts.next().unwrap();
        (trace_id, span_id)
    }

    struct TestTelemetry {
        exporter: InMemorySpanExporter,
        provider: TracerProvider,
    }

    impl Drop for TestTelemetry {
        fn drop(&mut self) {
            let _ = self.provider.shutdown();
        }
    }
}
