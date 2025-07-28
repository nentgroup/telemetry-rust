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

use crate::middleware::aws::{AwsSpan, AwsSpanBuilder};

struct NoOp;

impl RequestId for NoOp {
    fn request_id(&self) -> Option<&str> {
        None
    }
}

#[derive(Default)]
enum InstrumentedStreamState<'a> {
    Waiting(Box<AwsSpanBuilder<'a>>),
    Flowing(AwsSpan),
    Finished,
    #[default]
    Invalid,
}

impl<'a> InstrumentedStreamState<'a> {
    fn new(span: impl Into<AwsSpanBuilder<'a>>) -> Self {
        Self::Waiting(Box::new(span.into()))
    }
}

pin_project! {
    pub struct InstrumentedStream<'a, S: Stream> {
        #[pin]
        inner: S,
        state: Cell<InstrumentedStreamState<'a>>,
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
        let state = this.state.take();
        match state {
            InstrumentedStreamState::Waiting(span) => {
                this.state
                    .set(InstrumentedStreamState::Flowing(span.start()));
                this.inner.poll_next(cx)
            }
            InstrumentedStreamState::Flowing(aws_span) => {
                match this.inner.poll_next(cx) {
                    Poll::Ready(None) => {
                        aws_span.end::<NoOp, E>(&Ok(NoOp));
                        this.state.set(InstrumentedStreamState::Finished);
                        Poll::Ready(None)
                    }
                    Poll::Ready(Some(Err(err))) => {
                        let aws_result = Err(err);
                        aws_span.end::<NoOp, E>(&aws_result);
                        this.state.set(InstrumentedStreamState::Finished);
                        Poll::Ready(aws_result.err().map(Err))
                    }
                    result => {
                        this.state.set(InstrumentedStreamState::Flowing(aws_span));
                        result
                    }
                }
            }
            InstrumentedStreamState::Finished => {
                this.state.set(state);
                Poll::Ready(None)
            }
            InstrumentedStreamState::Invalid => {
                panic!("Invalid instrumented stream state")
            }
        }
    }
}

pub trait AwsStreamInstrument<T, E, S>
where
    E: RequestId + Error,
    S: Stream<Item = Result<T, E>>,
{
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
            state: Cell::new(InstrumentedStreamState::new(span)),
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
