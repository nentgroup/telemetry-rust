# telemetry-rust

```rust
use tracing::Level::INFO;
// middleware::axum is available if feature flag axum is on
use telemetry_rust::{init_tracing, middleware::axum::{
    OtelAxumLayer, OtelInResponseLayer
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

Creating new span:

```rust
// create new span directly by using current span context
let aws_span = AwsSpan::new(AwsTarget::Dynamo("table_name"), "TransactGetItems");

// or by providing an explicit parent context
let context = Span::current().context();
let aws_span = AwsSpan::with_context(AwsTarget::Dynamo("table_name"), "TransactGetItems", &context);

// or build it using builder pattern
let builder = AwsSpan::build(AwsTarget::Dynamo("table_name"), "TransactGetItems")
let aws_span = builder.start();
let aws_span = builder.start_with_context(&context);
```

Ending the span once AWS operation is complete:

```rust
let res = dynamo_client
    .transact_get_items()
    .set_transact_items(Some(transact_items))
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
