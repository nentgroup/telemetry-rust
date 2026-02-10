/// AWS SageMaker Runtime operations
///
/// API Reference: https://docs.aws.amazon.com/sagemaker/latest/APIReference/API_Operations_Amazon_SageMaker_Runtime.html
use crate::{KeyValue, StringValue};

use super::*;

/// Builder for SageMaker Runtime-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for SageMaker Runtime operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with SageMaker Runtime-specific attributes.
pub enum SageMakerRuntimeSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates a SageMaker Runtime operation span builder.
    ///
    /// This method creates a span builder configured for SageMaker Runtime inference
    /// operations with the endpoint name as the primary resource identifier.
    ///
    /// # Arguments
    ///
    /// * `method` - The SageMaker Runtime operation method name
    /// * `endpoint_name` - The name of the SageMaker endpoint being invoked
    pub fn sagemaker_runtime(
        method: impl Into<StringValue>,
        endpoint_name: impl Into<StringValue>,
    ) -> Self {
        let endpoint_name: StringValue = endpoint_name.into();
        let attributes =
            vec![KeyValue::new("aws.sagemaker.endpoint_name", endpoint_name)];
        Self::client("SageMakerRuntime", method, attributes)
    }
}

macro_rules! sagemaker_runtime_endpoint_operation {
    ($op: ident) => {
        impl SageMakerRuntimeSpanBuilder {
            #[doc = concat!("Creates a span builder for the SageMaker Runtime ", stringify!($op), " operation.")]
            ///
            /// # Arguments
            ///
            /// * `endpoint_name` - The name of the SageMaker endpoint
            pub fn $op<'a>(endpoint_name: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::sagemaker_runtime(stringify_camel!($op), endpoint_name)
            }
        }
    };
}

sagemaker_runtime_endpoint_operation!(invoke_endpoint);
sagemaker_runtime_endpoint_operation!(invoke_endpoint_async);
sagemaker_runtime_endpoint_operation!(invoke_endpoint_with_response_stream);
