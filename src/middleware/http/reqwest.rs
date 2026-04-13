//! Reqwest client instrumentation utilities.
//!
//! Provides fluent-builder instrumentation for async [`reqwest`] request builders,
//! mirroring the AWS builder ergonomics in this crate.
//!
//! # Example
//!
//! ```no_run
//! use telemetry_rust::middleware::http::reqwest::ReqwestBuilderInstrument;
//!
//! # async fn example() -> Result<(), reqwest::Error> {
//! let client = reqwest::Client::new();
//!
//! let response = client
//!     .get("https://example.com/health")
//!     .instrument()
//!     .send()
//!     .await?;
//!
//! println!("status = {}", response.status());
//! # Ok(())
//! # }
//! ```

use ::http::Request as HttpRequest;
use ::reqwest as reqwest_crate;
use std::convert::TryFrom;

use crate::{
    Context, OpenTelemetrySpanExt, http, middleware::http::client::HttpClientSpanBuilder,
};

/// Trait for instrumenting async [`reqwest::RequestBuilder`] values.
pub trait ReqwestBuilderInstrument<'a>
where
    Self: Sized,
{
    /// Instruments this request builder with OpenTelemetry tracing.
    fn instrument(self) -> InstrumentedRequestBuilder<'a>;
}

impl<'a> ReqwestBuilderInstrument<'a> for reqwest_crate::RequestBuilder {
    fn instrument(self) -> InstrumentedRequestBuilder<'a> {
        InstrumentedRequestBuilder::new(self)
    }
}

/// Wrapper for instrumented async [`reqwest::RequestBuilder`] values.
pub struct InstrumentedRequestBuilder<'a> {
    inner: reqwest_crate::RequestBuilder,
    context: Option<&'a Context>,
}

impl<'a> InstrumentedRequestBuilder<'a> {
    /// Creates a new instrumented reqwest request builder.
    pub fn new(inner: reqwest_crate::RequestBuilder) -> Self {
        Self {
            inner,
            context: None,
        }
    }

    /// Sets the OpenTelemetry context for this instrumented request.
    pub fn context(mut self, context: &'a Context) -> Self {
        self.context = Some(context);
        self
    }

    /// Sets the OpenTelemetry context for this instrumented request.
    pub fn set_context(mut self, context: Option<&'a Context>) -> Self {
        self.context = context;
        self
    }

    /// Sends the request and records an outbound HTTP client span around it.
    pub async fn send(self) -> Result<reqwest_crate::Response, reqwest_crate::Error> {
        let (client, request) = self.inner.build_split();
        let request = request?;

        let mut request = HttpRequest::try_from(request)?;
        let parent_context = self
            .context
            .cloned()
            .unwrap_or_else(|| tracing::Span::current().context());
        http::inject_context_on_context(&parent_context, request.headers_mut());

        let span_builder = HttpClientSpanBuilder::from_request(&request)
            .set_context(Some(&parent_context));
        let request = reqwest_crate::Request::try_from(request)?;
        let span = span_builder.start();

        let result = client.execute(request).await;
        match &result {
            Ok(response) => {
                span.end_response_parts(response.status(), response.version(), response.remote_addr())
            }
            Err(error) => span.end_error(error),
        }
        result
    }
}

#[cfg(test)]
#[allow(clippy::await_holding_lock)]
mod tests {
    use super::ReqwestBuilderInstrument;
    use crate::{OpenTelemetryLayer, Value, semconv};
    use assert2::assert;
    use axum::{
        Router,
        http::{HeaderMap, StatusCode},
        routing::get,
    };
    use opentelemetry::{
        global,
        trace::{SpanKind, TraceContextExt},
    };
    use opentelemetry_sdk::{
        propagation::TraceContextPropagator,
        trace::{InMemorySpanExporter, SdkTracerProvider as TracerProvider},
    };
    use std::{sync::Mutex, time::Duration};
    use tokio::{net::TcpListener, task::JoinHandle};
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
    use tracing_subscriber::{Registry, layer::SubscriberExt};

    static TEST_GUARD: Mutex<()> = Mutex::new(());

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_instrumentation_propagates_traceparent_from_current_span() {
        let _lock = test_lock();
        let telemetry = configure_test_tracing();
        let (addr, server) = spawn_server(Router::new().route(
            "/traceparent",
            get(|headers: HeaderMap| async move {
                headers
                    .get("traceparent")
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_owned()
            }),
        ))
        .await;

        let tracer = global::tracer("reqwest-propagation-test");
        let subscriber = Registry::default().with(OpenTelemetryLayer::new(tracer));
        let _guard = tracing::subscriber::set_default(subscriber);

        let parent = tracing::info_span!("parent");
        let parent_context = parent.context();
        let expected_trace_id = parent_context.span().span_context().trace_id();

        let response = tracing::Instrument::instrument(
            async {
                test_client()
                    .get(format!("http://{addr}/traceparent"))
                    .instrument()
                    .send()
                    .await
                    .unwrap()
                    .text()
                    .await
                    .unwrap()
            },
            parent,
        )
        .await;

        let traceparent = response.trim();
        assert!(traceparent.starts_with("00-"));
        let mut parts = traceparent.split('-');
        assert!(parts.next() == Some("00"));
        assert!(parts.next() == Some(expected_trace_id.to_string().as_str()));

        server.abort();
        let spans = force_flush_and_get_spans(&telemetry);
        let client_spans = client_spans(&spans);
        assert!(client_spans.len() == 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_instrumentation_exports_client_span_for_successful_response() {
        let _lock = test_lock();
        let telemetry = configure_test_tracing();
        let (addr, server) =
            spawn_server(Router::new().route("/ok", get(|| async { "ok" }))).await;

        let tracer = global::tracer("reqwest-exporter-test");
        let subscriber = Registry::default().with(OpenTelemetryLayer::new(tracer));
        let _guard = tracing::subscriber::set_default(subscriber);

        let parent = tracing::info_span!("parent");
        let parent_context = parent.context();
        let expected_trace_id = parent_context.span().span_context().trace_id();
        let expected_parent_span_id = parent_context.span().span_context().span_id();

        tracing::Instrument::instrument(
            async {
                test_client()
                    .get(format!("http://{addr}/ok"))
                    .instrument()
                    .send()
                    .await
                    .unwrap();
            },
            parent,
        )
        .await;

        server.abort();

        let spans = force_flush_and_get_spans(&telemetry);
        let client_spans = client_spans(&spans);
        assert!(client_spans.len() == 1);

        let span = client_spans[0];
        assert!(span.span_context.trace_id() == expected_trace_id);
        assert!(span.parent_span_id == expected_parent_span_id);
        assert!(span.span_kind == SpanKind::Client);
        assert!(string_attr(span, semconv::HTTP_REQUEST_METHOD) == Some("GET"));
        assert!(string_attr(span, semconv::URL_SCHEME) == Some("http"));
        assert!(string_attr(span, semconv::SERVER_ADDRESS) == Some("127.0.0.1"));
        assert!(string_attr(span, semconv::URL_PATH) == Some("/ok"));
        assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE) == Some(200));
        assert!(matches!(span.status, opentelemetry::trace::Status::Unset));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_instrumentation_marks_transport_failures_as_errors() {
        let _lock = test_lock();
        let telemetry = configure_test_tracing();
        let port = unused_port().await;

        let result = test_client_builder()
            .timeout(Duration::from_millis(250))
            .build()
            .unwrap()
            .get(format!("http://127.0.0.1:{port}/transport-error"))
            .instrument()
            .send()
            .await;

        assert!(result.is_err());

        let spans = force_flush_and_get_spans(&telemetry);
        let client_spans = client_spans(&spans);
        assert!(client_spans.len() == 1);
        assert!(matches!(
            client_spans[0].status,
            opentelemetry::trace::Status::Error { .. }
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_instrumentation_marks_500_responses_as_errors() {
        let _lock = test_lock();
        let telemetry = configure_test_tracing();
        let (addr, server) = spawn_server(Router::new().route(
            "/server-error",
            get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "boom") }),
        ))
        .await;

        test_client()
            .get(format!("http://{addr}/server-error"))
            .instrument()
            .send()
            .await
            .unwrap();

        server.abort();

        let spans = force_flush_and_get_spans(&telemetry);
        let client_spans = client_spans(&spans);
        assert!(client_spans.len() == 1);
        assert!(
            i64_attr(client_spans[0], semconv::HTTP_RESPONSE_STATUS_CODE) == Some(500)
        );
        assert!(matches!(
            client_spans[0].status,
            opentelemetry::trace::Status::Error { .. }
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reqwest_instrumentation_does_not_emit_span_for_invalid_builder() {
        let _lock = test_lock();
        let telemetry = configure_test_tracing();

        let result = test_client()
            .get("http://example.com")
            .header("bad\nheader", "value")
            .instrument()
            .send()
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().is_builder());

        let spans = force_flush_and_get_spans(&telemetry);
        assert!(client_spans(&spans).is_empty());
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

    fn test_client() -> ::reqwest::Client {
        test_client_builder().build().unwrap()
    }

    fn test_client_builder() -> ::reqwest::ClientBuilder {
        ::reqwest::Client::builder().no_proxy()
    }

    fn test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_GUARD.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn force_flush_and_get_spans(
        telemetry: &TestTelemetry,
    ) -> Vec<opentelemetry_sdk::trace::SpanData> {
        telemetry.provider.force_flush().unwrap();
        telemetry.exporter.get_finished_spans().unwrap()
    }

    fn client_spans(
        spans: &[opentelemetry_sdk::trace::SpanData],
    ) -> Vec<&opentelemetry_sdk::trace::SpanData> {
        spans
            .iter()
            .filter(|span| span.span_kind == SpanKind::Client)
            .collect()
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

    async fn spawn_server(app: Router) -> (std::net::SocketAddr, JoinHandle<()>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (addr, handle)
    }

    async fn unused_port() -> u16 {
        TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap()
            .local_addr()
            .unwrap()
            .port()
    }

    struct TestTelemetry {
        exporter: InMemorySpanExporter,
        provider: TracerProvider,
    }
}
