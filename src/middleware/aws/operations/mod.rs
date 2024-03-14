use opentelemetry::trace::SpanKind;

pub(super) use super::AwsSpanBuilder;

mod dynamodb;
mod firehose;
mod sns;

pub use dynamodb::DynamodbSpanBuilder;
pub use firehose::FirehoseSpanBuilder;
pub use sns::SnsSpanBuilder;

pub enum MessagingOperationKind {
    Publish,
    Create,
    Receive,
    Control,
}

impl MessagingOperationKind {
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
