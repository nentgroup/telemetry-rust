use crate::{Key, KeyValue, Value};

#[inline]
pub(crate) fn env_var(key: &str) -> Option<String> {
    match std::env::var(key) {
        Ok(value) => {
            if value.trim().is_empty() {
                None
            } else {
                Some(value)
            }
        }
        Err(_) => None,
    }
}

#[cfg(any(
    feature = "reqwest",
    feature = "hyper-http1",
    feature = "hyper-http2",
    feature = "hyper-client-legacy",
    feature = "aws-fluent-builder-instrumentation",
))]
#[inline]
pub(crate) fn as_attribute(
    key: impl Into<Key>,
    maybe_value: Option<impl Into<Value>>,
) -> Option<KeyValue> {
    maybe_value.map(|value| KeyValue::new(key, value))
}
