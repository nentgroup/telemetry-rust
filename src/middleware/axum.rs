//! Axum web framework middleware.
//!
//! Provides middleware for the Axum web framework to automatically
//! instrument HTTP requests with OpenTelemetry tracing.

// Originally retired from davidB/tracing-opentelemetry-instrumentation-sdk
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/axum-tracing-opentelemetry/src/middleware/trace_extractor.rs#L53
// which is licensed under CC0 1.0 Universal
// https://github.com/davidB/tracing-opentelemetry-instrumentation-sdk/blob/d3609ac2cc699d3a24fbf89754053cc8e938e3bf/LICENSE

use http::{Request, Response};
use pin_project_lite::pin_project;
use std::{
    error::Error,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::{Layer, Service};
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::http as otel_http;

/// Function type for filtering HTTP requests by path.
///
/// Takes a path string and returns true if the request should be traced.
pub type Filter = fn(&str) -> bool;

/// Function type for extracting string representation from a matched path type.
///
/// Used to convert Axum's matched path type to a string for span attributes.
pub type AsStr<T> = fn(&T) -> &str;

/// OpenTelemetry layer for Axum applications.
///
/// This layer provides automatic tracing instrumentation for Axum web applications,
/// creating spans for HTTP requests with appropriate semantic attributes.
///
/// The layer is generic over [`axum::extract::MatchedPath`](https://docs.rs/axum/latest/axum/extract/struct.MatchedPath.html),
/// making it compatible with different versions of axum without being tied to any specific one.
///
/// # Example
///
/// ```rust
/// use axum::{Router, routing};
/// use telemetry_rust::middleware::axum::OtelAxumLayer;
///
/// let app: Router = axum::Router::new()
///     .nest("/api", Router::new()) // api_routes would be your actual routes
///     .layer(OtelAxumLayer::new(axum::extract::MatchedPath::as_str));
/// ```
#[derive(Debug, Clone)]
pub struct OtelAxumLayer<P> {
    matched_path_as_str: AsStr<P>,
    filter: Option<Filter>,
    inject_context: bool,
}

// add a builder like api
impl<P> OtelAxumLayer<P> {
    /// Creates a new OpenTelemetry layer for Axum.
    ///
    /// # Arguments
    ///
    /// * `matched_path_as_str` - [`axum::extract::MatchedPath::as_str`] or any function to convert [`axum::extract::MatchedPath`] to a `&str`
    ///
    ///  [`axum::extract::MatchedPath::as_str`]: https://docs.rs/axum/latest/axum/extract/struct.MatchedPath.html#method.as_str
    ///  [`axum::extract::MatchedPath`]: https://docs.rs/axum/latest/axum/extract/struct.MatchedPath.html
    pub fn new(matched_path_as_str: AsStr<P>) -> Self {
        OtelAxumLayer {
            matched_path_as_str,
            filter: None,
            inject_context: false,
        }
    }

    /// Sets a filter function to selectively trace requests.
    ///
    /// # Arguments
    ///
    /// * `filter` - Function that returns true for paths that should be traced
    pub fn filter(self, filter: Filter) -> Self {
        OtelAxumLayer {
            filter: Some(filter),
            ..self
        }
    }

    /// Configures whether to inject OpenTelemetry context into responses.
    ///
    /// # Arguments
    ///
    /// * `inject_context` - Whether to inject trace context into response headers
    pub fn inject_context(self, inject_context: bool) -> Self {
        OtelAxumLayer {
            inject_context,
            ..self
        }
    }
}

impl<S, P> Layer<S> for OtelAxumLayer<P> {
    /// The wrapped service
    type Service = OtelAxumService<S, P>;
    fn layer(&self, inner: S) -> Self::Service {
        OtelAxumService {
            inner,
            matched_path_as_str: self.matched_path_as_str,
            filter: self.filter,
            inject_context: self.inject_context,
        }
    }
}

/// OpenTelemetry service wrapper for Axum applications.
///
/// This service wraps Axum services to provide automatic HTTP request tracing
/// with OpenTelemetry spans and context propagation.
#[derive(Debug, Clone)]
pub struct OtelAxumService<S, P> {
    inner: S,
    matched_path_as_str: AsStr<P>,
    filter: Option<Filter>,
    inject_context: bool,
}

impl<S, B, B2, P> Service<Request<B>> for OtelAxumService<S, P>
where
    S: Service<Request<B>, Response = Response<B2>> + Clone + Send + 'static,
    S::Error: Error + 'static, //fmt::Display + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
    P: Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // #[allow(clippy::type_complexity)]
    // type Future = futures_core::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        let span = if self.filter.is_none_or(|f| f(req.uri().path())) {
            let span = otel_http::http_server::make_span_from_request(&req);
            let matched_path = req.extensions().get::<P>();
            let route = matched_path.map_or("", self.matched_path_as_str);
            let method = otel_http::http_method(req.method());
            // let client_ip = parse_x_forwarded_for(req.headers())
            //     .or_else(|| {
            //         req.extensions()
            //             .get::<ConnectInfo<SocketAddr>>()
            //             .map(|ConnectInfo(client_ip)| Cow::from(client_ip.to_string()))
            //     })
            //     .unwrap_or_default();
            span.record("http.route", route);
            span.record("otel.name", format!("{method} {route}").trim());
            // span.record("trace_id", find_trace_id_from_tracing(&span));
            // span.record("client.address", client_ip);
            span.set_parent(otel_http::extract_context(req.headers()));
            span
        } else {
            tracing::Span::none()
        };
        let future = {
            let _ = span.enter();
            self.inner.call(req)
        };
        ResponseFuture {
            inner: future,
            inject_context: self.inject_context,
            span,
        }
    }
}

pin_project! {
    /// Response future for [`Trace`].
    ///
    /// [`Trace`]: super::Trace
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) inject_context: bool,
        pub(crate) span: Span,
        // pub(crate) start: Instant,
    }
}

impl<Fut, ResBody, E> Future for ResponseFuture<Fut>
where
    Fut: Future<Output = Result<Response<ResBody>, E>>,
    E: std::error::Error + 'static,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let mut result = futures_util::ready!(this.inner.poll(cx));
        otel_http::http_server::update_span_from_response_or_error(this.span, &result);
        if *this.inject_context
            && let Ok(response) = result.as_mut()
        {
            otel_http::inject_context(
                &tracing_opentelemetry_instrumentation_sdk::find_current_context(),
                response.headers_mut(),
            );
        }

        Poll::Ready(result)
    }
}
