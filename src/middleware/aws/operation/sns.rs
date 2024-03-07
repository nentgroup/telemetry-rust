use crate::{semcov, StringValue};

use super::{aws_target, AwsOperation};

aws_target!(SnsOperation);

impl SnsOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        topic_arn: impl Into<StringValue>,
    ) -> Self {
        let attributes = vec![
            semcov::MESSAGING_SYSTEM.string("aws_sns"),
            semcov::MESSAGING_OPERATION.string("publish"),
            semcov::MESSAGING_DESTINATION_NAME.string(topic_arn),
        ];
        Self(AwsOperation::producer("SNS", method, attributes))
    }
}
