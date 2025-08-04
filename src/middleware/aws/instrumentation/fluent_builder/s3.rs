/// AWS S3 fluent builder instrumentation implementations
use super::{utils::*, *};

// Object operations
impl<'a> AwsBuilderInstrument<'a> for ListObjectsV2FluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_prefix().as_attribute("aws.s3.prefix"),
            self.get_max_keys()
                .as_ref()
                .map(|max| KeyValue::new("aws.s3.max_keys", *max as i64)),
        ];
        S3SpanBuilder::list_objects_v2(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for ListObjectsV2Output {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            (!self.contents().is_empty()).then(|| KeyValue::new(
                "aws.s3.object_count",
                self.contents().len() as i64
            )),
            self.is_truncated()
                .as_ref()
                .map(|truncated| KeyValue::new("aws.s3.is_truncated", *truncated)),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::list_objects_v2);

impl<'a> AwsBuilderInstrument<'a> for GetObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::get_object(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.content_length()
                .as_ref()
                .map(|length| KeyValue::new("aws.s3.object.size", *length)),
            self.content_type()
                .as_attribute("aws.s3.object.content_type"),
            self.last_modified().as_ref().map(|modified| KeyValue::new(
                "aws.s3.object.last_modified",
                modified.to_string()
            )),
            self.e_tag().as_attribute("aws.s3.object.etag"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::get_object);

impl<'a> AwsBuilderInstrument<'a> for PutObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_content_length()
                .as_ref()
                .map(|length| KeyValue::new("aws.s3.object.size", *length)),
        ];
        S3SpanBuilder::put_object(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for PutObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.server_side_encryption()
                .as_ref()
                .map(|sse| KeyValue::new(
                    "aws.s3.object.server_side_encryption",
                    sse.as_str().to_string()
                )),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::put_object);

impl<'a> AwsBuilderInstrument<'a> for DeleteObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::delete_object(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.delete_marker()
                .as_ref()
                .map(|dm| KeyValue::new("aws.s3.object.delete_marker", *dm)),
            self.version_id().as_attribute("aws.s3.object.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::delete_object);

impl<'a> AwsBuilderInstrument<'a> for HeadObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::head_object(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for HeadObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.content_length()
                .as_ref()
                .map(|len| KeyValue::new("aws.s3.object.size", *len)),
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.content_type()
                .as_attribute("aws.s3.object.content_type"),
            self.last_modified()
                .as_ref()
                .map(|lm| KeyValue::new("aws.s3.object.last_modified", lm.to_string())),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::head_object);

impl<'a> AwsBuilderInstrument<'a> for CopyObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_copy_source().as_attribute("aws.s3.copy_source"),
        ];
        S3SpanBuilder::copy_object(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for CopyObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.copy_object_result()
                .as_ref()
                .and_then(|cor| cor.e_tag())
                .map(|etag| KeyValue::new("aws.s3.object.etag", etag.to_string())),
            self.server_side_encryption()
                .as_ref()
                .map(|sse| KeyValue::new(
                    "aws.s3.object.server_side_encryption",
                    sse.as_str().to_string()
                )),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::copy_object);

// Bucket operations
impl<'a> AwsBuilderInstrument<'a> for CreateBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::create_bucket(bucket_name)
    }
}
impl InstrumentedFluentBuilderOutput for CreateBucketOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.location().as_attribute("aws.s3.bucket.location"),]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::create_bucket);

impl<'a> AwsBuilderInstrument<'a> for DeleteBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket(bucket_name)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteBucketOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket);

impl<'a> AwsBuilderInstrument<'a> for HeadBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::head_bucket(bucket_name)
    }
}
impl InstrumentedFluentBuilderOutput for HeadBucketOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.bucket_region().as_attribute("aws.s3.bucket.region"),]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::head_bucket);

impl<'a> AwsBuilderInstrument<'a> for ListBucketsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        S3SpanBuilder::list_buckets()
    }
}
impl InstrumentedFluentBuilderOutput for ListBucketsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![self.buckets().len().as_attribute("aws.s3.bucket.count"),]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::list_buckets);

// Multipart upload operations
impl<'a> AwsBuilderInstrument<'a> for CreateMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![self.get_key().as_attribute("aws.s3.key"),];
        S3SpanBuilder::create_multipart_upload(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for CreateMultipartUploadOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.upload_id().as_attribute("aws.s3.multipart.upload_id"),
            self.server_side_encryption()
                .as_ref()
                .map(|sse| KeyValue::new(
                    "aws.s3.object.server_side_encryption",
                    sse.as_str().to_string()
                )),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::create_multipart_upload);

impl<'a> AwsBuilderInstrument<'a> for CompleteMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id()
                .as_attribute("aws.s3.multipart.upload_id"),
            self.get_multipart_upload().as_ref().map(|mu| KeyValue::new(
                "aws.s3.multipart.parts_count",
                mu.parts().len() as i64
            )),
        ];
        S3SpanBuilder::complete_multipart_upload(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for CompleteMultipartUploadOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.location().as_attribute("aws.s3.object.location"),
            self.server_side_encryption()
                .as_ref()
                .map(|sse| KeyValue::new(
                    "aws.s3.object.server_side_encryption",
                    sse.as_str().to_string()
                )),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::complete_multipart_upload);

impl<'a> AwsBuilderInstrument<'a> for AbortMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id()
                .as_attribute("aws.s3.multipart.upload_id"),
        ];
        S3SpanBuilder::abort_multipart_upload(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for AbortMultipartUploadOutput {}
instrument_aws_operation!(aws_sdk_s3::operation::abort_multipart_upload);

impl<'a> AwsBuilderInstrument<'a> for UploadPartFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id()
                .as_attribute("aws.s3.multipart.upload_id"),
            self.get_part_number()
                .as_ref()
                .map(|part| KeyValue::new("aws.s3.multipart.part_number", *part as i64)),
        ];
        S3SpanBuilder::upload_part(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for UploadPartOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.server_side_encryption()
                .as_ref()
                .map(|sse| KeyValue::new(
                    "aws.s3.object.server_side_encryption",
                    sse.as_str().to_string()
                )),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::upload_part);

impl<'a> AwsBuilderInstrument<'a> for ListPartsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id()
                .as_attribute("aws.s3.multipart.upload_id"),
            self.get_max_parts()
                .as_ref()
                .map(|max| KeyValue::new("aws.s3.multipart.max_parts", *max as i64)),
        ];
        S3SpanBuilder::list_parts(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for ListPartsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.parts()
                .len()
                .as_attribute("aws.s3.multipart.parts_count"),
            self.max_parts().as_attribute("aws.s3.multipart.max_parts"),
            self.is_truncated()
                .as_attribute("aws.s3.multipart.is_truncated"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::list_parts);

impl<'a> AwsBuilderInstrument<'a> for DeleteObjectsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![self.get_delete().as_ref().map(
            |del| KeyValue::new("aws.s3.batch.request_count", del.objects().len() as i64)
        ),];
        S3SpanBuilder::delete_objects(bucket_name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteObjectsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.deleted()
                .len()
                .as_attribute("aws.s3.batch.deleted_count"),
            self.errors().len().as_attribute("aws.s3.batch.error_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_s3::operation::delete_objects);
