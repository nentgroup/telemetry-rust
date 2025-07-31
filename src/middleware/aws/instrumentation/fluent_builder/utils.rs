use crate::{Key, KeyValue, StringValue, Value};

/// A trait for converting fluent builder properties into OpenTelemetry key-value attributes.
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
        self.map(Into::<i64>::into).as_attribute(key)
    }
}

impl AsAttribute for Option<i64> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        self.map(|value| KeyValue::new(key, value))
    }
}

impl AsAttribute for Option<f64> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        self.map(|value| KeyValue::new(key, value))
    }
}

impl AsAttribute for Option<usize> {
    fn as_attribute(&self, key: impl Into<Key>) -> Option<KeyValue> {
        self.map(TryInto::<i64>::try_into)
            .and_then(Result::ok)
            .as_attribute(key)
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

/// Helper macro to create attribute arrays that filter out None values
macro_rules! attributes {
    ($($expr:expr),* $(,)?) => {
        [$($expr,)*].into_iter().flatten()
    };
}

pub(super) use attributes;
