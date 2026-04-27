//! Async reqwest instrumentation helpers.
//!
//! # Example
//!
//! ```no_run
//! use telemetry_rust::instrumentations::http::reqwest::ReqwestBuilderInstrument;
//!
//! # async fn example() -> Result<(), reqwest::Error> {
//! let response = reqwest::Client::new()
//!     .get("https://example.com/health")
//!     .instrument()
//!     .send()
//!     .await?;
//! # let _ = response;
//! # Ok(())
//! # }
//! ```

use std::future::Future;

use crate::{
    Context, Value, http,
    instrumentations::http::client::{HttpClientSpanBuilder, UrlParts},
};

impl UrlParts for reqwest::Url {
    fn full_url(&self) -> Option<impl Into<Value>> {
        Some(self.as_str().to_owned())
    }

    fn path(&self) -> Option<impl Into<Value>> {
        Some(self.path().to_owned())
    }

    fn host(&self) -> Option<impl Into<Value>> {
        self.host_str().map(ToOwned::to_owned)
    }

    fn scheme(&self) -> Option<impl Into<Value>> {
        match self.scheme() {
            "" => None,
            scheme => Some(scheme.to_owned()),
        }
    }

    fn port(&self) -> Option<impl Into<Value>> {
        self.port_or_known_default().map(i64::from)
    }

    fn query(&self) -> Option<impl Into<Value>> {
        self.query().map(ToOwned::to_owned)
    }
}

/// A trait for instrumenting async reqwest request builders with OpenTelemetry tracing.
///
/// ```no_run
/// use telemetry_rust::instrumentations::http::reqwest::ReqwestBuilderInstrument;
///
/// # async fn example() -> Result<(), reqwest::Error> {
/// let response = reqwest::Client::new()
///     .get("https://example.com/health")
///     .instrument()
///     .send()
///     .await?;
/// # let _ = response;
/// # Ok(())
/// # }
/// ```
pub trait ReqwestBuilderInstrument
where
    Self: Sized,
{
    /// Instruments this reqwest builder with OpenTelemetry tracing.
    fn instrument(self) -> InstrumentedRequestBuilder;
}

impl ReqwestBuilderInstrument for reqwest::RequestBuilder {
    fn instrument(self) -> InstrumentedRequestBuilder {
        InstrumentedRequestBuilder::new(self)
    }
}

impl HttpClientSpanBuilder {
    pub(crate) fn from_reqwest_request(request: &reqwest::Request) -> Self {
        Self::from_parts(request.method(), request.headers(), request.url())
    }
}

/// A wrapper that instruments async reqwest request builders with OpenTelemetry tracing.
#[must_use = "RequestBuilder does nothing until you call send()"]
pub struct InstrumentedRequestBuilder {
    inner: reqwest::RequestBuilder,
    context: Option<Context>,
}

impl InstrumentedRequestBuilder {
    /// Creates a new instrumented reqwest request builder.
    pub fn new(inner: reqwest::RequestBuilder) -> Self {
        Self {
            inner,
            context: None,
        }
    }

    /// Sets the OpenTelemetry context for this instrumented request.
    pub fn context(mut self, context: &Context) -> Self {
        self.context = Some(context.clone());
        self
    }

    /// Sets the optional OpenTelemetry context for this instrumented request.
    pub fn set_context(mut self, context: Option<&Context>) -> Self {
        self.context = context.cloned();
        self
    }

    /// Sends the request and records an outbound HTTP client span around it.
    pub fn send(self) -> impl Future<Output = Result<reqwest::Response, reqwest::Error>> {
        let (client, request_result) = self.inner.build_split();
        let context = self.context;

        async move {
            let mut request = request_result?;
            let span_builder = HttpClientSpanBuilder::from_reqwest_request(&request);
            let span = match context.as_ref() {
                Some(context) => span_builder.start_with_context(context),
                None => span_builder.start(),
            };

            http::inject_context_on_context(span.context(), request.headers_mut());

            let result = client.execute(request).await;
            match &result {
                Ok(response) => {
                    span.end_response(
                        response.status(),
                        response.version(),
                        response.remote_addr(),
                    );
                }
                Err(error) => span.end_error(reqwest_error_type(error), error),
            }
            result
        }
    }
}

fn reqwest_error_type(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        "timeout"
    } else if error.is_connect() {
        "connect"
    } else if error.is_redirect() {
        "redirect"
    } else if error.is_request() {
        "request"
    } else if error.is_body() {
        "body"
    } else if error.is_decode() {
        "decode"
    } else if error.is_builder() {
        "builder"
    } else {
        "_OTHER"
    }
}

#[cfg(test)]
mod tests {
    use super::ReqwestBuilderInstrument;
    use crate::{Context, OpenTelemetryLayer, Value, semconv};
    use assert2::assert;
    use axum::{
        Router,
        extract::State,
        http::{HeaderMap, StatusCode},
        response::{IntoResponse, Redirect},
        routing::get,
    };
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
    use tokio::{net::TcpListener, task::JoinHandle};
    use tracing_opentelemetry::OpenTelemetrySpanExt as _;
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

    #[tokio::test]
    #[serial]
    async fn instruments_successful_requests() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;

        let response = test_client()
            .get(format!("{}/ok?ready=true", server.base_url))
            .header(::reqwest::header::USER_AGENT, "telemetry-rust-tests")
            .instrument()
            .send()
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
        assert!(string_attr(span, semconv::URL_PATH) == Some("/ok"));
        assert!(string_attr(span, semconv::URL_QUERY) == Some("ready=true"));
        assert!(
            string_attr(span, semconv::USER_AGENT_ORIGINAL)
                == Some("telemetry-rust-tests")
        );
        assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE) == Some(200));
        assert!(string_attr(span, semconv::NETWORK_PROTOCOL_VERSION).is_some());
        assert!(string_attr(span, semconv::NETWORK_PEER_ADDRESS).is_some());
        assert!(i64_attr(span, semconv::NETWORK_PEER_PORT).is_some());
    }

    #[tokio::test]
    #[serial]
    async fn propagates_traceparent_with_client_span_id() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;

        let tracer = global::tracer("reqwest-propagation-test");
        let subscriber = Registry::default().with(OpenTelemetryLayer::new(tracer));
        let _guard = tracing::subscriber::set_default(subscriber);

        let parent = tracing::info_span!("parent");
        let parent_context = parent.context();
        let expected_trace_id = parent_context.span().span_context().trace_id();

        tracing::Instrument::instrument(
            async {
                test_client()
                    .get(format!("{}/ok", server.base_url))
                    .instrument()
                    .send()
                    .await
                    .unwrap();
            },
            parent,
        )
        .await;

        let spans = force_flush_and_get_spans(&telemetry);
        let client_span = find_span(&spans, "GET");
        let traceparent = server.state.traceparent_for("/ok").unwrap();
        let (trace_id, span_id) = traceparent_ids(&traceparent);

        // The traceparent carries the client span's own span-id, not the parent's.
        assert!(trace_id == expected_trace_id.to_string());
        assert!(span_id == client_span.span_context.span_id().to_string());
    }

    #[tokio::test]
    #[serial]
    async fn marks_client_error_responses_as_errors() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;

        let response = test_client()
            .get(format!("{}/not-found", server.base_url))
            .instrument()
            .send()
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
    async fn marks_server_error_responses_as_errors() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;

        test_client()
            .get(format!("{}/server-error", server.base_url))
            .instrument()
            .send()
            .await
            .unwrap();

        let spans = force_flush_and_get_spans(&telemetry);
        let span = find_span(&spans, "GET");

        assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE) == Some(500));
        assert!(string_attr(span, semconv::ERROR_TYPE) == Some("500"));
        assert!(matches!(
            span.status,
            opentelemetry::trace::Status::Error { .. }
        ));
    }

    #[tokio::test]
    #[serial]
    async fn marks_transport_failures_as_errors() {
        let telemetry = configure_test_tracing();

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        let error = test_client()
            .get(format!("http://{addr}/unavailable"))
            .instrument()
            .send()
            .await
            .unwrap_err();

        assert!(error.is_connect());

        let spans = force_flush_and_get_spans(&telemetry);
        let span = find_span(&spans, "GET");

        assert!(matches!(
            span.status,
            opentelemetry::trace::Status::Error { .. }
        ));
        assert!(string_attr(span, semconv::ERROR_TYPE) == Some("connect"));
        assert!(i64_attr(span, semconv::HTTP_RESPONSE_STATUS_CODE).is_none());
    }

    #[tokio::test]
    #[serial]
    async fn preserves_original_url_when_redirects_are_followed() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;

        let response = test_client()
            .get(format!("{}/redirect?step=1", server.base_url))
            .instrument()
            .send()
            .await
            .unwrap();

        assert!(response.url().path() == "/final");

        let spans = force_flush_and_get_spans(&telemetry);
        let span = find_span(&spans, "GET");

        let expected_url = format!("{}/redirect?step=1", server.base_url);
        assert!(string_attr(span, semconv::URL_FULL) == Some(expected_url.as_str()));
        assert!(server.state.traceparent_for("/redirect").is_some());
        assert!(server.state.traceparent_for("/final").is_some());
    }

    #[tokio::test]
    #[serial]
    async fn uses_explicit_parent_context_when_provided() {
        let telemetry = configure_test_tracing();
        let server = spawn_server().await;
        let tracer = telemetry.provider.tracer("reqwest-tests");
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
                test_client()
                    .get(format!("{}/ok", server.base_url))
                    .instrument()
                    .context(&explicit_parent_cx)
                    .send()
                    .await
                    .unwrap();
            },
            current_parent,
        )
        .await;

        drop(guard);
        explicit_parent_cx.span().end();

        let spans = force_flush_and_get_spans(&telemetry);
        let reqwest_span = find_span(&spans, "GET");
        let current_span = find_span(&spans, "current-parent");

        assert!(reqwest_span.parent_span_id == explicit_parent_span_id);
        assert!(reqwest_span.parent_span_id != current_span.span_context.span_id());
    }

    #[tokio::test]
    #[serial]
    async fn does_not_emit_span_for_invalid_builder() {
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

    // --- helpers ---

    struct TestServer {
        base_url: String,
        state: TestState,
        _handle: JoinHandle<()>,
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

        async fn server_error(
            State(state): State<TestState>,
            headers: HeaderMap,
        ) -> impl IntoResponse {
            state.record("/server-error", &headers);
            StatusCode::INTERNAL_SERVER_ERROR
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
            .route("/server-error", get(server_error))
            .route("/redirect", get(redirect))
            .route("/final", get(final_route))
            .with_state(state.clone());

        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        TestServer {
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

    fn test_client() -> ::reqwest::Client {
        ::reqwest::Client::builder().no_proxy().build().unwrap()
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
