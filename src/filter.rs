use tracing::{subscriber::Interest, Level, Metadata, Subscriber};
use tracing_subscriber::layer::{Context, Filter, Layer};

pub struct OtelFilter {
    log_level: Level,
}

impl OtelFilter {
    pub fn new(log_level: Level) -> Self {
        Self { log_level }
    }

    #[inline(always)]
    fn _enabled(&self, meta: &Metadata<'_>) -> bool {
        meta.target() == "otel::tracing" || meta.level() <= &self.log_level
    }

    #[inline(always)]
    fn _callsite_enabled(&self, meta: &Metadata<'_>) -> Interest {
        if self._enabled(meta) {
            Interest::always()
        } else {
            Interest::never()
        }
    }
}

impl<S: Subscriber> Filter<S> for OtelFilter {
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        self._enabled(meta)
    }

    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        self._callsite_enabled(meta)
    }
}

impl<S: Subscriber> Layer<S> for OtelFilter {
    fn enabled(&self, meta: &Metadata<'_>, _: Context<'_, S>) -> bool {
        self._enabled(meta)
    }

    fn register_callsite(&self, meta: &'static Metadata<'static>) -> Interest {
        self._callsite_enabled(meta)
    }
}
