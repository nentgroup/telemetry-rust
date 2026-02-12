/// AWS S3 fluent builder instrumentation implementations
use super::{utils::*, *};
use crate::semconv;

// Object data operations
impl<'a> AwsBuilderInstrument<'a> for GetObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_part_number()
                .as_attribute(semconv::AWS_S3_PART_NUMBER),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::get_object(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.content_length().as_attribute("aws.s3.content_length"),
            self.e_tag().as_attribute("aws.s3.e_tag"),
            self.version_id().as_attribute("aws.s3.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::get_object);

impl<'a> AwsBuilderInstrument<'a> for PutObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_content_length()
                .as_attribute("aws.s3.content_length"),
        ];
        S3SpanBuilder::put_object(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for PutObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.e_tag"),
            self.version_id().as_attribute("aws.s3.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::put_object);

impl<'a> AwsBuilderInstrument<'a> for HeadObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_part_number()
                .as_attribute(semconv::AWS_S3_PART_NUMBER),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::head_object(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for HeadObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.content_length().as_attribute("aws.s3.content_length"),
            self.e_tag().as_attribute("aws.s3.e_tag"),
            self.version_id().as_attribute("aws.s3.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::head_object);

impl<'a> AwsBuilderInstrument<'a> for CopyObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_copy_source()
                .as_attribute(semconv::AWS_S3_COPY_SOURCE),
        ];
        S3SpanBuilder::copy_object(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for CopyObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.copy_object_result()
                .and_then(|r| r.e_tag())
                .as_attribute("aws.s3.e_tag"),
            self.version_id().as_attribute("aws.s3.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::copy_object);

impl<'a> AwsBuilderInstrument<'a> for DeleteObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::delete_object(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.delete_marker().as_attribute(semconv::AWS_S3_DELETE),
            self.version_id().as_attribute("aws.s3.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::delete_object);

impl<'a> AwsBuilderInstrument<'a> for DeleteObjectsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_objects(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteObjectsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.deleted()
                .len()
                .as_attribute("aws.s3.delete.deleted_count"),
            self.errors()
                .len()
                .as_attribute("aws.s3.delete.error_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::delete_objects);

impl<'a> AwsBuilderInstrument<'a> for RenameObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        S3SpanBuilder::rename_object(bucket, key)
    }
}
impl InstrumentedFluentBuilderOutput for RenameObjectOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::rename_object);

impl<'a> AwsBuilderInstrument<'a> for RestoreObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::restore_object(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for RestoreObjectOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::restore_object);

impl<'a> AwsBuilderInstrument<'a> for SelectObjectContentFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        S3SpanBuilder::select_object_content(bucket, key)
    }
}
impl InstrumentedFluentBuilderOutput for SelectObjectContentOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::select_object_content);

impl<'a> AwsBuilderInstrument<'a> for GetObjectTorrentFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        S3SpanBuilder::get_object_torrent(bucket, key)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectTorrentOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_object_torrent);

// Object listing operations
impl<'a> AwsBuilderInstrument<'a> for ListObjectsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_objects(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for ListObjectsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_objects);

impl<'a> AwsBuilderInstrument<'a> for ListObjectsV2FluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_objects_v2(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for ListObjectsV2Output {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.key_count().as_attribute("aws.s3.key_count")]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::list_objects_v2);

impl<'a> AwsBuilderInstrument<'a> for ListObjectVersionsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_object_versions(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for ListObjectVersionsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_object_versions);

// Multipart upload operations
impl<'a> AwsBuilderInstrument<'a> for CreateMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        S3SpanBuilder::create_multipart_upload(bucket, key)
    }
}
impl InstrumentedFluentBuilderOutput for CreateMultipartUploadOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.upload_id().as_attribute(semconv::AWS_S3_UPLOAD_ID),]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::create_multipart_upload);

impl<'a> AwsBuilderInstrument<'a> for UploadPartFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_upload_id().as_attribute(semconv::AWS_S3_UPLOAD_ID),
            self.get_part_number()
                .as_attribute(semconv::AWS_S3_PART_NUMBER),
            self.get_content_length()
                .as_attribute("aws.s3.content_length"),
        ];
        S3SpanBuilder::upload_part(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for UploadPartOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.e_tag().as_attribute("aws.s3.e_tag")]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::upload_part);

impl<'a> AwsBuilderInstrument<'a> for UploadPartCopyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_upload_id().as_attribute(semconv::AWS_S3_UPLOAD_ID),
            self.get_part_number()
                .as_attribute(semconv::AWS_S3_PART_NUMBER),
            self.get_copy_source()
                .as_attribute(semconv::AWS_S3_COPY_SOURCE),
        ];
        S3SpanBuilder::upload_part_copy(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for UploadPartCopyOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.copy_part_result()
                .and_then(|r| r.e_tag())
                .as_attribute("aws.s3.e_tag"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::upload_part_copy);

impl<'a> AwsBuilderInstrument<'a> for CompleteMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_upload_id().as_attribute(semconv::AWS_S3_UPLOAD_ID),];
        S3SpanBuilder::complete_multipart_upload(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for CompleteMultipartUploadOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.e_tag"),
            self.version_id().as_attribute("aws.s3.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::complete_multipart_upload);

impl<'a> AwsBuilderInstrument<'a> for AbortMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_upload_id().as_attribute(semconv::AWS_S3_UPLOAD_ID),];
        S3SpanBuilder::abort_multipart_upload(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for AbortMultipartUploadOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::abort_multipart_upload);

impl<'a> AwsBuilderInstrument<'a> for ListMultipartUploadsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_multipart_uploads(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for ListMultipartUploadsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_multipart_uploads);

impl<'a> AwsBuilderInstrument<'a> for ListPartsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_upload_id().as_attribute(semconv::AWS_S3_UPLOAD_ID),];
        S3SpanBuilder::list_parts(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for ListPartsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_parts);

// Bucket management operations
impl<'a> AwsBuilderInstrument<'a> for CreateBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::create_bucket(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for CreateBucketOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::create_bucket);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket);

impl<'a> AwsBuilderInstrument<'a> for HeadBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::head_bucket(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for HeadBucketOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::head_bucket);

impl<'a> AwsBuilderInstrument<'a> for CreateSessionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::create_session(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for CreateSessionOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::create_session);

// Global operations
impl<'a> AwsBuilderInstrument<'a> for ListBucketsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        S3SpanBuilder::list_buckets()
    }
}
impl InstrumentedFluentBuilderOutput for ListBucketsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_buckets);

impl<'a> AwsBuilderInstrument<'a> for ListDirectoryBucketsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        S3SpanBuilder::list_directory_buckets()
    }
}
impl InstrumentedFluentBuilderOutput for ListDirectoryBucketsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_directory_buckets);

impl<'a> AwsBuilderInstrument<'a> for WriteGetObjectResponseFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes = attributes![
            self.get_content_length()
                .as_attribute("aws.s3.content_length"),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::write_get_object_response().attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for WriteGetObjectResponseOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::write_get_object_response);

// Object ACL operations
impl<'a> AwsBuilderInstrument<'a> for GetObjectAclFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::get_object_acl(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectAclOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_object_acl);

impl<'a> AwsBuilderInstrument<'a> for PutObjectAclFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::put_object_acl(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for PutObjectAclOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_object_acl);

impl<'a> AwsBuilderInstrument<'a> for GetObjectAttributesFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::get_object_attributes(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectAttributesOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.e_tag"),
            self.version_id().as_attribute("aws.s3.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::get_object_attributes);

// Object tagging operations
impl<'a> AwsBuilderInstrument<'a> for GetObjectTaggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::get_object_tagging(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectTaggingOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.version_id().as_attribute("aws.s3.version_id"),]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::get_object_tagging);

impl<'a> AwsBuilderInstrument<'a> for PutObjectTaggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::put_object_tagging(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for PutObjectTaggingOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.version_id().as_attribute("aws.s3.version_id"),]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::put_object_tagging);

impl<'a> AwsBuilderInstrument<'a> for DeleteObjectTaggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::delete_object_tagging(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteObjectTaggingOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.version_id().as_attribute("aws.s3.version_id"),]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::delete_object_tagging);

// Object lock and retention operations
impl<'a> AwsBuilderInstrument<'a> for GetObjectLegalHoldFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::get_object_legal_hold(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectLegalHoldOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_object_legal_hold);

impl<'a> AwsBuilderInstrument<'a> for PutObjectLegalHoldFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::put_object_legal_hold(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for PutObjectLegalHoldOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_object_legal_hold);

impl<'a> AwsBuilderInstrument<'a> for GetObjectRetentionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::get_object_retention(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectRetentionOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_object_retention);

impl<'a> AwsBuilderInstrument<'a> for PutObjectRetentionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        let key = self.get_key().clone().unwrap_or_default();
        let attributes =
            attributes![self.get_version_id().as_attribute("aws.s3.version_id"),];
        S3SpanBuilder::put_object_retention(bucket, key).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for PutObjectRetentionOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_object_retention);

impl<'a> AwsBuilderInstrument<'a> for GetObjectLockConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_object_lock_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectLockConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_object_lock_configuration);

impl<'a> AwsBuilderInstrument<'a> for PutObjectLockConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_object_lock_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutObjectLockConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_object_lock_configuration);

// Bucket ACL and policy operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketAclFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_acl(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketAclOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_acl);

impl<'a> AwsBuilderInstrument<'a> for PutBucketAclFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_acl(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketAclOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_acl);

impl<'a> AwsBuilderInstrument<'a> for GetBucketPolicyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_policy(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketPolicyOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_policy);

impl<'a> AwsBuilderInstrument<'a> for PutBucketPolicyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_policy(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketPolicyOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_policy);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketPolicyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_policy(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketPolicyOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_policy);

impl<'a> AwsBuilderInstrument<'a> for GetBucketPolicyStatusFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_policy_status(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketPolicyStatusOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_policy_status);

// Bucket CORS operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketCorsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_cors(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketCorsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_cors);

impl<'a> AwsBuilderInstrument<'a> for PutBucketCorsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_cors(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketCorsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_cors);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketCorsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_cors(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketCorsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_cors);

// Bucket encryption operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketEncryptionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_encryption(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketEncryptionOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_encryption);

impl<'a> AwsBuilderInstrument<'a> for PutBucketEncryptionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_encryption(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketEncryptionOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_encryption);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketEncryptionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_encryption(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketEncryptionOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_encryption);

// Bucket lifecycle operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketLifecycleConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_lifecycle_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketLifecycleConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_lifecycle_configuration);

impl<'a> AwsBuilderInstrument<'a> for PutBucketLifecycleConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_lifecycle_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketLifecycleConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_lifecycle_configuration);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketLifecycleFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_lifecycle(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketLifecycleOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_lifecycle);

// Bucket replication operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketReplicationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_replication(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketReplicationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_replication);

impl<'a> AwsBuilderInstrument<'a> for PutBucketReplicationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_replication(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketReplicationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_replication);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketReplicationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_replication(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketReplicationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_replication);

// Bucket tagging operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketTaggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_tagging(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketTaggingOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_tagging);

impl<'a> AwsBuilderInstrument<'a> for PutBucketTaggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_tagging(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketTaggingOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_tagging);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketTaggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_tagging(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketTaggingOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_tagging);

// Bucket versioning operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketVersioningFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_versioning(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketVersioningOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_versioning);

impl<'a> AwsBuilderInstrument<'a> for PutBucketVersioningFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_versioning(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketVersioningOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_versioning);

// Bucket website operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketWebsiteFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_website(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketWebsiteOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_website);

impl<'a> AwsBuilderInstrument<'a> for PutBucketWebsiteFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_website(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketWebsiteOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_website);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketWebsiteFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_website(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketWebsiteOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_website);

// Bucket logging operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketLoggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_logging(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketLoggingOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_logging);

impl<'a> AwsBuilderInstrument<'a> for PutBucketLoggingFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_logging(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketLoggingOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_logging);

// Bucket notification operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketNotificationConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_notification_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketNotificationConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_notification_configuration);

impl<'a> AwsBuilderInstrument<'a> for PutBucketNotificationConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_notification_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketNotificationConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_notification_configuration);

// Bucket location operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketLocationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_location(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketLocationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_location);

// Bucket accelerate operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketAccelerateConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_accelerate_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketAccelerateConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_accelerate_configuration);

impl<'a> AwsBuilderInstrument<'a> for PutBucketAccelerateConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_accelerate_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketAccelerateConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_accelerate_configuration);

// Bucket request payment operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketRequestPaymentFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_request_payment(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketRequestPaymentOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_request_payment);

impl<'a> AwsBuilderInstrument<'a> for PutBucketRequestPaymentFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_request_payment(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketRequestPaymentOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_request_payment);

// Bucket ownership controls
impl<'a> AwsBuilderInstrument<'a> for GetBucketOwnershipControlsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_ownership_controls(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketOwnershipControlsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_ownership_controls);

impl<'a> AwsBuilderInstrument<'a> for PutBucketOwnershipControlsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_ownership_controls(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketOwnershipControlsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_ownership_controls);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketOwnershipControlsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_ownership_controls(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketOwnershipControlsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_ownership_controls);

// Bucket analytics configuration operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketAnalyticsConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_analytics_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketAnalyticsConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_analytics_configuration);

impl<'a> AwsBuilderInstrument<'a> for PutBucketAnalyticsConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_analytics_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketAnalyticsConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_analytics_configuration);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketAnalyticsConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_analytics_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketAnalyticsConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_analytics_configuration);

impl<'a> AwsBuilderInstrument<'a> for ListBucketAnalyticsConfigurationsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_bucket_analytics_configurations(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for ListBucketAnalyticsConfigurationsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_bucket_analytics_configurations);

// Bucket intelligent tiering configuration operations
impl<'a> AwsBuilderInstrument<'a>
    for GetBucketIntelligentTieringConfigurationFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_intelligent_tiering_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketIntelligentTieringConfigurationOutput {}
instrument_aws_operation!(
    aws_sdk_s3::operation::get_bucket_intelligent_tiering_configuration
);

impl<'a> AwsBuilderInstrument<'a>
    for PutBucketIntelligentTieringConfigurationFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_intelligent_tiering_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketIntelligentTieringConfigurationOutput {}
instrument_aws_operation!(
    aws_sdk_s3::operation::put_bucket_intelligent_tiering_configuration
);

impl<'a> AwsBuilderInstrument<'a>
    for DeleteBucketIntelligentTieringConfigurationFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_intelligent_tiering_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput
    for DeleteBucketIntelligentTieringConfigurationOutput
{
}
instrument_aws_operation!(
    aws_sdk_s3::operation::delete_bucket_intelligent_tiering_configuration
);

impl<'a> AwsBuilderInstrument<'a>
    for ListBucketIntelligentTieringConfigurationsFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_bucket_intelligent_tiering_configurations(bucket)
    }
}
impl InstrumentedFluentBuilderOutput
    for ListBucketIntelligentTieringConfigurationsOutput
{
}
instrument_aws_operation!(
    aws_sdk_s3::operation::list_bucket_intelligent_tiering_configurations
);

// Bucket inventory configuration operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketInventoryConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_inventory_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketInventoryConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_inventory_configuration);

impl<'a> AwsBuilderInstrument<'a> for PutBucketInventoryConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_inventory_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketInventoryConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_inventory_configuration);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketInventoryConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_inventory_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketInventoryConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_inventory_configuration);

impl<'a> AwsBuilderInstrument<'a> for ListBucketInventoryConfigurationsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_bucket_inventory_configurations(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for ListBucketInventoryConfigurationsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_bucket_inventory_configurations);

// Bucket metrics configuration operations
impl<'a> AwsBuilderInstrument<'a> for GetBucketMetricsConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_metrics_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketMetricsConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_metrics_configuration);

impl<'a> AwsBuilderInstrument<'a> for PutBucketMetricsConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_bucket_metrics_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutBucketMetricsConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_bucket_metrics_configuration);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketMetricsConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_metrics_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketMetricsConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_metrics_configuration);

impl<'a> AwsBuilderInstrument<'a> for ListBucketMetricsConfigurationsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::list_bucket_metrics_configurations(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for ListBucketMetricsConfigurationsOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::list_bucket_metrics_configurations);

// Public access block operations
impl<'a> AwsBuilderInstrument<'a> for GetPublicAccessBlockFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_public_access_block(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetPublicAccessBlockOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_public_access_block);

impl<'a> AwsBuilderInstrument<'a> for PutPublicAccessBlockFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::put_public_access_block(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for PutPublicAccessBlockOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::put_public_access_block);

impl<'a> AwsBuilderInstrument<'a> for DeletePublicAccessBlockFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_public_access_block(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeletePublicAccessBlockOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_public_access_block);

// Bucket metadata operations
impl<'a> AwsBuilderInstrument<'a> for CreateBucketMetadataConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::create_bucket_metadata_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for CreateBucketMetadataConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::create_bucket_metadata_configuration);

impl<'a> AwsBuilderInstrument<'a>
    for CreateBucketMetadataTableConfigurationFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::create_bucket_metadata_table_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for CreateBucketMetadataTableConfigurationOutput {}
instrument_aws_operation!(
    aws_sdk_s3::operation::create_bucket_metadata_table_configuration
);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketMetadataConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_metadata_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketMetadataConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket_metadata_configuration);

impl<'a> AwsBuilderInstrument<'a>
    for DeleteBucketMetadataTableConfigurationFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket_metadata_table_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketMetadataTableConfigurationOutput {}
instrument_aws_operation!(
    aws_sdk_s3::operation::delete_bucket_metadata_table_configuration
);

impl<'a> AwsBuilderInstrument<'a> for GetBucketMetadataConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_metadata_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketMetadataConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_metadata_configuration);

impl<'a> AwsBuilderInstrument<'a> for GetBucketMetadataTableConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::get_bucket_metadata_table_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput for GetBucketMetadataTableConfigurationOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::get_bucket_metadata_table_configuration);

impl<'a> AwsBuilderInstrument<'a>
    for UpdateBucketMetadataInventoryTableConfigurationFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::update_bucket_metadata_inventory_table_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput
    for UpdateBucketMetadataInventoryTableConfigurationOutput
{
}
instrument_aws_operation!(
    aws_sdk_s3::operation::update_bucket_metadata_inventory_table_configuration
);

impl<'a> AwsBuilderInstrument<'a>
    for UpdateBucketMetadataJournalTableConfigurationFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::update_bucket_metadata_journal_table_configuration(bucket)
    }
}
impl InstrumentedFluentBuilderOutput
    for UpdateBucketMetadataJournalTableConfigurationOutput
{
}
instrument_aws_operation!(
    aws_sdk_s3::operation::update_bucket_metadata_journal_table_configuration
);
