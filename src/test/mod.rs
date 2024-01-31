pub mod jaegar;

#[cfg(feature = "hyper-v1")]
use http_body_util::BodyExt;
#[cfg(not(feature = "hyper-v1"))]
use hyper::{body::Bytes, header::HeaderValue, HeaderMap, Result};
#[cfg(feature = "hyper-v1")]
use hyper_v1::{header::HeaderValue, HeaderMap, Result};

pub use opentelemetry_api::trace::{SpanId, TraceId};
use rand::Rng;

#[cfg(feature = "hyper-v1")]
type Response = hyper_v1::Response<hyper_v1::body::Incoming>;
#[cfg(not(feature = "hyper-v1"))]
type Response = hyper::Response<hyper::Body>;

#[derive(Debug)]
pub struct TracedResponse {
    resp: Response,
    pub trace_id: TraceId,
    pub span_id: SpanId,
}

impl TracedResponse {
    pub fn new(resp: Response, traceparent: Traceparent) -> Self {
        Self {
            resp,
            trace_id: traceparent.trace_id,
            span_id: traceparent.span_id,
        }
    }

    #[cfg(not(feature = "hyper-v1"))]
    pub async fn get_bytes(&mut self) -> Result<Bytes> {
        hyper::body::to_bytes(self.body_mut()).await
    }

    #[cfg(not(feature = "hyper-v1"))]
    pub async fn into_bytes(self) -> Result<Bytes> {
        hyper::body::to_bytes(self.resp).await
    }

    #[cfg(all(feature = "axum", feature = "hyper-v1"))]
    pub async fn into_axum_bytes(self) -> Result<axum::body::Bytes> {
        Ok(self.resp.into_body().collect().await?.to_bytes())
    }
}

impl std::ops::Deref for TracedResponse {
    type Target = Response;

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
                let value = format!("{}-{}-1", self.trace_id, self.span_id);
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
                map.append("X-B3-Sampled", HeaderValue::from_str("1").unwrap());
            }
        }

        map
    }
}
