use aws_types::request_id::RequestId;
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span as TelemetrySpan, SpanBuilder, SpanKind, Status, Tracer},
    Context, KeyValue,
};
use opentelemetry_semantic_conventions as semcov;
use std::error::Error;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub enum AwsTarget<'a> {
    Dynamo(&'a str),
    Firehose(&'a str),
    Sns(&'a str),
}

impl AwsTarget<'_> {
    pub fn system(&self) -> &'static str {
        match self {
            AwsTarget::Dynamo(_) => "dynamodb",
            AwsTarget::Firehose(_) => "firehose",
            AwsTarget::Sns(_) => "sns",
        }
    }

    pub fn attributes(&self) -> Vec<KeyValue> {
        match self {
            AwsTarget::Dynamo(table_name) => vec![
                KeyValue::new("dynamoDB", true),
                KeyValue::new("db.name", table_name.to_string()),
            ],
            AwsTarget::Firehose(stream_name) => vec![
                KeyValue::new("firehose", true),
                KeyValue::new("firehose.name", stream_name.to_string()),
            ],
            AwsTarget::Sns(topic_arn) => vec![
                KeyValue::new("sns", true),
                KeyValue::new("sns.topic.arn", topic_arn.to_string()),
            ],
        }
    }
}

pub enum AwsSpan {
    Pending(Box<SpanBuilder>, BoxedTracer),
    Started(BoxedSpan),
}

impl AwsSpan {
    pub fn new(aws_target: AwsTarget, operation: &str, method: &str) -> Self {
        let tracer = global::tracer("aws_sdk");
        let system = aws_target.system();
        let mut attributes = vec![
            semcov::trace::RPC_METHOD.string(method.to_string()),
            semcov::trace::RPC_SYSTEM.string("aws-api"),
            semcov::trace::RPC_SERVICE.string(system),
            KeyValue::new("aws_operation", operation.to_string()),
            KeyValue::new("db.system", system),
            KeyValue::new("db.operation", method.to_string()),
        ];
        attributes.extend(aws_target.attributes());
        let span_builder = tracer
            .span_builder(format!("aws_{system}"))
            .with_attributes(attributes)
            .with_kind(SpanKind::Client);

        Self::Pending(Box::new(span_builder), tracer)
    }

    pub fn start_with_context(self, parent_cx: &Context) -> Self {
        let span = match self {
            AwsSpan::Pending(builder, tracer) => {
                builder.start_with_context(&tracer, parent_cx)
            }
            AwsSpan::Started(_) => {
                panic!("Span already started");
            }
        };
        AwsSpan::Started(span)
    }

    pub fn start(self) -> Self {
        self.start_with_context(&Span::current().context())
    }

    pub fn end<T, E>(self, aws_response: &Result<T, E>)
    where
        T: RequestId,
        E: RequestId + Error,
    {
        let mut span = match self {
            AwsSpan::Pending(_, _) => {
                panic!("Span not started");
            }
            AwsSpan::Started(span) => span,
        };
        let (status, request_id) = match aws_response {
            Ok(resp) => (Status::Ok, resp.request_id()),
            Err(error) => {
                span.record_error(&error);
                (Status::error(error.to_string()), error.request_id())
            }
        };
        if let Some(value) = request_id {
            span.set_attribute(semcov::trace::AWS_REQUEST_ID.string(value.to_owned()));
        }
        span.set_attribute(KeyValue::new("success", status == Status::Ok));
        span.set_status(status);
    }
}
