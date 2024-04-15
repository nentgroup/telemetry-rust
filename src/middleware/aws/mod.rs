use aws_types::request_id::RequestId;
use opentelemetry::{
    global::{self, BoxedSpan, BoxedTracer},
    trace::{Span as _, SpanBuilder, SpanKind, Status, Tracer},
};
use std::error::Error;
use tracing::Span;

use crate::{semconv, Context, KeyValue, OpenTelemetrySpanExt, StringValue};

#[cfg(feature = "aws-instrumentation")]
mod instrumentation;
mod operations;

#[cfg(feature = "aws-instrumentation")]
pub use instrumentation::AwsInstrument;
pub use operations::*;

pub struct AwsSpan {
    span: BoxedSpan,
}

impl AwsSpan {
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
            span.set_attribute(semconv::AWS_REQUEST_ID.string(value.to_owned()));
        }
        span.set_status(status);
    }
}

impl From<BoxedSpan> for AwsSpan {
    #[inline]
    fn from(span: BoxedSpan) -> Self {
        Self { span }
    }
}

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
            semconv::RPC_METHOD.string(method),
            semconv::RPC_SYSTEM.string("aws-api"),
            semconv::RPC_SERVICE.string(service),
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

    pub fn client(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Client, service, method, attributes)
    }

    pub fn producer(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Producer, service, method, attributes)
    }

    pub fn consumer(
        service: impl Into<StringValue>,
        method: impl Into<StringValue>,
        attributes: impl IntoIterator<Item = KeyValue>,
    ) -> Self {
        Self::new(SpanKind::Consumer, service, method, attributes)
    }

    pub fn attributes(mut self, iter: impl IntoIterator<Item = KeyValue>) -> Self {
        if let Some(attributes) = &mut self.inner.attributes {
            attributes.extend(iter);
        }
        self
    }

    #[inline]
    pub fn attribute(self, attribute: KeyValue) -> Self {
        self.attributes(std::iter::once(attribute))
    }

    #[inline]
    pub fn context(mut self, context: &'a Context) -> Self {
        self.context = Some(context);
        self
    }

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

    #[inline]
    pub fn start(self) -> AwsSpan {
        match self.context {
            Some(context) => self.start_with_context(context),
            None => self.start_with_context(&Span::current().context()),
        }
    }
}
