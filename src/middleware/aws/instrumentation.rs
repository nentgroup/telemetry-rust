use aws_types::request_id::RequestId;
use std::{error::Error, future::Future};

use super::{AwsSpan, AwsSpanBuilder};
use crate::future::{InstrumentedFuture, InstrumentedFutureContext};

impl<T, E> InstrumentedFutureContext<Result<T, E>> for AwsSpan
where
    T: RequestId,
    E: RequestId + Error,
{
    fn on_result(self, result: &Result<T, E>) {
        self.end(result);
    }
}

/// Trait for instrumenting AWS futures with automatic span management.
///
/// This trait provides a convenient way to wrap AWS SDK futures with OpenTelemetry
/// instrumentation, automatically handling span creation, error recording, and cleanup.
///
/// # Example
///
/// ```rust
/// use aws_sdk_dynamodb::{Client as DynamoClient, types::AttributeValue};
/// use telemetry_rust::middleware::aws::{AwsInstrument, DynamodbSpanBuilder};
///
/// async fn query_table() {
///     let config = aws_config::load_from_env().await;
///     let dynamo_client = DynamoClient::new(&config);
///     let res = dynamo_client
///         .query()
///         .table_name("table_name")
///         .index_name("my_index")
///         .key_condition_expression("PK = :pk")
///         .expression_attribute_values(":pk", AttributeValue::S("Test".to_string()))
///         .send()
///         .instrument(DynamodbSpanBuilder::get_item("table_name"))
///         .await;
///     println!("DynamoDB response: {res:#?}");
/// }
/// ```
pub trait AwsInstrument<T, E, F>
where
    T: RequestId,
    E: RequestId + Error,
    F: Future<Output = Result<T, E>>,
{
    /// Instruments the future with an AWS span.
    ///
    /// Creates an instrumented future that will automatically start a span when polled
    /// and properly handle the result when the future completes.
    ///
    /// # Arguments
    ///
    /// * `span` - A span builder or span to use for instrumentation
    ///
    /// # Returns
    ///
    /// An instrumented future that will record AWS operation details
    fn instrument<'a>(
        self,
        span: impl Into<AwsSpanBuilder<'a>>,
    ) -> InstrumentedFuture<F, AwsSpan>;
}

impl<T, E, F> AwsInstrument<T, E, F> for F
where
    T: RequestId,
    E: RequestId + Error,
    F: Future<Output = Result<T, E>>,
{
    fn instrument<'a>(
        self,
        span: impl Into<AwsSpanBuilder<'a>>,
    ) -> InstrumentedFuture<F, AwsSpan> {
        let span = span.into().start();
        InstrumentedFuture::new(self, span)
    }
}
