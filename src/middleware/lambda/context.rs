use crate::semconv;
use lambda_runtime::LambdaInvocation;
use opentelemetry::trace::SpanKind;
use opentelemetry_sdk::trace::TracerProvider;
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;

use super::OtelLambdaLayer;

pub trait LambdaServiceContext {
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span;
}

// Generic lambda

pub struct GenericLambdaService {}

impl OtelLambdaLayer<GenericLambdaService> {
    pub fn new(provider: TracerProvider) -> Self {
        Self::with_context(GenericLambdaService {}, provider)
    }
}

impl LambdaServiceContext for GenericLambdaService {
    #[inline]
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span {
        tracing::trace_span!(
            target: TRACING_TARGET,
            "Lambda function invocation",
            "otel.kind" = ?SpanKind::Server,
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "other",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = coldstart,
        )
    }
}
