use tracing::{subscriber::Interest, Level, Metadata, Subscriber};
use tracing_subscriber::layer::{Context, Filter, Layer};

use crate::util;

const DEFAULT_TRACING_LEVEL: Level = Level::INFO;

pub struct TracingFilter {
    log_level: Level,
    tracing_level: Level,
}

impl TracingFilter {
    pub fn new(log_level: Level) -> Self {
        Self::new_with_tracing_level(log_level, read_tracing_level_from_env())
    }

    pub fn new_with_tracing_level(log_level: Level, tracing_level: Level) -> Self {
        Self {
            log_level,
            tracing_level,
        }
    }

    #[inline(always)]
    fn _enabled(&self, meta: &Metadata<'_>) -> bool {
        if meta.is_event() {
            meta.level() <= &self.log_level
        } else {
            meta.target() == "otel::tracing" || meta.level() <= &self.tracing_level
        }
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

impl<S: Subscriber> Filter<S> for TracingFilter {
    fn enabled(&self, meta: &Metadata<'_>, _: &Context<'_, S>) -> bool {
        self._enabled(meta)
    }

    fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest {
        self._callsite_enabled(meta)
    }
}

impl<S: Subscriber> Layer<S> for TracingFilter {
    fn enabled(&self, meta: &Metadata<'_>, _: Context<'_, S>) -> bool {
        self._enabled(meta)
    }

    fn register_callsite(&self, meta: &'static Metadata<'static>) -> Interest {
        self._callsite_enabled(meta)
    }
}

pub fn read_tracing_level_from_env() -> Level {
    if let Some(level_str) = util::env_var("OTEL_LOG_LEVEL") {
        level_str.parse().unwrap_or(DEFAULT_TRACING_LEVEL)
    } else {
        DEFAULT_TRACING_LEVEL
    }
}
