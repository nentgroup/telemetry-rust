use crate::{semcov, StringValue};

use super::*;

aws_target!(FirehoseOperation);

impl FirehoseOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        stream_name: impl Into<StringValue>,
    ) -> Self {
        let attributes = vec![
            semcov::MESSAGING_SYSTEM.string("aws_firehose"),
            semcov::MESSAGING_OPERATION.string("publish"),
            semcov::MESSAGING_DESTINATION_NAME.string(stream_name),
        ];
        Self(AwsOperation::producer("Firehose", method, attributes))
    }
}

macro_rules! firehose_operation {
    ($op: ident) => {
        impl<'a> FirehoseOperation<'a> {
            #[inline]
            pub fn $op(stream_name: impl Into<StringValue>) -> Self {
                Self::new(stringify_camel!($op), stream_name)
            }
        }
    };
}

firehose_operation!(create_delivery_stream);
firehose_operation!(delete_delivery_stream);
firehose_operation!(describe_delivery_stream);
firehose_operation!(list_delivery_streams);
firehose_operation!(list_tags_for_delivery_stream);
firehose_operation!(put_record); // publish
firehose_operation!(put_record_batch); // publish
firehose_operation!(start_delivery_stream_encryption);
firehose_operation!(stop_delivery_stream_encryption);
firehose_operation!(tag_delivery_stream);
firehose_operation!(untag_delivery_stream);
firehose_operation!(update_destination);
