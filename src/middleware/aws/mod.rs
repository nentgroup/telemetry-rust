use aws_types::request_id::RequestId;
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span as TelemetrySpan, SpanBuilder, SpanKind, Status, Tracer},
};
use std::error::Error;
use tracing::Span;

use crate::{semcov, Context, KeyValue, OpenTelemetrySpanExt, StringValue};

#[cfg(feature = "aws-instrumentation")]
mod instrumentation;

#[cfg(feature = "aws-instrumentation")]
pub use instrumentation::AwsInstrumented;

pub struct AwsSpan {
    span: BoxedSpan,
}

impl AwsSpan {
    pub fn end<T, E>(self, aws_response: &Result<T, E>)
    where
        T: RequestId,
        E: RequestId + Error,
    {
        let mut span = self.span;
        let (status, request_id) = match aws_response {
            Ok(resp) => (Status::Ok, resp.request_id()),
            Err(error) => {
                span.record_error(&error);
                (Status::error(error.to_string()), error.request_id())
            }
        };
        if let Some(value) = request_id {
            span.set_attribute(semcov::AWS_REQUEST_ID.string(value.to_owned()));
        }
        span.set_status(status);
    }
}

impl From<BoxedSpan> for AwsSpan {
    #[inline]
    fn from(span: BoxedSpan) -> Self {
        Self { span }
    }
}

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

    #[inline]
    pub fn attribute(mut self, attribute: KeyValue) -> Self {
        if let Some(attributes) = &mut self.inner.attributes {
            attributes.push(attribute);
        }
        self
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

pub struct DynamoDBOperation<'a>(AwsOperation<'a>);

impl DynamoDBOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        table_name: impl Into<StringValue>,
    ) -> Self {
        let method: StringValue = method.into();
        let table_name: StringValue = table_name.into();
        let attributes = vec![
            semcov::DB_SYSTEM.string("dynamodb"),
            semcov::DB_NAME.string(table_name.clone()),
            semcov::DB_OPERATION.string(method.clone()),
            semcov::AWS_DYNAMODB_TABLE_NAMES.array(vec![table_name]),
        ];
        Self(AwsOperation::client("DynamoDB", method, attributes))
    }
}

impl<'a> Into<AwsOperation<'a>> for DynamoDBOperation<'a> {
    #[inline]
    fn into(self) -> AwsOperation<'a> {
        self.0
    }
}

impl<'a> DynamoDBOperation<'a> {
    #[inline]
    pub fn attribute(self, attribute: KeyValue) -> Self {
        Self(self.0.attribute(attribute))
    }

    #[inline]
    pub fn context(self, context: &'a Context) -> Self {
        Self(self.0.context(context))
    }

    #[inline]
    pub fn set_context(self, context: Option<&'a Context>) -> Self {
        Self(self.0.set_context(context))
    }

    #[inline]
    pub fn start(self) -> AwsSpan {
        self.0.start()
    }
}

pub struct FirehoseOperation<'a>(AwsOperation<'a>);

impl FirehoseOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        stream_name: impl Into<StringValue>,
    ) -> Self {
        let attributes = vec![
            semcov::MESSAGING_SYSTEM.string("aws_firehose"),
            semcov::MESSAGING_OPERATION.string("publish"),
            semcov::MESSAGING_DESTINATION_NAME.string(stream_name),
        ];
        Self(AwsOperation::producer("Firehose", method, attributes))
    }
}

impl<'a> Into<AwsOperation<'a>> for FirehoseOperation<'a> {
    #[inline]
    fn into(self) -> AwsOperation<'a> {
        self.0
    }
}

impl<'a> FirehoseOperation<'a> {
    #[inline]
    pub fn attribute(self, attribute: KeyValue) -> Self {
        Self(self.0.attribute(attribute))
    }

    #[inline]
    pub fn context(self, context: &'a Context) -> Self {
        Self(self.0.context(context))
    }

    #[inline]
    pub fn set_context(self, context: Option<&'a Context>) -> Self {
        Self(self.0.set_context(context))
    }

    #[inline]
    pub fn start(self) -> AwsSpan {
        self.0.start()
    }
}

pub struct SnsOperation<'a>(AwsOperation<'a>);

impl SnsOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        topic_arn: impl Into<StringValue>,
    ) -> Self {
        let attributes = vec![
            semcov::MESSAGING_SYSTEM.string("aws_sns"),
            semcov::MESSAGING_OPERATION.string("publish"),
            semcov::MESSAGING_DESTINATION_NAME.string(topic_arn),
        ];
        Self(AwsOperation::producer("SNS", method, attributes))
    }
}

impl<'a> Into<AwsOperation<'a>> for SnsOperation<'a> {
    #[inline]
    fn into(self) -> AwsOperation<'a> {
        self.0
    }
}

impl<'a> SnsOperation<'a> {
    #[inline]
    pub fn attribute(self, attribute: KeyValue) -> Self {
        Self(self.0.attribute(attribute))
    }

    #[inline]
    pub fn context(self, context: &'a Context) -> Self {
        Self(self.0.context(context))
    }

    #[inline]
    pub fn set_context(self, context: Option<&'a Context>) -> Self {
        Self(self.0.set_context(context))
    }

    #[inline]
    pub fn start(self) -> AwsSpan {
        self.0.start()
    }
}
