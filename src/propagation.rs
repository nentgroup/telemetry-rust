use opentelemetry::{
    propagation::{
        text_map_propagator::FieldIter, Extractor, Injector, TextMapPropagator,
    },
    Context,
};
use opentelemetry_sdk::propagation::{
    TextMapCompositePropagator, TraceContextPropagator,
};
use opentelemetry_zipkin::{B3Encoding, Propagator as B3Propagator};
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct TextMapSplitPropagator {
    extract_propagator: Box<dyn TextMapPropagator + Send + Sync>,
    inject_propagator: Box<dyn TextMapPropagator + Send + Sync>,
    fields: Vec<String>,
}

impl TextMapSplitPropagator {
    pub fn new(
        extract_propagator: Box<dyn TextMapPropagator + Send + Sync>,
        inject_propagator: Box<dyn TextMapPropagator + Send + Sync>,
    ) -> Self {
        let mut fields = BTreeSet::from_iter(extract_propagator.fields());
        fields.extend(inject_propagator.fields());
        let fields = fields.into_iter().map(String::from).collect();

        Self {
            extract_propagator,
            inject_propagator,
            fields,
        }
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
        let trace_context_propagator = TraceContextPropagator::new();
        let b3_propagator = B3Propagator::with_encoding(B3Encoding::SingleAndMultiHeader);
        let composite_propagator = TextMapCompositePropagator::new(vec![
            Box::new(trace_context_propagator.clone()),
            Box::new(b3_propagator),
        ]);

        Self::new(
            Box::new(composite_propagator),
            Box::new(trace_context_propagator),
        )
    }
}
