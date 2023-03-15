# telemetry-rust

Telemetry rust provides all logic setled up for start tracing and also TraceLayer provided by [axum-tracing-opentelemetry](https://github.com/davidB/axum-tracing-opentelemetry)

```rust
use tracing::Level::INFO;
use telemetry_rust::{init_tracing, opentelemetry_tracing_layer};

#[tokio::main]
async fn main() {
    init_tracing(INFO);

    // ...

    let app = axum::Router::new().layer(opentelemetry_tracing_layer())
```
