use async_trait::async_trait;
use aws_types::request_id::RequestId;
use futures_util::Future;
use std::error::Error;

use super::AwsSpanBuilder;

#[async_trait]
pub trait AwsInstrumented<T, E>
where
    T: RequestId,
    E: RequestId + Error,
{
    async fn instrument<'a>(self, span: AwsSpanBuilder<'a>) -> Result<T, E>;
}

#[async_trait]
impl<T, E, F> AwsInstrumented<T, E> for F
where
    T: RequestId,
    E: RequestId + Error,
    F: Future<Output = Result<T, E>> + Send,
{
    async fn instrument<'a>(self, span: AwsSpanBuilder<'a>) -> Result<T, E> {
        let span = span.start();
        let result = self.await;
        span.end(&result);
        result
    }
}
