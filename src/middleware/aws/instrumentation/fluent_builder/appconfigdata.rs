/// AWS AppConfig Data fluent builder instrumentation implementations
use super::{utils::*, *};

// Session operations
impl<'a> AwsBuilderInstrument<'a> for StartConfigurationSessionFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let application_id = self
            .get_application_identifier()
            .clone()
            .unwrap_or_default();
        let attributes = attributes![
            self.get_environment_identifier()
                .as_attribute("aws.appconfigdata.environment_id"),
            self.get_configuration_profile_identifier()
                .as_attribute("aws.appconfigdata.configuration_profile_id"),
            self.get_required_minimum_poll_interval_in_seconds()
                .map(|v| KeyValue::new(
                    "aws.appconfigdata.required_minimum_poll_interval_in_seconds",
                    v as i64,
                )),
        ];
        AppConfigDataSpanBuilder::start_configuration_session(application_id)
            .attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for StartConfigurationSessionOutput {}
instrument_aws_operation!(aws_sdk_appconfigdata::operation::start_configuration_session);

// Configuration retrieval operations
impl<'a> AwsBuilderInstrument<'a> for GetLatestConfigurationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        AppConfigDataSpanBuilder::get_latest_configuration()
    }
}
impl InstrumentedFluentBuilderOutput for GetLatestConfigurationOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.content_type()
                .as_attribute("aws.appconfigdata.content_type"),
            self.version_label()
                .as_attribute("aws.appconfigdata.version_label"),
            Some(KeyValue::new(
                "aws.appconfigdata.next_poll_interval_in_seconds",
                self.next_poll_interval_in_seconds() as i64,
            )),
        ]
    }
}
instrument_aws_operation!(aws_sdk_appconfigdata::operation::get_latest_configuration);
