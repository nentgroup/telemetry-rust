use crate::{semcov, Context, KeyValue, StringValue};

use super::{AwsOperation, AwsSpan};

pub struct FirehoseOperation<'a>(AwsOperation<'a>);

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

impl<'a> From<FirehoseOperation<'a>> for AwsOperation<'a> {
    #[inline]
    fn from(outer: FirehoseOperation<'a>) -> Self {
        outer.0
    }
}

impl<'a> FirehoseOperation<'a> {
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
