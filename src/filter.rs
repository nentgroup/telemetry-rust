use tracing::{Level, Metadata, Subscriber, subscriber::Interest};
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;
use tracing_subscriber::layer::{Context, Filter, Layer};

use crate::util;

const DEFAULT_TRACING_LEVEL: Level = Level::INFO;

pub struct TracingFilter {
    log_level: Level,
    tracing_level: Level,
}

impl TracingFilter {
    pub fn new(log_level: Level, tracing_level: Level) -> Self {
        Self {
            log_level,
            tracing_level,
        }
    }

    pub fn from_level(log_level: Level) -> Self {
        Self::new(log_level, read_tracing_level_from_env())
    }

    #[inline(always)]
    fn _enabled(&self, meta: &Metadata<'_>) -> bool {
        if meta.is_event() {
            meta.level() <= &self.log_level
        } else {
            meta.target() == TRACING_TARGET || meta.level() <= &self.tracing_level
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

impl From<Level> for TracingFilter {
    #[inline]
    fn from(level: Level) -> Self {
        Self::from_level(level)
    }
}

/// Reads the tracing level configuration from environment variables.
///
/// This function checks the `OTEL_LOG_LEVEL` environment variable to determine
/// the minimum tracing level. If the variable is not set or contains an invalid
/// value, it defaults to [`Level::INFO`].
///
/// # Supported Values
///
/// The environment variable should contain one of:
/// - `ERROR` or `error`
/// - `WARN` or `warn`  
/// - `INFO` or `info`
/// - `DEBUG` or `debug`
/// - `TRACE` or `trace`
///
/// # Returns
///
/// The configured [`Level`] for tracing, or [`Level::INFO`] as default
///
/// # Examples
///
/// ```bash
/// export OTEL_LOG_LEVEL=DEBUG
/// ```
///
/// ```rust
/// use telemetry_rust::otlp::read_otel_log_level_from_env;
///
/// let level = read_otel_log_level_from_env();
/// ```
pub(crate) fn read_tracing_level_from_env() -> Level {
    if let Some(level_str) = util::env_var("OTEL_LOG_LEVEL") {
        level_str.parse().unwrap_or(DEFAULT_TRACING_LEVEL)
    } else {
        DEFAULT_TRACING_LEVEL
    }
}
