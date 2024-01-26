use crate::otlp::read_otel_log_level_from_env;
use tracing::{subscriber::Interest, Level, Metadata, Subscriber};
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;
use tracing_subscriber::layer::{Context, Filter};

pub struct OtelFilter {
    log_level: Level,
}

impl OtelFilter {
    pub fn new(log_level: Level) -> Self {
        Self { log_level }
    }

    #[inline(always)]
    fn _enabled(&self, meta: &Metadata<'_>) -> bool {
        meta.target() == TRACING_TARGET || meta.level() <= &self.log_level
    }
}

impl Default for OtelFilter {
    fn default() -> Self {
        Self::new(read_otel_log_level_from_env())
    }
}

impl<S: Subscriber> Filter<S> for OtelFilter {
    #[inline]
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        self._enabled(meta)
    }

    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        if self._enabled(meta) {
            Interest::always()
        } else {
            Interest::never()
        }
    }
}
