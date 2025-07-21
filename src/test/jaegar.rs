//! Data structures for deserializing and traversing Jaeger API responses.

use opentelemetry_api::trace::{SpanId, TraceId};
use serde::{
    Deserialize, Deserializer, Serialize, Serializer, de::Error as DeserializationError,
};
use serde_json::Value;
use std::collections::HashMap;

fn trace_from_hex<'de, D>(deserializer: D) -> Result<TraceId, D::Error>
where
    D: Deserializer<'de>,
{
    let hex: &str = Deserialize::deserialize(deserializer)?;
    match TraceId::from_hex(hex) {
        Ok(trace_id) => Ok(trace_id),
        Err(error) => Err(D::Error::custom(error)),
    }
}

fn span_from_hex<'de, D>(deserializer: D) -> Result<SpanId, D::Error>
where
    D: Deserializer<'de>,
{
    let hex: &str = Deserialize::deserialize(deserializer)?;
    match SpanId::from_hex(hex) {
        Ok(trace_id) => Ok(trace_id),
        Err(error) => Err(D::Error::custom(error)),
    }
}

fn as_hex<T, S>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: std::fmt::LowerHex,
    S: Serializer,
{
    serializer.serialize_str(&format!("{val:x}"))
}

/// Response structure for Jaeger trace query API.
///
/// This structure represents the response from Jaeger's trace query API,
/// containing trace data along with pagination information and potential errors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceResponse {
    /// Any errors that occurred during the trace query
    pub errors: Option<Vec<Error>>,
    /// The actual trace data returned by the query
    pub data: Option<Vec<TraceData>>,
    /// Total number of traces matching the query criteria
    pub total: i64,
    /// Maximum number of traces to return in this response
    pub limit: i64,
    /// Number of traces to skip from the beginning of the result set
    pub offset: i64,
}

/// Error information from Jaeger trace operations.
///
/// Represents an error condition that occurred during trace query or processing
/// operations in Jaeger, containing both an error code and descriptive message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    /// Numeric error code identifying the type of error
    pub code: i64,
    /// Human-readable error message describing what went wrong
    pub msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.code, self.msg)
    }
}

impl std::error::Error for Error {}

/// Complete trace data for a single distributed trace.
///
/// This structure contains all the information for a complete trace, including
/// all spans that are part of the trace, process information, and any warnings
/// that occurred during trace collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceData {
    /// The unique identifier for this trace
    #[serde(
        rename = "traceID",
        deserialize_with = "trace_from_hex",
        serialize_with = "as_hex"
    )]
    pub trace_id: TraceId,
    /// All spans that belong to this trace
    pub spans: Vec<Span>,
    /// Process information mapped by process ID
    pub processes: HashMap<String, Process>,
    /// Any warnings generated during trace collection
    pub warnings: Option<Vec<String>>,
}

impl TraceData {
    /// Finds a span within this trace by its operation name.
    ///
    /// # Arguments
    ///
    /// * `operation_name` - The name of the operation to search for
    ///
    /// # Returns
    ///
    /// The first span found with the matching operation name, or `None` if not found
    pub fn find_span(&self, operation_name: &str) -> Option<&Span> {
        self.spans
            .iter()
            .find(|&span| span.operation_name == operation_name)
    }
}

/// Individual span within a distributed trace.
///
/// A span represents a single operation within a trace, containing timing information,
/// metadata, references to other spans, and associated process information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    /// The trace ID this span belongs to
    #[serde(
        rename = "traceID",
        deserialize_with = "trace_from_hex",
        serialize_with = "as_hex"
    )]
    pub trace_id: TraceId,
    /// Unique identifier for this span
    #[serde(
        rename = "spanID",
        deserialize_with = "span_from_hex",
        serialize_with = "as_hex"
    )]
    pub span_id: SpanId,
    /// Human-readable name of the operation this span represents
    pub operation_name: String,
    /// References to other spans (e.g., parent-child relationships)
    pub references: Vec<Reference>,
    /// Timestamp when this span started (in microseconds since epoch)
    pub start_time: i64,
    /// Duration of this span in microseconds
    pub duration: i64,
    /// Key-value tags providing metadata about this span
    pub tags: Vec<Tag>,
    /// Log entries recorded during this span's execution
    pub logs: Vec<Log>,
    /// ID of the process that created this span
    #[serde(rename = "processID")]
    pub process_id: String,
    /// Any warnings associated with this span
    #[serde(default)]
    pub warnings: Option<Vec<String>>,
}

impl Span {
    /// Finds a reference of the specified type within this span.
    ///
    /// # Arguments
    ///
    /// * `ref_type` - The type of reference to search for (e.g., "CHILD_OF", "FOLLOWS_FROM")
    ///
    /// # Returns
    ///
    /// The first reference found with the matching type, or `None` if not found
    pub fn find_reference(&self, ref_type: &str) -> Option<&Reference> {
        self.references
            .iter()
            .find(|&refer| refer.ref_type == ref_type)
    }

    /// Gets the parent reference for this span.
    ///
    /// This is a convenience method that specifically looks for a "CHILD_OF" reference,
    /// which indicates the parent span in the trace hierarchy.
    ///
    /// # Returns
    ///
    /// The parent reference if this span has one, or `None` if this is a root span
    pub fn get_parent_reference(&self) -> Option<&Reference> {
        self.find_reference("CHILD_OF")
    }
}

/// Reference between spans in a trace.
///
/// Represents a relationship between two spans, such as parent-child relationships
/// or follows-from relationships that indicate causal dependencies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    /// Type of reference (e.g., "CHILD_OF", "FOLLOWS_FROM")
    pub ref_type: String,
    /// Trace ID of the referenced span
    #[serde(
        rename = "traceID",
        deserialize_with = "trace_from_hex",
        serialize_with = "as_hex"
    )]
    pub trace_id: TraceId,
    /// Span ID of the referenced span
    #[serde(
        rename = "spanID",
        deserialize_with = "span_from_hex",
        serialize_with = "as_hex"
    )]
    pub span_id: SpanId,
}

/// Key-value tag metadata attached to spans.
///
/// Tags provide additional context and metadata about spans, such as HTTP status codes,
/// database names, or other application-specific information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// The tag key/name
    pub key: String,
    /// The data type of the tag value
    #[serde(rename = "type")]
    pub type_field: String,
    /// The tag value (can be various types: string, number, boolean, etc.)
    pub value: Value,
}

/// Log event recorded during span execution.
///
/// Represents a structured log event that occurred during the span's lifetime,
/// containing a timestamp and associated field data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    /// Timestamp when this log event occurred (in microseconds since epoch)
    pub timestamp: i64,
    /// Key-value fields providing details about the log event
    pub fields: Vec<LogEntry>,
}

/// Individual field within a log event.
///
/// Represents a single key-value pair within a log event, providing structured
/// data about what occurred during the span execution.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    /// The field key/name
    pub key: String,
    /// The data type of the field value
    #[serde(rename = "type")]
    pub entry_type: String,
    /// The field value (can be various types: string, number, boolean, etc.)
    pub value: Value,
}

/// Process information for spans in a trace.
///
/// Represents information about the process that generated spans,
/// including service identification and process-level metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    /// Name of the service that generated the spans
    pub service_name: String,
    /// Tags providing additional metadata about the process
    pub tags: Vec<Tag>,
}
