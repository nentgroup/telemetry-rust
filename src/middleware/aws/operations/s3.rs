/// AWS S3 operations
///
/// API Reference: https://docs.aws.amazon.com/AmazonS3/latest/API/API_Operations_Amazon_Simple_Storage_Service.html
use crate::{KeyValue, StringValue, semconv};

use super::*;

/// Builder for S3-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for S3 operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with S3-specific attributes following OpenTelemetry semantic conventions
/// for object stores.
pub enum S3SpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates an S3 operation span builder.
    ///
    /// This method creates a span builder configured for S3 operations with
    /// appropriate semantic attributes according to OpenTelemetry conventions
    /// for object stores.
    ///
    /// # Arguments
    ///
    /// * `method` - The S3 operation method name (e.g., "GetObject", "PutObject")
    /// * `bucket` - Optional bucket name for operations that target specific buckets
    /// * `key` - Optional object key for operations that target specific objects
    pub fn s3(
        method: impl Into<StringValue>,
        bucket: Option<impl Into<StringValue>>,
        key: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = Vec::new();
        if let Some(bucket) = bucket {
            attributes.push(KeyValue::new(semconv::AWS_S3_BUCKET, bucket.into()));
        }
        if let Some(key) = key {
            attributes.push(KeyValue::new(semconv::AWS_S3_KEY, key.into()));
        }
        Self::client("S3", method, attributes)
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
            /// * `bucket` - The name of the S3 bucket
            pub fn $op<'a>(bucket: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::s3(
                    stringify_camel!($op),
                    Some(bucket),
                    None::<StringValue>,
                )
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
            /// * `bucket` - The name of the S3 bucket
            /// * `key` - The key of the S3 object
            pub fn $op<'a>(
                bucket: impl Into<StringValue>,
                key: impl Into<StringValue>,
            ) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::s3(stringify_camel!($op), Some(bucket), Some(key))
            }
        }
    };
}

// global operations
s3_global_operation!(list_buckets);
s3_global_operation!(list_directory_buckets);

// bucket management operations
s3_bucket_operation!(create_bucket);
s3_bucket_operation!(delete_bucket);
s3_bucket_operation!(head_bucket);
s3_bucket_operation!(create_session);

// object data operations
s3_object_operation!(copy_object);
s3_object_operation!(delete_object);
s3_object_operation!(get_object);
s3_object_operation!(head_object);
s3_object_operation!(put_object);
s3_object_operation!(rename_object);
s3_object_operation!(restore_object);
s3_object_operation!(select_object_content);
s3_object_operation!(get_object_torrent);

// batch delete operation
s3_bucket_operation!(delete_objects);

// object listing operations
s3_bucket_operation!(list_objects);
s3_bucket_operation!(list_objects_v2);
s3_bucket_operation!(list_object_versions);

// multipart upload operations
s3_object_operation!(create_multipart_upload);
s3_object_operation!(upload_part);
s3_object_operation!(upload_part_copy);
s3_object_operation!(complete_multipart_upload);
s3_object_operation!(abort_multipart_upload);
s3_bucket_operation!(list_multipart_uploads);
s3_object_operation!(list_parts);

// object ACL operations
s3_object_operation!(get_object_acl);
s3_object_operation!(put_object_acl);
s3_object_operation!(get_object_attributes);

// object tagging operations
s3_object_operation!(get_object_tagging);
s3_object_operation!(put_object_tagging);
s3_object_operation!(delete_object_tagging);

// object lock and retention operations
s3_object_operation!(get_object_legal_hold);
s3_object_operation!(put_object_legal_hold);
s3_object_operation!(get_object_retention);
s3_object_operation!(put_object_retention);
s3_bucket_operation!(get_object_lock_configuration);
s3_bucket_operation!(put_object_lock_configuration);

// bucket ACL and policy operations
s3_bucket_operation!(get_bucket_acl);
s3_bucket_operation!(put_bucket_acl);
s3_bucket_operation!(get_bucket_policy);
s3_bucket_operation!(put_bucket_policy);
s3_bucket_operation!(delete_bucket_policy);
s3_bucket_operation!(get_bucket_policy_status);

// bucket CORS operations
s3_bucket_operation!(get_bucket_cors);
s3_bucket_operation!(put_bucket_cors);
s3_bucket_operation!(delete_bucket_cors);

// bucket encryption operations
s3_bucket_operation!(get_bucket_encryption);
s3_bucket_operation!(put_bucket_encryption);
s3_bucket_operation!(delete_bucket_encryption);

// bucket lifecycle operations
s3_bucket_operation!(get_bucket_lifecycle_configuration);
s3_bucket_operation!(put_bucket_lifecycle_configuration);
s3_bucket_operation!(delete_bucket_lifecycle);

// bucket replication operations
s3_bucket_operation!(get_bucket_replication);
s3_bucket_operation!(put_bucket_replication);
s3_bucket_operation!(delete_bucket_replication);

// bucket tagging operations
s3_bucket_operation!(get_bucket_tagging);
s3_bucket_operation!(put_bucket_tagging);
s3_bucket_operation!(delete_bucket_tagging);

// bucket versioning operations
s3_bucket_operation!(get_bucket_versioning);
s3_bucket_operation!(put_bucket_versioning);

// bucket website operations
s3_bucket_operation!(get_bucket_website);
s3_bucket_operation!(put_bucket_website);
s3_bucket_operation!(delete_bucket_website);

// bucket logging operations
s3_bucket_operation!(get_bucket_logging);
s3_bucket_operation!(put_bucket_logging);

// bucket notification operations
s3_bucket_operation!(get_bucket_notification_configuration);
s3_bucket_operation!(put_bucket_notification_configuration);

// bucket location operations
s3_bucket_operation!(get_bucket_location);

// bucket accelerate operations
s3_bucket_operation!(get_bucket_accelerate_configuration);
s3_bucket_operation!(put_bucket_accelerate_configuration);

// bucket request payment operations
s3_bucket_operation!(get_bucket_request_payment);
s3_bucket_operation!(put_bucket_request_payment);

// bucket ownership controls
s3_bucket_operation!(get_bucket_ownership_controls);
s3_bucket_operation!(put_bucket_ownership_controls);
s3_bucket_operation!(delete_bucket_ownership_controls);

// bucket analytics configuration operations
s3_bucket_operation!(get_bucket_analytics_configuration);
s3_bucket_operation!(put_bucket_analytics_configuration);
s3_bucket_operation!(delete_bucket_analytics_configuration);
s3_bucket_operation!(list_bucket_analytics_configurations);

// bucket intelligent tiering configuration operations
s3_bucket_operation!(get_bucket_intelligent_tiering_configuration);
s3_bucket_operation!(put_bucket_intelligent_tiering_configuration);
s3_bucket_operation!(delete_bucket_intelligent_tiering_configuration);
s3_bucket_operation!(list_bucket_intelligent_tiering_configurations);

// bucket inventory configuration operations
s3_bucket_operation!(get_bucket_inventory_configuration);
s3_bucket_operation!(put_bucket_inventory_configuration);
s3_bucket_operation!(delete_bucket_inventory_configuration);
s3_bucket_operation!(list_bucket_inventory_configurations);

// bucket metrics configuration operations
s3_bucket_operation!(get_bucket_metrics_configuration);
s3_bucket_operation!(put_bucket_metrics_configuration);
s3_bucket_operation!(delete_bucket_metrics_configuration);
s3_bucket_operation!(list_bucket_metrics_configurations);

// public access block operations
s3_bucket_operation!(get_public_access_block);
s3_bucket_operation!(put_public_access_block);
s3_bucket_operation!(delete_public_access_block);

// bucket metadata operations
s3_bucket_operation!(create_bucket_metadata_configuration);
s3_bucket_operation!(create_bucket_metadata_table_configuration);
s3_bucket_operation!(delete_bucket_metadata_configuration);
s3_bucket_operation!(delete_bucket_metadata_table_configuration);
s3_bucket_operation!(get_bucket_metadata_configuration);
s3_bucket_operation!(get_bucket_metadata_table_configuration);
s3_bucket_operation!(update_bucket_metadata_inventory_table_configuration);
s3_bucket_operation!(update_bucket_metadata_journal_table_configuration);

// S3 Object Lambda operation
s3_global_operation!(write_get_object_response);
