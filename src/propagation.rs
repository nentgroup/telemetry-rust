//! Context propagation utilities for distributed tracing across service boundaries.

use opentelemetry::{
    Context,
    propagation::{
        Extractor, Injector, TextMapCompositePropagator, TextMapPropagator,
        text_map_propagator::FieldIter,
    },
};
#[cfg(feature = "xray")]
use opentelemetry_aws::trace::XrayPropagator;
#[cfg(feature = "zipkin")]
use opentelemetry_propagator_b3::{B3Encoding, Propagator as B3Propagator};
use opentelemetry_sdk::{
    error::OTelSdkError,
    propagation::{BaggagePropagator, TraceContextPropagator},
};
use std::collections::BTreeSet;

use crate::util;

/// Type alias for a boxed text map propagator.
///
/// This type represents a thread-safe, heap-allocated text map propagator that can be
/// used for OpenTelemetry context propagation across service boundaries.
pub type Propagator = Box<dyn TextMapPropagator + Send + Sync>;

/// A no-op propagator that performs no context injection or extraction.
///
/// This propagator can be used when context propagation is explicitly disabled
/// or not needed. It implements the [`TextMapPropagator`] trait but performs
/// no actual propagation operations.
#[derive(Debug)]
pub struct NonePropagator;

impl TextMapPropagator for NonePropagator {
    fn inject_context(&self, _: &Context, _: &mut dyn Injector) {}

    fn extract_with_context(&self, cx: &Context, _: &dyn Extractor) -> Context {
        cx.clone()
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(&[])
    }
}

/// A text map propagator that uses different propagators for injection and extraction.
///
/// This propagator allows for asymmetric context propagation where different
/// propagation strategies can be used for outgoing requests (injection) versus
/// incoming requests (extraction). This is useful when you need to maintain
/// compatibility with multiple tracing systems or protocols.
///
/// # Use Cases
///
/// - Migrating between tracing systems while maintaining compatibility
/// - Supporting multiple trace context formats in a single service
/// - Using environment-specific propagation strategies
#[derive(Debug)]
pub struct TextMapSplitPropagator {
    extract_propagator: Propagator,
    inject_propagator: Propagator,
    fields: Vec<String>,
}

impl TextMapSplitPropagator {
    /// Creates a new split propagator with separate propagators for extraction and injection.
    ///
    /// # Arguments
    ///
    /// - `extract_propagator`: Propagator used for extracting context from incoming requests
    /// - `inject_propagator`: Propagator used for injecting context into outgoing requests
    ///
    /// # Returns
    ///
    /// A new [`TextMapSplitPropagator`] instance
    pub fn new(extract_propagator: Propagator, inject_propagator: Propagator) -> Self {
        let mut fields = BTreeSet::from_iter(extract_propagator.fields());
        fields.extend(inject_propagator.fields());
        let fields = fields.into_iter().map(String::from).collect();

        Self {
            extract_propagator,
            inject_propagator,
            fields,
        }
    }

    /// Creates a split propagator based on the `OTEL_PROPAGATORS` environment variable.
    ///
    /// This method reads the `OTEL_PROPAGATORS` environment variable to determine which
    /// propagators to use. The first propagator in the list is used for injection,
    /// while all propagators are composed together for extraction.
    ///
    /// # Environment Variable Format
    ///
    /// The `OTEL_PROPAGATORS` variable should contain a comma-separated list of propagator names:
    /// - `tracecontext`: W3C Trace Context propagator
    /// - `baggage`: W3C Baggage propagator
    /// - `b3`: B3 single header propagator (requires "zipkin" feature)
    /// - `b3multi`: B3 multiple header propagator (requires "zipkin" feature)
    /// - `xray`: AWS X-Ray propagator (requires "xray" feature)
    /// - `none`: No-op propagator
    ///
    /// # Returns
    ///
    /// A configured [`TextMapSplitPropagator`] on success, or an [`OTelSdkError`] if
    /// the environment variable contains unsupported propagator names.
    ///
    /// # Examples
    ///
    /// ```bash
    /// export OTEL_PROPAGATORS=tracecontext,baggage
    /// ```
    ///
    /// ```rust
    /// use telemetry_rust::propagation::TextMapSplitPropagator;
    ///
    /// let propagator = TextMapSplitPropagator::from_env()?;
    /// # Ok::<(), opentelemetry_sdk::error::OTelSdkError>(())
    /// ```
    pub fn from_env() -> Result<Self, OTelSdkError> {
        let value_from_env = match util::env_var("OTEL_PROPAGATORS") {
            Some(value) => value,
            None => {
                return Ok(Self::default());
            }
        };
        let propagators: Vec<String> = value_from_env
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        tracing::info!(target: "otel::setup", propagators = propagators.join(","));

        let inject_propagator = match propagators.first() {
            Some(s) => propagator_from_string(s)?,
            None => Box::new(NonePropagator),
        };
        let propagators = propagators
            .iter()
            .rev()
            .map(|s| propagator_from_string(s))
            .collect::<Result<Vec<_>, _>>()?;
        let extract_propagator = Box::new(TextMapCompositePropagator::new(propagators));

        Ok(Self::new(extract_propagator, inject_propagator))
    }
}

impl TextMapPropagator for TextMapSplitPropagator {
    fn inject_context(&self, cx: &Context, injector: &mut dyn Injector) {
        self.inject_propagator.inject_context(cx, injector)
    }

    fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context {
        self.extract_propagator.extract_with_context(cx, extractor)
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(self.fields.as_slice())
    }
}

impl Default for TextMapSplitPropagator {
    fn default() -> Self {
        let trace_context_propagator = Box::new(TraceContextPropagator::new());
        #[cfg(feature = "zipkin")]
        let b3_propagator = Box::new(B3Propagator::with_encoding(
            B3Encoding::SingleAndMultiHeader,
        ));
        let composite_propagator = Box::new(TextMapCompositePropagator::new(vec![
            trace_context_propagator.clone(),
            #[cfg(feature = "zipkin")]
            b3_propagator,
        ]));

        Self::new(composite_propagator, trace_context_propagator)
    }
}

fn propagator_from_string(v: &str) -> Result<Propagator, OTelSdkError> {
    match v.trim() {
        "tracecontext" => Ok(Box::new(TraceContextPropagator::new())),
        "baggage" => Ok(Box::new(BaggagePropagator::new())),
        "none" => Ok(Box::new(NonePropagator)),
        #[cfg(feature = "zipkin")]
        "b3" => Ok(Box::new(B3Propagator::with_encoding(
            B3Encoding::SingleHeader,
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3" => Err(OTelSdkError::InternalFailure(
            "unsupported propagator from env OTEL_PROPAGATORS: 'b3', try to enable compile feature 'zipkin'"
                .to_owned(),
        )),
        #[cfg(feature = "zipkin")]
        "b3multi" => Ok(Box::new(B3Propagator::with_encoding(
            B3Encoding::MultipleHeader,
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3multi" => Err(OTelSdkError::InternalFailure(
            "unsupported propagator from env OTEL_PROPAGATORS: 'b3multi', try to enable compile feature 'zipkin'"
                .to_owned(),
        )),
        #[cfg(feature = "xray")]
        "xray" => Ok(Box::new(XrayPropagator::new())),
        #[cfg(not(feature = "xray"))]
        "xray" => Err(OTelSdkError::InternalFailure(
            "unsupported propagator from env OTEL_PROPAGATORS: 'xray', try to enable compile feature 'xray'"
                .to_owned(),
        )),
        unknown => Err(OTelSdkError::InternalFailure(format!(
            "unsupported propagator from env OTEL_PROPAGATORS: {unknown:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use assert2::assert;

    #[test]
    fn init_tracing_failed_on_invalid_propagator() {
        assert!(let Err(_) = super::propagator_from_string("xxxxxx"));
    }

    #[cfg(feature = "zipkin")]
    #[test]
    fn b3_encodings_preserve_span_context() {
        use opentelemetry::{
            Context,
            propagation::TextMapPropagator,
            trace::{
                SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState,
            },
        };
        use std::collections::HashMap;

        let span_context = SpanContext::new(
            TraceId::from(42),
            SpanId::from(7),
            TraceFlags::SAMPLED,
            true,
            TraceState::default(),
        );
        let context = Context::new().with_remote_span_context(span_context.clone());

        for (name, header) in [("b3", "b3"), ("b3multi", "x-b3-traceid")] {
            let propagator = super::propagator_from_string(name).unwrap();
            let mut headers = HashMap::new();
            propagator.inject_context(&context, &mut headers);
            assert!(headers.contains_key(header));
            assert!(headers.contains_key("b3") == (name == "b3"));
            assert!(propagator.extract(&headers).span().span_context() == &span_context);
            assert!(
                super::TextMapSplitPropagator::default()
                    .extract(&headers)
                    .span()
                    .span_context()
                    == &span_context
            );
        }
    }
}
