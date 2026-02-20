/// AWS Secrets Manager fluent builder instrumentation implementations
use super::{utils::*, *};

macro_rules! instrument_secrets_manager_operation {
    ($op: ident) => {
        paste::paste! {
            impl<'a> AwsBuilderInstrument<'a> for [<$op:camel FluentBuilder>] {
                fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
                    let attributes = attributes![Some("secretsmanager")
                        .as_attribute("cloud.service.name"),];
                    SecretsManagerSpanBuilder::$op().attributes(attributes)
                }
            }

            impl InstrumentedFluentBuilderOutput for [<$op:camel Output>] {}
        }

        instrument_aws_operation!(aws_sdk_secretsmanager::operation::$op);
    };
}

instrument_secrets_manager_operation!(batch_get_secret_value);
instrument_secrets_manager_operation!(cancel_rotate_secret);
instrument_secrets_manager_operation!(create_secret);
instrument_secrets_manager_operation!(delete_resource_policy);
instrument_secrets_manager_operation!(delete_secret);
instrument_secrets_manager_operation!(describe_secret);
instrument_secrets_manager_operation!(get_random_password);
instrument_secrets_manager_operation!(get_resource_policy);
instrument_secrets_manager_operation!(get_secret_value);
instrument_secrets_manager_operation!(list_secret_version_ids);
instrument_secrets_manager_operation!(list_secrets);
instrument_secrets_manager_operation!(put_resource_policy);
instrument_secrets_manager_operation!(put_secret_value);
instrument_secrets_manager_operation!(remove_regions_from_replication);
instrument_secrets_manager_operation!(replicate_secret_to_regions);
instrument_secrets_manager_operation!(restore_secret);
instrument_secrets_manager_operation!(rotate_secret);
instrument_secrets_manager_operation!(stop_replication_to_replica);
instrument_secrets_manager_operation!(tag_resource);
instrument_secrets_manager_operation!(untag_resource);
instrument_secrets_manager_operation!(update_secret);
instrument_secrets_manager_operation!(update_secret_version_stage);
instrument_secrets_manager_operation!(validate_resource_policy);
