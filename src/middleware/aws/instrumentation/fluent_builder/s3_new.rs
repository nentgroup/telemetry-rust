/// AWS S3 fluent builder instrumentation implementations
#[cfg(feature = "aws-s3")]
#[allow(unused_imports)]
use super::{utils::*, *};

// Object operations
#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::get_object::builders::GetObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::get_object(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::get_object::GetObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.content_length().as_ref()
                .map(|length| KeyValue::new("aws.s3.object.size", *length)),
            self.content_type().as_attribute("aws.s3.object.content_type"),
            self.last_modified().as_ref()
                .map(|modified| KeyValue::new("aws.s3.object.last_modified", modified.to_string())),
            self.e_tag().as_attribute("aws.s3.object.etag"),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::get_object);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::put_object::builders::PutObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_content_length().as_ref()
                .map(|length| KeyValue::new("aws.s3.object.size", *length)),
        ];
        S3SpanBuilder::put_object(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::put_object::PutObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.server_side_encryption().as_ref()
                .map(|sse| KeyValue::new("aws.s3.object.server_side_encryption", sse.as_str().to_string())),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::put_object);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::delete_object::builders::DeleteObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::delete_object(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::delete_object::DeleteObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.delete_marker().as_ref()
                .map(|dm| KeyValue::new("aws.s3.object.delete_marker", *dm)),
            self.version_id().as_attribute("aws.s3.object.version_id"),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::delete_object);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::head_object::builders::HeadObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_version_id().as_attribute("aws.s3.version_id"),
        ];
        S3SpanBuilder::head_object(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::head_object::HeadObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.content_length().as_ref()
                .map(|len| KeyValue::new("aws.s3.object.size", *len)),
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.content_type().as_attribute("aws.s3.object.content_type"),
            self.last_modified().as_ref()
                .map(|lm| KeyValue::new("aws.s3.object.last_modified", lm.to_string())),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::head_object);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::copy_object::builders::CopyObjectFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_copy_source().as_attribute("aws.s3.copy_source"),
        ];
        S3SpanBuilder::copy_object(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::copy_object::CopyObjectOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.copy_object_result().as_ref()
                .and_then(|cor| cor.e_tag())
                .map(|etag| KeyValue::new("aws.s3.object.etag", etag.to_string())),
            self.server_side_encryption().as_ref()
                .map(|sse| KeyValue::new("aws.s3.object.server_side_encryption", sse.as_str().to_string())),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::copy_object);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::list_objects_v2::builders::ListObjectsV2FluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_prefix().as_attribute("aws.s3.list.prefix"),
            self.get_max_keys().as_ref()
                .map(|max| KeyValue::new("aws.s3.list.max_keys", *max as i64)),
        ];
        S3SpanBuilder::list_objects_v2(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.key_count().as_ref()
                .map(|count| KeyValue::new("aws.s3.object.count", *count as i64)),
            self.is_truncated().as_ref()
                .map(|truncated| KeyValue::new("aws.s3.object.is_truncated", *truncated)),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::list_objects_v2);

// Bucket operations
#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::create_bucket::builders::CreateBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::create_bucket(bucket_name)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::create_bucket::CreateBucketOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.location().as_attribute("aws.s3.bucket.location"),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::create_bucket);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::delete_bucket::builders::DeleteBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::delete_bucket(bucket_name)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::delete_bucket::DeleteBucketOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        // Delete operations typically don't have meaningful output attributes
        None
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::delete_bucket);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::head_bucket::builders::HeadBucketFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        S3SpanBuilder::head_bucket(bucket_name)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::head_bucket::HeadBucketOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.bucket_region().as_attribute("aws.s3.bucket.region"),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::head_bucket);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::list_buckets::builders::ListBucketsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        S3SpanBuilder::list_buckets()
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::list_buckets::ListBucketsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.buckets().as_ref()
                .map(|buckets| KeyValue::new("aws.s3.bucket.count", buckets.len() as i64)),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::list_buckets);

// Multipart upload operations
#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::create_multipart_upload::builders::CreateMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
        ];
        S3SpanBuilder::create_multipart_upload(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::create_multipart_upload::CreateMultipartUploadOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.upload_id().as_attribute("aws.s3.multipart.upload_id"),
            self.server_side_encryption().as_ref()
                .map(|sse| KeyValue::new("aws.s3.object.server_side_encryption", sse.as_str().to_string())),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::create_multipart_upload);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::complete_multipart_upload::builders::CompleteMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id().as_attribute("aws.s3.multipart.upload_id"),
            self.get_multipart_upload().as_ref()
                .and_then(|mu| mu.parts())
                .map(|parts| KeyValue::new("aws.s3.multipart.parts_count", parts.len() as i64)),
        ];
        S3SpanBuilder::complete_multipart_upload(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::complete_multipart_upload::CompleteMultipartUploadOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.location().as_attribute("aws.s3.object.location"),
            self.server_side_encryption().as_ref()
                .map(|sse| KeyValue::new("aws.s3.object.server_side_encryption", sse.as_str().to_string())),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::complete_multipart_upload);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::abort_multipart_upload::builders::AbortMultipartUploadFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id().as_attribute("aws.s3.multipart.upload_id"),
        ];
        S3SpanBuilder::abort_multipart_upload(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::abort_multipart_upload::AbortMultipartUploadOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        // Abort operations typically don't have meaningful output attributes
        None
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::abort_multipart_upload);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::upload_part::builders::UploadPartFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id().as_attribute("aws.s3.multipart.upload_id"),
            self.get_part_number().as_ref()
                .map(|part| KeyValue::new("aws.s3.multipart.part_number", *part as i64)),
        ];
        S3SpanBuilder::upload_part(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::upload_part::UploadPartOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.e_tag().as_attribute("aws.s3.object.etag"),
            self.server_side_encryption().as_ref()
                .map(|sse| KeyValue::new("aws.s3.object.server_side_encryption", sse.as_str().to_string())),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::upload_part);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::list_parts::builders::ListPartsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_key().as_attribute("aws.s3.key"),
            self.get_upload_id().as_attribute("aws.s3.multipart.upload_id"),
            self.get_max_parts().as_ref()
                .map(|max| KeyValue::new("aws.s3.multipart.max_parts", *max as i64)),
        ];
        S3SpanBuilder::list_parts(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::list_parts::ListPartsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.parts().as_ref()
                .map(|parts| KeyValue::new("aws.s3.multipart.parts_count", parts.len() as i64)),
            self.max_parts().as_ref()
                .map(|max| KeyValue::new("aws.s3.multipart.max_parts", *max as i64)),
            self.is_truncated().as_ref()
                .map(|truncated| KeyValue::new("aws.s3.multipart.is_truncated", *truncated)),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::list_parts);

#[cfg(feature = "aws-s3")]
impl<'a> AwsBuilderInstrument<'a> for aws_sdk_s3::operation::delete_objects::builders::DeleteObjectsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let bucket_name = self.get_bucket().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_delete().as_ref()
                .and_then(|del| del.objects())
                .map(|objects| KeyValue::new("aws.s3.batch.request_count", objects.len() as i64)),
        ];
        S3SpanBuilder::delete_objects(bucket_name).attributes(attributes)
    }
}

#[cfg(feature = "aws-s3")]
impl InstrumentedFluentBuilderOutput for aws_sdk_s3::operation::delete_objects::DeleteObjectsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.deleted().as_ref()
                .map(|deleted| KeyValue::new("aws.s3.batch.deleted_count", deleted.len() as i64)),
            self.errors().as_ref()
                .map(|errors| KeyValue::new("aws.s3.batch.error_count", errors.len() as i64)),
        ]
    }
}

#[cfg(feature = "aws-s3")]
instrument_aws_operation!(aws_sdk_s3::operation::delete_objects);
