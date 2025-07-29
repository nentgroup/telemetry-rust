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
