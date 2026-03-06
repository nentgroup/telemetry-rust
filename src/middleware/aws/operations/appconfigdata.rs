/// AWS AppConfig Data operations
///
/// API Reference: https://docs.aws.amazon.com/appconfig/2019-10-09/APIReference/API_Operations_AWS_AppConfig_Data.html
use crate::{KeyValue, StringValue};

use super::*;

/// Builder for AppConfig Data-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for AppConfig Data operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with AppConfig Data-specific attributes.
pub enum AppConfigDataSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates an AppConfig Data operation span builder.
    ///
    /// This method creates a span builder configured for AppConfig Data
    /// operations with the application identifier as the primary resource identifier.
    ///
    /// # Arguments
    ///
    /// * `method` - The AppConfig Data operation method name
    /// * `application_id` - Optional application identifier for the operation
    pub fn appconfigdata(
        method: impl Into<StringValue>,
        application_id: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = Vec::new();
        if let Some(id) = application_id {
            attributes.push(KeyValue::new("aws.appconfigdata.application_id", id.into()));
        }
        Self::client("AppConfigData", method, attributes)
    }
}

macro_rules! appconfigdata_global_operation {
    ($op: ident) => {
        impl AppConfigDataSpanBuilder {
            #[doc = concat!("Creates a span builder for the AppConfig Data ", stringify!($op), " operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::appconfigdata(stringify_camel!($op), None::<StringValue>)
            }
        }
    };
}

macro_rules! appconfigdata_application_operation {
    ($op: ident) => {
        impl AppConfigDataSpanBuilder {
            #[doc = concat!("Creates a span builder for the AppConfig Data ", stringify!($op), " operation.")]
            ///
            /// # Arguments
            ///
            /// * `application_id` - The application identifier
            pub fn $op<'a>(
                application_id: impl Into<StringValue>,
            ) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::appconfigdata(stringify_camel!($op), Some(application_id))
            }
        }
    };
}

// Session operations
appconfigdata_application_operation!(start_configuration_session);

// Configuration retrieval operations
appconfigdata_global_operation!(get_latest_configuration);
