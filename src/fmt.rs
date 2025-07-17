use opentelemetry::trace::TraceContextExt;
use serde::ser::{Serialize, SerializeMap, SerializeSeq, Serializer as _};
use serde_json::Serializer;
use std::{collections::HashMap, fmt, io, marker::PhantomData, ops::Deref, str};
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
                    serializer.serialize_entry(
                        "spans",
                        &SpanScope(leaf_span, PhantomData::<N>),
                    )?;
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

struct SpanData<'a, R, N>(SpanRef<'a, R>, PhantomData<N>)
where
    R: for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static;

impl<R, N> Serialize for SpanData<'_, R, N>
where
    R: for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serializer = serializer.serialize_map(None)?;

        let extensions = self.0.extensions();
        let mut name = self.0.name();

        if let Some(fields) = extensions.get::<FormattedFields<N>>() {
            match serde_json::from_str::<HashMap<&str, &str>>(fields) {
                Ok(data) => {
                    for (key, value) in data.into_iter() {
                        if key == "name" {
                            name = value;
                        } else {
                            serializer.serialize_entry(key, value)?;
                        }
                    }
                }
                Err(err) => {
                    serializer.serialize_entry("raw_fields", fields.deref())?;
                    serializer.serialize_entry("fields_error", &format!("{err:?}"))?;
                }
            }
        }
        serializer.serialize_entry("name", name)?;

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serializer = serializer.serialize_seq(None)?;
        for span in self.0.scope().from_root() {
            serializer.serialize_element(&SpanData(span, self.1))?;
        }
        serializer.end()
    }
}
