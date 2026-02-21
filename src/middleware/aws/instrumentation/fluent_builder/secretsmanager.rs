/// AWS Secrets Manager fluent builder instrumentation implementations
use super::{utils::*, *};

// Secret value operations
impl<'a> AwsBuilderInstrument<'a> for GetSecretValueFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_version_id()
                .as_attribute("aws.secretsmanager.version_id"),
            self.get_version_stage()
                .as_attribute("aws.secretsmanager.version_stage"),
        ];
        SecretsManagerSpanBuilder::get_secret_value(secret_id).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetSecretValueOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.version_id()
                .as_attribute("aws.secretsmanager.version_id"),
            self.version_stages()
                .len()
                .as_attribute("aws.secretsmanager.version_stages_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::get_secret_value);

impl<'a> AwsBuilderInstrument<'a> for PutSecretValueFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::put_secret_value(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for PutSecretValueOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.version_id()
                .as_attribute("aws.secretsmanager.version_id"),
            self.version_stages()
                .len()
                .as_attribute("aws.secretsmanager.version_stages_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::put_secret_value);

impl<'a> AwsBuilderInstrument<'a> for CreateSecretFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let name = self.get_name().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::create_secret(name)
    }
}
impl InstrumentedFluentBuilderOutput for CreateSecretOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.version_id()
                .as_attribute("aws.secretsmanager.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::create_secret);

// Secret lifecycle operations
impl<'a> AwsBuilderInstrument<'a> for DeleteSecretFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_force_delete_without_recovery()
                .as_attribute("aws.secretsmanager.force_delete"),
            self.get_recovery_window_in_days()
                .map(|d| KeyValue::new("aws.secretsmanager.recovery_window_days", d)),
        ];
        SecretsManagerSpanBuilder::delete_secret(secret_id).attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteSecretOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::delete_secret);

impl<'a> AwsBuilderInstrument<'a> for DescribeSecretFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::describe_secret(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for DescribeSecretOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.rotation_enabled()
                .as_attribute("aws.secretsmanager.rotation_enabled"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::describe_secret);

impl<'a> AwsBuilderInstrument<'a> for UpdateSecretFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::update_secret(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for UpdateSecretOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.version_id()
                .as_attribute("aws.secretsmanager.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::update_secret);

impl<'a> AwsBuilderInstrument<'a> for RestoreSecretFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::restore_secret(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for RestoreSecretOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::restore_secret);

// Rotation operations
impl<'a> AwsBuilderInstrument<'a> for RotateSecretFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::rotate_secret(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for RotateSecretOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.version_id()
                .as_attribute("aws.secretsmanager.version_id"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::rotate_secret);

impl<'a> AwsBuilderInstrument<'a> for CancelRotateSecretFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::cancel_rotate_secret(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for CancelRotateSecretOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::cancel_rotate_secret);

// Version operations
impl<'a> AwsBuilderInstrument<'a> for UpdateSecretVersionStageFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_version_stage()
                .as_attribute("aws.secretsmanager.version_stage"),
            self.get_move_to_version_id()
                .as_attribute("aws.secretsmanager.move_to_version_id"),
            self.get_remove_from_version_id()
                .as_attribute("aws.secretsmanager.remove_from_version_id"),
        ];
        SecretsManagerSpanBuilder::update_secret_version_stage(secret_id)
            .attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for UpdateSecretVersionStageOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::update_secret_version_stage);

impl<'a> AwsBuilderInstrument<'a> for ListSecretVersionIdsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        let attributes = attributes![
            self.get_max_results()
                .map(|v| KeyValue::new("aws.secretsmanager.max_results", v as i64)),
        ];
        SecretsManagerSpanBuilder::list_secret_version_ids(secret_id)
            .attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for ListSecretVersionIdsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.versions()
                .len()
                .as_attribute("aws.secretsmanager.version_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::list_secret_version_ids);

// Tagging operations
impl<'a> AwsBuilderInstrument<'a> for TagResourceFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::tag_resource(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for TagResourceOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::tag_resource);

impl<'a> AwsBuilderInstrument<'a> for UntagResourceFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::untag_resource(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for UntagResourceOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::untag_resource);

// Resource policy operations
impl<'a> AwsBuilderInstrument<'a> for GetResourcePolicyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::get_resource_policy(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for GetResourcePolicyOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::get_resource_policy);

impl<'a> AwsBuilderInstrument<'a> for PutResourcePolicyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::put_resource_policy(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for PutResourcePolicyOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::put_resource_policy);

impl<'a> AwsBuilderInstrument<'a> for DeleteResourcePolicyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::delete_resource_policy(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for DeleteResourcePolicyOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::delete_resource_policy);

impl<'a> AwsBuilderInstrument<'a> for ValidateResourcePolicyFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::validate_resource_policy(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for ValidateResourcePolicyOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            Some(KeyValue::new(
                "aws.secretsmanager.policy_validation_passed",
                self.policy_validation_passed(),
            )),
            self.validation_errors()
                .len()
                .as_attribute("aws.secretsmanager.validation_error_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::validate_resource_policy);

// Replication operations
impl<'a> AwsBuilderInstrument<'a> for RemoveRegionsFromReplicationFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::remove_regions_from_replication(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for RemoveRegionsFromReplicationOutput {}
instrument_aws_operation!(
    aws_sdk_secretsmanager::operation::remove_regions_from_replication
);

impl<'a> AwsBuilderInstrument<'a> for ReplicateSecretToRegionsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::replicate_secret_to_regions(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for ReplicateSecretToRegionsOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::replicate_secret_to_regions);

impl<'a> AwsBuilderInstrument<'a> for StopReplicationToReplicaFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let secret_id = self.get_secret_id().clone().unwrap_or_default();
        SecretsManagerSpanBuilder::stop_replication_to_replica(secret_id)
    }
}
impl InstrumentedFluentBuilderOutput for StopReplicationToReplicaOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::stop_replication_to_replica);

// Global operations
impl<'a> AwsBuilderInstrument<'a> for ListSecretsFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes = attributes![
            self.get_max_results()
                .map(|v| KeyValue::new("aws.secretsmanager.max_results", v as i64)),
        ];
        SecretsManagerSpanBuilder::list_secrets().attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for ListSecretsOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.secret_list()
                .len()
                .as_attribute("aws.secretsmanager.secret_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::list_secrets);

impl<'a> AwsBuilderInstrument<'a> for BatchGetSecretValueFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes =
            attributes![self.get_secret_id_list().as_ref().map(|ids| KeyValue::new(
                "aws.secretsmanager.secret_count",
                ids.len() as i64
            )),];
        SecretsManagerSpanBuilder::batch_get_secret_value().attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for BatchGetSecretValueOutput {
    fn extract_attributes(&self) -> impl IntoIterator<Item = KeyValue> {
        attributes![
            self.secret_values()
                .len()
                .as_attribute("aws.secretsmanager.secret_values_count"),
            self.errors()
                .len()
                .as_attribute("aws.secretsmanager.error_count"),
        ]
    }
}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::batch_get_secret_value);

impl<'a> AwsBuilderInstrument<'a> for GetRandomPasswordFluentBuilder {
    fn build_aws_span(&self) -> AwsSpanBuilder<'a> {
        let attributes = attributes![
            self.get_password_length()
                .map(|v| KeyValue::new("aws.secretsmanager.password_length", v)),
        ];
        SecretsManagerSpanBuilder::get_random_password().attributes(attributes)
    }
}
impl InstrumentedFluentBuilderOutput for GetRandomPasswordOutput {}
instrument_aws_operation!(aws_sdk_secretsmanager::operation::get_random_password);
