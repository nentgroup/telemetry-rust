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

pub trait AwsInstrument<T, E, F>
where
    T: RequestId,
    E: RequestId + Error,
    F: Future<Output = Result<T, E>>,
{
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
