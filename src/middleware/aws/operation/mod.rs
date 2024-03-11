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

pub use dynamodb::DynamoDBOperation;
pub use firehose::FirehoseOperation;
pub use sns::SnsOperation;

pub struct AwsOperation<'a> {
    inner: SpanBuilder,
    tracer: BoxedTracer,
    context: Option<&'a Context>,
}

impl<'a> AwsOperation<'a> {
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

macro_rules! aws_target {
    ($target: ident) => {
        pub struct $target<'a>($crate::middleware::aws::AwsOperation<'a>);

        impl<'a> From<$target<'a>> for $crate::middleware::aws::AwsOperation<'a> {
            #[inline]
            fn from(outer: $target<'a>) -> Self {
                outer.0
            }
        }

        impl<'a> $target<'a> {
            pub fn attributes(
                self,
                iter: impl IntoIterator<Item = $crate::KeyValue>,
            ) -> Self {
                Self(self.0.attributes(iter))
            }

            #[inline]
            pub fn attribute(self, attribute: $crate::KeyValue) -> Self {
                Self(self.0.attribute(attribute))
            }

            #[inline]
            pub fn context(self, context: &'a $crate::Context) -> Self {
                Self(self.0.context(context))
            }

            #[inline]
            pub fn set_context(self, context: Option<&'a $crate::Context>) -> Self {
                Self(self.0.set_context(context))
            }

            #[inline]
            pub fn start(self) -> $crate::middleware::aws::AwsSpan {
                self.0.start()
            }
        }
    };
}

macro_rules! stringify_camel {
    ($var: ident) => {
        paste::paste! { stringify!([<$var:camel>]) }
    };
}

pub(super) use {aws_target, stringify_camel};
