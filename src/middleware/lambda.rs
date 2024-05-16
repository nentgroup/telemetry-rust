use crate::{
    future::{InstrumentedFuture, InstrumentedFutureContext},
    semconv, OpenTelemetrySpanExt,
};
use lambda_runtime::LambdaInvocation;
use opentelemetry::{
    trace::{SpanKind, TraceContextExt},
    Context,
};
use opentelemetry_aws::trace::xray_propagator::span_context_from_str;
use opentelemetry_sdk::trace::TracerProvider;
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
        self.force_flush();
    }
}

pub struct OtelLambdaService<S> {
    inner: S,
    provider: TracerProvider,
    coldstart: bool,
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
            { semconv::FAAS_TRIGGER } = "other",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOKED_NAME } = req.context.env_config.function_name,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = self.coldstart,
        );

        let context = req
            .context
            .xray_trace_id
            .as_deref()
            .and_then(span_context_from_str)
            .map(|span_context| {
                Context::map_current(|cx| cx.with_remote_span_context(span_context))
            });
        if let Some(cx) = context {
            span.set_parent(cx);
        }

        self.coldstart = false;

        let future = self.inner.call(req).instrument(span);
        InstrumentedFuture::new(future, self.provider.clone())
    }
}
