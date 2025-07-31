/// AWS Firehose operations
///
/// API Reference: https://docs.aws.amazon.com/firehose/latest/APIReference/API_Operations.html
use crate::{KeyValue, StringValue, semconv};

use super::*;

/// Builder for Firehose-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for Firehose operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with Firehose-specific messaging attributes.
pub enum FirehoseSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates a Firehose operation span builder.
    ///
    /// This method creates a span builder configured for Firehose operations with
    /// appropriate messaging semantic attributes.
    ///
    /// # Arguments
    ///
    /// * `operation_kind` - The type of messaging operation being performed
    /// * `method` - The Firehose operation method name
    /// * `stream_name` - Optional stream name for operations that target specific streams
    pub fn firehose(
        operation_kind: MessagingOperationKind,
        method: impl Into<StringValue>,
        stream_name: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = vec![
            KeyValue::new(semconv::MESSAGING_SYSTEM, "aws_firehose"),
            KeyValue::new(semconv::MESSAGING_OPERATION_TYPE, operation_kind.as_str()),
        ];
        if let Some(stream_name) = stream_name {
            attributes.push(KeyValue::new(
                semconv::MESSAGING_DESTINATION_NAME,
                stream_name.into(),
            ))
        }
        Self::new(operation_kind.into(), "Firehose", method, attributes)
    }
}

macro_rules! firehose_global_operation {
    ($op: ident) => {
        impl FirehoseSpanBuilder {
            #[doc = concat!("Creates a span builder for the Firehose ", stringify!($op), " global operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::firehose(
                    MessagingOperationKind::Control,
                    stringify_camel!($op),
                    None::<StringValue>,
                )
            }
        }
    };
}

macro_rules! firehose_publish_operation {
    ($op: ident, $kind: expr) => {
        impl FirehoseSpanBuilder {
            #[doc = concat!("Creates a span builder for the Firehose ", stringify!($op), " operation.")]
            ///
            /// # Arguments
            ///
            /// * `stream_name` - The name of the Firehose delivery stream
            pub fn $op<'a>(stream_name: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::firehose($kind, stringify_camel!($op), Some(stream_name))
            }
        }
    };
}

macro_rules! firehose_stream_operation {
    ($op: ident) => {
        impl FirehoseSpanBuilder {
            #[doc = concat!("Creates a span builder for the Firehose ", stringify!($op), " stream operation.")]
            ///
            /// # Arguments
            ///
            /// * `stream_name` - The name of the Firehose delivery stream
            pub fn $op<'a>(stream_name: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::firehose(
                    MessagingOperationKind::Control,
                    stringify_camel!($op),
                    Some(stream_name),
                )
            }
        }
    };
}

// publish operation
firehose_publish_operation!(put_record, MessagingOperationKind::Create);
firehose_publish_operation!(put_record_batch, MessagingOperationKind::Send);

// global operations
firehose_global_operation!(list_delivery_streams);

// control plane stream operations
firehose_stream_operation!(create_delivery_stream);
firehose_stream_operation!(delete_delivery_stream);
firehose_stream_operation!(describe_delivery_stream);
firehose_stream_operation!(list_tags_for_delivery_stream);
firehose_stream_operation!(start_delivery_stream_encryption);
firehose_stream_operation!(stop_delivery_stream_encryption);
firehose_stream_operation!(tag_delivery_stream);
firehose_stream_operation!(untag_delivery_stream);
firehose_stream_operation!(update_destination);
