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
    fn span_kind(&self) -> SpanKind {
        SpanKind::Client
    }
    fn into_attributes(self, method: &'static str) -> Vec<KeyValue>;
}

impl<T: Into<StringValue>> IntoAttributes for AwsTarget<T> {
    fn service(&self) -> &'static str {
        match self {
            AwsTarget::Dynamo(_) => "DynamoDB",
            AwsTarget::Firehose(_) => "Firehose",
            AwsTarget::Sns(_) => "SNS",
        }
    }

    fn span_kind(&self) -> SpanKind {
        match self {
            AwsTarget::Dynamo(_) => SpanKind::Client,
            AwsTarget::Firehose(_) => SpanKind::Producer,
            AwsTarget::Sns(_) => SpanKind::Producer,
        }
    }

    fn into_attributes(self, method: &'static str) -> Vec<KeyValue> {
        match self {
            AwsTarget::Dynamo(table_name) => {
                let table_name: StringValue = table_name.into();
                vec![
                    semcov::DB_SYSTEM.string("dynamodb"),
                    semcov::DB_NAME.string(table_name.clone()),
                    semcov::DB_OPERATION.string(method),
                    semcov::AWS_DYNAMODB_TABLE_NAMES.array(vec![table_name]),
                ]
            }
            AwsTarget::Firehose(stream_name) => vec![
                semcov::MESSAGING_SYSTEM.string("aws_firehose"),
                semcov::MESSAGING_OPERATION.string("publish"),
                semcov::MESSAGING_DESTINATION_NAME.string(stream_name),
            ],
            AwsTarget::Sns(topic_arn) => vec![
                semcov::MESSAGING_SYSTEM.string("aws_sns"),
                semcov::MESSAGING_OPERATION.string("publish"),
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
    pub fn new(aws_target: impl IntoAttributes, method: impl Into<&'static str>) -> Self {
        let tracer = global::tracer("aws_sdk");
        let service = aws_target.service();
        let method: &'static str = method.into();
        let span_name = format!("{service}.{method}");
        let span_kind = aws_target.span_kind();
        let mut attributes = aws_target.into_attributes(method);
        attributes.extend(vec![
            semcov::RPC_METHOD.string(method),
            semcov::RPC_SYSTEM.string("aws-api"),
            semcov::RPC_SERVICE.string(service),
        ]);
        let inner = tracer
            .span_builder(span_name)
            .with_attributes(attributes)
            .with_kind(span_kind);

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
    #[inline]
    pub fn build(
        aws_target: impl IntoAttributes,
        method: impl Into<&'static str>,
    ) -> AwsSpanBuilder {
        AwsSpanBuilder::new(aws_target, method)
    }

    #[inline]
    pub fn new(aws_target: impl IntoAttributes, method: impl Into<&'static str>) -> Self {
        Self::build(aws_target, method).start()
    }

    #[inline]
    pub fn with_context(
        aws_target: impl IntoAttributes,
        method: impl Into<&'static str>,
        parent_cx: &Context,
    ) -> Self {
        Self::build(aws_target, method).start_with_context(parent_cx)
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
