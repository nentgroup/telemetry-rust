use crate::{semcov, StringValue};

use super::*;

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

macro_rules! dynamodb_operation {
    ($op: ident) => {
        impl<'a> DynamoDBOperation<'a> {
            #[inline]
            pub fn $op(table_name: impl Into<StringValue>) -> Self {
                Self::new(stringify_camel!($op), table_name)
            }
        }
    };
}

dynamodb_operation!(batch_execute_statement);
dynamodb_operation!(batch_get_item);
dynamodb_operation!(batch_write_item);
dynamodb_operation!(create_backup);
dynamodb_operation!(create_global_table);
dynamodb_operation!(create_table);
dynamodb_operation!(delete_backup);
dynamodb_operation!(delete_item);
dynamodb_operation!(delete_table);
dynamodb_operation!(describe_backup);
dynamodb_operation!(describe_continuous_backups);
dynamodb_operation!(describe_contributor_insights);
dynamodb_operation!(describe_endpoints);
dynamodb_operation!(describe_export);
dynamodb_operation!(describe_global_table);
dynamodb_operation!(describe_global_table_settings);
dynamodb_operation!(describe_import);
dynamodb_operation!(describe_kinesis_streaming_destination);
dynamodb_operation!(describe_limits);
dynamodb_operation!(describe_table);
dynamodb_operation!(describe_table_replica_auto_scaling);
dynamodb_operation!(describe_time_to_live);
dynamodb_operation!(disable_kinesis_streaming_destination);
dynamodb_operation!(enable_kinesis_streaming_destination);
dynamodb_operation!(execute_statement);
dynamodb_operation!(execute_transaction);
dynamodb_operation!(export_table_to_point_in_time);
dynamodb_operation!(get_item);
dynamodb_operation!(import_table);
dynamodb_operation!(list_backups);
dynamodb_operation!(list_contributor_insights);
dynamodb_operation!(list_exports);
dynamodb_operation!(list_global_tables);
dynamodb_operation!(list_imports);
dynamodb_operation!(list_tables);
dynamodb_operation!(list_tags_of_resource);
dynamodb_operation!(put_item);
dynamodb_operation!(query);
dynamodb_operation!(restore_table_from_backup);
dynamodb_operation!(restore_table_to_point_in_time);
dynamodb_operation!(scan);
dynamodb_operation!(tag_resource);
dynamodb_operation!(transact_get_items);
dynamodb_operation!(transact_write_items);
dynamodb_operation!(untag_resource);
dynamodb_operation!(update_continuous_backups);
dynamodb_operation!(update_contributor_insights);
dynamodb_operation!(update_global_table);
dynamodb_operation!(update_global_table_settings);
dynamodb_operation!(update_item);
dynamodb_operation!(update_kinesis_streaming_destination);
dynamodb_operation!(update_table);
dynamodb_operation!(update_table_replica_auto_scaling);
dynamodb_operation!(update_time_to_live);
