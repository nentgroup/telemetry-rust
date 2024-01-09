# telemetry-rust

```rust
use tracing::Level::INFO;
use telemetry_rust::{init_tracing, middleware::axum::OtelAxumLayer};

#[tokio::main]
async fn main() {
    init_tracing!(INFO);

    // ...

    let app = axum::Router::new().layer(OtelAxumLayer::default())
```
