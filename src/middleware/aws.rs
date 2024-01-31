use opentelemetry::{
    global,
    trace::{SpanKind, Tracer},
};
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;


pub use opentelemetry::{KeyValue, trace::Span};

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
        .with_kind(SpanKind::Client)
        .start_with_context(&tracer, &parent_context);   
    span.set_attribute(KeyValue::new("dynamoDB", true));
    span.set_attribute(KeyValue::new("operation", operation.to_string()));
    span.set_attribute(KeyValue::new("tableName", table_name.to_string()));
    span.set_attribute(KeyValue::new("method", method.to_string()));
    span.set_attribute(KeyValue::new("service", "AWS::DynamoDB"));

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
) -> tracing::Span {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let config = firehose_client.config();
    if let Some(region) = config.region() {
        let span = tracing::info_span!(
            target: TRACING_TARGET,
            "aws_firehose",
            firehose = tracing::field::Empty,
            operation = tracing::field::Empty,
            firehose_stream_name = tracing::field::Empty,
            method = tracing::field::Empty,
            service = tracing::field::Empty,
            cloud.region = tracing::field::Empty,
            success = tracing::field::Empty,
        );
        span.record("firehose", &"true");
        span.record("operation", &operation);
        span.record("firehose_stream_name", &firehose_stream_name);
        span.record("method", &method);
        span.record("service", "AWS::Firehose");
        span.record("cloud.region", region.as_ref());
        span
    } else {
        tracing::Span::none()
    }
}

#[cfg(any(feature = "aws", feature = "aws_sns"))]
pub fn info_span_sns(
    sns_client: &aws_sdk_sns::Client,
    operation: &str,
    method: &str,
) -> tracing::Span {
    // Spans will be sent to the configured OpenTelemetry exporter
    // use telemetry_rust::OpenTelemetrySpanExt;
    let config = sns_client.config();
    if let Some(region) = config.region() {
        let span = tracing::info_span!(
            target: TRACING_TARGET,
            "aws_sns",
            sns = tracing::field::Empty,
            operation = tracing::field::Empty,
            method = tracing::field::Empty,
            service = tracing::field::Empty,
            cloud.region = tracing::field::Empty,
            success = tracing::field::Empty,
        );
        span.record("SNS", &"true");
        span.record("operation", &operation);
        span.record("method", &method);
        span.record("service", "AWS::SNS");
        span.record("cloud.region", region.as_ref());
        span
    } else {
        tracing::Span::none()
    }
}
