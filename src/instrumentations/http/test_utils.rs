use crate::Value;
use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::get,
};
use opentelemetry::{global, trace::SpanKind};
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{InMemorySpanExporter, SdkTracerProvider as TracerProvider},
};
use std::sync::{Arc, Mutex};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Clone, Default)]
pub struct TestState {
    traceparents: Arc<Mutex<Vec<(String, String)>>>,
}

impl TestState {
    pub fn record(&self, path: &str, headers: &HeaderMap) {
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

    pub fn traceparent_for(&self, path: &str) -> Option<String> {
        self.traceparents
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|(recorded_path, _)| recorded_path == path)
            .map(|(_, traceparent)| traceparent.clone())
    }
}

pub struct TestServer {
    pub addr: std::net::SocketAddr,
    pub base_url: String,
    pub state: TestState,
    _handle: JoinHandle<()>,
}

impl TestServer {
    pub fn authority(&self) -> String {
        self.addr.to_string()
    }
}

pub async fn spawn_server() -> TestServer {
    async fn ok(State(state): State<TestState>, headers: HeaderMap) -> impl IntoResponse {
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
        addr,
        base_url: format!("http://{addr}"),
        state,
        _handle: handle,
    }
}

pub fn configure_test_tracing() -> TestTelemetry {
    let exporter = InMemorySpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    global::set_tracer_provider(provider.clone());
    global::set_text_map_propagator(TraceContextPropagator::new());
    TestTelemetry { exporter, provider }
}

pub fn test_client() -> reqwest::Client {
    reqwest::Client::builder().no_proxy().build().unwrap()
}

pub fn force_flush_and_get_spans(
    telemetry: &TestTelemetry,
) -> Vec<opentelemetry_sdk::trace::SpanData> {
    telemetry.provider.force_flush().unwrap();
    telemetry.exporter.get_finished_spans().unwrap()
}

pub fn client_spans(
    spans: &[opentelemetry_sdk::trace::SpanData],
) -> Vec<&opentelemetry_sdk::trace::SpanData> {
    spans
        .iter()
        .filter(|span| span.span_kind == SpanKind::Client)
        .collect()
}

pub fn find_span<'a>(
    spans: &'a [opentelemetry_sdk::trace::SpanData],
    name: &str,
) -> &'a opentelemetry_sdk::trace::SpanData {
    spans.iter().find(|span| span.name == name).unwrap()
}

pub fn string_attr<'a>(
    span: &'a opentelemetry_sdk::trace::SpanData,
    key: &str,
) -> Option<&'a str> {
    match attr(span, key) {
        Some(Value::String(value)) => Some(value.as_str()),
        _ => None,
    }
}

pub fn i64_attr(span: &opentelemetry_sdk::trace::SpanData, key: &str) -> Option<i64> {
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

pub fn traceparent_ids(traceparent: &str) -> (&str, &str) {
    let mut parts = traceparent.split('-');
    let _version = parts.next().unwrap();
    let trace_id = parts.next().unwrap();
    let span_id = parts.next().unwrap();
    (trace_id, span_id)
}

pub struct TestTelemetry {
    pub exporter: InMemorySpanExporter,
    pub provider: TracerProvider,
}

impl Drop for TestTelemetry {
    fn drop(&mut self) {
        let _ = self.provider.shutdown();
    }
}
