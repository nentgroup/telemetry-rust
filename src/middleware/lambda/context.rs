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

macro_rules! lambda_service {
    ($trigger: ident, $kind: ident, $service:ident {
        $($prop:ident: $type:ty,)*
        $({ $key:path } = $value:literal,)*
    }) => {
        #[allow(non_snake_case)]
        pub struct $service {
            $($prop: $type,)*
        }

        impl OtelLambdaLayer<$service> {
            #[inline]
            #[allow(non_snake_case)]
            pub fn $trigger(
                provider: TracerProvider,
                $($prop: impl Into<$type>,)*
            ) -> Self {
                let context = $service {
                    $($prop: $prop.into(),)*
                };
                Self::with_context(context, provider)
            }
        }

        impl LambdaServiceContext for $service {
            #[inline]
            fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span {
                tracing::trace_span!(
                    target: TRACING_TARGET,
                    "Lambda function invocation",
                    "otel.kind" = ?SpanKind::$kind,
                    "otel.name" = req.context.env_config.function_name,
                    { semconv::FAAS_TRIGGER } = stringify!($trigger),
                    { semconv::AWS_LAMBDA_INVOKED_ARN } = req.context.invoked_function_arn,
                    { semconv::FAAS_INVOCATION_ID } = req.context.request_id,
                    { semconv::FAAS_COLDSTART } = coldstart,
                    $({ semconv::$prop } = self.$prop.as_str(),)*
                    $({ $key } = $value,)*
                )
            }
        }
    };
}

lambda_service!(other, Server, GenericLambdaService {});
lambda_service!(http, Server, HttpLambdaService {});
lambda_service!(pubsub, Consumer, PubSubLambdaService {
    MESSAGING_SYSTEM: Value,
    MESSAGING_DESTINATION_NAME: OptionalValue,
    { semconv::MESSAGING_OPERATION } = "process",
});
lambda_service!(datasource, Consumer, DatasourceLambdaService {
    FAAS_DOCUMENT_COLLECTION: Value,
    FAAS_DOCUMENT_OPERATION: Value,
    FAAS_DOCUMENT_NAME: OptionalValue,
});
lambda_service!(timer, Consumer, TimerLambdaService {
    FAAS_CRON: OptionalValue,
});

impl OtelLambdaLayer<GenericLambdaService> {
    #[inline]
    pub fn new(provider: TracerProvider) -> Self {
        Self::other(provider)
    }
}

impl OtelLambdaLayer<PubSubLambdaService> {
    pub fn sqs(provider: TracerProvider, queue_arn: impl Into<StringValue>) -> Self {
        Self::pubsub(provider, "AmazonSQS", Some(queue_arn))
    }
}

impl OtelLambdaLayer<PubSubLambdaService> {
    pub fn sns(provider: TracerProvider, topic_arn: impl Into<StringValue>) -> Self {
        Self::pubsub(provider, "AmazonSNS", Some(topic_arn))
    }
}
