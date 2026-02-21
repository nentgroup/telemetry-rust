/// AWS Secrets Manager operations
///
/// API Reference: https://docs.aws.amazon.com/secretsmanager/latest/apireference/API_Operations.html
use crate::{KeyValue, StringValue};

use super::*;

/// Builder for Secrets Manager-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for Secrets Manager operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with Secrets Manager-specific attributes.
pub enum SecretsManagerSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates a Secrets Manager operation span builder.
    ///
    /// This method creates a span builder configured for Secrets Manager
    /// operations with the secret identifier as the primary resource identifier.
    ///
    /// # Arguments
    ///
    /// * `method` - The Secrets Manager operation method name
    /// * `secret_id` - Optional secret name or ARN for the operation
    pub fn secretsmanager(
        method: impl Into<StringValue>,
        secret_id: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = Vec::new();
        if let Some(id) = secret_id {
            attributes.push(KeyValue::new("aws.secretsmanager.secret_id", id.into()));
        }
        Self::client("SecretsManager", method, attributes)
    }
}

macro_rules! secretsmanager_global_operation {
    ($op: ident) => {
        impl SecretsManagerSpanBuilder {
            #[doc = concat!("Creates a span builder for the Secrets Manager ", stringify!($op), " operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::secretsmanager(stringify_camel!($op), None::<StringValue>)
            }
        }
    };
}

macro_rules! secretsmanager_secret_operation {
    ($op: ident) => {
        impl SecretsManagerSpanBuilder {
            #[doc = concat!("Creates a span builder for the Secrets Manager ", stringify!($op), " operation.")]
            ///
            /// # Arguments
            ///
            /// * `secret_id` - The name or ARN of the secret
            pub fn $op<'a>(
                secret_id: impl Into<StringValue>,
            ) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::secretsmanager(stringify_camel!($op), Some(secret_id))
            }
        }
    };
}

// Secret value operations
secretsmanager_secret_operation!(get_secret_value);
secretsmanager_secret_operation!(put_secret_value);
secretsmanager_secret_operation!(create_secret);

// Secret lifecycle operations
secretsmanager_secret_operation!(delete_secret);
secretsmanager_secret_operation!(describe_secret);
secretsmanager_secret_operation!(update_secret);
secretsmanager_secret_operation!(restore_secret);

// Rotation operations
secretsmanager_secret_operation!(rotate_secret);
secretsmanager_secret_operation!(cancel_rotate_secret);

// Version operations
secretsmanager_secret_operation!(update_secret_version_stage);
secretsmanager_secret_operation!(list_secret_version_ids);

// Tagging operations
secretsmanager_secret_operation!(tag_resource);
secretsmanager_secret_operation!(untag_resource);

// Resource policy operations
secretsmanager_secret_operation!(get_resource_policy);
secretsmanager_secret_operation!(put_resource_policy);
secretsmanager_secret_operation!(delete_resource_policy);
secretsmanager_secret_operation!(validate_resource_policy);

// Replication operations
secretsmanager_secret_operation!(remove_regions_from_replication);
secretsmanager_secret_operation!(replicate_secret_to_regions);
secretsmanager_secret_operation!(stop_replication_to_replica);

// Global operations
secretsmanager_global_operation!(list_secrets);
secretsmanager_global_operation!(batch_get_secret_value);
secretsmanager_global_operation!(get_random_password);
