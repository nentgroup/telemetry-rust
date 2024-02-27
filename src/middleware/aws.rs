use aws_types::request_id::RequestId;
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span as TelemetrySpan, SpanBuilder, SpanKind, Status, Tracer},
};
use std::error::Error;
use tracing::Span;

use crate::{semcov, Context, KeyValue, OpenTelemetrySpanExt, StringValue};

pub enum AwsTarget<T: Into<StringValue>> {
    Dynamo(T),
    Firehose(T),
    Sns(T),
}

pub trait IntoAttributes {
    fn service(&self) -> &'static str;
    fn into_attributes(self, operation: impl Into<StringValue>) -> Vec<KeyValue>;
}

impl<T: Into<StringValue>> IntoAttributes for AwsTarget<T> {
    fn service(&self) -> &'static str {
        match self {
            AwsTarget::Dynamo(_) => "dynamodb",
            AwsTarget::Firehose(_) => "firehose",
            AwsTarget::Sns(_) => "sns",
        }
    }

    fn into_attributes(self, operation: impl Into<StringValue>) -> Vec<KeyValue> {
        match self {
            AwsTarget::Dynamo(table_name) => vec![
                semcov::DB_SYSTEM.string("dynamodb"),
                semcov::DB_OPERATION.string(operation),
                semcov::AWS_DYNAMODB_TABLE_NAMES
                    .array(vec![Into::<StringValue>::into(table_name)]),
            ],
            AwsTarget::Firehose(stream_name) => vec![
                semcov::MESSAGING_SYSTEM.string("firehose"),
                semcov::MESSAGING_OPERATION.string(operation),
                semcov::MESSAGING_DESTINATION_NAME.string(stream_name),
            ],
            AwsTarget::Sns(topic_arn) => vec![
                semcov::MESSAGING_SYSTEM.string("sns"),
                semcov::MESSAGING_OPERATION.string(operation),
                semcov::MESSAGING_DESTINATION_NAME.string(topic_arn),
            ],
        }
    }
}

pub struct AwsSpanBuilder {
    inner: SpanBuilder,
    tracer: BoxedTracer,
}

impl AwsSpanBuilder {
    pub fn new(
        aws_target: impl IntoAttributes,
        operation: impl Into<StringValue>,
        method: impl Into<StringValue>,
    ) -> Self {
        let tracer = global::tracer("aws_sdk");
        let service = aws_target.service();
        let mut attributes: Vec<KeyValue> = vec![
            semcov::RPC_METHOD.string(method),
            semcov::RPC_SYSTEM.string("aws-api"),
            semcov::RPC_SERVICE.string(service),
        ];
        attributes.extend(aws_target.into_attributes(operation));
        let inner = tracer
            .span_builder(format!("aws_{service}"))
            .with_attributes(attributes)
            .with_kind(SpanKind::Client);

        Self { inner, tracer }
    }

    pub fn start_with_context(self, parent_cx: &Context) -> AwsSpan {
        self.inner
            .start_with_context(&self.tracer, parent_cx)
            .into()
    }

    pub fn start(self) -> AwsSpan {
        self.start_with_context(&Span::current().context())
    }
}

pub struct AwsSpan {
    span: BoxedSpan,
}

impl AwsSpan {
    pub fn new(
        aws_target: impl IntoAttributes,
        operation: impl Into<StringValue>,
        method: impl Into<StringValue>,
    ) -> AwsSpanBuilder {
        AwsSpanBuilder::new(aws_target, operation, method)
    }

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
