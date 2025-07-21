use opentelemetry::trace::TraceContextExt;
use serde::{
    Deserialize, Deserializer as _, Serialize, Serializer as _,
    de::{Error, MapAccess, Visitor as DeVisitor},
    ser::{SerializeMap, SerializeSeq},
};
use serde_json::{Deserializer, Serializer, Value};
use std::{fmt, io, marker::PhantomData, ops::Deref, str};
use tracing::{Event, Span, Subscriber};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_serde::{AsSerde, SerdeMapVisitor};
use tracing_subscriber::{
    fmt::{
        FmtContext, FormatEvent, FormatFields, FormattedFields,
        format::Writer,
        time::{FormatTime, SystemTime},
    },
    registry::{LookupSpan, SpanRef},
};

/// JSON event formatter for structured logging with OpenTelemetry integration.
///
/// This formatter serializes tracing events into JSON format with additional OpenTelemetry
/// metadata including trace IDs, span IDs, and span hierarchy information. It's designed
/// to work with the tracing-subscriber ecosystem and provides rich context for log analysis.
///
/// # Features
///
/// - Structured JSON output with consistent field names
/// - Automatic inclusion of OpenTelemetry trace and span IDs
/// - Hierarchical span information for request tracing
/// - ISO 8601 timestamp formatting
/// - Proper handling of event fields and metadata
///
/// # JSON Structure
///
/// The generated JSON includes the following fields:
/// - `timestamp`: ISO 8601 formatted timestamp
/// - `level`: Log level (ERROR, WARN, INFO, DEBUG, TRACE)
/// - `target`: The module path where the event was recorded
/// - `trace_id`: OpenTelemetry trace ID (if available)
/// - `span_id`: OpenTelemetry span ID (if available)
/// - `spans`: Array of parent spans with their fields
/// - Additional fields from the log event
pub struct JsonFormat;

impl<S, N> FormatEvent<S, N> for JsonFormat
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let mut timestamp = String::new();
        SystemTime.format_time(&mut Writer::new(&mut timestamp))?;

        let meta = event.metadata();

        let mut visit = || {
            let mut serializer = Serializer::new(IoWriter(&mut writer));
            let mut serializer = serializer.serialize_map(None)?;

            serializer.serialize_entry("timestamp", &timestamp)?;
            serializer.serialize_entry("level", &meta.level().as_serde())?;

            // add all event fields to the json object
            let mut visitor = SerdeMapVisitor::new(serializer);
            event.record(&mut visitor);
            serializer = visitor.take_serializer()?;

            serializer.serialize_entry("target", meta.target())?;

            // extract tracing information from the current span context
            let current_span = Span::current();
            if let Some(id) = current_span.id() {
                let otel_ctx = current_span.context();
                let span_ref = otel_ctx.span();
                let span_context = span_ref.span_context();

                if let Some(leaf_span) = ctx.span(&id).or_else(|| ctx.lookup_current()) {
                    let spans = SpanScope(leaf_span, PhantomData::<N>);
                    serializer.serialize_entry("spans", &spans)?;
                }

                let trace_id = span_context.trace_id().to_string();
                serializer.serialize_entry("trace_id", &trace_id)?;

                let span_id = span_context.span_id().to_string();
                serializer.serialize_entry("span_id", &span_id)?;
            }

            SerializeMap::end(serializer)
        };

        visit().map_err(|_| fmt::Error)?;
        writeln!(writer)
    }
}

struct IoWriter<'a>(&'a mut dyn fmt::Write);

impl io::Write for IoWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = str::from_utf8(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.0.write_str(s).map_err(io::Error::other)?;

        Ok(s.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct SpanData<'a, R, N>(SpanRef<'a, R>, PhantomData<N>)
where
    R: for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static;

impl<R, N> Serialize for SpanData<'_, R, N>
where
    R: for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_map(None)?;
        serializer.serialize_entry("name", self.0.name())?;

        let extensions = self.0.extensions();
        if let Some(fields) = extensions.get::<FormattedFields<N>>() {
            let mut deserializer = Deserializer::from_str(fields);
            let visitor = SerializerVisior(&mut serializer);
            if let Err(error) = deserializer.deserialize_map(visitor) {
                serializer.serialize_entry("formatted_fields", fields.deref())?;
                serializer.serialize_entry("parsing_error", &format!("{error:?}"))?;
            }
        }

        serializer.end()
    }
}

struct SpanScope<'a, R, N>(SpanRef<'a, R>, PhantomData<N>)
where
    R: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static;

impl<R, N> Serialize for SpanScope<'_, R, N>
where
    R: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut serializer = serializer.serialize_seq(None)?;
        for span in self.0.scope().from_root() {
            serializer.serialize_element(&SpanData(span, self.1))?;
        }
        serializer.end()
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum FieldValue<'a> {
    Borrowed(&'a str),
    Owned(Value),
}

struct SerializerVisior<'a, S: SerializeMap>(&'a mut S);

impl<'de, S: SerializeMap> DeVisitor<'de> for SerializerVisior<'_, S> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a map of values")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        while let Some((key, value)) = map.next_entry::<&str, Value>()? {
            self.0
                .serialize_entry(key, &value)
                .map_err(A::Error::custom)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use assert2::assert;
    use rstest::rstest;
    use serde::Serialize;

    use super::*;

    // Normal strings should be parsed as borrowed &str
    #[rstest]
    #[case("Hello worlds!")]
    #[case("123.456")]
    #[case("true")]
    #[case("null")]
    #[case("42")]
    fn test_borrowed_field_value(#[case] value: &str) {
        let json = format!("\"{value}\"");
        let actual = serde_json::from_str::<FieldValue>(&json)
            .map_err(|err| format!("Error parsing {json:?}: {err:?}"))
            .unwrap();
        assert!(actual == FieldValue::Borrowed(value));
    }

    // Strings containeng escape sequences and non-string values should be parsed as owned values
    #[rstest]
    #[case("Qwe\\rty")]
    #[case("\"\"")]
    #[case(true)]
    #[case(false)]
    #[case(123.456)]
    #[case(i64::MIN)]
    #[case(u64::MAX)]
    #[case(())]
    fn test_owned_field_value<T: Serialize + Into<Value>>(#[case] value: T) {
        let json = serde_json::to_string(&value).unwrap();
        let actual = serde_json::from_str::<FieldValue>(&json)
            .map_err(|err| format!("Error parsing {json:?}: {err:?}"))
            .unwrap();
        assert!(actual == FieldValue::Owned(value.into()));
    }
}
