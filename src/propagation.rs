use opentelemetry::{
    propagation::{
        text_map_propagator::FieldIter, Extractor, Injector, TextMapCompositePropagator,
        TextMapPropagator,
    },
    trace::TraceError,
    Context,
};
use opentelemetry_sdk::propagation::{BaggagePropagator, TraceContextPropagator};
#[cfg(feature = "zipkin")]
use opentelemetry_zipkin::{B3Encoding, Propagator as B3Propagator};
use std::collections::BTreeSet;

use crate::util;

pub type Propagator = Box<dyn TextMapPropagator + Send + Sync>;

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

#[derive(Debug)]
pub struct TextMapSplitPropagator {
    extract_propagator: Propagator,
    inject_propagator: Propagator,
    fields: Vec<String>,
}

impl TextMapSplitPropagator {
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

    pub fn from_env() -> Result<Self, TraceError> {
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

fn propagator_from_string(v: &str) -> Result<Propagator, TraceError> {
    match v.trim() {
        "tracecontext" => Ok(Box::new(TraceContextPropagator::new())),
        "baggage" => Ok(Box::new(BaggagePropagator::new())),
        "none" => Ok(Box::new(NonePropagator)),
        #[cfg(feature = "zipkin")]
        "b3" => Ok(Box::new(B3Propagator::with_encoding(
            B3Encoding::SingleHeader,
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3" => Err(TraceError::from(
            "unsupported propagator form env OTEL_PROPAGATORS: 'b3', try to enable compile feature 'zipkin'"
        )),
        #[cfg(feature = "zipkin")]
        "b3multi" => Ok(Box::new(B3Propagator::with_encoding(
            B3Encoding::MultipleHeader,
        ))),
        #[cfg(not(feature = "zipkin"))]
        "b3multi" => Err(TraceError::from(
            "unsupported propagator form env OTEL_PROPAGATORS: 'b3multi', try to enable compile feature 'zipkin'"
        )),
        unknown => Err(TraceError::from(format!(
            "unsupported propagator form env OTEL_PROPAGATORS: {unknown:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use assert2::let_assert;

    #[test]
    fn init_tracing_failed_on_invalid_propagator() {
        let_assert!(Err(_) = super::propagator_from_string("xxxxxx"));
    }
}
