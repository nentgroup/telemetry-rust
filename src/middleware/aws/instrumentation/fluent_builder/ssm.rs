/// AWS SSM Parameter Store fluent builder instrumentation implementations
use super::{utils::*, *};

// Single parameter operations
impl<'a> AwsBuilderInstrument<'a> for GetParameterFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let name = self.get_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_with_decryption()
                .as_attribute("aws.ssm.with_decryption"),
        ];
        SsmSpanBuilder::get_parameter(name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetParameterOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.parameter()
                .map(|p| KeyValue::new("aws.ssm.parameter_version", p.version())),
        ]
    }
}
instrument_aws_operation!(aws_sdk_ssm::operation::get_parameter);

impl<'a> AwsBuilderInstrument<'a> for PutParameterFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let name = self.get_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_overwrite().as_attribute("aws.ssm.overwrite"),
            self.get_type()
                .as_ref()
                .map(|t| KeyValue::new("aws.ssm.type", t.as_str().to_owned())),
            self.get_tier()
                .as_ref()
                .map(|t| KeyValue::new("aws.ssm.tier", t.as_str().to_owned())),
        ];
        SsmSpanBuilder::put_parameter(name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for PutParameterOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.tier()
                .as_ref()
                .map(|t| KeyValue::new("aws.ssm.tier", t.as_str().to_owned())),
        ]
    }
}
instrument_aws_operation!(aws_sdk_ssm::operation::put_parameter);

impl<'a> AwsBuilderInstrument<'a> for DeleteParameterFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let name = self.get_name().clone().unwrap_or_default();
        SsmSpanBuilder::delete_parameter(name)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteParameterOutput {}
instrument_aws_operation!(aws_sdk_ssm::operation::delete_parameter);

impl<'a> AwsBuilderInstrument<'a> for GetParameterHistoryFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let name = self.get_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_with_decryption()
                .as_attribute("aws.ssm.with_decryption"),
            self.get_max_results()
                .map(|v| KeyValue::new("aws.ssm.max_results", v as i64)),
        ];
        SsmSpanBuilder::get_parameter_history(name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetParameterHistoryOutput {}
instrument_aws_operation!(aws_sdk_ssm::operation::get_parameter_history);

impl<'a> AwsBuilderInstrument<'a> for LabelParameterVersionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let name = self.get_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_parameter_version()
                .map(|v| KeyValue::new("aws.ssm.parameter_version", v)),
        ];
        SsmSpanBuilder::label_parameter_version(name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for LabelParameterVersionOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.invalid_labels()
                .len()
                .as_attribute("aws.ssm.invalid_labels_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_ssm::operation::label_parameter_version);

impl<'a> AwsBuilderInstrument<'a> for UnlabelParameterVersionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let name = self.get_name().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_parameter_version()
                .map(|v| KeyValue::new("aws.ssm.parameter_version", v)),
        ];
        SsmSpanBuilder::unlabel_parameter_version(name).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for UnlabelParameterVersionOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.removed_labels()
                .len()
                .as_attribute("aws.ssm.removed_labels_count"),
            self.invalid_labels()
                .len()
                .as_attribute("aws.ssm.invalid_labels_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_ssm::operation::unlabel_parameter_version);

// Multi-parameter operations
impl<'a> AwsBuilderInstrument<'a> for GetParametersFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes = attributes![
            self.get_names().as_ref().map(|names| KeyValue::new(
                "aws.ssm.parameter_count",
                names.len() as i64
            )),
            self.get_with_decryption()
                .as_attribute("aws.ssm.with_decryption"),
        ];
        SsmSpanBuilder::get_parameters().attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetParametersOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.invalid_parameters()
                .len()
                .as_attribute("aws.ssm.invalid_parameters_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_ssm::operation::get_parameters);

impl<'a> AwsBuilderInstrument<'a> for DeleteParametersFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes =
            attributes![self.get_names().as_ref().map(|names| KeyValue::new(
                "aws.ssm.parameter_count",
                names.len() as i64
            )),];
        SsmSpanBuilder::delete_parameters().attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteParametersOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.deleted_parameters()
                .len()
                .as_attribute("aws.ssm.deleted_count"),
            self.invalid_parameters()
                .len()
                .as_attribute("aws.ssm.invalid_parameters_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_ssm::operation::delete_parameters);

// Path-based operations
impl<'a> AwsBuilderInstrument<'a> for GetParametersByPathFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let path = self.get_path().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_recursive().as_attribute("aws.ssm.recursive"),
            self.get_with_decryption()
                .as_attribute("aws.ssm.with_decryption"),
            self.get_max_results()
                .map(|v| KeyValue::new("aws.ssm.max_results", v as i64)),
        ];
        SsmSpanBuilder::get_parameters_by_path(path).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetParametersByPathOutput {}
instrument_aws_operation!(aws_sdk_ssm::operation::get_parameters_by_path);

// List/describe operations
impl<'a> AwsBuilderInstrument<'a> for DescribeParametersFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes = attributes![
            self.get_max_results()
                .map(|v| KeyValue::new("aws.ssm.max_results", v as i64)),
        ];
        SsmSpanBuilder::describe_parameters().attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for DescribeParametersOutput {}
instrument_aws_operation!(aws_sdk_ssm::operation::describe_parameters);
