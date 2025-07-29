# telemetry-rust

```rust
use tracing::Level::INFO;
// middleware::axum is available if feature flag axum is on
use telemetry_rust::{
    TracerProvider, init_tracing,
    middleware::axum::{OtelAxumLayer, OtelInResponseLayer},
    shutdown_tracer_provider,
};

#[tracing::instrument]
async fn route_otel() -> impl axum::response::IntoResponse {
    let trace_id =
        telemetry_rust::tracing_opentelemetry_instrumentation_sdk::find_current_trace_id();
    dbg!(&trace_id);
    axum::Json(serde_json::json!({ "trace-id": trace_id }))
}

#[tokio::main]
async fn main() {
    let provider: TracerProvider = init_tracing!(INFO);

    // ...

    let app = axum::Router::new()
        // request processed inside span
        .route("/otel", axum::routing::get(route_otel))
        // start OpenTelemetry trace on incoming request + include trace context as header into the response
        .layer(OtelAxumLayer::new(axum::extract::MatchedPath::as_str).inject_context(true));

    // ...
}

async fn graceful_shutdown(provider: TracerProvider) {
    // ...
    shutdown_tracer_provider(&provider);
}
```

## AWS SDK instrumentation

### `AwsInstrumented` trait

```rust
let res = dynamo_client
    .get_item()
    .table_name("table_name")
    .index_name("my_index")
    .set_key(primary_key)
    .send()
    .instrument(DynamodbSpanBuilder::get_item("table_name"))
    .await;
```

### Low level API

Creating new span:

```rust
// create new span in the current span's context using either a dedicated constructor
let aws_span = DynamodbSpanBuilder::get_item("table_name").start();
// or a generic one
let aws_span = AwsSpanBuilder::dynamodb("GetItem", vec!["table_name"]).start();

// optionally, provide an explicit parent context
let context = Span::current().context();
let aws_span = DynamodbSpanBuilder::get_item("table_name").context(&context).start();

// or set custom span attributes
let aws_span = DynamodbSpanBuilder::get_item("table_name")
    .attribute(KeyValue::new(semconv::AWS_DYNAMODB_INDEX_NAME, "my_index"))
    .attributes(vec![
        KeyValue::new(semconv::AWS_DYNAMODB_LIMIT, 6),
        KeyValue::new(semconv::AWS_DYNAMODB_SELECT, "ALL_ATTRIBUTES"),
    ])
    .start();
```

Ending the span once AWS operation is complete:

```rust
let res = dynamo_client
    .get_item()
    .table_name("table_name")
    .index_name("my_index")
    .set_key(primary_key)
    .send()
    .await;
aws_span.end(&res);
```

Only the following AWS targets are fully supported at the moment:

 * DynamoDB
 * SNS
 * Firehose

But a generic `AwsSpanBuilder` could be used to instrument any other AWS SDK:

```rust
let s3_span = AwsSpanBuilder::client(
    "S3",
    "GetObject",
    vec![KeyValue::new(semconv::AWS_S3_BUCKET, "my_bucket")],
)
.start();
```

## AWS Lambda instrumentation

### [Generic](https://opentelemetry.io/docs/specs/semconv/faas/faas-spans/#other) layer

Generic lambda layer could be created using either `OtelLambdaLayer::new` or `OtelLambdaLayer::other` factory functions.

```rust
#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // Grab TracerProvider after telemetry initialisation
    let provider = telemetry_rust::init_tracing!(tracing::Level::WARN);

    // Create a generic lambda telemetry layer
    let telemetry_layer = telemetry_rust::middleware::lambda::OtelLambdaLayer::new(provider);

    // Run lambda runtime with telemetry layer
    lambda_runtime::Runtime::new(tower::service_fn(handler))
        .layer(telemetry_layer)
        .run()
        .await?;

    // Tracer provider will be automatically shutdown when the runtime is dropped

    Ok(())
}
```

Generic layer could be used for all kinds of lambdas, but it is recommended to use a dedicated layer when possible.

### [PubSub](https://opentelemetry.io/docs/specs/semconv/faas/faas-spans/#pubsub) layer

PubSub layer could be used when the lambda is triggered by some event, i.e. when it's subscribed to Kinesis Data Streams or DynamoDB Streams.

```rust
let pubsub_telemetry_layer = OtelLambdaLayer::pubsub(
    provider,
    // The messaging system
    "AmazonKinesis",
    // The message destination arn or unique name
    Some("arn:aws:kinesis:us-east-2:123456789012:stream/mystream"),
);
```

[SQS](https://opentelemetry.io/docs/specs/semconv/faas/aws-lambda/#sqs) and SNS layers could be created using their own factory functions for convenience:

```rust
let sqs_telemetry_layer = OtelLambdaLayer::sqs(
    provider,
    Some("arn:aws:sqs:us-east-2:123456789012:MyQueue"),
);
let sns_telemetry_layer = OtelLambdaLayer::sns(
    provider,
    Some("arn:aws:sns:us-east-2:123456789012:MyTopic"),
);
```

### [Datasource](https://opentelemetry.io/docs/specs/semconv/faas/faas-spans/#datasource) layer

Datasource layer could be used when the lambda is invoked in response to some data source operation such as a database or filesystem read/write.

It's recommended to use Datasource layer when processing Amazon Simple Storage Service event notifications.

```rust
let s3_telemetry_layer = OtelLambdaLayer::datasource(
    provider,
    // The name of the source on which the triggering operation was performed
    "myBucketName",
    // The type of the operation that was performed on the data (usually "insert", "edit" or "delete")
    "edit",
    // The document name/table subjected to the operation
    Some("/myFolder/myFile.txt"),
);
```

Even though DynamoDB is a data source, it's recommended to use a `pubsub` layer when processing DynamoDB Streams events.

### [Timer](https://opentelemetry.io/docs/specs/semconv/faas/faas-spans/#timer) layer

Timer layer could be used when the lambda is invoked periodically by the Amazon EventBridge Scheduler.

```rust
let cron_telemetry_layer = OtelLambdaLayer::timer(
    provider,
    // The schedule period as Cron Expression
    Some("0/5 * * * ? *"),
);
```

### [HTTP](https://opentelemetry.io/docs/specs/semconv/faas/faas-spans/#http) layer

Tracing for [API Gateway](https://opentelemetry.io/docs/specs/semconv/faas/aws-lambda/#api-gateway) events is not fully supported since that would require extracting tracking metadata from the event payload, but parsing event body is not supported by the `OtelLambdaLayer` implementation.

Though it's still possible to create a simple HTTP layer to report the correct trigger type:

```rust
let http_telemetry_layer = OtelLambdaLayer::http(provider);
```

## Publishing new version

New version could be published using [cargo-release](https://github.com/crate-ci/cargo-release?tab=readme-ov-file#install):

```sh
cargo release -x <level>
```
