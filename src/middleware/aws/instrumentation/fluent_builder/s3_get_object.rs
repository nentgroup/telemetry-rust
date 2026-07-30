//! Extended instrumentation for S3 `GetObject` operations.
//!
//! Provides additional methods to allow instrumenting the full operation, including response body transfer.
use crate::{
    future::InstrumentedFutureContext,
    middleware::aws::{InstrumentedFluentBuilder, InstrumentedFluentBuilderOutput},
    semconv,
};
use aws_sdk_s3::{
    error::SdkError,
    operation::get_object::{GetObjectError, builders::GetObjectFluentBuilder},
};
use aws_smithy_types::byte_stream::{AggregatedBytes, error::Error as ByteStreamError};
use aws_types::request_id::RequestId;
use opentelemetry::{KeyValue, trace::Status};

/// Error returned by [`InstrumentedFluentBuilder::collect`] on a [`GetObjectFluentBuilder`].
#[derive(thiserror::Error, Debug)]
pub enum GetObjectCollectError {
    /// The S3 `GetObject` request failed.
    #[error(transparent)]
    SdkError(#[from] Box<SdkError<GetObjectError>>),

    /// Reading the response body stream failed.
    #[error(transparent)]
    ByteStreamError(#[from] ByteStreamError),
}

impl InstrumentedFluentBuilder<'_, GetObjectFluentBuilder> {
    /// Sends the `GetObject` request and collects the full response body as [`AggregatedBytes`].
    ///
    /// This method instruments the **entire operation** — both the SDK call and the
    /// subsequent body transfer — within a single span. It provides accurate timing
    /// for the complete download, not just getting the initial response headers.
    ///
    /// Use this instead of calling [`InstrumentedFluentBuilder::send()`] followed by `body.collect()`
    /// when you need full tracing coverage. If you need the entire [`GetObjectOutput`] object
    /// or the raw [`ByteStream`] (i.e. to stream the body chunk-by-chunk), use `.send()` directly,
    /// but aware that the span will not include object body fetching in that case.
    ///
    /// # Errors
    ///
    /// Returns [`GetObjectCollectError::SdkError`] if the HTTP request fails, or
    /// [`GetObjectCollectError::ByteStreamError`] if reading the response body fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aws_sdk_s3::Client as S3Client;
    /// use telemetry_rust::middleware::aws::{AwsBuilderInstrument, S3SpanBuilder};
    ///
    /// async fn download_object(s3_client: &S3Client) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    ///     let body = s3_client
    ///         .get_object()
    ///         .bucket("my_bucket")
    ///         .key("my_key")
    ///         .instrument()
    ///         .collect()
    ///         .await?;
    ///     Ok(body.to_vec())
    /// }
    /// ```
    ///
    /// [`GetObjectOutput`]: aws_sdk_s3::operation::get_object::GetObjectOutput
    /// [`ByteStream`]: aws_smithy_types::byte_stream::ByteStream
    pub async fn collect(self) -> Result<AggregatedBytes, GetObjectCollectError> {
        let mut span = self.span.start();
        span.set_attribute(KeyValue::new("aws.s3.body.mode", "collect"));

        let result = self.inner.send().await;
        let Ok(output) = result else {
            span.on_result(&result);
            return Err(Box::new(result.unwrap_err()).into());
        };

        if let Some(value) = output.request_id() {
            span.set_attribute(KeyValue::new(semconv::AWS_REQUEST_ID, value.to_owned()));
        }

        span.set_attributes(output.extract_attributes());
        match output.body.collect().await {
            Ok(body) => {
                span.set_status(Status::Ok);
                Ok(body)
            }
            Err(err) => {
                span.record_error(&err);
                span.set_status(Status::error(err.to_string()));
                Err(err.into())
            }
        }
    }
}
