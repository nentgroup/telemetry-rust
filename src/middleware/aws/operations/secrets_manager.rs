/// AWS Secrets Manager operations
///
/// API Reference: https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_Operations.html
use crate::StringValue;

use super::*;

/// Builder for Secrets Manager-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for Secrets Manager operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans for AWS Secrets Manager operations.
pub enum SecretsManagerSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates a Secrets Manager operation span builder.
    ///
    /// # Arguments
    ///
    /// * `method` - The Secrets Manager operation method name
    pub fn secrets_manager(method: impl Into<StringValue>) -> Self {
        Self::client(
            "SecretsManager",
            method,
            std::iter::empty::<crate::KeyValue>(),
        )
    }
}

macro_rules! secrets_manager_operation {
    ($op: ident) => {
        impl SecretsManagerSpanBuilder {
            #[doc = concat!("Creates a span builder for the Secrets Manager ", stringify!($op), " operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::secrets_manager(stringify_camel!($op))
            }
        }
    };
}

secrets_manager_operation!(batch_get_secret_value);
secrets_manager_operation!(cancel_rotate_secret);
secrets_manager_operation!(create_secret);
secrets_manager_operation!(delete_resource_policy);
secrets_manager_operation!(delete_secret);
secrets_manager_operation!(describe_secret);
secrets_manager_operation!(get_random_password);
secrets_manager_operation!(get_resource_policy);
secrets_manager_operation!(get_secret_value);
secrets_manager_operation!(list_secret_version_ids);
secrets_manager_operation!(list_secrets);
secrets_manager_operation!(put_resource_policy);
secrets_manager_operation!(put_secret_value);
secrets_manager_operation!(remove_regions_from_replication);
secrets_manager_operation!(replicate_secret_to_regions);
secrets_manager_operation!(restore_secret);
secrets_manager_operation!(rotate_secret);
secrets_manager_operation!(stop_replication_to_replica);
secrets_manager_operation!(tag_resource);
secrets_manager_operation!(untag_resource);
secrets_manager_operation!(update_secret);
secrets_manager_operation!(update_secret_version_stage);
secrets_manager_operation!(validate_resource_policy);
