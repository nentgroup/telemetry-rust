use opentelemetry::trace::SpanKind;

pub(super) use super::AwsSpanBuilder;

mod dynamodb;
mod firehose;
mod sns;

pub use dynamodb::DynamodbSpanBuilder;
pub use firehose::FirehoseSpanBuilder;
pub use sns::SnsSpanBuilder;

/// Messaging operation kinds for AWS services.
///
/// Defines the different types of messaging operations that can be performed
/// with AWS messaging services like SQS, SNS, etc. Each operation kind maps
/// to an appropriate OpenTelemetry span kind.
pub enum MessagingOperationKind {
    /// Publishing one or mpde messages to a messaging service
    Publish,
    /// Creating a single message
    Create,
    /// Receiving or consuming messages from a messaging service
    Receive,
    /// Control operations (delete, update, list resources, etc.)
    Control,
}

impl MessagingOperationKind {
    /// Returns the string representation of the operation kind.
    ///
    /// This follows OpenTelemetry semantic conventions for messaging operations.
    pub fn as_str(&self) -> &'static str {
        match self {
            MessagingOperationKind::Publish => "publish",
            MessagingOperationKind::Create => "create",
            MessagingOperationKind::Receive => "receive",
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
            MessagingOperationKind::Publish => SpanKind::Producer,
            MessagingOperationKind::Create => SpanKind::Producer,
            MessagingOperationKind::Receive => SpanKind::Consumer,
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
