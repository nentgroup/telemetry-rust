use opentelemetry::{
    global::{self, BoxedTracer},
    trace::{SpanBuilder, SpanKind, Tracer},
};
use tracing::Span;

pub(super) use super::AwsSpan;
use crate::{semcov, Context, KeyValue, OpenTelemetrySpanExt, StringValue};

mod dynamodb;
mod firehose;
mod sns;

pub use dynamodb::DynamodbSpanBuilder;
pub use firehose::FirehoseSpanBuilder;
pub use sns::SnsSpanBuilder;

pub struct AwsSpanBuilder<'a> {
    inner: SpanBuilder,
    tracer: BoxedTracer,
    context: Option<&'a Context>,
}

impl<'a> AwsSpanBuilder<'a> {
    fn new(
        span_kind: SpanKind,
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        custom_attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        let service: StringValue = service.into();
        let method: StringValue = method.into();
        let tracer = global::tracer("aws_sdk");
        let span_name = format!("{service}.{method}");
        let mut attributes = vec![
            semcov::RPC_METHOD.string(method),
            semcov::RPC_SYSTEM.string("aws-api"),
            semcov::RPC_SERVICE.string(service),
        ];
        attributes.extend(custom_attributes);
        let inner = tracer
            .span_builder(span_name)
            .with_attributes(attributes)
            .with_kind(span_kind);

        Self {
            inner,
            tracer,
            context: None,
        }
    }

    pub fn client(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Client, service, method, attributes)
    }

    pub fn producer(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Producer, service, method, attributes)
    }

    pub fn consumer(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Consumer, service, method, attributes)
    }

    pub fn attributes(mut self, iter: impl IntoIterator<Item = KeyValue>) -> Self {
        if let Some(attributes) = &mut self.inner.attributes {
            attributes.extend(iter);
        }
        self
    }

    #[inline]
    pub fn attribute(self, attribute: KeyValue) -> Self {
        self.attributes(std::iter::once(attribute))
    }

    #[inline]
    pub fn context(mut self, context: &'a Context) -> Self {
        self.context = Some(context);
        self
    }

    #[inline]
    pub fn set_context(mut self, context: Option<&'a Context>) -> Self {
        self.context = context;
        self
    }

    #[inline(always)]
    fn start_with_context(self, parent_cx: &Context) -> AwsSpan {
        self.inner
            .start_with_context(&self.tracer, parent_cx)
            .into()
    }

    #[inline]
    pub fn start(self) -> AwsSpan {
        match self.context {
            Some(context) => self.start_with_context(context),
            None => self.start_with_context(&Span::current().context()),
        }
    }
}

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
