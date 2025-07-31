/// AWS DynamoDB operations
///
/// API Reference: https://docs.aws.amazon.com/amazondynamodb/latest/APIReference/API_Operations_Amazon_DynamoDB.html
use crate::{KeyValue, StringValue, Value, semconv};

use super::*;

#[allow(deprecated)]
pub const LEGACY_DB_NAME: &str = semconv::DB_NAME;

#[allow(deprecated)]
pub const LEGACY_DB_SYSTEM: &str = semconv::DB_SYSTEM;

/// Builder for DynamoDB-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for DynamoDB operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with DynamoDB-specific attributes.
pub enum DynamodbSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates a DynamoDB operation span builder.
    ///
    /// This method creates a span builder configured for DynamoDB operations with
    /// appropriate semantic attributes according to OpenTelemetry conventions.
    ///
    /// # Arguments
    ///
    /// * `method` - The DynamoDB operation method name (e.g., "GetItem", "PutItem")
    /// * `table_names` - Iterator of table names involved in the operation
    ///
    /// # Returns
    ///
    /// A configured AWS span builder for the DynamoDB operation
    pub fn dynamodb(
        method: impl Into<StringValue>,
        table_names: impl IntoIterator<Item = impl Into<StringValue>>,
    ) -> Self {
        let method: StringValue = method.into();
        let table_names: Vec<StringValue> =
            table_names.into_iter().map(|item| item.into()).collect();
        let mut attributes = vec![
            KeyValue::new(LEGACY_DB_SYSTEM, "dynamodb"),
            KeyValue::new(semconv::DB_OPERATION_NAME, method.clone()),
        ];
        match table_names.len() {
            0 => {}
            1 => {
                attributes.extend([
                    KeyValue::new(LEGACY_DB_NAME, table_names[0].clone()),
                    KeyValue::new(semconv::DB_NAMESPACE, table_names[0].clone()),
                    KeyValue::new(
                        semconv::AWS_DYNAMODB_TABLE_NAMES,
                        Value::Array(table_names.into()),
                    ),
                ]);
            }
            _ => {
                attributes.push(KeyValue::new(
                    semconv::AWS_DYNAMODB_TABLE_NAMES,
                    Value::Array(table_names.into()),
                ));
            }
        }
        Self::client("DynamoDB", method, attributes)
    }
}

macro_rules! dynamodb_global_operation {
    ($op: ident) => {
        impl DynamodbSpanBuilder {
            #[doc = concat!("Creates a span builder for the DynamoDB ", stringify!($op), " operation.")]
            ///
            /// This operation does not require specific table names as it operates globally.
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::dynamodb(
                    stringify_camel!($op),
                    std::iter::empty::<StringValue>(),
                )
            }
        }
    };
}

macro_rules! dynamodb_table_operation {
    ($op: ident) => {
        impl DynamodbSpanBuilder {
            #[doc = concat!("Creates a span builder for the DynamoDB ", stringify!($op), " operation on a specific table.")]
            ///
            /// # Arguments
            ///
            /// * `table_name` - The name of the DynamoDB table
            pub fn $op<'a>(table_name: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::dynamodb(
                    stringify_camel!($op),
                    std::iter::once(table_name),
                )
            }
        }
    };
}

macro_rules! dynamodb_table_arn_operation {
    ($op: ident) => {
        impl DynamodbSpanBuilder {
            #[doc = concat!("Creates a span builder for the DynamoDB ", stringify!($op), " operation using a table ARN.")]
            ///
            /// # Arguments
            ///
            /// * `table_arn` - The ARN of the DynamoDB table
            pub fn $op<'a>(table_arn: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                let table_arn = table_arn.into();
                AwsSpanBuilder::dynamodb(
                    stringify_camel!($op),
                    std::iter::empty::<StringValue>(),
                )
                .attributes(vec![
                    KeyValue::new(LEGACY_DB_NAME, table_arn.clone()),
                    KeyValue::new(semconv::DB_NAMESPACE, table_arn),
                ])
            }
        }
    };
}

macro_rules! dynamodb_batch_operation {
    ($op: ident) => {
        impl DynamodbSpanBuilder {
            #[doc = concat!("Creates a span builder for the DynamoDB ", stringify!($op), " batch operation.")]
            ///
            /// # Arguments
            ///
            /// * `table_names` - Iterator of table names involved in the batch operation
            pub fn $op<'a>(
                table_names: impl IntoIterator<Item = impl Into<StringValue>>,
            ) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::dynamodb(stringify_camel!($op), table_names)
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
dynamodb_table_operation!(delete_resource_policy);
dynamodb_table_operation!(delete_table);
dynamodb_table_operation!(describe_continuous_backups);
dynamodb_table_operation!(describe_contributor_insights);
dynamodb_table_operation!(describe_kinesis_streaming_destination);
dynamodb_table_operation!(describe_table);
dynamodb_table_operation!(describe_table_replica_auto_scaling);
dynamodb_table_operation!(describe_time_to_live);
dynamodb_table_operation!(disable_kinesis_streaming_destination);
dynamodb_table_operation!(enable_kinesis_streaming_destination);
dynamodb_table_operation!(get_item);
dynamodb_table_operation!(get_resource_policy);
dynamodb_table_operation!(import_table);
dynamodb_table_operation!(list_backups);
dynamodb_table_operation!(list_contributor_insights);
dynamodb_table_operation!(list_tags_of_resource);
dynamodb_table_operation!(put_item);
dynamodb_table_operation!(put_resource_policy);
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
dynamodb_batch_operation!(batch_get_item);
dynamodb_batch_operation!(batch_write_item);
dynamodb_batch_operation!(transact_get_items);
dynamodb_batch_operation!(transact_write_items);

// PartiQL operations
dynamodb_batch_operation!(execute_statement);
dynamodb_batch_operation!(batch_execute_statement);
dynamodb_batch_operation!(execute_transaction);
