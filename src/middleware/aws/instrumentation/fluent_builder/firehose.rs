use paste::paste;
use std::collections::HashSet;

use super::{AwsInstrumentBuilder, InstrumentedFluentBuilder, utils::*};
use crate::{middleware::aws::*, semconv};

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_firehose::operation::put_record::builders::PutRecordFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let stream_name = self.get_delivery_stream_name().clone().unwrap_or_default();
        FirehoseSpanBuilder::put_record(stream_name)
    }
}
instrument_aws_operation!(aws_sdk_firehose::operation::put_record);
