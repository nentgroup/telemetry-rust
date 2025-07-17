use crate::{KeyValue, StringValue, semconv};

use super::*;

pub enum SnsSpanBuilder {}

impl AwsSpanBuilder<'_> {
    pub fn sns(
        operation_kind: MessagingOperationKind,
        method: impl Into<StringValue>,
        topic_arn: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = vec![
            KeyValue::new(semconv::MESSAGING_SYSTEM, "aws_sns"),
            KeyValue::new(semconv::MESSAGING_OPERATION_TYPE, operation_kind.as_str()),
        ];
        if let Some(topic_arn) = topic_arn {
            attributes.push(KeyValue::new(
                semconv::MESSAGING_DESTINATION_NAME,
                topic_arn.into(),
            ))
        }
        Self::new(operation_kind.into(), "SNS", method, attributes)
    }
}

macro_rules! sns_global_operation {
    ($op: ident) => {
        impl SnsSpanBuilder {
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::sns(
                    MessagingOperationKind::Control,
                    stringify_camel!($op),
                    None::<StringValue>,
                )
            }
        }
    };
}

macro_rules! sns_publish_operation {
    ($op: ident, $kind: expr) => {
        impl SnsSpanBuilder {
            pub fn $op<'a>(topic_arn: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::sns($kind, stringify_camel!($op), Some(topic_arn))
            }
        }
    };
}

macro_rules! sns_topic_operation {
    ($op: ident) => {
        impl SnsSpanBuilder {
            pub fn $op<'a>(topic_arn: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::sns(
                    MessagingOperationKind::Control,
                    stringify_camel!($op),
                    Some(topic_arn),
                )
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
