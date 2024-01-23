#[cfg(any(feature = "aws", feature = "aws_dynamo"))]
pub fn info_span_dynamo(dynamo_client: aws_sdk_dynamodb::model::DynamoClient, table_name: &str, operation: &str, method: &str) -> tracing::Span {
    {
        // Spans will be sent to the configured OpenTelemetry exporter
        // use telemetry_rust::OpenTelemetrySpanExt;
        // region().await.unwrap()
        let config = dynamo_client.config();
        if let Some(region) = config.region() {
            println!("config>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> {:?}", config);
            // let http_client = config.http_client().unwrap();

            let span = tracing::info_span!(
                "aws_dynamo",
                dynamoDB = tracing::field::Empty,
                operation = tracing::field::Empty,
                tableName = tracing::field::Empty,
                method = tracing::field::Empty,
                service = tracing::field::Empty,
                cloud.region = tracing::field::Empty,
                http_client = tracing::field::Empty,
                // childSpan = tracing::field::Empty,
            );
            let _guard = span.enter();
            span.record("dynamoDB", &"true");
            span.record("operation", &operation);
            span.record("tableName", &table_name);
            span.record("method", &method);
            span.record("service", "AWS::DynamoDB");
            span.record("cloud.region", region.as_ref() );
        }
        // span.record("childSpan",  dynamo_client.config().instrument(span!(tracing::Level::INFO, "aws_dynamo")));
    }
}