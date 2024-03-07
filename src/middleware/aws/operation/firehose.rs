use crate::{semcov, StringValue};

use super::{aws_target, AwsOperation};

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
