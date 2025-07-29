use aws_smithy_async::future::pagination_stream::PaginationStream;
use aws_smithy_types_convert::stream::{PaginationStreamExt, PaginationStreamImplStream};
use aws_types::request_id::RequestId;
use futures_util::Stream;
use pin_project_lite::pin_project;
use std::{
    cell::Cell,
    error::Error,
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    KeyValue,
    middleware::aws::{AwsSpan, AwsSpanBuilder},
};

/// A no-op implementation of [`RequestId`] for internal use.
///
/// This is used in place of an AWS response to satisfy [`AwsSpan::end`] trait bounds,
/// because we don't have access to the real response with request ID information.
struct Void;

impl RequestId for Void {
    fn request_id(&self) -> Option<&str> {
        None
    }
}

enum StreamStateKind {
    Waiting,
    Flowing,
    Finished,
}

#[derive(Default)]
enum StreamState<'a> {
    Waiting(Box<AwsSpanBuilder<'a>>),
    Flowing(AwsSpan),
    Finished,
    #[default]
    Invalid,
}

impl<'a> StreamState<'a> {
    fn new(span: impl Into<AwsSpanBuilder<'a>>) -> Self {
        let span = Into::<AwsSpanBuilder>::into(span);
        Self::Waiting(Box::new(
            span.attribute(KeyValue::new("aws.pagination_stream", true)),
        ))
    }

    fn kind(&self) -> StreamStateKind {
        match self {
            StreamState::Waiting(_) => StreamStateKind::Waiting,
            StreamState::Flowing(_) => StreamStateKind::Flowing,
            StreamState::Finished => StreamStateKind::Finished,
            StreamState::Invalid => {
                panic!("Invalid instrumented stream state")
            }
        }
    }

    fn start(self) -> Self {
        let Self::Waiting(span) = self else {
            panic!("Instrumented stream state is not Waiting");
        };
        Self::Flowing(span.start())
    }

    fn end<E: RequestId + Error>(self, aws_response: &Result<Void, E>) -> Self {
        let Self::Flowing(span) = self else {
            panic!("Instrumented stream state is not Flowing");
        };
        span.end(aws_response);
        Self::Finished
    }
}

pin_project! {
    /// A wrapper around a Stream that provides OpenTelemetry instrumentation for AWS operations.
    ///
    /// This struct automatically creates spans for stream operations and handles proper
    /// span lifecycle management including error handling and completion tracking.
    ///
    /// The instrumented stream automatically adds the `aws.pagination_stream = true` attribute
    /// to help identify pagination/streaming operations in traces.
    ///
    /// The instrumented stream maintains state to track the span lifecycle:
    /// - `Waiting`: Initial state with a span builder ready to start
    /// - `Flowing`: Active state with an ongoing span
    /// - `Finished`: Terminal state after the stream completes or errors
    pub struct InstrumentedStream<'a, S: Stream> {
        #[pin]
        inner: S,
        state: Cell<StreamState<'a>>,
    }
}

impl<T, E, S> Stream for InstrumentedStream<'_, S>
where
    E: RequestId + Error,
    S: Stream<Item = Result<T, E>>,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.state.get_mut().kind() {
            StreamStateKind::Waiting => {
                this.state.set(this.state.take().start());
                this.inner.poll_next(cx)
            }
            StreamStateKind::Flowing => match this.inner.poll_next(cx) {
                Poll::Ready(None) => {
                    this.state.set(this.state.take().end(&Ok::<_, E>(Void)));
                    Poll::Ready(None)
                }
                Poll::Ready(Some(Err(err))) => {
                    let aws_result = Err(err);
                    this.state.set(this.state.take().end(&aws_result));
                    Poll::Ready(aws_result.err().map(Err))
                }
                result => result,
            },
            StreamStateKind::Finished => Poll::Ready(None),
        }
    }
}

/// A trait for adding OpenTelemetry instrumentation to AWS pagination streams.
///
/// This trait provides the `instrument` method that wraps streams with telemetry
/// capabilities, automatically creating and managing spans for AWS operations.
/// It is designed for AWS pagination streams, but it is implemented for any
/// [`TryStream`][`futures_util::TryStream`] yielding AWS SDK errors.
///
/// All instrumented streams automatically include the `aws.pagination_stream = true`
/// attribute to help identify streaming operations in traces.
///
/// # Examples
///
/// ```rust
/// use aws_sdk_dynamodb::{Client as DynamoClient, types::AttributeValue};
/// use futures_util::TryStreamExt;
/// use telemetry_rust::{
///     KeyValue,
///     middleware::aws::{AwsStreamInstrument, DynamodbSpanBuilder},
///     semconv,
/// };
///
/// async fn query_table() -> Result<usize, Box<dyn std::error::Error>> {
///     let config = aws_config::load_from_env().await;
///     let dynamo_client = DynamoClient::new(&config);
///     let items =
///         dynamo_client
///             .query()
///             .table_name("table_name")
///             .index_name("my_index")
///             .key_condition_expression("PK = :pk")
///             .expression_attribute_values(":pk", AttributeValue::S("Test".to_string()))
///             .into_paginator()
///             .items()
///             .send()
///             .instrument(DynamodbSpanBuilder::query("table_name").attribute(
///                 KeyValue::new(semconv::AWS_DYNAMODB_INDEX_NAME, "my_index"),
///             ))
///             .try_collect::<Vec<_>>()
///             .await?;
///     println!("DynamoDB items: {items:#?}");
///     Ok(items.len())
/// }
/// ```
pub trait AwsStreamInstrument<T, E, S>
where
    E: RequestId + Error,
    S: Stream<Item = Result<T, E>>,
{
    /// Instruments the stream with OpenTelemetry tracing.
    ///
    /// This method wraps a [`TryStream`][`futures_util::TryStream`] in an [`InstrumentedStream`] that will:
    /// - Start a span when the stream begins polling
    /// - End the span with success when the stream completes normally
    /// - End the span with error information if the stream encounters an error
    ///
    /// # Arguments
    ///
    /// * `span` - The span builder or span configuration to use for instrumentation
    ///
    /// # Returns
    ///
    /// An [`InstrumentedStream`] that wraps the original stream with telemetry capabilities.
    fn instrument<'a>(
        self,
        span: impl Into<AwsSpanBuilder<'a>>,
    ) -> InstrumentedStream<'a, S>;
}

impl<T, E, S> AwsStreamInstrument<T, E, S> for S
where
    E: RequestId + Error,
    S: Stream<Item = Result<T, E>>,
{
    fn instrument<'a>(
        self,
        span: impl Into<AwsSpanBuilder<'a>>,
    ) -> InstrumentedStream<'a, S> {
        InstrumentedStream {
            inner: self,
            state: Cell::new(StreamState::new(span)),
        }
    }
}

impl<T, E> AwsStreamInstrument<T, E, PaginationStreamImplStream<Result<T, E>>>
    for PaginationStream<Result<T, E>>
where
    E: RequestId + Error,
{
    fn instrument<'a>(
        self,
        span: impl Into<AwsSpanBuilder<'a>>,
    ) -> InstrumentedStream<'a, PaginationStreamImplStream<Result<T, E>>> {
        self.into_stream_03x().instrument(span)
    }
}
