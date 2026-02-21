/// AWS SSM Parameter Store operations
///
/// API Reference: https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_Operations.html
use crate::{KeyValue, StringValue};

use super::*;

/// Builder for SSM Parameter Store-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for SSM Parameter Store operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with SSM-specific attributes.
pub enum SsmSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates an SSM operation span builder.
    ///
    /// This method creates a span builder configured for SSM Parameter Store
    /// operations with the parameter name as the primary resource identifier.
    ///
    /// # Arguments
    ///
    /// * `method` - The SSM operation method name
    /// * `parameter_name` - Optional parameter name for the operation
    pub fn ssm(
        method: impl Into<StringValue>,
        parameter_name: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = Vec::new();
        if let Some(name) = parameter_name {
            attributes.push(KeyValue::new("aws.ssm.parameter_name", name.into()));
        }
        Self::client("SSM", method, attributes)
    }
}

macro_rules! ssm_global_operation {
    ($op: ident) => {
        impl SsmSpanBuilder {
            #[doc = concat!("Creates a span builder for the SSM ", stringify!($op), " operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::ssm(stringify_camel!($op), None::<StringValue>)
            }
        }
    };
}

macro_rules! ssm_parameter_operation {
    ($op: ident) => {
        impl SsmSpanBuilder {
            #[doc = concat!("Creates a span builder for the SSM ", stringify!($op), " parameter operation.")]
            ///
            /// # Arguments
            ///
            /// * `parameter_name` - The name of the SSM parameter
            pub fn $op<'a>(
                parameter_name: impl Into<StringValue>,
            ) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::ssm(stringify_camel!($op), Some(parameter_name))
            }
        }
    };
}

// Single parameter operations
ssm_parameter_operation!(get_parameter);
ssm_parameter_operation!(put_parameter);
ssm_parameter_operation!(delete_parameter);
ssm_parameter_operation!(get_parameter_history);
ssm_parameter_operation!(label_parameter_version);
ssm_parameter_operation!(unlabel_parameter_version);

// Multi-parameter operations
ssm_global_operation!(get_parameters);
ssm_global_operation!(delete_parameters);

// Path-based operations
ssm_parameter_operation!(get_parameters_by_path);

// List/describe operations
ssm_global_operation!(describe_parameters);
