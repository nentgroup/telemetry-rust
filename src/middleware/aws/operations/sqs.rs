use http::Uri;

use crate::{KeyValue, StringValue, semconv};

use super::*;

/// Builder for SQS-specific OpenTelemetry spans.
///
/// This enum serves as a namespace for SQS operation span builders.
/// Each operation provides a specific method to create properly configured
/// spans with SQS-specific messaging attributes.
pub enum SqsSpanBuilder {}

impl AwsSpanBuilder<'_> {
    /// Creates an SQS operation span builder.
    ///
    /// This method creates a span builder configured for SQS operations with
    /// appropriate messaging semantic attributes.
    ///
    /// # Arguments
    ///
    /// * `operation_kind` - The type of messaging operation being performed
    /// * `method` - The SQS operation method name
    /// * `queue` - Optional SNS queue URL or name for operations that target specific queues
    pub fn sqs(
        operation_kind: MessagingOperationKind,
        method: impl Into<StringValue>,
        queue: Option<impl Into<StringValue>>,
    ) -> Self {
        let mut attributes = vec![
            KeyValue::new(semconv::MESSAGING_SYSTEM, "aws_sqs"),
            KeyValue::new(semconv::MESSAGING_OPERATION_TYPE, operation_kind.as_str()),
        ];
        if let Some(queue) = queue.map(Into::<StringValue>::into) {
            let queue_url = queue.as_str().parse::<Uri>().ok();
            let queue_name = queue_url
                .as_ref()
                .and_then(|uri| uri.path().split('/').next_back())
                .unwrap_or(queue.as_str())
                .to_owned();
            attributes.push(KeyValue::new(
                semconv::MESSAGING_DESTINATION_NAME,
                queue_name,
            ));
            if queue_url.is_some() {
                attributes.push(KeyValue::new("aws.sqs.queue.url", queue));
            }
        }
        Self::new(operation_kind.into(), "SQS", method, attributes)
    }
}

macro_rules! sqs_global_operation {
    ($op: ident) => {
        impl SqsSpanBuilder {
            #[doc = concat!("Creates a span builder for the SQS ", stringify!($op), " global operation.")]
            #[inline]
            pub fn $op<'a>() -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::sqs(
                    MessagingOperationKind::Control,
                    stringify_camel!($op),
                    None::<StringValue>,
                )
            }
        }
    };
}

macro_rules! sqs_messaging_operation {
    ($op: ident, $kind: expr) => {
        impl SqsSpanBuilder {
            #[doc = concat!("Creates a span builder for the SNS ", stringify!($op), " messaging operation.")]
            ///
            /// # Arguments
            ///
            /// * `queue` - SNS queue URL or name
            pub fn $op<'a>(queue: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::sns($kind, stringify_camel!($op), Some(queue))
            }
        }
    };
}

macro_rules! sqs_queue_operation {
    ($op: ident) => {
        impl SqsSpanBuilder {
            #[doc = concat!("Creates a span builder for the SQS ", stringify!($op), " queue operation.")]
            ///
            /// # Arguments
            ///
            /// * `queue` - SNS queue URL or name
            pub fn $op<'a>(queue: impl Into<StringValue>) -> AwsSpanBuilder<'a> {
                AwsSpanBuilder::sqs(
                    MessagingOperationKind::Control,
                    stringify_camel!($op),
                    Some(queue),
                )
            }
        }
    };
}

// messaging operations
sqs_messaging_operation!(receive_message, MessagingOperationKind::Receive);
sqs_messaging_operation!(send_message, MessagingOperationKind::Send);
sqs_messaging_operation!(send_message_batch, MessagingOperationKind::Send);

// queue management operations
sqs_queue_operation!(add_permission);
sqs_queue_operation!(change_message_visibility);
sqs_queue_operation!(change_message_visibility_batch);
sqs_queue_operation!(create_queue);
sqs_queue_operation!(delete_message);
sqs_queue_operation!(delete_message_batch);
sqs_queue_operation!(delete_queue);
sqs_queue_operation!(get_queue_attributes);
sqs_queue_operation!(get_queue_url);
sqs_queue_operation!(list_dead_letter_source_queues);
sqs_queue_operation!(list_queue_tags);
sqs_queue_operation!(purge_queue);
sqs_queue_operation!(remove_permission);
sqs_queue_operation!(set_queue_attributes);
sqs_queue_operation!(tag_queue);
sqs_queue_operation!(untag_queue);

// global operations
sqs_global_operation!(cancel_message_move_task);
sqs_global_operation!(list_message_move_tasks);
sqs_global_operation!(list_queues);
sqs_global_operation!(start_message_move_task);
