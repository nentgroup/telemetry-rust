use lambda_runtime::LambdaInvocation;
use opentelemetry::{StringValue, trace::SpanKind};
use opentelemetry_sdk::trace::SdkTracerProvider as TracerProvider;
use tracing::Span;
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;

use super::OtelLambdaLayer;
use crate::{middleware::aws::MessagingOperationKind, semconv};

/// Trait for creating OpenTelemetry spans for AWS Lambda invocations.
///
/// This trait defines the interface for context providers that create appropriate
/// spans for different types of Lambda triggers (HTTP, PubSub, Timer, etc.).
/// Each implementation provides trigger-specific span attributes and metadata.
pub trait LambdaServiceContext {
    /// Creates an OpenTelemetry span for a Lambda invocation.
    ///
    /// # Arguments
    ///
    /// * `req` - The Lambda invocation request containing context and payload
    /// * `coldstart` - Whether this is a cold start invocation
    ///
    /// # Returns
    ///
    /// A configured [`Span`] with appropriate OpenTelemetry attributes for the trigger type
    fn create_span(&self, req: &LambdaInvocation, coldstart: bool) -> Span;
}

/// Wrapper for required string values in Lambda span attributes.
///
/// This struct wraps OpenTelemetry's [`StringValue`] to provide a consistent
/// interface for Lambda context attributes that must have a value.
pub struct Value(StringValue);

impl Value {
    /// Returns the string value as a string slice.
    ///
    /// # Returns
    ///
    /// The underlying string value as `&str`
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl<T: Into<StringValue>> From<T> for Value {
    fn from(inner: T) -> Self {
        Self(inner.into())
    }
}

/// Wrapper for optional string values in Lambda span attributes.
///
/// This struct wraps an optional OpenTelemetry [`StringValue`] to provide a
/// consistent interface for Lambda context attributes that may or may not have a value.
pub struct OptionalValue(Option<StringValue>);

impl OptionalValue {
    /// Returns the string value as an optional string slice.
    ///
    /// # Returns
    ///
    /// The underlying string value as `Option<&str>`, or `None` if no value is set
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
    ($trigger: ident, $kind: ident, $service:ident { $($prop:ident: $type:ty,)* }$(, $($field:tt)*)?) => {
        #[allow(non_snake_case)]
        #[doc = concat!("Context provider for ", stringify!($trigger), " Lambda triggers.")]
        #[doc = ""]
        #[doc = concat!("This struct implements [`LambdaServiceContext`] to create appropriate ")]
        #[doc = concat!("OpenTelemetry spans for Lambda functions triggered by ", stringify!($trigger), " events.")]
        pub struct $service {
            $($prop: $type,)*
        }

        impl OtelLambdaLayer<$service> {
            #[inline]
            #[allow(non_snake_case)]
            #[doc = concat!("Creates a new OpenTelemetry layer for ", stringify!($trigger), " Lambda triggers.")]
            #[doc = ""]
            #[doc = "# Arguments"]
            #[doc = ""]
            #[doc = "* `provider` - The tracer provider to use for creating spans"]
            $(#[doc = concat!("* `", stringify!($prop), "` - ", stringify!($type))])*
            #[doc = ""]
            #[doc = "# Returns"]
            #[doc = ""]
            #[doc = concat!("A configured [`OtelLambdaLayer`] for ", stringify!($trigger), " triggers")]
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
                    $($($field)*)?
                )
            }
        }
    };
}

lambda_service!(other, Server, GenericLambdaService {});
lambda_service!(http, Server, HttpLambdaService {});
lambda_service!(
    pubsub,
    Consumer,
    PubSubLambdaService {
        MESSAGING_SYSTEM: Value,
        MESSAGING_DESTINATION_NAME: OptionalValue,
    },
    { semconv::MESSAGING_OPERATION_TYPE } = MessagingOperationKind::Process.as_str(),
);
lambda_service!(
    datasource,
    Consumer,
    DatasourceLambdaService {
        FAAS_DOCUMENT_COLLECTION: Value,
        FAAS_DOCUMENT_OPERATION: Value,
        FAAS_DOCUMENT_NAME: OptionalValue,
    },
);
lambda_service!(
    timer,
    Consumer,
    TimerLambdaService {
        FAAS_CRON: OptionalValue,
    },
);

impl OtelLambdaLayer<GenericLambdaService> {
    /// Creates a new OpenTelemetry layer for Lambda functions.
    ///
    /// # Arguments
    ///
    /// * `provider` - The tracer provider to use for creating spans
    #[inline]
    pub fn new(provider: TracerProvider) -> Self {
        Self::other(provider)
    }
}

impl OtelLambdaLayer<PubSubLambdaService> {
    /// Creates a new OpenTelemetry layer for Amazon SQS Lambda event source.
    ///
    /// This is a convenience method that creates a PubSub layer specifically
    /// configured for Amazon SQS events.
    ///
    /// # Arguments
    ///
    /// * `provider` - The tracer provider to use for creating spans
    /// * `queue_arn` - Optional SQS queue ARN for the messaging destination
    ///
    /// # Returns
    ///
    /// A configured [`OtelLambdaLayer`] for SQS triggers
    pub fn sqs(provider: TracerProvider, queue_arn: impl Into<OptionalValue>) -> Self {
        Self::pubsub(provider, "aws_sqs", queue_arn)
    }
}

impl OtelLambdaLayer<PubSubLambdaService> {
    /// Creates a new OpenTelemetry layer for Amazon SNS Lambda event source.
    ///
    /// This is a convenience method that creates a PubSub layer specifically
    /// configured for Amazon SNS events.
    ///
    /// # Arguments
    ///
    /// * `provider` - The tracer provider to use for creating spans
    /// * `topic_arn` - Optional SNS topic ARN for the messaging destination
    ///
    /// # Returns
    ///
    /// A configured [`OtelLambdaLayer`] for SNS triggers
    pub fn sns(provider: TracerProvider, topic_arn: impl Into<OptionalValue>) -> Self {
        Self::pubsub(provider, "aws_sns", topic_arn)
    }
}
