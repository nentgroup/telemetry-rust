use aws_types::request_id::RequestId;
use std::{error::Error, future::Future};

use super::{AwsSpan, AwsSpanBuilder};
use crate::future::HookedFuture;

pub trait AwsInstrument<T, E, F>
where
    T: RequestId,
    E: RequestId + Error,
    F: Future<Output = Result<T, E>>,
{
    fn instrument<'a>(
        self,
        span: impl Into<AwsSpanBuilder<'a>>,
    ) -> HookedFuture<F, AwsSpan>;
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
    ) -> HookedFuture<F, AwsSpan> {
        let span = span.into().start();
        HookedFuture::new(self, span, |span, result| span.end(result))
    }
}
