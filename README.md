# telemetry-rust

OpenTelemetry instrumentation library for Rust. Provides middleware for Axum and AWS Lambda, instrumentation helpers for outbound HTTP and AWS SDK clients, and utilities for context propagation.

## Axum middleware

Requires the `axum` feature flag.

```rust
use tracing::Level::INFO;
use telemetry_rust::{
    TracerProvider, init_tracing,
    middleware::axum::OtelAxumLayer,
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

## HTTP client instrumentation

### Reqwest

Requires the `reqwest` feature flag.

```rust
use telemetry_rust::instrumentations::http::reqwest::ReqwestBuilderInstrument;

let response = reqwest::Client::new()
    .get("https://example.com/health")
    .instrument()
    .send()
    .await?;
```

### Hyper legacy client

Requires the `hyper-client-legacy` feature flag. Wraps a `hyper_util::client::legacy::Client` once and reuses it across requests.

```rust
use telemetry_rust::instrumentations::http::hyper::HyperLegacyClientInstrument;

let client = hyper_util::client::legacy::Client::builder(TokioExecutor::new())
    .build_http::<Empty<Bytes>>()
    .instrument();

let response = client
    .request(
        Request::builder()
            .uri("http://example.com/health")
            .body(Empty::<Bytes>::new())?,
    )
    .await?;
```

### Hyper low-level send

Requires the `hyper-http1` or `hyper-http2` feature flag. Wraps a per-connection `SendRequest`.

```rust
use telemetry_rust::instrumentations::http::hyper::HyperSendRequestInstrument;

let (send_request, connection) = hyper::client::conn::http1::handshake(io).await?;
tokio::spawn(async move { let _ = connection.await; });

let mut sender = send_request.instrument();
let response = sender
    .send_request(
        Request::builder()
            .uri("/health")
            .header(HOST, "example.com")
            .body(Empty::<Bytes>::new())?,
    )
    .await?;
```

## AWS SDK instrumentation

The following AWS services have full first-class support:

- DynamoDB (`aws-dynamodb`)
- SNS (`aws-sns`)
- SQS (`aws-sqs`)
- S3 (`aws-s3`)
- Firehose (`aws-firehose`)
- SageMaker Runtime (`aws-sagemaker-runtime`)
- Secrets Manager (`aws-secretsmanager`)
- SSM Parameter Store (`aws-ssm`)
- AppConfig Data (`aws-appconfigdata`)

Enable `aws-full` to turn on all of the above at once.

Each per-service feature flag enables the `AwsBuilderInstrument` trait for that service. Call `.instrument()` on any fluent builder before `.send()` — attributes are automatically extracted from both the request and response following OpenTelemetry semantic conventions.

```rust
let res = dynamo_client
    .get_item()
    .table_name("table_name")
    .set_key(primary_key)
    .instrument()
    .send()
    .await;
// Automatically extracts:
// - Request attributes from fluent builder: table name, consistent read, projection expression, etc.
// - Output attributes: consumed capacity, item found status, etc.
```

### S3 `GetObject`

`GetObject` requires special handling because the response body is a `ByteStream` transferred separately from the SDK call. `.instrument().send()` only covers the API call itself, not the body transfer.

Use `.collect()` or `.stream()` to instrument the full operation:

```rust
// `.collect()` — loads the full body into memory:
let body = s3_client
    .get_object()
    .bucket("my_bucket")
    .key("my_key")
    .instrument()
    .collect()
    .await?;

// `.stream()` — yields chunks as they arrive:
let mut stream = s3_client
    .get_object()
    .bucket("my_bucket")
    .key("my_key")
    .instrument()
    .stream()
    .await?;

// `.send()` is still available when you need the full `GetObjectOutput`:
let res = s3_client
    .get_object()
    .bucket("my_bucket")
    .key("my_key")
    .instrument()
    .send()
    .await;
let body = res.body.collect().await?; // not instrumented
```

### Paginator streams: `AwsStreamInstrument` trait

Requires the `aws-stream-instrumentation` feature flag.

Paginator streams can't use `AwsBuilderInstrument` directly. Use `.build_aws_span()` on the fluent builder (available with any per-service feature flag) to extract request attributes automatically, then pass the span builder to `.instrument()`. Response attributes are not extracted — there is no single response object for a paginated stream.

```rust
let query = dynamodb_client
    .query()
    .table_name(&table_name)
    .index_name(&index_name)
    .key_condition_expression("PK = :pk")
    .expression_attribute_values(":pk", AttributeValue::S("Hello".to_string()));

// Extracts the same request attributes as `.instrument().send()` would
let span = query.build_aws_span();

let items = query
    .into_paginator()
    .items()
    .send()
    .instrument(span)
    .try_collect::<Vec<_>>()
    .await?;
```

## AWS Lambda instrumentation

Requires the `aws-lambda` feature flag.

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

    // Tracer provider will be automatically shutdown when the runtime is dropped

    Ok(())
}
```

## Context Propagation

The following context propagation formats are supported:

- `tracecontext`: W3C Trace Context (default)
- `baggage`: W3C Baggage
- `b3`: B3 single header (requires `zipkin` feature)
- `b3multi`: B3 multiple headers (requires `zipkin` feature)
- `xray`: AWS X-Ray (requires `xray` feature)

## Advanced AWS instrumentation

### `AwsInstrument` trait

Requires the `aws-instrumentation` feature flag.

Use this when you need explicit control over span attributes — for example, to attach attributes not automatically extracted, or to instrument a service that lacks a per-service feature flag.

Call `.instrument(SpanBuilder)` on the future returned by `.send()`:

```rust
// DynamoDB
let res = dynamo_client
    .get_item()
    .table_name("table_name")
    .index_name("my_index")
    .set_key(primary_key)
    .send()
    .instrument(DynamodbSpanBuilder::get_item("table_name"))
    .await;

// SQS
let res = sqs_client
    .send_message()
    .queue_url("https://sqs.region.amazonaws.com/account/queue_name")
    .message_body("Hello World")
    .send()
    .instrument(SqsSpanBuilder::send_message("https://sqs.region.amazonaws.com/account/queue_name"))
    .await;

// SNS
let res = sns_client
    .publish()
    .topic_arn("arn:aws:sns:region:account:topic_name")
    .message("Hello World")
    .send()
    .instrument(SnsSpanBuilder::publish("arn:aws:sns:region:account:topic_name"))
    .await;

// Firehose
let res = firehose_client
    .put_record()
    .delivery_stream_name("stream_name")
    .record(record)
    .send()
    .instrument(FirehoseSpanBuilder::put_record("stream_name"))
    .await;

// S3
let res = s3_client
    .get_object()
    .bucket("my_bucket")
    .key("my_key")
    .send()
    .instrument(S3SpanBuilder::get_object("my_bucket", "my_key"))
    .await;
```

### Low-level span API

Requires the `aws-span` feature flag. Use this for AWS services not listed above, or when you need full manual control over the span lifecycle (e.g. the span must cross an async boundary).

```rust
// Dedicated constructor for a supported service
let aws_span = DynamodbSpanBuilder::get_item("table_name").start();

// Generic constructor for any AWS service
let aws_span = AwsSpanBuilder::dynamodb("GetItem", vec!["table_name"]).start();

// Explicit parent context
let context = Span::current().context();
let aws_span = DynamodbSpanBuilder::get_item("table_name").context(&context).start();

// Custom attributes
let aws_span = DynamodbSpanBuilder::get_item("table_name")
    .attribute(KeyValue::new(semconv::AWS_DYNAMODB_INDEX_NAME, "my_index"))
    .attributes([
        KeyValue::new(semconv::AWS_DYNAMODB_LIMIT, 6),
        KeyValue::new(semconv::AWS_DYNAMODB_SELECT, "ALL_ATTRIBUTES"),
    ])
    .start();
```

End the span once the operation completes:

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

For unsupported services, use the generic `AwsSpanBuilder`:

```rust
let lambda_span = AwsSpanBuilder::client(
    "Lambda",
    "Invoke",
    vec![KeyValue::new("aws.lambda.function_name", "my_function")],
)
.start();
```

## Publishing new version

New version could be published using [cargo-release](https://github.com/crate-ci/cargo-release?tab=readme-ov-file#install):

```sh
cargo release -x <level>
```
