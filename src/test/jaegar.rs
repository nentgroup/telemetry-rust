use opentelemetry_api::trace::{SpanId, TraceId};
use serde::{
    de::Error as DeserializationError, Deserialize, Deserializer, Serialize, Serializer,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceResponse {
    pub errors: Option<Vec<Error>>,
    pub data: Option<Vec<TraceData>>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub code: i64,
    pub msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.code, self.msg)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceData {
    #[serde(
        rename = "traceID",
        deserialize_with = "trace_from_hex",
        serialize_with = "as_hex"
    )]
    pub trace_id: TraceId,
    pub spans: Vec<Span>,
    pub processes: HashMap<String, Process>,
    pub warnings: Option<Vec<String>>,
}

impl TraceData {
    pub fn find_span(&self, operation_name: &str) -> Option<&Span> {
        self.spans
            .iter()
            .find(|&span| span.operation_name == operation_name)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    #[serde(
        rename = "traceID",
        deserialize_with = "trace_from_hex",
        serialize_with = "as_hex"
    )]
    pub trace_id: TraceId,
    #[serde(
        rename = "spanID",
        deserialize_with = "span_from_hex",
        serialize_with = "as_hex"
    )]
    pub span_id: SpanId,
    pub operation_name: String,
    pub references: Vec<Reference>,
    pub start_time: i64,
    pub duration: i64,
    pub tags: Vec<Tag>,
    pub logs: Vec<Log>,
    #[serde(rename = "processID")]
    pub process_id: String,
    #[serde(default)]
    pub warnings: Option<Vec<String>>,
}

impl Span {
    pub fn find_reference(&self, ref_type: &str) -> Option<&Reference> {
        self.references
            .iter()
            .find(|&refer| refer.ref_type == ref_type)
    }

    pub fn get_parent_reference(&self) -> Option<&Reference> {
        self.find_reference("CHILD_OF")
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub ref_type: String,
    #[serde(
        rename = "traceID",
        deserialize_with = "trace_from_hex",
        serialize_with = "as_hex"
    )]
    pub trace_id: TraceId,
    #[serde(
        rename = "spanID",
        deserialize_with = "span_from_hex",
        serialize_with = "as_hex"
    )]
    pub span_id: SpanId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub key: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    pub timestamp: i64,
    pub fields: Vec<LogEntry>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub key: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    pub service_name: String,
    pub tags: Vec<Tag>,
}
