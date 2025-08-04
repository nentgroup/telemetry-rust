/// AWS S3 operations
///
/// API Reference: https://docs.aws.amazon.com/AmazonS3/latest/API/API_Operations.html
use crate::{KeyValue, StringValue, semconv};

use super::*;

/// Builder for S3-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for S3 operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with S3-specific object store attributes.
pub enum S3SpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates an S3 operation span builder.
    ///
    /// This method creates a span builder configured for S3 operations with
    /// appropriate semantic attributes according to OpenTelemetry conventions.
    ///
    /// # Arguments
    ///
    /// * `method` - The S3 operation method name (e.g., "GetObject", "PutObject")
    /// * `bucket_name` - Optional bucket name for operations that target specific buckets
    ///
    /// # Returns
    ///
    /// A configured AWS span builder for the S3 operation
    pub fn s3(
        method: impl Into<StringValue>,
        bucket_name: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = vec![
            KeyValue::new(semconv::RPC_SYSTEM, "aws-api"),
            KeyValue::new(semconv::RPC_SERVICE, "S3"),
            KeyValue::new(semconv::RPC_METHOD, method.into()),
        ];
        if let Some(bucket_name) = bucket_name {
            attributes.push(KeyValue::new("aws.s3.bucket", bucket_name.into()));
        }
        Self::new(SpanKind::Client, "AWS", "S3", attributes)
    }
}

macro_rules! s3_global_operation {
    ($op: ident) => {
        impl S3SpanBuilder {
            #[doc = concat!("Creates a span builder for the S3 ", stringify!($op), " global operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::s3(
                    stringify_camel!($op),
                    None::<StringValue>,
                )
            }
        }
    };
}

macro_rules! s3_bucket_operation {
    ($op: ident) => {
        impl S3SpanBuilder {
            #[doc = concat!("Creates a span builder for the S3 ", stringify!($op), " bucket operation.")]
            ///
            /// # Arguments
            ///
            /// * `bucket_name` - The name of the S3 bucket
            pub fn $op<'a>(bucket_name: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::s3(stringify_camel!($op), Some(bucket_name))
            }
        }
    };
}

macro_rules! s3_object_operation {
    ($op: ident) => {
        impl S3SpanBuilder {
            #[doc = concat!("Creates a span builder for the S3 ", stringify!($op), " object operation.")]
            ///
            /// # Arguments
            ///
            /// * `bucket_name` - The name of the S3 bucket
            pub fn $op<'a>(bucket_name: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::s3(stringify_camel!($op), Some(bucket_name))
            }
        }
    };
}

// Global operations (no bucket required)
s3_global_operation!(list_buckets);

// Bucket operations
s3_bucket_operation!(create_bucket);
s3_bucket_operation!(delete_bucket);
s3_bucket_operation!(head_bucket);
s3_bucket_operation!(list_objects);
s3_bucket_operation!(list_objects_v2);
s3_bucket_operation!(list_object_versions);
s3_bucket_operation!(list_multipart_uploads);
s3_bucket_operation!(get_bucket_accelerate_configuration);
s3_bucket_operation!(get_bucket_acl);
s3_bucket_operation!(get_bucket_analytics_configuration);
s3_bucket_operation!(get_bucket_cors);
s3_bucket_operation!(get_bucket_encryption);
s3_bucket_operation!(get_bucket_intelligent_tiering_configuration);
s3_bucket_operation!(get_bucket_inventory_configuration);
s3_bucket_operation!(get_bucket_lifecycle);
s3_bucket_operation!(get_bucket_lifecycle_configuration);
s3_bucket_operation!(get_bucket_location);
s3_bucket_operation!(get_bucket_logging);
s3_bucket_operation!(get_bucket_metrics_configuration);
s3_bucket_operation!(get_bucket_notification);
s3_bucket_operation!(get_bucket_notification_configuration);
s3_bucket_operation!(get_bucket_ownership_controls);
s3_bucket_operation!(get_bucket_policy);
s3_bucket_operation!(get_bucket_policy_status);
s3_bucket_operation!(get_bucket_replication);
s3_bucket_operation!(get_bucket_request_payment);
s3_bucket_operation!(get_bucket_tagging);
s3_bucket_operation!(get_bucket_versioning);
s3_bucket_operation!(get_bucket_website);
s3_bucket_operation!(get_public_access_block);
s3_bucket_operation!(put_bucket_accelerate_configuration);
s3_bucket_operation!(put_bucket_acl);
s3_bucket_operation!(put_bucket_analytics_configuration);
s3_bucket_operation!(put_bucket_cors);
s3_bucket_operation!(put_bucket_encryption);
s3_bucket_operation!(put_bucket_intelligent_tiering_configuration);
s3_bucket_operation!(put_bucket_inventory_configuration);
s3_bucket_operation!(put_bucket_lifecycle);
s3_bucket_operation!(put_bucket_lifecycle_configuration);
s3_bucket_operation!(put_bucket_logging);
s3_bucket_operation!(put_bucket_metrics_configuration);
s3_bucket_operation!(put_bucket_notification);
s3_bucket_operation!(put_bucket_notification_configuration);
s3_bucket_operation!(put_bucket_ownership_controls);
s3_bucket_operation!(put_bucket_policy);
s3_bucket_operation!(put_bucket_replication);
s3_bucket_operation!(put_bucket_request_payment);
s3_bucket_operation!(put_bucket_tagging);
s3_bucket_operation!(put_bucket_versioning);
s3_bucket_operation!(put_bucket_website);
s3_bucket_operation!(put_public_access_block);
s3_bucket_operation!(delete_bucket_analytics_configuration);
s3_bucket_operation!(delete_bucket_cors);
s3_bucket_operation!(delete_bucket_encryption);
s3_bucket_operation!(delete_bucket_intelligent_tiering_configuration);
s3_bucket_operation!(delete_bucket_inventory_configuration);
s3_bucket_operation!(delete_bucket_lifecycle);
s3_bucket_operation!(delete_bucket_metrics_configuration);
s3_bucket_operation!(delete_bucket_ownership_controls);
s3_bucket_operation!(delete_bucket_policy);
s3_bucket_operation!(delete_bucket_replication);
s3_bucket_operation!(delete_bucket_tagging);
s3_bucket_operation!(delete_bucket_website);
s3_bucket_operation!(delete_public_access_block);
s3_bucket_operation!(list_bucket_analytics_configurations);
s3_bucket_operation!(list_bucket_intelligent_tiering_configurations);
s3_bucket_operation!(list_bucket_inventory_configurations);
s3_bucket_operation!(list_bucket_metrics_configurations);

// Object operations (require bucket, may have object key)
s3_object_operation!(get_object);
s3_object_operation!(put_object);
s3_object_operation!(delete_object);
s3_object_operation!(delete_objects);
s3_object_operation!(head_object);
s3_object_operation!(copy_object);
s3_object_operation!(get_object_acl);
s3_object_operation!(put_object_acl);
s3_object_operation!(get_object_attributes);
s3_object_operation!(get_object_legal_hold);
s3_object_operation!(put_object_legal_hold);
s3_object_operation!(get_object_lock_configuration);
s3_object_operation!(put_object_lock_configuration);
s3_object_operation!(get_object_retention);
s3_object_operation!(put_object_retention);
s3_object_operation!(get_object_tagging);
s3_object_operation!(put_object_tagging);
s3_object_operation!(delete_object_tagging);
s3_object_operation!(get_object_torrent);
s3_object_operation!(restore_object);
s3_object_operation!(select_object_content);

// Multipart upload operations
s3_object_operation!(create_multipart_upload);
s3_object_operation!(complete_multipart_upload);
s3_object_operation!(abort_multipart_upload);
s3_object_operation!(upload_part);
s3_object_operation!(upload_part_copy);
s3_object_operation!(list_parts);
