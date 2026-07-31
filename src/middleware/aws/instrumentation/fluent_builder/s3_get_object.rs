//! Extended instrumentation for S3 `GetObject` operations.
//!
//! Provides additional methods to allow instrumenting the full operation, including response body transfer:
//! - [`InstrumentedFluentBuilder::collect()`] — buffers the entire body into [`AggregatedBytes`]
//! - [`InstrumentedFluentBuilder::stream()`] — yields body chunks via [`InstrumentedByteStream`]
use bytes::Bytes;
use futures_util::Stream;
use pin_project_lite::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::{
    future::InstrumentedFutureContext,
    middleware::aws::{
        AwsSpan, InstrumentedFluentBuilder, InstrumentedFluentBuilderOutput,
    },
    semconv,
};
use aws_sdk_s3::{
    error::SdkError,
    operation::get_object::{GetObjectError, builders::GetObjectFluentBuilder},
};
use aws_smithy_types::byte_stream::{
    AggregatedBytes, ByteStream, error::Error as ByteStreamError,
};
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

pin_project! {
    /// An instrumented S3 `GetObject` response body stream.
    ///
    /// Created by [`InstrumentedFluentBuilder::stream()`] on a [`GetObjectFluentBuilder`].
    /// Implements [`Stream`]`<Item = Result<`[`Bytes`]`, `[`ByteStreamError`]`>>`, yielding the
    /// response body chunk by chunk.
    ///
    /// The associated span ends when the stream is exhausted or an error is encountered.
    /// If the stream is dropped before completion, the span ends via `Drop` with no explicit
    /// status set (remains `Status::Unset`).
    pub struct InstrumentedByteStream {
        #[pin]
        inner: ByteStream,
        span: Option<AwsSpan>,
    }
}

impl InstrumentedByteStream {
    fn new(body: ByteStream, span: AwsSpan) -> Self {
        Self {
            inner: body,
            span: Some(span),
        }
    }
}

impl Stream for InstrumentedByteStream {
    type Item = Result<Bytes, ByteStreamError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match this.inner.poll_next(cx) {
            Poll::Ready(None) => {
                if let Some(mut span) = this.span.take() {
                    span.set_status(Status::Ok);
                }
                Poll::Ready(None)
            }
            Poll::Ready(Some(Err(err))) => {
                if let Some(mut span) = this.span.take() {
                    span.record_error(&err);
                    span.set_status(Status::error(err.to_string()));
                }
                Poll::Ready(Some(Err(err)))
            }
            other => other,
        }
    }
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
    /// (e.g. to inspect response headers or metadata), use `.send()` directly,
    /// but be aware that the span will not cover body transfer in that case.
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
    /// use telemetry_rust::middleware::aws::AwsBuilderInstrument;
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

    /// Sends the `GetObject` request and returns an instrumented stream of the response body.
    ///
    /// Unlike [`collect`][Self::collect], this method does not buffer the body in memory.
    /// Instead it returns an [`InstrumentedByteStream`] that yields chunks as they arrive.
    /// The span covers the **entire operation** — both the SDK call and the body transfer —
    /// ending only when the stream is exhausted or an error occurs.
    ///
    /// If you need the full body in memory, prefer [`collect`][Self::collect].
    /// If you need the raw [`GetObjectOutput`] or [`ByteStream`] (e.g. to pass to another
    /// API), use `.send()` directly, but be aware the span will not cover body transfer.
    ///
    /// # Errors
    ///
    /// Returns `Err` if the `GetObject` request itself fails. Body transfer errors surface
    /// as [`Err`] items yielded by the returned stream.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aws_sdk_s3::Client as S3Client;
    /// use futures_util::TryStreamExt;
    /// use telemetry_rust::middleware::aws::AwsBuilderInstrument;
    ///
    /// async fn stream_object(s3_client: &S3Client) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    ///     let mut stream = s3_client
    ///         .get_object()
    ///         .bucket("my_bucket")
    ///         .key("my_key")
    ///         .instrument()
    ///         .stream()
    ///         .await?;
    ///
    ///     let mut body = Vec::new();
    ///     while let Some(chunk) = stream.try_next().await? {
    ///         body.extend_from_slice(&chunk);
    ///     }
    ///     Ok(body)
    /// }
    /// ```
    ///
    /// [`GetObjectOutput`]: aws_sdk_s3::operation::get_object::GetObjectOutput
    /// [`ByteStream`]: aws_smithy_types::byte_stream::ByteStream
    pub async fn stream(
        self,
    ) -> Result<InstrumentedByteStream, Box<SdkError<GetObjectError>>> {
        let mut span = self.span.start();
        span.set_attribute(KeyValue::new("aws.s3.body.mode", "stream"));

        let result = self.inner.send().await;
        let Ok(output) = result else {
            span.on_result(&result);
            return Err(Box::new(result.unwrap_err()));
        };

        if let Some(value) = output.request_id() {
            span.set_attribute(KeyValue::new(semconv::AWS_REQUEST_ID, value.to_owned()));
        }

        span.set_attributes(output.extract_attributes());
        Ok(InstrumentedByteStream::new(output.body, span))
    }
}
