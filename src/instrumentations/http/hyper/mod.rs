//! Async hyper connection instrumentation helpers.
//!
//! This module instruments `hyper::client::conn::http1::SendRequest`,
//! `hyper::client::conn::http2::SendRequest`, and, with the
//! `hyper-client-legacy` feature, `hyper_util::client::legacy::Client`.
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

use crate::{
    Context, Value,
    instrumentations::http::client::{HttpClientSpanBuilder, HttpError, UrlParts},
};

/// Async instrumentation helpers for `hyper_util::client::legacy::Client`.
#[cfg(feature = "hyper-client-legacy")]
pub mod legacy_client;

#[cfg(feature = "hyper-client-legacy")]
pub use legacy_client::HyperLegacyClientInstrument;

impl UrlParts for hyper::Uri {
    fn full_url(&self) -> Option<impl Into<Value>> {
        self.host().map(|_| self.to_string())
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
        self.port_u16().map(i64::from)
    }

    fn query(&self) -> Option<impl Into<Value>> {
        self.query().map(ToOwned::to_owned)
    }
}

impl HttpError for hyper::Error {
    fn error_type(&self) -> &'static str {
        if self.is_timeout() {
            "timeout"
        } else if self.is_closed() {
            "closed"
        } else if self.is_canceled() {
            "canceled"
        } else if self.is_parse_status() {
            "parse_status"
        } else if self.is_parse() {
            "parse"
        } else if self.is_incomplete_message() {
            "incomplete_message"
        } else if self.is_body_write_aborted() {
            "body_write_aborted"
        } else if self.is_shutdown() {
            "shutdown"
        } else if self.is_user() {
            "user"
        } else {
            "_OTHER"
        }
    }
}

impl<B> From<&hyper::Request<B>> for HttpClientSpanBuilder {
    fn from(request: &hyper::Request<B>) -> Self {
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
impl<B> HyperSendRequestInstrument for hyper::client::conn::http1::SendRequest<B> {
    fn instrument(self) -> InstrumentedSendRequest<Self> {
        InstrumentedSendRequest::new(self)
    }
}

#[cfg(feature = "hyper-http2")]
impl<B> HyperSendRequestInstrument for hyper::client::conn::http2::SendRequest<B> {
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

#[cfg(any(feature = "hyper-http1", feature = "hyper-http2"))]
macro_rules! impl_instrumented_send_request {
    ($http:ident) => {
        mod $http {
            use std::future::Future;
            use std::task::{Context as TaskContext, Poll};

            use hyper::{
                Request, Response, Result,
                body::{Body, Incoming},
                client::conn::$http::SendRequest,
            };

            use super::InstrumentedSendRequest;
            use crate::{
                future::InstrumentedFuture, http,
                instrumentations::http::client::HttpClientSpanBuilder,
            };

            impl<B: Body + 'static> InstrumentedSendRequest<SendRequest<B>> {
                /// Polls until the underlying sender is ready to send a request.
                pub fn poll_ready(
                    &mut self,
                    cx: &mut TaskContext<'_>,
                ) -> Poll<Result<()>> {
                    self.inner.poll_ready(cx)
                }

                /// Waits until the underlying sender is ready to send a request.
                pub async fn ready(&mut self) -> Result<()> {
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
                ) -> impl Future<Output = Result<Response<Incoming>>> + '_ {
                    let span = HttpClientSpanBuilder::from(&request).start(&self.context);

                    http::inject_context_on_context(
                        span.context(),
                        request.headers_mut(),
                    );

                    let future = self.inner.send_request(request);
                    InstrumentedFuture::new(future, span)
                }
            }
        }
    };
}

#[cfg(feature = "hyper-http1")]
impl_instrumented_send_request!(http1);
#[cfg(feature = "hyper-http2")]
impl_instrumented_send_request!(http2);

#[cfg(all(test, any(feature = "hyper-http1", feature = "hyper-http2")))]
mod tests {
    use super::HyperSendRequestInstrument;
    use crate::{Context, instrumentations::http::test_utils::*, semconv};
    use axum::http::StatusCode;
    use bytes::Bytes;
    use http_body_util::Empty;
    use hyper::{
        Request,
        header::{HOST, USER_AGENT},
    };
    use hyper_util::rt::TokioIo;
    use opentelemetry::trace::{Span, SpanKind, TraceContextExt, Tracer, TracerProvider};
    use serial_test::serial;
    use tokio::net::TcpStream;
    use tracing_subscriber::{Registry, layer::SubscriberExt};

    #[cfg(feature = "hyper-http1")]
    mod http1 {
        use super::*;
        use assert2::assert;

        #[tokio::test]
        #[serial]
        async fn instruments_successful_requests() {
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
                i64_attr(span, semconv::SERVER_PORT)
                    == Some(i64::from(server.addr.port()))
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

        #[tokio::test]
        #[serial]
        async fn falls_back_to_host_header_for_url_full_on_origin_form_uri() {
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
                        .uri("/ok?ready=true")
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

            let expected_url = format!("//{}/ok?ready=true", server.authority());
            assert!(string_attr(span, semconv::URL_FULL) == Some(expected_url.as_str()));
            assert!(string_attr(span, semconv::SERVER_ADDRESS) == Some("127.0.0.1"));
            assert!(
                i64_attr(span, semconv::SERVER_PORT)
                    == Some(i64::from(server.addr.port()))
            );
        }

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
    }

    #[cfg(feature = "hyper-http2")]
    mod http2 {
        use super::*;
        use assert2::assert;
        use hyper_util::rt::TokioExecutor;

        #[tokio::test]
        #[serial]
        async fn instruments_successful_requests() {
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
                i64_attr(span, semconv::SERVER_PORT)
                    == Some(i64::from(server.addr.port()))
            );
            assert!(string_attr(span, semconv::URL_PATH) == Some("/ok"));
            assert!(string_attr(span, semconv::URL_QUERY) == Some("ready=true"));
            assert!(string_attr(span, semconv::URL_FULL) == Some(request_url.as_str()));
            assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE) == Some(200));
            assert!(string_attr(span, semconv::NETWORK_PROTOCOL_VERSION).is_some());
        }
    }
}
