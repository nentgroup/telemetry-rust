use crate::{semcov, Context, KeyValue, StringValue};

use super::{AwsOperation, AwsSpan};

pub struct DynamoDBOperation<'a>(AwsOperation<'a>);

impl DynamoDBOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        table_name: impl Into<StringValue>,
    ) -> Self {
        let method: StringValue = method.into();
        let table_name: StringValue = table_name.into();
        let attributes = vec![
            semcov::DB_SYSTEM.string("dynamodb"),
            semcov::DB_NAME.string(table_name.clone()),
            semcov::DB_OPERATION.string(method.clone()),
            semcov::AWS_DYNAMODB_TABLE_NAMES.array(vec![table_name]),
        ];
        Self(AwsOperation::client("DynamoDB", method, attributes))
    }
}

impl<'a> From<DynamoDBOperation<'a>> for AwsOperation<'a> {
    #[inline]
    fn from(outer: DynamoDBOperation<'a>) -> Self {
        outer.0
    }
}

impl<'a> DynamoDBOperation<'a> {
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
