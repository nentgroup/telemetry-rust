pub use opentelemetry_semantic_conventions::attribute::*;

#[allow(deprecated)]
pub mod legacy {
    pub const MESSAGING_OPERATION: &str = super::MESSAGING_OPERATION;
    pub const DB_NAME: &str = super::DB_NAME;
    pub const DB_OPERATION: &str = super::DB_OPERATION;
}
