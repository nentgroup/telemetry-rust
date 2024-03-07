use aws_types::request_id::RequestId;
use opentelemetry::{
    global::BoxedSpan,
    trace::{Span, Status},
};
use std::error::Error;

use crate::semcov;

#[cfg(feature = "aws-instrumentation")]
mod instrumentation;
mod operation;

#[cfg(feature = "aws-instrumentation")]
pub use instrumentation::AwsInstrumented;
pub use operation::*;

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
            span.set_attribute(semcov::AWS_REQUEST_ID.string(value.to_owned()));
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
