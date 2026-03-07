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

#[cfg(test)]
mod tests {
    use super::*;
    use aws_sdk_appconfigdata::{
        Client,
        operation::{
            get_latest_configuration::GetLatestConfigurationOutput,
            start_configuration_session::StartConfigurationSessionOutput,
        },
    };
    use aws_smithy_mocks::{mock, mock_client};

    #[tokio::test]
    async fn start_configuration_session_instrument_send_accepts_no_arguments() {
        let rule = mock!(Client::start_configuration_session)
            .match_requests(|req| {
                req.application_identifier() == Some("app")
                    && req.configuration_profile_identifier() == Some("profile")
                    && req.environment_identifier() == Some("env")
                    && req.required_minimum_poll_interval_in_seconds() == Some(30)
            })
            .then_output(|| {
                StartConfigurationSessionOutput::builder()
                    .initial_configuration_token("initial-token")
                    .build()
            });

        let client = mock_client!(aws_sdk_appconfigdata, [&rule]);

        let response = client
            .start_configuration_session()
            .application_identifier("app")
            .configuration_profile_identifier("profile")
            .environment_identifier("env")
            .required_minimum_poll_interval_in_seconds(30)
            .instrument()
            .send()
            .await
            .expect("mocked start configuration session should succeed");

        assert_eq!(
            response.initial_configuration_token(),
            Some("initial-token")
        );
        assert_eq!(rule.num_calls(), 1);
    }

    #[test]
    fn get_latest_configuration_extracts_expected_attributes() {
        let output = GetLatestConfigurationOutput::builder()
            .content_type("application/json")
            .version_label("v1")
            .next_poll_interval_in_seconds(45)
            .build();

        let attributes = output.extract_attributes().into_iter().collect::<Vec<_>>();

        assert_eq!(
            attributes,
            vec![
                KeyValue::new("aws.appconfigdata.content_type", "application/json"),
                KeyValue::new("aws.appconfigdata.version_label", "v1"),
                KeyValue::new("aws.appconfigdata.next_poll_interval_in_seconds", 45_i64),
            ]
        );
    }
}
