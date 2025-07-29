use crate::future::{InstrumentedFuture, InstrumentedFutureContext};
use lambda_runtime::LambdaInvocation;
use opentelemetry_sdk::trace::SdkTracerProvider as TracerProvider;
use std::{
    sync::Arc,
    task::{Context as TaskContext, Poll},
};
use tower::{Layer, Service};
use tracing::{Instrument, instrument::Instrumented};

use super::context::LambdaServiceContext;

/// OpenTelemetry layer for AWS Lambda functions.
///
/// This layer provides automatic tracing instrumentation for AWS Lambda functions,
/// creating spans for each invocation with appropriate FaaS semantic attributes.
///
/// # Example
///
/// ```rust,no_run
/// use lambda_runtime::{
///     Error as LambdaRuntimeError, Error as LambdaError, LambdaEvent, Runtime,
///     service_fn,
/// };
/// use telemetry_rust::{init_tracing, middleware::lambda::OtelLambdaLayer};
///
/// #[tracing::instrument(skip_all, err, fields(req_id = %event.context.request_id))]
/// pub async fn handle(event: LambdaEvent<()>) -> Result<String, LambdaError> {
///     Ok(String::from("Hello!"))
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), lambda_runtime::Error> {
///     // Grab TracerProvider after telemetry initialisation
///     let provider = init_tracing!(tracing::Level::WARN);
///
///     // Create lambda telemetry layer
///     let telemetry_layer = OtelLambdaLayer::new(provider);
///
///     // Run lambda runtime with telemetry layer
///     Runtime::new(service_fn(handle))
///         .layer(telemetry_layer)
///         .run()
///         .await?;
///
///     // Tracer provider will be automatically shutdown when the runtime is dropped
///
///     Ok(())
/// }
/// ```
pub struct OtelLambdaLayer<C> {
    context: Arc<C>,
    provider: TracerProvider,
}

impl<C> OtelLambdaLayer<C> {
    pub fn with_context(context: C, provider: TracerProvider) -> Self {
        Self {
            context: Arc::new(context),
            provider,
        }
    }
}

impl<S, C> Layer<S> for OtelLambdaLayer<C> {
    type Service = OtelLambdaService<S, C>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelLambdaService {
            inner,
            context: self.context.clone(),
            provider: self.provider.clone(),
            coldstart: true,
        }
    }
}

impl<T> InstrumentedFutureContext<T> for TracerProvider {
    fn on_result(self, _: &T) {
        if let Err(err) = self.force_flush() {
            tracing::warn!("failed to flush tracer provider: {err:?}");
        }
    }
}

/// OpenTelemetry service wrapper for AWS Lambda functions.
///
/// This service wraps Lambda services to provide automatic invocation tracing
/// with proper span lifecycle management and cold start detection.
pub struct OtelLambdaService<S, C> {
    inner: S,
    context: Arc<C>,
    provider: TracerProvider,
    coldstart: bool,
}

impl<S, C> Drop for OtelLambdaService<S, C> {
    fn drop(&mut self) {
        crate::shutdown_tracer_provider(&self.provider)
    }
}

impl<S, R, C> Service<LambdaInvocation> for OtelLambdaService<S, C>
where
    S: Service<LambdaInvocation, Response = R>,
    C: LambdaServiceContext,
{
    type Response = R;
    type Error = S::Error;
    type Future = InstrumentedFuture<Instrumented<S::Future>, TracerProvider>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let span = self.context.create_span(&req, self.coldstart);

        self.coldstart = false;

        let future = self.inner.call(req).instrument(span);
        InstrumentedFuture::new(future, self.provider.clone())
    }
}
