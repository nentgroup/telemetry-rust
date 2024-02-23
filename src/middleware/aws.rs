use opentelemetry::{
    global,
    trace::{SpanKind, Tracer},
};
pub use opentelemetry::{trace::Span, KeyValue};
pub use opentelemetry_semantic_conventions as semcov;

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
    parent_context: &opentelemetry::Context,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter

    let tracer = global::tracer("aws_sdk");
    let mut span = tracer
        .span_builder("aws_dynamo")
        .with_attributes(vec![
            semcov::trace::RPC_METHOD.string(method.to_string()),
            semcov::trace::RPC_SYSTEM.string("aws-api"),
            semcov::trace::RPC_SERVICE.string("DynamoDB"),
        ])
        .with_kind(SpanKind::Client)
        .start_with_context(&tracer, parent_context);

    span.set_attribute(KeyValue::new("dynamoDB", true));
    span.set_attribute(KeyValue::new("aws_operation", operation.to_string()));
    span.set_attribute(KeyValue::new("db.name", table_name.to_string()));
    span.set_attribute(KeyValue::new("db.system", "dynamodb"));
    span.set_attribute(KeyValue::new("db.operation", method.to_string()));
    span
}

#[cfg(any(feature = "aws", feature = "aws_firehose"))]
pub fn info_span_firehose(
    firehose_stream_name: &str,
    operation: &str,
    method: &str,
    parent_context: &opentelemetry::Context,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let tracer = global::tracer("aws_sdk");
    let mut span = tracer
        .span_builder("aws_firehose")
        .with_attributes(vec![
            semcov::trace::RPC_METHOD.string(method.to_string()),
            semcov::trace::RPC_SYSTEM.string("aws-api"),
            semcov::trace::RPC_SERVICE.string("Firehose"),
        ])
        .with_kind(SpanKind::Client)
        .start_with_context(&tracer, parent_context);

    span.set_attribute(KeyValue::new("firehose", true));
    span.set_attribute(KeyValue::new(
        "firehose.name",
        firehose_stream_name.to_string(),
    ));
    span.set_attribute(KeyValue::new("system", "firehose"));
    span.set_attribute(KeyValue::new("operation", operation.to_string()));
    span
}

pub fn info_span_sns(
    operation: &str,
    method: &str,
    parent_context: &opentelemetry::Context,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let tracer = global::tracer("aws_sdk");
    let mut span = tracer
        .span_builder("aws_sns")
        .with_attributes(vec![
            semcov::trace::RPC_METHOD.string(method.to_string()),
            semcov::trace::RPC_SYSTEM.string("aws-api"),
            semcov::trace::RPC_SERVICE.string("SNS"),
        ])
        .with_kind(SpanKind::Client)
        .start_with_context(&tracer, parent_context);

    span.set_attribute(KeyValue::new("sns", true));
    span.set_attribute(KeyValue::new("system", "sns"));
    span.set_attribute(KeyValue::new("operation", operation.to_string()));

    span
}
