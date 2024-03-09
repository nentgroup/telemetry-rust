use crate::{semcov, StringValue};

use super::*;

aws_target!(SnsOperation);

impl SnsOperation<'_> {
    pub fn new(
        method: impl Into<StringValue>,
        topic_arn: impl Into<StringValue>,
    ) -> Self {
        let attributes = vec![
            semcov::MESSAGING_SYSTEM.string("aws_sns"),
            semcov::MESSAGING_OPERATION.string("publish"),
            semcov::MESSAGING_DESTINATION_NAME.string(topic_arn),
        ];
        Self(AwsOperation::producer("SNS", method, attributes))
    }
}

macro_rules! sns_operation {
    ($op: ident) => {
        impl<'a> SnsOperation<'a> {
            #[inline]
            pub fn $op(topic_arn: impl Into<StringValue>) -> Self {
                Self::new(stringify_camel!($op), topic_arn)
            }
        }
    };
}

sns_operation!(add_permission);
sns_operation!(check_if_phone_number_is_opted_out);
sns_operation!(confirm_subscription);
sns_operation!(create_platform_application);
sns_operation!(create_platform_endpoint);
sns_operation!(create_sms_sandbox_phone_number);
sns_operation!(create_topic);
sns_operation!(delete_endpoint);
sns_operation!(delete_platform_application);
sns_operation!(delete_sms_sandbox_phone_number);
sns_operation!(delete_topic);
sns_operation!(get_data_protection_policy);
sns_operation!(get_endpoint_attributes);
sns_operation!(get_platform_application_attributes);
sns_operation!(get_sms_attributes);
sns_operation!(get_sms_sandbox_account_status);
sns_operation!(get_subscription_attributes);
sns_operation!(get_topic_attributes);
sns_operation!(list_endpoints_by_platform_application);
sns_operation!(list_origination_numbers);
sns_operation!(list_phone_numbers_opted_out);
sns_operation!(list_platform_applications);
sns_operation!(list_sms_sandbox_phone_numbers);
sns_operation!(list_subscriptions);
sns_operation!(list_subscriptions_by_topic);
sns_operation!(list_tags_for_resource);
sns_operation!(list_topics);
sns_operation!(opt_in_phone_number);
sns_operation!(publish); // publish
sns_operation!(publish_batch); // publish
sns_operation!(put_data_protection_policy);
sns_operation!(remove_permission);
sns_operation!(set_endpoint_attributes);
sns_operation!(set_platform_application_attributes);
sns_operation!(set_sms_attributes);
sns_operation!(set_subscription_attributes);
sns_operation!(set_topic_attributes);
sns_operation!(subscribe);
sns_operation!(tag_resource);
sns_operation!(unsubscribe);
sns_operation!(untag_resource);
sns_operation!(verify_sms_sandbox_phone_number);
