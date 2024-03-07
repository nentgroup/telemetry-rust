use crate::{semcov, Context, KeyValue, StringValue};

use super::{AwsOperation, AwsSpan};

pub struct SnsOperation<'a>(AwsOperation<'a>);

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

impl<'a> From<SnsOperation<'a>> for AwsOperation<'a> {
    #[inline]
    fn from(outer: SnsOperation<'a>) -> Self {
        outer.0
    }
}

impl<'a> SnsOperation<'a> {
    #[inline]
    pub fn attribute(self, attribute: KeyValue) -> Self {
        Self(self.0.attribute(attribute))
    }

    #[inline]
    pub fn context(self, context: &'a Context) -> Self {
        Self(self.0.context(context))
    }

    #[inline]
    pub fn set_context(self, context: Option<&'a Context>) -> Self {
        Self(self.0.set_context(context))
    }

    #[inline]
    pub fn start(self) -> AwsSpan {
        self.0.start()
    }
}
