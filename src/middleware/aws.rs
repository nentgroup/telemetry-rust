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
#[cfg(any(feature = "aws", feature = "aws_dynamo"))]
pub fn info_span_dynamo(
    dynamo_client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    operation: &str,
    method: &str,
    parent_context: &opentelemetry::Context,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter
    let config = dynamo_client.config();

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

    if let Some(r) = config.region() {
        span.set_attribute(KeyValue::new("cloud.region", r.to_string()));
    }
    span
}

#[cfg(any(feature = "aws", feature = "aws_firehose"))]
pub fn info_span_firehose(
    firehose_client: &aws_sdk_firehose::Client,
    firehose_stream_name: &str,
    operation: &str,
    method: &str,
    parent_context: &opentelemetry::Context,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let config = firehose_client.config();

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

    if let Some(r) = config.region() {
        span.set_attribute(KeyValue::new("cloud.region", r.to_string()));
    }
    span
}

#[cfg(any(feature = "aws", feature = "aws_sns"))]
pub fn info_span_sns(
    sns_client: &aws_sdk_sns::Client,
    operation: &str,
    method: &str,
    parent_context: &opentelemetry::Context,
) -> opentelemetry::global::BoxedSpan {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let config = sns_client.config();

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

    if let Some(r) = config.region() {
        span.set_attribute(KeyValue::new("cloud.region", r.to_string()));
    }
    span
}
