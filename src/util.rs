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
