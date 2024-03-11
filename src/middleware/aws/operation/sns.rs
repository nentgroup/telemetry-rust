use crate::{semcov, StringValue};

use super::*;

aws_target!(SnsOperation);

impl SnsOperation<'_> {
    pub fn with_operation_kind(
        operation_kind: MessagingOperationKind,
        method: impl Into<StringValue>,
        topic_arn: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = vec![
            semcov::MESSAGING_SYSTEM.string("aws_sns"),
            semcov::MESSAGING_OPERATION.string(operation_kind.as_str()),
        ];
        if let Some(topic_arn) = topic_arn {
            attributes.push(semcov::MESSAGING_DESTINATION_NAME.string(topic_arn))
        }
        Self(AwsOperation::new(
            operation_kind.into(),
            "SNS",
            method,
            attributes,
        ))
    }

    pub fn new(
        method: impl Into<StringValue>,
        topic_arn: Option<impl Into<StringValue>>,
    ) -> Self {
        Self::with_operation_kind(MessagingOperationKind::Control, method, topic_arn)
    }
}

macro_rules! sns_global_operation {
    ($op: ident) => {
        impl<'a> SnsOperation<'a> {
            #[inline]
            pub fn $op() -> Self {
                Self::new(stringify_camel!($op), None::<StringValue>)
            }
        }
    };
}

macro_rules! sns_publish_operation {
    ($op: ident, $kind: expr) => {
        impl<'a> SnsOperation<'a> {
            pub fn $op(topic_arn: impl Into<StringValue>) -> Self {
                Self::with_operation_kind($kind, stringify_camel!($op), Some(topic_arn))
            }
        }
    };
}

macro_rules! sns_topic_operation {
    ($op: ident) => {
        impl<'a> SnsOperation<'a> {
            pub fn $op(topic_arn: impl Into<StringValue>) -> Self {
                Self::new(stringify_camel!($op), Some(topic_arn))
            }
        }
    };
}

// publish operation
sns_publish_operation!(publish, MessagingOperationKind::Create);
sns_publish_operation!(publish_batch, MessagingOperationKind::Publish);

// global operations
sns_global_operation!(check_if_phone_number_is_opted_out);
sns_global_operation!(create_platform_application);
sns_global_operation!(create_platform_endpoint);
sns_global_operation!(create_sms_sandbox_phone_number);
sns_global_operation!(create_topic);
sns_global_operation!(delete_endpoint);
sns_global_operation!(delete_platform_application);
sns_global_operation!(delete_sms_sandbox_phone_number);
sns_global_operation!(get_data_protection_policy);
sns_global_operation!(get_endpoint_attributes);
sns_global_operation!(get_platform_application_attributes);
sns_global_operation!(get_sms_attributes);
sns_global_operation!(get_sms_sandbox_account_status);
sns_global_operation!(get_subscription_attributes);
sns_global_operation!(list_endpoints_by_platform_application);
sns_global_operation!(list_origination_numbers);
sns_global_operation!(list_phone_numbers_opted_out);
sns_global_operation!(list_platform_applications);
sns_global_operation!(list_sms_sandbox_phone_numbers);
sns_global_operation!(list_subscriptions);
sns_global_operation!(list_tags_for_resource);
sns_global_operation!(list_topics);
sns_global_operation!(opt_in_phone_number);
sns_global_operation!(put_data_protection_policy);
sns_global_operation!(set_endpoint_attributes);
sns_global_operation!(set_platform_application_attributes);
sns_global_operation!(set_sms_attributes);
sns_global_operation!(set_subscription_attributes);
sns_global_operation!(tag_resource);
sns_global_operation!(unsubscribe);
sns_global_operation!(untag_resource);
sns_global_operation!(verify_sms_sandbox_phone_number);

// control plane topic operations
sns_topic_operation!(add_permission);
sns_topic_operation!(confirm_subscription);
sns_topic_operation!(delete_topic);
sns_topic_operation!(get_topic_attributes);
sns_topic_operation!(list_subscriptions_by_topic);
sns_topic_operation!(remove_permission);
sns_topic_operation!(set_topic_attributes);
sns_topic_operation!(subscribe);
