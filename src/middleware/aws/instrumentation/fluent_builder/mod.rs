use crate::{Context, middleware::aws::*};

pub(super) mod utils;

#[cfg(feature = "aws-dynamodb")]
mod dynamodb;
#[cfg(feature = "aws-firehose")]
mod firehose;
#[cfg(feature = "aws-sns")]
mod sns;
#[cfg(feature = "aws-sqs")]
mod sqs;

/// A trait for AWS service clients that can be instrumented with OpenTelemetry tracing.
///
/// This trait provides methods to build spans for AWS operations and instrument the
/// fluent builders returned by AWS SDK operations.
pub trait AwsInstrumentBuilder<'a>
where
    Self: Sized,
{
    /// Builds an AWS span for the specific operation represented by this builder.
    ///
    /// Returns an [`AwsSpanBuilder`] that contains the necessary span attributes
    /// and metadata for the AWS operation.
    fn build_aws_span(&self) -> AwsSpanBuilder<'a>;

    /// Instruments this fluent builder with OpenTelemetry tracing.
    ///
    /// Returns an [`InstrumentedFluentBuilder`] that will automatically create
    /// and manage spans when the operation is executed.
    fn instrument(self) -> InstrumentedFluentBuilder<'a, Self> {
        let span = self.build_aws_span();
        InstrumentedFluentBuilder::new(self, span)
    }
}

/// A wrapper that instruments AWS fluent builders with OpenTelemetry tracing.
///
/// This struct wraps AWS SDK fluent builders and automatically creates spans
/// when operations are executed, providing distributed tracing capabilities
/// for AWS service calls.
pub struct InstrumentedFluentBuilder<'a, T: AwsInstrumentBuilder<'a>> {
    inner: T,
    span: AwsSpanBuilder<'a>,
}

impl<'a, T: AwsInstrumentBuilder<'a>> InstrumentedFluentBuilder<'a, T> {
    /// Creates a new instrumented fluent builder.
    ///
    /// # Arguments
    /// * `inner` - The AWS SDK fluent builder to wrap
    /// * `span` - The span builder with AWS operation metadata
    pub fn new(inner: T, span: AwsSpanBuilder<'a>) -> Self {
        Self { inner, span }
    }

    /// Sets the OpenTelemetry context for this instrumented builder.
    ///
    /// # Arguments
    /// * `context` - The OpenTelemetry context to use for span creation
    pub fn context(mut self, context: &'a Context) -> Self {
        self.span = self.span.context(context);
        self
    }

    /// Sets the OpenTelemetry context for this instrumented builder.
    ///
    /// # Arguments
    /// * `context` - Optional OpenTelemetry context to use for span creation
    pub fn set_context(mut self, context: Option<&'a Context>) -> Self {
        self.span = self.span.set_context(context);
        self
    }
}

/// Generates [`super::InstrumentedFluentBuilder`] implementation for AWS SDK operations.
macro_rules! instrument_aws_operation {
    ($sdk:ident::operation::$op:ident, $builder:ident, $output:ident, $error:ident) => {
        use $sdk::operation::$op::builders::$builder;
        impl
            super::InstrumentedFluentBuilder<'_, $sdk::operation::$op::builders::$builder>
        {
            /// Executes the AWS operation with instrumentation.
            ///
            /// This method creates a span for the operation and executes it within
            /// that span context, providing automatic distributed tracing.
            pub async fn send(
                self,
            ) -> Result<
                $sdk::operation::$op::$output,
                $sdk::error::SdkError<$sdk::operation::$op::$error>,
            > {
                self.inner.send().instrument(self.span).await
            }
        }
    };
    ($sdk:ident::operation::$op:ident) => {
        paste::paste! {
            instrument_aws_operation!(
                $sdk::operation::$op,
                [<$op:camel FluentBuilder>],
                [<$op:camel Output>],
                [<$op:camel Error>]
            );
        }
    };
}

pub(super) use instrument_aws_operation;
