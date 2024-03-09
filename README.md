# telemetry-rust

```rust
use tracing::Level::INFO;
// middleware::axum is available if feature flag axum is on
use telemetry_rust::{
    init_tracing,
    middleware::axum::{OtelAxumLayer, OtelInResponseLayer},
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
    init_tracing!(INFO);

    // ...

    let app = axum::Router::new()
        // request processed inside span
        .route("/otel", axum::routing::get(route_otel))
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        // start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default());

    // ...
```

## AWS instrumentation

### `AwsInstrumented` trait

```rust
let res = dynamo_client
    .get_item()
    .table_name("table_name")
    .index_name("my_index")
    .set_key(primary_key)
    .send()
    .instrument(DynamoDBOperation::get_item("table_name"))
    .await;
```

### Low level API

Creating new span:

```rust
// create new span in the current span's context using either a dedicated constructor
let aws_span = DynamoDBOperation::get_item("table_name").start();
// or a generic one
let aws_span = DynamoDBOperation::new("GetItem", "table_name").start();

// optionally, provide an explicit parent context
let context = Span::current().context();
let aws_span = DynamoDBOperation::get_item("table_name").context(&context).start();

// or set custom span attributes
let builder = DynamoDBOperation::get_item("table_name")
    .attribute(semcov::AWS_DYNAMODB_INDEX_NAME.string("my_index"))
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

But a generic `AwsOperation` could be used to instrument any other AWS SDK:

```rust
let s3_span = AwsOperation::client(
    "S3",
    "GetObject",
    vec![semcov::AWS_S3_BUCKET.string("my_bucket")],
)
.start();
```
