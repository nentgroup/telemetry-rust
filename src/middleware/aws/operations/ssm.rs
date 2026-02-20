/// AWS Systems Manager (SSM) operations
///
/// API Reference: https://docs.aws.amazon.com/systems-manager/latest/APIReference/API_Operations.html
use crate::StringValue;

use super::*;

/// Builder for SSM-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for SSM operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans for AWS Systems Manager operations.
pub enum SsmSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates an SSM operation span builder.
    ///
    /// # Arguments
    ///
    /// * `method` - The SSM operation method name
    pub fn ssm(method: impl Into<StringValue>) -> Self {
        Self::client("SSM", method, std::iter::empty::<crate::KeyValue>())
    }
}

macro_rules! ssm_operation {
    ($op: ident) => {
        impl SsmSpanBuilder {
            #[doc = concat!("Creates a span builder for the SSM ", stringify!($op), " operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::ssm(stringify_camel!($op))
            }
        }
    };
}

ssm_operation!(add_tags_to_resource);
ssm_operation!(associate_ops_item_related_item);
ssm_operation!(cancel_command);
ssm_operation!(cancel_maintenance_window_execution);
ssm_operation!(create_activation);
ssm_operation!(create_association);
ssm_operation!(create_association_batch);
ssm_operation!(create_document);
ssm_operation!(create_maintenance_window);
ssm_operation!(create_ops_item);
ssm_operation!(create_ops_metadata);
ssm_operation!(create_patch_baseline);
ssm_operation!(create_resource_data_sync);
ssm_operation!(delete_activation);
ssm_operation!(delete_association);
ssm_operation!(delete_document);
ssm_operation!(delete_inventory);
ssm_operation!(delete_maintenance_window);
ssm_operation!(delete_ops_item);
ssm_operation!(delete_ops_metadata);
ssm_operation!(delete_parameter);
ssm_operation!(delete_parameters);
ssm_operation!(delete_patch_baseline);
ssm_operation!(delete_resource_data_sync);
ssm_operation!(delete_resource_policy);
ssm_operation!(deregister_managed_instance);
ssm_operation!(deregister_patch_baseline_for_patch_group);
ssm_operation!(deregister_target_from_maintenance_window);
ssm_operation!(deregister_task_from_maintenance_window);
ssm_operation!(describe_activations);
ssm_operation!(describe_association);
ssm_operation!(describe_association_execution_targets);
ssm_operation!(describe_association_executions);
ssm_operation!(describe_automation_executions);
ssm_operation!(describe_automation_step_executions);
ssm_operation!(describe_available_patches);
ssm_operation!(describe_document);
ssm_operation!(describe_document_permission);
ssm_operation!(describe_effective_instance_associations);
ssm_operation!(describe_effective_patches_for_patch_baseline);
ssm_operation!(describe_instance_associations_status);
ssm_operation!(describe_instance_information);
ssm_operation!(describe_instance_patch_states);
ssm_operation!(describe_instance_patch_states_for_patch_group);
ssm_operation!(describe_instance_patches);
ssm_operation!(describe_instance_properties);
ssm_operation!(describe_inventory_deletions);
ssm_operation!(describe_maintenance_window_execution_task_invocations);
ssm_operation!(describe_maintenance_window_execution_tasks);
ssm_operation!(describe_maintenance_window_executions);
ssm_operation!(describe_maintenance_window_schedule);
ssm_operation!(describe_maintenance_window_targets);
ssm_operation!(describe_maintenance_window_tasks);
ssm_operation!(describe_maintenance_windows);
ssm_operation!(describe_maintenance_windows_for_target);
ssm_operation!(describe_ops_items);
ssm_operation!(describe_parameters);
ssm_operation!(describe_patch_baselines);
ssm_operation!(describe_patch_group_state);
ssm_operation!(describe_patch_groups);
ssm_operation!(describe_patch_properties);
ssm_operation!(describe_sessions);
ssm_operation!(disassociate_ops_item_related_item);
ssm_operation!(get_access_token);
ssm_operation!(get_automation_execution);
ssm_operation!(get_calendar_state);
ssm_operation!(get_command_invocation);
ssm_operation!(get_connection_status);
ssm_operation!(get_default_patch_baseline);
ssm_operation!(get_deployable_patch_snapshot_for_instance);
ssm_operation!(get_document);
ssm_operation!(get_execution_preview);
ssm_operation!(get_inventory);
ssm_operation!(get_inventory_schema);
ssm_operation!(get_maintenance_window);
ssm_operation!(get_maintenance_window_execution);
ssm_operation!(get_maintenance_window_execution_task);
ssm_operation!(get_maintenance_window_execution_task_invocation);
ssm_operation!(get_maintenance_window_task);
ssm_operation!(get_ops_item);
ssm_operation!(get_ops_metadata);
ssm_operation!(get_ops_summary);
ssm_operation!(get_parameter);
ssm_operation!(get_parameter_history);
ssm_operation!(get_parameters);
ssm_operation!(get_parameters_by_path);
ssm_operation!(get_patch_baseline);
ssm_operation!(get_patch_baseline_for_patch_group);
ssm_operation!(get_resource_policies);
ssm_operation!(get_service_setting);
ssm_operation!(label_parameter_version);
ssm_operation!(list_association_versions);
ssm_operation!(list_associations);
ssm_operation!(list_command_invocations);
ssm_operation!(list_commands);
ssm_operation!(list_compliance_items);
ssm_operation!(list_compliance_summaries);
ssm_operation!(list_document_metadata_history);
ssm_operation!(list_document_versions);
ssm_operation!(list_documents);
ssm_operation!(list_inventory_entries);
ssm_operation!(list_nodes);
ssm_operation!(list_nodes_summary);
ssm_operation!(list_ops_item_events);
ssm_operation!(list_ops_item_related_items);
ssm_operation!(list_ops_metadata);
ssm_operation!(list_resource_compliance_summaries);
ssm_operation!(list_resource_data_sync);
ssm_operation!(list_tags_for_resource);
ssm_operation!(modify_document_permission);
ssm_operation!(put_compliance_items);
ssm_operation!(put_inventory);
ssm_operation!(put_parameter);
ssm_operation!(put_resource_policy);
ssm_operation!(register_default_patch_baseline);
ssm_operation!(register_patch_baseline_for_patch_group);
ssm_operation!(register_target_with_maintenance_window);
ssm_operation!(register_task_with_maintenance_window);
ssm_operation!(remove_tags_from_resource);
ssm_operation!(reset_service_setting);
ssm_operation!(resume_session);
ssm_operation!(send_automation_signal);
ssm_operation!(send_command);
ssm_operation!(start_access_request);
ssm_operation!(start_associations_once);
ssm_operation!(start_automation_execution);
ssm_operation!(start_change_request_execution);
ssm_operation!(start_execution_preview);
ssm_operation!(start_session);
ssm_operation!(stop_automation_execution);
ssm_operation!(terminate_session);
ssm_operation!(unlabel_parameter_version);
ssm_operation!(update_association);
ssm_operation!(update_association_status);
ssm_operation!(update_document);
ssm_operation!(update_document_default_version);
ssm_operation!(update_document_metadata);
ssm_operation!(update_maintenance_window);
ssm_operation!(update_maintenance_window_target);
ssm_operation!(update_maintenance_window_task);
ssm_operation!(update_managed_instance_role);
ssm_operation!(update_ops_item);
ssm_operation!(update_ops_metadata);
ssm_operation!(update_patch_baseline);
ssm_operation!(update_resource_data_sync);
ssm_operation!(update_service_setting);
