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

trait AsValue {
    fn as_value(&self) -> Option<&str>;
}

impl AsValue for StringValue {
    fn as_value(&self) -> Option<&str> {
        Some(self.as_str())
    }
}

impl AsValue for Option<StringValue> {
    fn as_value(&self) -> Option<&str> {
        self.as_ref().map(|value| value.as_str())
    }
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
    destination: Option<StringValue>,
}

impl OtelLambdaLayer<PubSubLambdaService> {
    pub fn pub_sub(
        provider: TracerProvider,
        system: impl Into<StringValue>,
        destination: Option<impl Into<StringValue>>,
    ) -> Self {
        let context = PubSubLambdaService {
            system: system.into(),
            destination: destination.map(|value| value.into()),
        };
        Self::with_context(context, provider)
    }
}

impl OtelLambdaLayer<PubSubLambdaService> {
    pub fn sqs(provider: TracerProvider, topic_arn: impl Into<StringValue>) -> Self {
        Self::pub_sub(provider, "AmazonSQS", Some(topic_arn))
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
            { semconv::MESSAGING_SYSTEM } = self.system.as_value(),
            { semconv::MESSAGING_DESTINATION_NAME } = self.destination.as_value(),
        )
    }
}

// Datasource lambda

pub struct DatasourceLambdaService {
    collection: StringValue,
    operation: StringValue,
    document_name: Option<StringValue>,
}

impl OtelLambdaLayer<DatasourceLambdaService> {
    pub fn pub_sub(
        provider: TracerProvider,
        collection: impl Into<StringValue>,
        operation: impl Into<StringValue>,
        document_name: Option<impl Into<StringValue>>,
    ) -> Self {
        let context = DatasourceLambdaService {
            collection: collection.into(),
            operation: operation.into(),
            document_name: document_name.map(|value| value.into()),
        };
        Self::with_context(context, provider)
    }
}

impl LambdaServiceContext for DatasourceLambdaService {
    #[inline]
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span {
        tracing::trace_span!(
            target: TRACING_TARGET,
            "Lambda function invocation",
            "otel.kind" = ?SpanKind::Consumer,
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "datasource",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = coldstart,
            { semconv::FAAS_DOCUMENT_COLLECTION } = self.collection.as_value(),
            { semconv::FAAS_DOCUMENT_OPERATION } = self.operation.as_value(),
            { semconv::FAAS_DOCUMENT_NAME } = self.document_name.as_value(),
        )
    }
}

// Timer lambda

pub struct TimerLambdaService {
    cron: Option<StringValue>,
}

impl OtelLambdaLayer<TimerLambdaService> {
    pub fn pub_sub(
        provider: TracerProvider,
        cron: Option<impl Into<StringValue>>,
    ) -> Self {
        let context = TimerLambdaService {
            cron: cron.map(|value| value.into()),
        };
        Self::with_context(context, provider)
    }
}

impl LambdaServiceContext for TimerLambdaService {
    #[inline]
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span {
        tracing::trace_span!(
            target: TRACING_TARGET,
            "Lambda function invocation",
            "otel.kind" = ?SpanKind::Consumer,
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "timer",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = coldstart,
            { semconv::FAAS_CRON } = self.cron.as_value(),
        )
    }
}

// HTTP lambda

pub struct HttpLambdaService {}

impl OtelLambdaLayer<HttpLambdaService> {
    #[inline]
    pub fn new(provider: TracerProvider) -> Self {
        Self::with_context(HttpLambdaService {}, provider)
    }
}

impl LambdaServiceContext for HttpLambdaService {
    #[inline]
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span {
        tracing::trace_span!(
            target: TRACING_TARGET,
            "Lambda function invocation",
            "otel.kind" = ?SpanKind::Server,
            "otel.name" = req.context.env_config.function_name,
            { semconv::FAAS_TRIGGER } = "http",
            { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
            { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
            { semconv::FAAS_COLDSTART } = coldstart,
        )
    }
}
