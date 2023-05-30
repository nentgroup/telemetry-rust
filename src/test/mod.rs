pub mod jaegar;

use opentelemetry_api::trace::{SpanId, TraceId};
use rand::Rng;

pub trait Response {
    fn text(&self) -> String;
}

pub struct TracedResponse<T> {
    response: T,
    pub trace_id: TraceId,
    pub span_id: SpanId,
}

impl<T> TracedResponse<T>
where
    T: Response,
{
    pub fn new(response: T, trace_id: TraceId, span_id: SpanId) -> Self {
        Self {
            response,
            trace_id,
            span_id,
        }
    }

    pub fn text(self) -> String {
        self.response.text()
    }
}

impl<T> std::ops::Deref for TracedResponse<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}

pub fn generate_traceparent() -> (String, TraceId, SpanId) {
    let mut rng = rand::thread_rng();
    let trace_id = TraceId::from_u128(rng.gen());
    let span_id = SpanId::from_u64(rng.gen());

    let traceparent = format!("00-{trace_id}-{span_id}-01");

    (traceparent, trace_id, span_id)
}

pub enum TracingHeaderKind {
    Traceparent,
    B3Single,
    B3Multi,
}

pub struct Traceparent {
    pub trace_id: TraceId,
    pub span_id: SpanId,
}

impl Traceparent {
    pub fn generate() -> Self {
        let mut rng = rand::thread_rng();
        let trace_id = TraceId::from_u128(rng.gen());
        let span_id = SpanId::from_u64(rng.gen());
        Self { trace_id, span_id }
    }
}
