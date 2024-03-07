use crate::{semcov, StringValue};

use super::{aws_target, AwsOperation};

aws_target!(DynamoDBOperation);

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
