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

pub struct Value(StringValue);

impl Value {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<T: Into<StringValue>> From<T> for Value {
    fn from(inner: T) -> Self {
        Self(inner.into())
    }
}

pub struct OptionalValue(Option<StringValue>);

impl OptionalValue {
    pub fn as_str(&self) -> Option<&str> {
        self.0.as_ref().map(|value| value.as_str())
    }
}

impl<T: Into<StringValue>> From<Option<T>> for OptionalValue {
    fn from(inner: Option<T>) -> Self {
        Self(inner.map(|value| value.into()))
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
    system: Value,
    destination: OptionalValue,
}

impl OtelLambdaLayer<PubSubLambdaService> {
    pub fn pubsub(
        provider: TracerProvider,
        system: impl Into<Value>,
        destination: impl Into<OptionalValue>,
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
        Self::pubsub(provider, "AmazonSQS", Some(topic_arn))
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

// Datasource lambda

pub struct DatasourceLambdaService {
    collection: Value,
    operation: Value,
    document_name: OptionalValue,
}

impl OtelLambdaLayer<DatasourceLambdaService> {
    pub fn datasource(
        provider: TracerProvider,
        collection: impl Into<Value>,
        operation: impl Into<Value>,
        document_name: impl Into<OptionalValue>,
    ) -> Self {
        let context = DatasourceLambdaService {
            collection: collection.into(),
            operation: operation.into(),
            document_name: document_name.into(),
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
            { semconv::FAAS_DOCUMENT_COLLECTION } = self.collection.as_str(),
            { semconv::FAAS_DOCUMENT_OPERATION } = self.operation.as_str(),
            { semconv::FAAS_DOCUMENT_NAME } = self.document_name.as_str(),
        )
    }
}

// Timer lambda

pub struct TimerLambdaService {
    cron: OptionalValue,
}

impl OtelLambdaLayer<TimerLambdaService> {
    pub fn timer(
        provider: TracerProvider,
        cron: impl Into<OptionalValue>,
    ) -> Self {
        let context = TimerLambdaService {
            cron: cron.into(),
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
            { semconv::FAAS_CRON } = self.cron.as_str(),
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
