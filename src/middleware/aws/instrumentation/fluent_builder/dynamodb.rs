use std::collections::HashSet;

use super::{AwsInstrumentBuilder, utils::*};
use crate::{middleware::aws::*, semconv};

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::get_item::builders::GetItemFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        let attributes = [
            self.get_consistent_read()
                .as_attribute(semconv::AWS_DYNAMODB_CONSISTENT_READ),
            self.get_projection_expression()
                .as_attribute(semconv::AWS_DYNAMODB_PROJECTION),
        ];
        DynamodbSpanBuilder::get_item(table_name)
            .attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::get_item);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::put_item::builders::PutItemFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::put_item(table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::put_item);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::update_item::builders::UpdateItemFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::update_item(table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::update_item);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::delete_item::builders::DeleteItemFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::delete_item(table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::delete_item);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::query::builders::QueryFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        let attributes = [
            self.get_attributes_to_get()
                .as_attribute(semconv::AWS_DYNAMODB_ATTRIBUTES_TO_GET),
            self.get_consistent_read()
                .as_attribute(semconv::AWS_DYNAMODB_CONSISTENT_READ),
            self.get_index_name()
                .as_attribute(semconv::AWS_DYNAMODB_INDEX_NAME),
            self.get_limit().as_attribute(semconv::AWS_DYNAMODB_LIMIT),
            self.get_projection_expression()
                .as_attribute(semconv::AWS_DYNAMODB_PROJECTION),
            self.get_scan_index_forward()
                .as_attribute(semconv::AWS_DYNAMODB_SCAN_FORWARD),
            self.get_select().as_attribute(semconv::AWS_DYNAMODB_SELECT),
        ];
        DynamodbSpanBuilder::query(table_name)
            .attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::query);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::scan::builders::ScanFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        let attributes = [
            self.get_attributes_to_get()
                .as_attribute(semconv::AWS_DYNAMODB_ATTRIBUTES_TO_GET),
            self.get_consistent_read()
                .as_attribute(semconv::AWS_DYNAMODB_CONSISTENT_READ),
            self.get_index_name()
                .as_attribute(semconv::AWS_DYNAMODB_INDEX_NAME),
            self.get_limit().as_attribute(semconv::AWS_DYNAMODB_LIMIT),
            self.get_projection_expression()
                .as_attribute(semconv::AWS_DYNAMODB_PROJECTION),
            self.get_select().as_attribute(semconv::AWS_DYNAMODB_SELECT),
            self.get_segment()
                .as_attribute(semconv::AWS_DYNAMODB_SEGMENT),
            self.get_total_segments()
                .as_attribute(semconv::AWS_DYNAMODB_TOTAL_SEGMENTS),
        ];
        DynamodbSpanBuilder::scan(table_name).attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::scan);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::batch_get_item::builders::BatchGetItemFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_names = self
            .get_request_items()
            .iter()
            .flat_map(|items| items.keys())
            .map(ToOwned::to_owned);
        DynamodbSpanBuilder::batch_get_item(table_names)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::batch_get_item);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::batch_write_item::builders::BatchWriteItemFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_names = self
            .get_request_items()
            .iter()
            .flat_map(|items| items.keys())
            .map(ToOwned::to_owned);
        DynamodbSpanBuilder::batch_write_item(table_names)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::batch_write_item);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::transact_get_items::builders::TransactGetItemsFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_names = self
            .get_transact_items()
            .iter()
            .flatten()
            .flat_map(|item| item.get().map(|op| op.table_name()))
            .collect::<HashSet<_>>()
            .into_iter()
            .map(ToOwned::to_owned);
        DynamodbSpanBuilder::transact_get_items(table_names)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::transact_get_items);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::transact_write_items::builders::TransactWriteItemsFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_names = self
            .get_transact_items()
            .iter()
            .flatten()
            .flat_map(|item| {
                [
                    item.condition_check().map(|op| op.table_name()),
                    item.put().map(|op| op.table_name()),
                    item.delete().map(|op| op.table_name()),
                    item.update().map(|op| op.table_name()),
                ]
            })
            .flatten()
            .collect::<HashSet<_>>()
            .into_iter()
            .map(ToOwned::to_owned);
        DynamodbSpanBuilder::transact_write_items(table_names)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::transact_write_items);

// Table management operations
impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::create_table::builders::CreateTableFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        let throughput = self.get_provisioned_throughput().as_ref();
        let attributes = [
            throughput
                .map(|pt| pt.read_capacity_units())
                .as_attribute(semconv::AWS_DYNAMODB_PROVISIONED_READ_CAPACITY),
            throughput
                .map(|pt| pt.write_capacity_units())
                .as_attribute(semconv::AWS_DYNAMODB_PROVISIONED_WRITE_CAPACITY),
        ];
        DynamodbSpanBuilder::create_table(table_name)
            .attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::create_table);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::delete_table::builders::DeleteTableFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::delete_table(table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::delete_table);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::describe_table::builders::DescribeTableFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::describe_table(table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::describe_table);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::update_table::builders::UpdateTableFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        let throughput = self.get_provisioned_throughput().as_ref();
        let attributes = [
            throughput
                .map(|pt| pt.read_capacity_units())
                .as_attribute(semconv::AWS_DYNAMODB_PROVISIONED_READ_CAPACITY),
            throughput
                .map(|pt| pt.write_capacity_units())
                .as_attribute(semconv::AWS_DYNAMODB_PROVISIONED_WRITE_CAPACITY),
        ];
        DynamodbSpanBuilder::update_table(table_name)
            .attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::update_table);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::list_tables::builders::ListTablesFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes = [
            self.get_exclusive_start_table_name()
                .as_attribute(semconv::AWS_DYNAMODB_EXCLUSIVE_START_TABLE),
            self.get_limit().as_attribute(semconv::AWS_DYNAMODB_LIMIT),
        ];
        DynamodbSpanBuilder::list_tables().attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::list_tables);

// Backup operations
impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::create_backup::builders::CreateBackupFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::create_backup(table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::create_backup);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::delete_backup::builders::DeleteBackupFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        DynamodbSpanBuilder::delete_backup()
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::delete_backup);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::describe_backup::builders::DescribeBackupFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        DynamodbSpanBuilder::describe_backup()
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::describe_backup);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::list_backups::builders::ListBackupsFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let table_name = self.get_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::list_backups(table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::list_backups);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::restore_table_from_backup::builders::RestoreTableFromBackupFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let target_table_name = self.get_target_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::restore_table_from_backup(target_table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::restore_table_from_backup);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::restore_table_to_point_in_time::builders::RestoreTableToPointInTimeFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let target_table_name = self.get_target_table_name().clone().unwrap_or_default();
        DynamodbSpanBuilder::restore_table_to_point_in_time(target_table_name)
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::restore_table_to_point_in_time);

// Execute operations
impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::execute_statement::builders::ExecuteStatementFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes = [
            self.get_consistent_read()
                .as_attribute(semconv::AWS_DYNAMODB_CONSISTENT_READ),
            self.get_limit().as_attribute(semconv::AWS_DYNAMODB_LIMIT),
        ];
        DynamodbSpanBuilder::execute_statement().attributes(attributes.into_iter().flatten())
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::execute_statement);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::batch_execute_statement::builders::BatchExecuteStatementFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        DynamodbSpanBuilder::batch_execute_statement()
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::batch_execute_statement);

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_dynamodb::operation::execute_transaction::builders::ExecuteTransactionFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        DynamodbSpanBuilder::execute_transaction()
    }
}
instrument_aws_operation!(aws_sdk_dynamodb::operation::execute_transaction);
