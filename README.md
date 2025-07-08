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
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        // start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default());

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

```rust
#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    // Grab TracerProvider after telemetry initialisation
    let provider = telemetry_rust::init_tracing!(tracing::Level::WARN);

    // Create lambda telemetry layer
    let telemetry_layer = telemetry_rust::middleware::lambda::OtelLambdaLayer::new(provider);

    // Run lambda runtime with telemetry layer
    lambda_runtime::Runtime::new(tower::service_fn(handler))
        .layer(telemetry_layer)
        .run()
        .await?;

    // Tracer provider will be automatically shutdown when the layer is dropped

    Ok(())
}
```

## Publishing new version

New version could be published using [cargo-release](https://github.com/crate-ci/cargo-release?tab=readme-ov-file#install):

```sh
cargo release -x <level>
```
