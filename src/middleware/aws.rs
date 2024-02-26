use aws_types::request_id::RequestId;
use opentelemetry::{
    global::{self, BoxedSpan},
    trace::{Span as TelemetrySpan, SpanKind, Status, Tracer},
    Context, KeyValue,
};
use opentelemetry_semantic_conventions as semcov;
use std::error::Error;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
// TODO: Write as macro
//
// #[instrument_aws(table_name = "cars", operation = "CreateCar", method = "Post")]
// fn create_car_in_database() {
//      info_span_dynamo();
// }

// Once this scope is closed, all spans inside are closed as well
pub fn info_span_dynamo(
    table_name: &str,
    operation: &str,
    method: &str,
    parent_context: Option<&Context>,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter

    let tracer = global::tracer("aws_sdk");
    let span = tracer
        .span_builder("aws_dynamo")
        .with_attributes(vec![
            semcov::trace::RPC_METHOD.string(method.to_string()),
            semcov::trace::RPC_SYSTEM.string("aws-api"),
            semcov::trace::RPC_SERVICE.string("DynamoDB"),
            KeyValue::new("dynamoDB", true),
            KeyValue::new("aws_operation", operation.to_string()),
            KeyValue::new("db.name", table_name.to_string()),
            KeyValue::new("db.system", "dynamodb"),
            KeyValue::new("db.operation", method.to_string()),
        ])
        .with_kind(SpanKind::Client);
    match parent_context {
        Some(ctx) => span.start_with_context(&tracer, ctx),
        None => span.start_with_context(&tracer, &Span::current().context()),
    }
}

pub fn info_span_firehose(
    firehose_stream_name: &str,
    operation: &str,
    method: &str,
    parent_context: Option<&Context>,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let tracer = global::tracer("aws_sdk");
    let span = tracer
        .span_builder("aws_firehose")
        .with_attributes(vec![
            semcov::trace::RPC_METHOD.string(method.to_string()),
            semcov::trace::RPC_SYSTEM.string("aws-api"),
            semcov::trace::RPC_SERVICE.string("Firehose"),
            KeyValue::new("firehose", true),
            KeyValue::new("firehose.name", firehose_stream_name.to_string()),
            KeyValue::new("system", "firehose"),
            KeyValue::new("operation", operation.to_string()),
        ])
        .with_kind(SpanKind::Client);
    match parent_context {
        Some(ctx) => span.start_with_context(&tracer, ctx),
        None => span.start_with_context(&tracer, &Span::current().context()),
    }
}

pub fn info_span_sns(
    topic_arn: &str,
    operation: &str,
    method: &str,
    parent_context: Option<&Context>,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let tracer = global::tracer("aws_sdk");
    let span = tracer
        .span_builder("aws_sns")
        .with_attributes(vec![
            semcov::trace::RPC_METHOD.string(method.to_string()),
            semcov::trace::RPC_SYSTEM.string("aws-api"),
            semcov::trace::RPC_SERVICE.string("SNS"),
            KeyValue::new("sns", true),
            KeyValue::new("sns.topic.arn", topic_arn.to_string()),
            KeyValue::new("system", "sns"),
            KeyValue::new("operation", operation.to_string()),
        ])
        .with_kind(SpanKind::Client);
    match parent_context {
        Some(ctx) => span.start_with_context(&tracer, ctx),
        None => span.start_with_context(&tracer, &Span::current().context()),
    }
}

pub fn end_aws_span<T, E>(mut span: BoxedSpan, aws_response: &Result<T, E>)
where
    T: RequestId,
    E: RequestId + Error,
{
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
