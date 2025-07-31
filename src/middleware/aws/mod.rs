//! Instrumentation utilities for AWS SDK operations.
//!
//! This module provides comprehensive instrumentation for AWS services,
//! including automatic instrumentation and a low-level API for manual span creation.
//! It supports both individual AWS SDK operations and streaming/pagination.
//!
//! # Features
//!
//! - **Span Creation**: Manual span creation with [`AwsSpan`] and [`AwsSpanBuilder`]
//! - **Instrumentation**: Automatic instrumentation for AWS SDK operations with [`AwsInstrument`] trait
//! - **Stream Instrumentation**: Automatic instrumentation for AWS [`PaginationStream`](`aws_smithy_async::future::pagination_stream::PaginationStream`) with [`AwsStreamInstrument`] trait
//!
//! # Feature Flags
//!
//! - `aws-instrumentation`: Enables [`Future`] instrumentation via [`AwsInstrument`] trait
//! - `aws-stream-instrumentation`: Enables [`Stream`][`futures_util::Stream`] instrumentation via [`AwsStreamInstrument`] trait

use aws_types::request_id::RequestId;
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span as _, SpanBuilder, SpanKind, Status, Tracer},
};
use std::error::Error;
use tracing::Span;

use crate::{Context, KeyValue, OpenTelemetrySpanExt, StringValue, semconv};

mod instrumentation;
mod operations;

pub use instrumentation::*;
pub use operations::*;

/// A wrapper around an OpenTelemetry span specifically designed for AWS operations.
///
/// This struct provides convenient methods for handling AWS-specific span attributes
/// and status updates, particularly for recording request IDs and error handling.
pub struct AwsSpan {
    span: BoxedSpan,
}

impl AwsSpan {
    /// Ends the span with AWS response information.
    ///
    /// This method finalizes the span by recording the outcome of an AWS operation.
    /// It automatically extracts request IDs and handles error reporting.
    ///
    /// # Arguments
    ///
    /// * `aws_response` - The result of the AWS operation, which must implement
    ///   `RequestId` for both success and error cases
    ///
    /// # Behavior
    ///
    /// - On success: Sets span status to OK and records the request ID
    /// - On error: Records the error, sets error status, and records the request ID if available
    pub fn end<T, E>(self, aws_response: &Result<T, E>)
    where
        T: RequestId,
        E: RequestId + Error,
    {
        let mut span = self.span;
        let (status, request_id) = match aws_response {
            Ok(resp) => (Status::Ok, resp.request_id()),
            Err(error) => {
                span.record_error(&error);
                (Status::error(error.to_string()), error.request_id())
            }
        };
        if let Some(value) = request_id {
            span.set_attribute(KeyValue::new(semconv::AWS_REQUEST_ID, value.to_owned()));
        }
        span.set_status(status);
    }

    /// Sets a single attribute on the span.
    ///
    /// This method allows you to add custom attributes to the span after it has been created.
    /// This is useful for adding dynamic attributes that become available during operation execution.
    ///
    /// For more information see [`BoxedSpan::set_attribute`]
    ///
    /// # Arguments
    ///
    /// * `attribute` - The key-value attribute to add to the span
    ///
    /// # Example
    ///
    /// ```rust
    /// use telemetry_rust::{KeyValue, middleware::aws::AwsSpanBuilder};
    ///
    /// let mut span = AwsSpanBuilder::client("DynamoDB", "GetItem", []).start();
    /// span.set_attribute(KeyValue::new("custom.attribute", "value"));
    /// ```
    pub fn set_attribute(&mut self, attribute: KeyValue) {
        self.span.set_attribute(attribute);
    }

    /// Sets multiple attributes on the span.
    ///
    /// This method allows you to add multiple custom attributes to the span at once.
    /// This is more efficient than calling `set_attribute` multiple times.
    ///
    /// For more information see [`BoxedSpan::set_attributes`]
    ///
    /// # Arguments
    ///
    /// * `attributes` - An iterator of key-value attributes to add to the span
    ///
    /// # Example
    ///
    /// ```rust
    /// use telemetry_rust::{KeyValue, middleware::aws::AwsSpanBuilder, semconv};
    ///
    /// let mut span = AwsSpanBuilder::client("DynamoDB", "GetItem", []).start();
    /// span.set_attributes([
    ///     KeyValue::new(semconv::DB_NAMESPACE, "my_table"),
    ///     KeyValue::new("custom.attribute", "value"),
    /// ]);
    /// ```
    pub fn set_attributes(&mut self, attributes: impl IntoIterator<Item = KeyValue>) {
        self.span.set_attributes(attributes);
    }
}

impl From<BoxedSpan> for AwsSpan {
    #[inline]
    fn from(span: BoxedSpan) -> Self {
        Self { span }
    }
}

/// Builder for creating AWS-specific OpenTelemetry spans.
///
/// This builder provides a fluent interface for constructing spans with AWS-specific
/// attributes and proper span kinds for different types of AWS operations.
pub struct AwsSpanBuilder<'a> {
    inner: SpanBuilder,
    tracer: BoxedTracer,
    context: Option<&'a Context>,
}

impl<'a> AwsSpanBuilder<'a> {
    fn new(
        span_kind: SpanKind,
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        custom_attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        let service: StringValue = service.into();
        let method: StringValue = method.into();
        let tracer = global::tracer("aws_sdk");
        let span_name = format!("{service}.{method}");
        let mut attributes = vec![
            KeyValue::new(semconv::RPC_METHOD, method),
            KeyValue::new(semconv::RPC_SYSTEM, "aws-api"),
            KeyValue::new(semconv::RPC_SERVICE, service),
        ];
        attributes.extend(custom_attributes);
        let inner = tracer
            .span_builder(span_name)
            .with_attributes(attributes)
            .with_kind(span_kind);

        Self {
            inner,
            tracer,
            context: None,
        }
    }

    /// Creates a client span builder for AWS operations.
    ///
    /// Client spans represent outbound calls to AWS services from your application.
    ///
    /// # Arguments
    ///
    /// * `service` - The AWS service name (e.g., "S3", "DynamoDB")
    /// * `method` - The operation name (e.g., "GetObject", "PutItem")
    /// * `attributes` - Additional custom attributes for the span
    pub fn client(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Client, service, method, attributes)
    }

    /// Creates a producer span builder for AWS operations.
    ///
    /// Producer spans represent operations that send messages or data to AWS services.
    ///
    /// # Arguments
    ///
    /// * `service` - The AWS service name (e.g., "SQS", "SNS")
    /// * `method` - The operation name (e.g., "SendMessage", "Publish")
    /// * `attributes` - Additional custom attributes for the span
    pub fn producer(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Producer, service, method, attributes)
    }

    /// Creates a consumer span builder for AWS operations.
    ///
    /// Consumer spans represent operations that receive messages or data from AWS services.
    ///
    /// # Arguments
    ///
    /// * `service` - The AWS service name (e.g., "SQS", "Kinesis")
    /// * `method` - The operation name (e.g., "ReceiveMessage", "GetRecords")
    /// * `attributes` - Additional custom attributes for the span
    pub fn consumer(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Consumer, service, method, attributes)
    }

    /// Adds multiple attributes to the span being built.
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator of key-value attributes to add to the span
    pub fn attributes(mut self, iter: impl IntoIterator<Item = KeyValue>) -> Self {
        if let Some(attributes) = &mut self.inner.attributes {
            attributes.extend(iter);
        }
        self
    }

    /// Adds a single attribute to the span being built.
    ///
    /// This is a convenience method for adding one attribute at a time.
    ///
    /// # Arguments
    ///
    /// * `attribute` - The key-value attribute to add to the span
    #[inline]
    pub fn attribute(self, attribute: KeyValue) -> Self {
        self.attributes(std::iter::once(attribute))
    }

    /// Sets the parent context for the span.
    ///
    /// # Arguments
    ///
    /// * `context` - The OpenTelemetry context to use as the parent
    #[inline]
    pub fn context(mut self, context: &'a Context) -> Self {
        self.context = Some(context);
        self
    }

    /// Optionally sets the parent context for the span.
    ///
    /// # Arguments
    ///
    /// * `context` - An optional OpenTelemetry context to use as the parent
    #[inline]
    pub fn set_context(mut self, context: Option<&'a Context>) -> Self {
        self.context = context;
        self
    }

    #[inline(always)]
    fn start_with_context(self, parent_cx: &Context) -> AwsSpan {
        self.inner
            .start_with_context(&self.tracer, parent_cx)
            .into()
    }

    /// Starts the span and returns an AwsSpan.
    ///
    /// This method creates and starts the span using either the explicitly set context
    /// or the current tracing span's context as the parent.
    #[inline]
    pub fn start(self) -> AwsSpan {
        match self.context {
            Some(context) => self.start_with_context(context),
            None => self.start_with_context(&Span::current().context()),
        }
    }
}
