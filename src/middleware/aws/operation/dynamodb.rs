use crate::{semcov, StringValue};

use super::*;

aws_target!(DynamoDBOperation);

impl DynamoDBOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        table_names: impl IntoIterator<Item = impl Into<StringValue>>,
    ) -> Self {
        let method: StringValue = method.into();
        let table_names: Vec<StringValue> =
            table_names.into_iter().map(|item| item.into()).collect();
        let mut attributes = vec![
            semcov::DB_SYSTEM.string("dynamodb"),
            semcov::DB_OPERATION.string(method.clone()),
        ];
        match table_names.len() {
            0 => {}
            1 => {
                attributes.extend([
                    semcov::DB_NAME.string(table_names[0].clone()),
                    semcov::AWS_DYNAMODB_TABLE_NAMES.array(table_names),
                ]);
            }
            _ => {
                attributes.push(semcov::AWS_DYNAMODB_TABLE_NAMES.array(table_names));
            }
        }
        Self(AwsOperation::client("DynamoDB", method, attributes))
    }
}

macro_rules! dynamodb_global_operation {
    ($op: ident) => {
        impl<'a> DynamoDBOperation<'a> {
            #[inline]
            pub fn $op() -> Self {
                Self::new(stringify_camel!($op), std::iter::empty::<StringValue>())
            }
        }
    };
}

macro_rules! dynamodb_table_operation {
    ($op: ident) => {
        impl<'a> DynamoDBOperation<'a> {
            pub fn $op(table_name: impl Into<StringValue>) -> Self {
                Self::new(stringify_camel!($op), std::iter::once(table_name))
            }
        }
    };
}

macro_rules! dynamodb_table_arn_operation {
    ($op: ident) => {
        impl<'a> DynamoDBOperation<'a> {
            pub fn $op(table_arn: impl Into<StringValue>) -> Self {
                Self::new(stringify_camel!($op), std::iter::empty::<StringValue>())
                    .attribute(semcov::DB_NAME.string(table_arn))
            }
        }
    };
}

macro_rules! dynamodb_batch_operation {
    ($op: ident) => {
        impl<'a> DynamoDBOperation<'a> {
            pub fn $op(
                table_names: impl IntoIterator<Item = impl Into<StringValue>>,
            ) -> Self {
                Self::new(stringify_camel!($op), table_names)
            }
        }
    };
}

// global operations
dynamodb_global_operation!(describe_endpoints);
dynamodb_global_operation!(describe_limits);
dynamodb_global_operation!(list_global_tables);
dynamodb_global_operation!(list_tables);

// operations on custom resources
dynamodb_global_operation!(delete_backup);
dynamodb_global_operation!(describe_backup);
dynamodb_global_operation!(describe_export);
dynamodb_global_operation!(describe_import);

// table operations (by name)
dynamodb_table_operation!(create_backup);
dynamodb_table_operation!(create_table);
dynamodb_table_operation!(delete_item);
dynamodb_table_operation!(delete_table);
dynamodb_table_operation!(describe_continuous_backups);
dynamodb_table_operation!(describe_contributor_insights);
dynamodb_table_operation!(describe_kinesis_streaming_destination);
dynamodb_table_operation!(describe_table);
dynamodb_table_operation!(describe_table_replica_auto_scaling);
dynamodb_table_operation!(describe_time_to_live);
dynamodb_table_operation!(disable_kinesis_streaming_destination);
dynamodb_table_operation!(enable_kinesis_streaming_destination);
dynamodb_table_operation!(execute_statement);
dynamodb_table_operation!(get_item);
dynamodb_table_operation!(import_table);
dynamodb_table_operation!(list_backups);
dynamodb_table_operation!(list_contributor_insights);
dynamodb_table_operation!(list_tags_of_resource);
dynamodb_table_operation!(put_item);
dynamodb_table_operation!(query);
dynamodb_table_operation!(restore_table_from_backup);
dynamodb_table_operation!(restore_table_to_point_in_time);
dynamodb_table_operation!(scan);
dynamodb_table_operation!(tag_resource);
dynamodb_table_operation!(untag_resource);
dynamodb_table_operation!(update_continuous_backups);
dynamodb_table_operation!(update_contributor_insights);
dynamodb_table_operation!(update_item);
dynamodb_table_operation!(update_kinesis_streaming_destination);
dynamodb_table_operation!(update_table);
dynamodb_table_operation!(update_table_replica_auto_scaling);
dynamodb_table_operation!(update_time_to_live);

// table operations (by arn)
dynamodb_table_arn_operation!(export_table_to_point_in_time);
dynamodb_table_arn_operation!(list_exports);
dynamodb_table_arn_operation!(list_imports);

// global table operations
dynamodb_table_operation!(create_global_table);
dynamodb_table_operation!(describe_global_table);
dynamodb_table_operation!(describe_global_table_settings);
dynamodb_table_operation!(update_global_table);
dynamodb_table_operation!(update_global_table_settings);

// batch operations
dynamodb_batch_operation!(batch_execute_statement);
dynamodb_batch_operation!(batch_get_item);
dynamodb_batch_operation!(batch_write_item);
dynamodb_batch_operation!(execute_transaction);
dynamodb_batch_operation!(transact_get_items);
dynamodb_batch_operation!(transact_write_items);
