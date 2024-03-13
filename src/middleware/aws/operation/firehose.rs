use crate::{semcov, StringValue};

use super::*;

pub enum FirehoseOperation {}

impl<'a> AwsOperation<'a> {
    pub fn firehose(
        operation_kind: MessagingOperationKind,
        method: impl Into<StringValue>,
        stream_name: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = vec![
            semcov::MESSAGING_SYSTEM.string("aws_firehose"),
            semcov::MESSAGING_OPERATION.string(operation_kind.as_str()),
        ];
        if let Some(stream_name) = stream_name {
            attributes.push(semcov::MESSAGING_DESTINATION_NAME.string(stream_name))
        }
        Self::new(operation_kind.into(), "Firehose", method, attributes)
    }
}

macro_rules! firehose_global_operation {
    ($op: ident) => {
        impl FirehoseOperation {
            #[inline]
            pub fn $op<'a>() -> AwsOperation<'a> {
                AwsOperation::firehose(
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
        impl FirehoseOperation {
            pub fn $op<'a>(stream_name: impl Into<StringValue>) -> AwsOperation<'a> {
                AwsOperation::firehose($kind, stringify_camel!($op), Some(stream_name))
            }
        }
    };
}

macro_rules! firehose_stream_operation {
    ($op: ident) => {
        impl FirehoseOperation {
            pub fn $op<'a>(stream_name: impl Into<StringValue>) -> AwsOperation<'a> {
                AwsOperation::firehose(
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
firehose_publish_operation!(put_record_batch, MessagingOperationKind::Publish);

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
