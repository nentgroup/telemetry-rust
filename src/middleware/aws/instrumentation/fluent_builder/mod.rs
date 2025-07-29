use crate::{middleware::aws::*, Context};

pub(super) mod utils;

#[cfg(feature = "aws-dynamodb")]
mod dynamodb;
#[cfg(feature = "aws-firehose")]
mod firehose;
#[cfg(feature = "aws-sns")]
mod sns;

pub trait AwsInstrumentBuilder<'a>
where
    Self: Sized,
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a>;
    fn instrument(self) -> InstrumentedFluentBuilder<'a, Self> {
        let span = self.build_aws_span();
        InstrumentedFluentBuilder::new(self, span)
    }
}

pub struct InstrumentedFluentBuilder<'a, T: AwsInstrumentBuilder<'a>> {
    inner: T,
    span: AwsSpanBuilder<'a>,
}

impl<'a, T: AwsInstrumentBuilder<'a>> InstrumentedFluentBuilder<'a, T> {
    pub fn new(inner: T, span: AwsSpanBuilder<'a>) -> Self {
        Self { inner, span }
    }

    pub fn context(mut self, context: &'a Context) -> Self {
        self.span = self.span.context(context);
        self
    }

    pub fn set_context(mut self, context: Option<&'a Context>) -> Self {
        self.span = self.span.set_context(context);
        self
    }
}
