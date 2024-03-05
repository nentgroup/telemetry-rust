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
    .instrument(AwsSpan::build(
        AwsTarget::Dynamo("table_name"),
        "GetItem",
    ))
    .await;
```

### Low level API

Creating new span:

```rust
// create new span directly by using current span context
let aws_span = AwsSpan::new(AwsTarget::Dynamo("table_name"), "GetItem");

// or by providing an explicit parent context
let context = Span::current().context();
let aws_span = AwsSpan::with_context(AwsTarget::Dynamo("table_name"), "GetItem", &context);

// or build it using builder pattern
let builder = AwsSpan::build(AwsTarget::Dynamo("table_name"), "GetItem")
    .set_attribute(semcov::AWS_DYNAMODB_INDEX_NAME.string("my_index"));
// and manually start it using either start or start_with_context
let aws_span = builder.start();
let aws_span = builder.context(&context).start();
let aws_span = builder.start_with_context(&context);
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

Defining a custom aws target:

```rust
struct S3Target {}
impl IntoAttributes for S3Target {
    fn service(&self) -> &'static str {
        "s3"
    }

    fn into_attributes(self, _method: &'static str) -> Vec<KeyValue> {
        vec![semcov::AWS_S3_BUCKET.string("my_bucket")]
    }
}
```

```rust
let s3_span = AwsSpan::new(S3Target {}, "GetObject");
```
