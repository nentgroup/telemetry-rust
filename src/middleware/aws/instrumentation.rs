use async_trait::async_trait;
use aws_types::request_id::RequestId;
use futures_util::Future;
use std::error::Error;

use super::AwsOperation;

#[async_trait]
pub trait AwsInstrument<T, E>
where
    T: RequestId,
    E: RequestId + Error,
{
    async fn instrument<'a>(
        self,
        span: impl Into<AwsOperation<'a>> + Send,
    ) -> Result<T, E>;
}

#[async_trait]
impl<T, E, F> AwsInstrument<T, E> for F
where
    T: RequestId,
    E: RequestId + Error,
    F: Future<Output = Result<T, E>> + Send,
{
    async fn instrument<'a>(
        self,
        span: impl Into<AwsOperation<'a>> + Send,
    ) -> Result<T, E> {
        let span = span.into().start();
        let result = self.await;
        span.end(&result);
        result
    }
}
