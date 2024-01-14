pub mod jaegar;

use http_body_util::BodyExt;
use hyper::{header::HeaderValue, Error, HeaderMap, Response};
pub use opentelemetry_api::trace::{SpanId, TraceId};
use rand::Rng;

#[derive(Debug)]
pub struct TracedResponse {
    resp: Response<hyper::body::Incoming>,
    pub trace_id: TraceId,
    pub span_id: SpanId,
}

impl TracedResponse {
    pub fn new(resp: Response<hyper::body::Incoming>, traceparent: Traceparent) -> Self {
        Self {
            resp,
            trace_id: traceparent.trace_id,
            span_id: traceparent.span_id,
        }
    }

    #[cfg(feature = "axum")]
    pub async fn into_axum_bytes(self) -> Result<axum::body::Bytes, Error> {
        Ok(self.resp.into_body().collect().await?.to_bytes())
    }
}

impl std::ops::Deref for TracedResponse {
    type Target = Response<hyper::body::Incoming>;

    fn deref(&self) -> &Self::Target {
        &self.resp
    }
}

impl std::ops::DerefMut for TracedResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.resp
    }
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

    pub fn get_headers(&self, kind: TracingHeaderKind) -> HeaderMap {
        let mut map = HeaderMap::new();

        match kind {
            TracingHeaderKind::Traceparent => {
                let value = format!("00-{}-{}-01", self.trace_id, self.span_id);
                map.append("traceparent", HeaderValue::from_str(&value).unwrap());
            }
            TracingHeaderKind::B3Single => {
                let value = format!("{}-{}", self.trace_id, self.span_id);
                map.append("b3", HeaderValue::from_str(&value).unwrap());
            }
            TracingHeaderKind::B3Multi => {
                map.append(
                    "X-B3-TraceId",
                    HeaderValue::from_str(&self.trace_id.to_string()).unwrap(),
                );
                map.append(
                    "X-B3-SpanId",
                    HeaderValue::from_str(&self.span_id.to_string()).unwrap(),
                );
            }
        }

        map
    }
}
