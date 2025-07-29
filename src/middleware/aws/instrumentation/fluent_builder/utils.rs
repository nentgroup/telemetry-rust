use crate::{Key, KeyValue, StringValue, Value};

macro_rules! instrument_aws_operation {
    ($sdk:ident::operation::$op:ident) => {
        paste::paste! {
            impl super::InstrumentedFluentBuilder<'_, $sdk::operation::$op::builders::[<$op:camel FluentBuilder>]> {
                pub async fn send(self) -> Result<
                    $sdk::operation::$op::[<$op:camel Output>],
                    $sdk::error::SdkError<$sdk::operation::$op::[<$op:camel Error>]>,
                > {
                    self.inner.send().instrument(self.span).await
                }
            }
        }
    };
}

pub(super) use instrument_aws_operation;

pub(super) trait AsAttribute {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue>;
}

impl AsAttribute for Option<String> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        self.as_ref().map(|value| KeyValue::new(key, value.clone()))
    }
}

impl AsAttribute for Option<bool> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        self.map(|value| KeyValue::new(key, value))
    }
}

impl AsAttribute for Option<i32> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        self.map(|value| KeyValue::new(key, value as i64))
    }
}

impl AsAttribute for Option<Vec<String>> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        self.as_ref().map(|value| {
            let items = value
                .iter()
                .map(|item| item.clone().into())
                .collect::<Vec<StringValue>>();
            KeyValue::new(key, Value::Array(items.into()))
        })
    }
}

impl AsAttribute for Option<aws_sdk_dynamodb::types::Select> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        use aws_sdk_dynamodb::types::Select::*;

        self.as_ref().map(|value| {
            let value: StringValue = match value {
                AllAttributes => "ALL_ATTRIBUTES".into(),
                AllProjectedAttributes => "ALL_PROJECTED_ATTRIBUTES".into(),
                Count => "COUNT".into(),
                SpecificAttributes => "SPECIFIC_ATTRIBUTES".into(),
                other => other.as_str().to_owned().into(),
            };
            KeyValue::new(key, value)
        })
    }
}
