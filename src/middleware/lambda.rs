use crate::{
    future::{InstrumentedFuture, InstrumentedFutureContext},
    semconv,
};
use lambda_runtime::LambdaInvocation;
use opentelemetry::trace::SpanKind;
use opentelemetry_sdk::trace::SdkTracerProvider as TracerProvider;
use std::task::{Context as TaskContext, Poll};
use tower::{Layer, Service};
use tracing::{instrument::Instrumented, Instrument};
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;

pub struct OtelLambdaLayer {
    provider: TracerProvider,
}

impl OtelLambdaLayer {
    pub fn new(provider: TracerProvider) -> Self {
        Self { provider }
    }
}

impl<S> Layer<S> for OtelLambdaLayer {
    type Service = OtelLambdaService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        OtelLambdaService {
            inner,
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

pub struct OtelLambdaService<S> {
    inner: S,
    provider: TracerProvider,
    coldstart: bool,
}

impl<S> Drop for OtelLambdaService<S> {
    fn drop(&mut self) {
        crate::shutdown_tracer_provider(&self.provider)
    }
}

impl<S, R> Service<LambdaInvocation> for OtelLambdaService<S>
where
    S: Service<LambdaInvocation, Response = R>,
{
    type Response = R;
    type Error = S::Error;
    type Future = InstrumentedFuture<Instrumented<S::Future>, TracerProvider>;

    fn poll_ready(&mut self, cx: &mut TaskContext<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: LambdaInvocation) -> Self::Future {
        let span = tracing::trace_span!(
            target: TRACING_TARGET,
            "Lambda function invocation",
            // TODO: set correct otel.kind and faas.trigger
            // see https://opentelemetry.io/docs/specs/semconv/faas/aws-lambda/
            "otel.kind" = ?SpanKind::Server,
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "other",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = self.coldstart,
        );

        self.coldstart = false;

        let future = self.inner.call(req).instrument(span);
        InstrumentedFuture::new(future, self.provider.clone())
    }
}
