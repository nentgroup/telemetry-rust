use opentelemetry::trace::SpanKind;

pub(super) use super::AwsSpanBuilder;

mod dynamodb;
mod firehose;
mod s3;
mod sagemaker_runtime;
mod secrets_manager;
mod sns;
mod sqs;
mod ssm;

pub use dynamodb::DynamodbSpanBuilder;
pub use firehose::FirehoseSpanBuilder;
pub use s3::S3SpanBuilder;
pub use sagemaker_runtime::SageMakerRuntimeSpanBuilder;
pub use secrets_manager::SecretsManagerSpanBuilder;
pub use sns::SnsSpanBuilder;
pub use sqs::SqsSpanBuilder;
pub use ssm::SsmSpanBuilder;

/// Messaging operation type
///
/// Represents well-known `messaging.operation.type` values from
/// [Semantic conventions specification](https://opentelemetry.io/docs/specs/semconv/registry/attributes/messaging/).
pub enum MessagingOperationKind {
    /// A message is created. “Create” spans always refer to a single message
    /// and are used to provide a unique creation context for messages in batch sending scenarios.
    Create,
    /// One or more messages are processed by a consumer.
    Process,
    /// One or more messages are requested by a consumer. This operation refers to pull-based scenarios,
    /// where consumers explicitly call methods of messaging SDKs to receive messages.
    Receive,
    /// One or more messages are provided for sending to an intermediary.
    /// If a single message is sent, the context of the “Send” span can be used as the creation context
    /// and no “Create” span needs to be created.
    Send,
    /// One or more messages are settled.
    Settle,
    /// Custom value representing control operations over messaging resources.
    Control,
}

impl MessagingOperationKind {
    /// Returns the string representation of the operation kind.
    ///
    /// This follows OpenTelemetry semantic conventions for messaging operations.
    pub fn as_str(&self) -> &'static str {
        match self {
            MessagingOperationKind::Create => "create",
            MessagingOperationKind::Process => "process",
            MessagingOperationKind::Receive => "receive",
            MessagingOperationKind::Send => "send",
            MessagingOperationKind::Settle => "settle",
            MessagingOperationKind::Control => "control",
        }
    }
}

impl std::fmt::Display for MessagingOperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl From<MessagingOperationKind> for SpanKind {
    #[inline]
    fn from(kind: MessagingOperationKind) -> Self {
        match kind {
            MessagingOperationKind::Create => SpanKind::Producer,
            MessagingOperationKind::Process => SpanKind::Consumer,
            MessagingOperationKind::Receive => SpanKind::Consumer,
            MessagingOperationKind::Settle => SpanKind::Producer,
            MessagingOperationKind::Send => SpanKind::Producer,
            MessagingOperationKind::Control => SpanKind::Client,
        }
    }
}

macro_rules! stringify_camel {
    ($var: ident) => {
        paste::paste! { stringify!([<$var:camel>]) }
    };
}

pub(super) use stringify_camel;
