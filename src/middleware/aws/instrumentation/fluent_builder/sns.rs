use paste::paste;
use std::collections::HashSet;

use super::{AwsInstrumentBuilder, InstrumentedFluentBuilder, utils::*};
use crate::{middleware::aws::*, semconv};

impl<'a> AwsInstrumentBuilder<'a>
    for aws_sdk_sns::operation::publish::builders::PublishFluentBuilder
{
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let topic_arn = self.get_target_arn().clone().unwrap_or_default();
        SnsSpanBuilder::publish(topic_arn)
    }
}
instrument_aws_operation!(aws_sdk_sns::operation::publish);
