use opentelemetry::{
    global,
    trace::{SpanKind, Tracer},
    Context,
};
pub use opentelemetry::{trace::Span, KeyValue};
pub use opentelemetry_semantic_conventions as semcov;
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
        None => span.start_with_context(&tracer, &tracing::Span::current().context()),
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
        None => span.start_with_context(&tracer, &tracing::Span::current().context()),
    }
}

pub fn info_span_sns(
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
            KeyValue::new("system", "sns"),
            KeyValue::new("operation", operation.to_string()),
        ])
        .with_kind(SpanKind::Client);
    // .start_with_context(&tracer, parent_context);
    match parent_context {
        Some(ctx) => span.start_with_context(&tracer, ctx),
        None => span.start_with_context(&tracer, &tracing::Span::current().context()),
    }
}
