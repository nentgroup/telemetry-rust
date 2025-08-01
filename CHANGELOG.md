# Changelog

## v6.0.0

- New AWS SDK instrumentation with automatic attribute extraction https://github.com/nentgroup/telemetry-rust/pull/140

This release is not breaking, but `aws-full` feature now adds AWS SDK dependencies.

## v5.3.0

- Add missing DynamoDB operations instrumentation https://github.com/nentgroup/telemetry-rust/pull/138
- Support SQS: https://github.com/nentgroup/telemetry-rust/pull/139

## v5.2.0

- Implement `AwsStreamInstrument` trait to instrument AWS pagination streams https://github.com/nentgroup/telemetry-rust/pull/134

## v5.1.1

- Fix reading `service.name` and `service.version` from `OTEL_RESOURCE_ATTRIBUTES` https://github.com/nentgroup/telemetry-rust/pull/132

## v5.1.0

- Update `MessagingOperationKind` to follow the latest Semantic conventions spec https://github.com/nentgroup/telemetry-rust/pull/129
- Make `telemetry_rust::test::TracedResponse` more generic https://github.com/nentgroup/telemetry-rust/pull/130
- Add documentation https://github.com/nentgroup/telemetry-rust/pull/128

## v5.0.0

- **breaking:** New axum version agnostic way to construct `OtelAxumLayer` https://github.com/nentgroup/telemetry-rust/pull/123

Use

```rust
OtelAxumLayer::new(axum::extract::MatchedPath::as_str)
```

instead of

```rust
OtelAxumLayer::default()
```

- **breaking:** Deprecate `OtelInResponseLayer` https://github.com/nentgroup/telemetry-rust/pull/124

Instead use `OtelAxumLayer` with `inject_context` set to `true`

```rust
OtelAxumLayer::new(MatchedPath::as_str).inject_context(true)
```

## v4.1.0

- Add `trace_id` and `span_id` to json logs https://github.com/nentgroup/telemetry-rust/pull/126

## v4.0.0

- **breaking:** Update `axum` to `0.8` https://github.com/nentgroup/telemetry-rust/pull/100

## v3.7.0

- Update opentelemetry SDK to `0.30` https://github.com/nentgroup/telemetry-rust/pull/109

## v3.6.0

- Update opentelemetry SDK to `0.27` https://github.com/nentgroup/telemetry-rust/pull/77 & https://github.com/nentgroup/telemetry-rust/pull/91

## v3.5.0

- Update opentelemetry SDK to `0.26` https://github.com/nentgroup/telemetry-rust/pull/63

## v3.4.0

- Follow latest Semantic Conventions https://github.com/nentgroup/telemetry-rust/pull/66

## v3.3.0

- Update opentelemetry SDK to `0.25` https://github.com/nentgroup/telemetry-rust/pull/54

## v3.2.0

- Update opentelemetry SDK to `0.24` https://github.com/nentgroup/telemetry-rust/pull/51

## v3.1.1

- Update dependencies

## v3.1.0

- Support AWS lambdas https://github.com/nentgroup/telemetry-rust/pull/29

## v3.0.0

- Update dependencies and toolchain

## v2.0.0

- New interface for AWS instrumentation https://github.com/nentgroup/telemetry-rust/pull/15

## v1.5.0

- Implement `AwsInstrument` trait to instrument AWS operations https://github.com/nentgroup/telemetry-rust/pull/13

## v1.4.0

- Experimental support fir AWS spans https://github.com/nentgroup/telemetry-rust/pull/11

## v1.3.0

- Fix performance issues https://github.com/nentgroup/telemetry-rust/pull/9

## v1.2.0

- Handle `OTEL_RESOURCE_ATTRIBUTES` and `OTEL_PROPAGATORS` Env vars https://github.com/nentgroup/telemetry-rust/pull/8

## v1.1.0

- Support http protocol for sending traces https://github.com/nentgroup/telemetry-rust/pull/7

## v1.0.1

- Correction on shutdown_signal function.

## v1.0.0

- Lambda support
- Support to hyper 1.0.0 and axum 0.7.3

## v0.0.3

- Support to axum 0.6.x
