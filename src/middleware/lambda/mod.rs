use crate::{
    future::{InstrumentedFuture, InstrumentedFutureContext},
    semconv,
};
use lambda_runtime::LambdaInvocation;
use opentelemetry::trace::SpanKind;
use opentelemetry_sdk::trace::TracerProvider;
use std::{
    sync::Arc,
    task::{Context as TaskContext, Poll},
};
use tower::{Layer, Service};
use tracing::{instrument::Instrumented, Instrument, Span};
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;

trait OtelLambdaServiceContext {
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span;
}

pub struct GenericLambdaInvocation {}

impl OtelLambdaServiceContext for GenericLambdaInvocation {
    #[inline]
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span {
        tracing::trace_span!(
            target: TRACING_TARGET,
            "Lambda function invocation",
            // TODO: set correct otel.kind and faas.trigger
            // see https://opentelemetry.io/docs/specs/semconv/faas/aws-lambda/
            "otel.kind" = ?SpanKind::Server,
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "other",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = coldstart,
        )
    }
}

pub struct OtelLambdaLayer<C> {
    context: Arc<C>,
    provider: TracerProvider,
}

impl OtelLambdaLayer<GenericLambdaInvocation> {
    pub fn new(provider: TracerProvider) -> Self {
        let context = Arc::new(GenericLambdaInvocation {});
        Self { context, provider }
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
        self.force_flush();
    }
}

pub struct OtelLambdaService<S, C> {
    inner: S,
    context: Arc<C>,
    provider: TracerProvider,
    coldstart: bool,
}

impl<S, R, C> Service<LambdaInvocation> for OtelLambdaService<S, C>
where
    S: Service<LambdaInvocation, Response = R>,
    C: OtelLambdaServiceContext,
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
