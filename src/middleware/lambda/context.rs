use crate::semconv;
use lambda_runtime::LambdaInvocation;
use opentelemetry::{trace::SpanKind, StringValue};
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
    #[inline]
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

// PubSub lambda

pub struct PubSubLambdaService {
    system: StringValue,
    destination: StringValue,
}

impl OtelLambdaLayer<PubSubLambdaService> {
    pub fn pub_sub(
        provider: TracerProvider,
        system: impl Into<StringValue>,
        destination: impl Into<StringValue>,
    ) -> Self {
        let context = PubSubLambdaService {
            system: system.into(),
            destination: destination.into(),
        };
        Self::with_context(context, provider)
    }
}

impl OtelLambdaLayer<PubSubLambdaService> {
    pub fn sqs(provider: TracerProvider, topic_arn: impl Into<StringValue>) -> Self {
        Self::pub_sub(provider, "AmazonSQS", topic_arn)
    }
}

impl LambdaServiceContext for PubSubLambdaService {
    #[inline]
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span {
        tracing::trace_span!(
            target: TRACING_TARGET,
            "Lambda function invocation",
            "otel.kind" = ?SpanKind::Consumer,
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "pubsub",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = coldstart,
            { semconv::MESSAGING_OPERATION } = "process",
            { semconv::MESSAGING_SYSTEM } = self.system.as_str(),
            { semconv::MESSAGING_DESTINATION_NAME } = self.destination.as_str(),
        )
    }
}
